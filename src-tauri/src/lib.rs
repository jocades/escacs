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
    sync::{mpsc, oneshot, Mutex},
};
use tracing::{debug, error, Instrument};

use crate::chess::{
    openings::{find_opening, gather_openings},
    search, Engine, Go, Info, Search,
};

pub mod chess;
pub mod database;

struct AppState {
    manager: Arc<Mutex<EngineManager>>,
    client_restart_count: AtomicUsize,
}

static mut CALL_COUNT: usize = 0;

enum Op {
    Go(Go),
    NewGame,
}

struct EngineEntry {
    tx: mpsc::Sender<Op>,
    stop_tx: mpsc::Sender<oneshot::Sender<()>>,
    is_searching: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct EngineManager {
    engines: Vec<EngineEntry>,
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
    mut rx: mpsc::Receiver<Op>,
    mut stop_rx: mpsc::Receiver<oneshot::Sender<()>>,
    chan: tauri::ipc::Channel<Info>,
    is_searching: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    while let Some(op) = rx.recv().await {
        match op {
            Op::Go(job) => {
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
            Op::NewGame => {
                engine.tx.send("ucinewgame".into()).await?;
                engine.isready().await?;
                debug!("NEW GAME READY");
            }
        }
    }
    Ok(())
}

impl EngineManager {
    async fn start_engine(&mut self, chan: tauri::ipc::Channel<Info>) -> anyhow::Result<()> {
        if !self.engines.is_empty() {
            return Ok(());
        }

        let mut engine = Engine::new("stockfish")?;
        let opts = [("Threads", "8"), ("UCI_ShowWDL", "true"), ("MultiPV", "3")];
        engine.uci().await?;
        engine.opts(&opts).await?;
        engine.isready().await?;

        let (tx, rx) = mpsc::channel(32);
        let (stop_tx, stop_rx) = mpsc::channel(1);
        let is_searching = Arc::new(AtomicBool::new(false));

        self.engines.push(EngineEntry {
            tx,
            stop_tx,
            is_searching: is_searching.clone(),
        });

        tauri::async_runtime::spawn(
            async move {
                if let Err(e) = controller(engine, rx, stop_rx, chan, is_searching).await {
                    tracing::error!(cause = %e, "controller error");
                }
            }
            .instrument(tracing::trace_span!("controller")),
        );

        Ok(())
    }

    async fn go(&mut self, fen: &str) -> anyhow::Result<()> {
        let EngineEntry {
            tx,
            stop_tx,
            is_searching,
        } = &mut self.engines[0];

        debug!(?is_searching);

        if is_searching.load(Ordering::SeqCst) {
            let (ack, syn) = oneshot::channel();
            stop_tx.send(ack).await?;
            syn.await?;
        }

        let job = Go::new().fen(fen).depth(26);
        tx.send(Op::Go(job)).await?;

        Ok(())
    }

    async fn new_game(&mut self) -> anyhow::Result<()> {
        let EngineEntry {
            tx,
            stop_tx,
            is_searching,
        } = &mut self.engines[0];

        if is_searching.load(Ordering::SeqCst) {
            let (ack, syn) = oneshot::channel();
            stop_tx.send(ack).await?;
            syn.await?;
        }

        tx.send(Op::NewGame).await?;

        Ok(())
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
        state.manager.lock().await.start_engine(chan).await.unwrap();
    }
    Ok(())
}

#[tauri::command]
async fn go(fen: &str, state: State<'_, AppState>) -> Result<(), String> {
    state.manager.lock().await.go(fen).await.unwrap();
    Ok(())
}

#[tauri::command]
async fn new_game(state: State<'_, AppState>) -> Result<(), String> {
    state.manager.lock().await.new_game().await.unwrap();
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
            new_game,
            test_what,
            find_opening,
            test_obj,
        ])
        .setup(|app| {
            setup_logging();
            gather_openings();
            let state = AppState {
                manager: Arc::new(Mutex::new(EngineManager::default())),
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
