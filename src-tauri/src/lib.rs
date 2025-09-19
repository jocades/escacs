use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use shakmaty::{
    fen::Fen, san::San, uci::UciMove, CastlingMode, Chess, Color, FromSetup, Position, Setup,
};
use tauri::{Builder, Manager, State};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, Instrument};

use crate::chess::{
    openings::{find_opening, gather_openings},
    search, Engine, Go, Info, Search,
};

pub mod chess;
pub mod database;

struct AppState {
    handle: Handle,
    client_restart_count: AtomicUsize,
}

static mut CALL_COUNT: usize = 0;

enum Op {
    Start(tauri::ipc::Channel<Info>, oneshot::Sender<usize>),
    Go(Go),
}

struct EngineEntry {
    job_tx: mpsc::Sender<Go>,
    stop_tx: mpsc::Sender<oneshot::Sender<()>>,
    is_searching: Arc<AtomicBool>,
}

pub struct EngineManager {
    engines: Vec<EngineEntry>,
    rx: mpsc::Receiver<Op>,
}

fn uci_to_san(chess: &mut Chess, uci_move: &str) -> anyhow::Result<String> {
    let m = uci_move.parse::<UciMove>()?.to_move(chess)?;
    chess.play_unchecked(m);
    Ok(San::from_move(chess, m).to_string())
}

fn prettyfy(fen: &str, info: &mut Info) -> anyhow::Result<()> {
    let fen = fen.parse::<Fen>()?;
    let mut chess: Chess = fen.into_position(CastlingMode::Standard)?;

    for uci_move in info.pv.iter_mut() {
        *uci_move = uci_to_san(&mut chess, uci_move)?;
    }
    Ok(())
}

async fn controller(
    mut engine: Engine,
    mut job_rx: mpsc::Receiver<Go>,
    mut stop_rx: mpsc::Receiver<oneshot::Sender<()>>,
    chan: tauri::ipc::Channel<Info>,
    is_searching: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    while let Some(job) = job_rx.recv().await {
        debug!("new job");
        engine.tx.send(job.to_cmd()).await?;
        is_searching.store(true, Ordering::SeqCst);

        loop {
            select! {
                Some(line) = engine.rx.recv() => match search(&line)? {
                    Some(Search::Info(mut info)) => {
                        prettyfy(job.fen.as_ref().unwrap(), &mut info)?;
                        chan.send(info)?;
                    },
                    Some(Search::BestMove(_)) => {},
                    None => continue,
                },
                Some(ack) = stop_rx.recv() => {
                    engine.stop().await?;
                    debug!("engine stop");
                    _ = ack.send(());
                    break;
                },
                else => break,
            }
        }
        is_searching.store(false, Ordering::SeqCst);
    }
    Ok(())
}

impl EngineManager {
    async fn run(&mut self) -> anyhow::Result<()> {
        let _span = tracing::debug_span!("manager");
        while let Some(op) = self.rx.recv().await {
            match op {
                Op::Start(chan, ack) => {
                    if self.engines.len() > 0 {
                        continue;
                    }
                    let mut engine = Engine::new("stockfish")?;
                    let opts = [("Threads", "8"), ("UCI_ShowWDL", "true"), ("MultiPV", "3")];
                    engine.uci().await?;
                    engine.opts(&opts).await?;
                    engine.isready().await?;

                    let (job_tx, job_rx) = mpsc::channel(32);
                    let (stop_tx, stop_rx) = mpsc::channel(1);
                    let is_searching = Arc::new(AtomicBool::new(false));

                    let entry = EngineEntry {
                        job_tx,
                        stop_tx,
                        is_searching: is_searching.clone(),
                    };
                    self.engines.push(entry);

                    tauri::async_runtime::spawn(
                        async move {
                            if let Err(e) =
                                controller(engine, job_rx, stop_rx, chan, is_searching).await
                            {
                                tracing::error!(cause = %e, "controller error");
                            }
                        }
                        .instrument(tracing::trace_span!("controller")),
                    );

                    _ = ack.send(self.engines.len() - 1);
                }
                Op::Go(job) => {
                    let EngineEntry {
                        job_tx,
                        stop_tx,
                        is_searching,
                    } = &mut self.engines[0];

                    debug!(?is_searching);

                    if is_searching.load(Ordering::SeqCst) {
                        let (ack, syn) = oneshot::channel();
                        stop_tx.send(ack).await?;
                        syn.await?;
                    }

                    job_tx.send(job).await?;
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
                error!(cause = %e, "manager error");
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
        debug!("start engine");
        state.handle.start_engine(chan).await;
    }
    Ok(())
}

#[tauri::command]
async fn go(fen: &str, state: State<'_, AppState>) -> Result<(), String> {
    unsafe {
        CALL_COUNT += 1;
        debug!("call_count = {CALL_COUNT}");
    };
    debug!(fen, "GO");
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
fn test_obj(value: serde_json::Map<String, serde_json::Value>) {
    debug!(?value);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            start_engine,
            go,
            test_what,
            find_opening,
            test_obj,
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
    use tracing_subscriber::filter::EnvFilter;

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::TRACE.into())
        .from_env_lossy()
        .add_directive("escacs_lib::chess::engine=debug".parse().unwrap())
        .add_directive(
            "tao::platform_impl::platform::window_delegate=info"
                .parse()
                .unwrap(),
        );

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .with_target(true)
        .compact()
        .init();
}
