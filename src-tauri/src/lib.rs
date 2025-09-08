use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use tauri::{Builder, Manager, State};
use tokio::{
    select,
    sync::{mpsc, oneshot, Notify},
};

use crate::chess::{
    openings::{find_opening, gather_openings},
    BestMove, Engine, Go, Info,
};

pub mod chess;

struct AppState {
    handle: Handle,
    client_restart_count: AtomicUsize,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command]
// fn greet(name: &str) -> String {
//     println!("GREET CMD");
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

struct Handler {
    engines: Vec<Engine>,
}

static mut CALL_COUNT: usize = 0;

enum Op {
    Start(tauri::ipc::Channel<Info>, oneshot::Sender<usize>),
    Go(Go),
    Stop,
}

struct EngineEntry {
    job_tx: mpsc::Sender<Go>,
    notify: Arc<Notify>,
    is_searching: Arc<AtomicBool>,
}

pub struct EngineManager {
    engines: Vec<EngineEntry>,
    rx: mpsc::Receiver<Op>,
}

pub struct Visitor {
    chan: tauri::ipc::Channel<Info>,
}

impl chess::Visitor for Visitor {
    fn info(&mut self, info: Info) {
        println!("INFO: depth = {}", info.depth);
        self.chan.send(info).unwrap();
    }

    fn best(&mut self, best: BestMove) {}
}

enum Command {
    Go(Go),
}

async fn controller(
    mut engine: Engine,
    mut visitor: Visitor,
    mut job_rx: mpsc::Receiver<Go>,
    stop: Arc<Notify>,
    is_searching: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    loop {
        select! {
            Some(job) = job_rx.recv() => {
                println!("[controller] is_searching = true");
                is_searching.store(true, Ordering::SeqCst);
                let fut = engine.go_with(job, &mut visitor);
                select! {
                    res = fut => res?,
                    () = stop.notified() => {
                        println!("notified");
                        engine.stop().await?;
                    }
                }
            }
            else => {
                println!("[controller] break");
                break;
            },
        }
        is_searching.store(false, Ordering::SeqCst);
        println!("[controller] is_searching = false");
    }
    Ok(())
}

impl EngineManager {
    async fn run(&mut self) -> anyhow::Result<()> {
        while let Some(op) = self.rx.recv().await {
            match op {
                Op::Start(chan, ack) => {
                    if self.engines.len() > 0 {
                        continue;
                    }
                    let mut engine = Engine::new("stockfish").unwrap();
                    let opts = [("Threads", "8"), ("UCI_ShowWDL", "true"), ("MultiPV", "3")];
                    engine.uci().await?;
                    engine.opts(&opts).await?;
                    engine.isready().await?;

                    let (job_tx, job_rx) = mpsc::channel(32);
                    let notify = Arc::new(Notify::new());
                    let is_searching = Arc::new(AtomicBool::new(false));

                    let entry = EngineEntry {
                        job_tx,
                        notify: notify.clone(),
                        is_searching: is_searching.clone(),
                    };
                    self.engines.push(entry);

                    let visitor = Visitor { chan };

                    tauri::async_runtime::spawn(async move {
                        if let Err(e) =
                            controller(engine, visitor, job_rx, notify, is_searching).await
                        {
                            eprintln!("handler error: {e}");
                        }
                    });

                    _ = ack.send(self.engines.len() - 1);
                }
                Op::Go(job) => {
                    println!("[manager] job = {job:?}");
                    let EngineEntry {
                        job_tx,
                        notify,
                        is_searching,
                    } = &mut self.engines[0];
                    if is_searching.load(Ordering::SeqCst) {
                        println!("[manager] is_searching = true");
                        notify.notify_one();
                    }
                    job_tx.send(job).await?;
                }
                Op::Stop => {
                    // let (_, stop_tx) = &mut self.engines[0];
                    // stop_tx.send(()).await?;
                }
            }
        }
        Ok(())
    }
}

pub struct Handle {
    tx: mpsc::Sender<Op>,
}

impl Handle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut manager = EngineManager {
            rx,
            engines: Vec::new(),
        };
        tauri::async_runtime::spawn(async move {
            if let Err(e) = manager.run().await {
                eprintln!("manager error: {e}");
            }
        });
        Self { tx }
    }

    pub async fn start_engine(&self, chan: tauri::ipc::Channel<Info>) {
        let (ack, syn) = oneshot::channel();
        self.tx
            .send(Op::Start(chan, ack))
            .await
            .expect("manager died");
        syn.await.unwrap();
    }

    pub async fn go(&self, job: Go) {
        self.tx.send(Op::Go(job)).await.expect("manager died");
    }

    pub async fn stop_engine(&self) {
        self.tx.send(Op::Stop).await.expect("manager died");
    }
}

#[tauri::command]
async fn start_engine(
    state: State<'_, AppState>,
    chan: tauri::ipc::Channel<Info>,
) -> Result<(), String> {
    let n = state
        .client_restart_count
        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    if n == 0 {
        println!("start engine");
        state.handle.start_engine(chan).await;
    }
    Ok(())
}

#[tauri::command]
async fn go(fen: &str, state: State<'_, AppState>) -> Result<(), String> {
    unsafe {
        CALL_COUNT += 1;
        println!("call_count = {CALL_COUNT}");
    };
    println!("go fen: {fen}");
    let job = Go::new().fen(fen).depth(26);
    state.handle.go(job).await;
    Ok(())
}

#[tauri::command]
async fn stop_engine(state: State<'_, AppState>) -> Result<(), String> {
    state.handle.stop_engine().await;
    Ok(())
}

#[tauri::command]
fn test_what() -> &'static str {
    println!("test what");
    "Hello World!"
}

#[tauri::command]
fn call_count() {
    unsafe {
        CALL_COUNT += 1;
        println!("call_count = {CALL_COUNT}");
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_engine,
            go,
            stop_engine,
            test_what,
            call_count,
            find_opening,
        ])
        .setup(|app| {
            setup_logging();
            gather_openings();
            let state = AppState {
                handle: Handle::new(),
                client_restart_count: AtomicUsize::default(),
            };
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_logging() {
    use tracing::level_filters::LevelFilter;

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .without_time()
        .with_target(false)
        .compact()
        .init();
}
