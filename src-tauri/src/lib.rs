use std::sync::atomic::AtomicUsize;

use dashmap::DashMap;
use tauri::{Builder, Manager, State};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::chess::{BestMove, Engine, Go, Info};

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
    Start(tauri::ipc::Channel<Info>, oneshot::Sender<()>),
    Go(Go),
}

struct EngineEntry {
    engine: Engine,
    visitor: Visitor,
    stop_tx: mpsc::Sender<()>,
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
        self.chan.send(info).unwrap();
    }

    fn best(&mut self, best: BestMove) {}
}

impl EngineManager {
    async fn run(&mut self) {
        while let Some(op) = self.rx.recv().await {
            match op {
                Op::Start(chan, resp) => {
                    if self.engines.len() > 0 {
                        continue;
                    }
                    let (stop_tx, stop_rx) = mpsc::channel(32);
                    let mut engine = Engine::new("stockfish", stop_rx).unwrap();
                    engine.uci().await.unwrap();
                    engine.isready().await.unwrap();
                    let visitor = Visitor { chan };
                    self.engines.push(EngineEntry {
                        engine,
                        visitor,
                        stop_tx,
                    });
                    _ = resp.send(());
                }
                Op::Go(job) => {
                    let EngineEntry {
                        engine,
                        visitor,
                        stop_tx,
                    } = &mut self.engines[0];
                    if engine.is_searching {
                        println!("manager stop");
                        stop_tx.send(()).await.unwrap();
                    }
                    engine.go_with(job, visitor).await.unwrap();
                }
            }
        }
    }
}

pub struct Handle {
    tx: mpsc::Sender<Op>,
}

impl Handle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut manager = EngineManager {
            engines: Vec::new(),
            rx,
        };
        tauri::async_runtime::spawn(async move { manager.run().await });
        Self { tx }
    }

    pub async fn start_engine(&self, chan: tauri::ipc::Channel<Info>) {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Op::Start(chan, tx))
            .await
            .expect("manager died");
        rx.await.unwrap();
    }

    pub async fn go(&self, job: Go) {
        self.tx.send(Op::Go(job)).await.expect("manager died");
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
            test_what,
            call_count
        ])
        .setup(|app| {
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
