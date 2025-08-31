use dashmap::DashMap;
use tauri::{Builder, Manager, State};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::chess::{BestMove, Engine, Go, Info};

pub mod chess;

struct AppState {
    handle: Handle,
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

enum Op {
    Init(tauri::ipc::Channel<Info>),
    Go(Go),
}

struct EngineManager {
    engines: Vec<(Engine, Visitor)>,
    rx: mpsc::Receiver<Op>,
}

struct Visitor {
    chan: tauri::ipc::Channel<Info>,
}

impl chess::Visitor for Visitor {
    fn info(&mut self, info: Info) {
        println!("INFO: {info:?}");
        self.chan.send(info).unwrap();
    }

    fn best(&mut self, best: BestMove) {
        println!("BEST: {best:?}");
    }
}

impl EngineManager {
    async fn run(&mut self) {
        while let Some(op) = self.rx.recv().await {
            match op {
                Op::Init(chan) => {
                    if self.engines.len() > 0 {
                        continue;
                    }
                    let mut engine = Engine::new("stockfish").unwrap();
                    engine.uci().await.unwrap();
                    engine.isready().await.unwrap();
                    let visitor = Visitor { chan };
                    self.engines.push((engine, visitor));
                }
                Op::Go(job) => {
                    let (engine, visitor) = &mut self.engines[0];
                    engine.go_with(job, visitor).await.unwrap();
                }
            }
        }
    }
}

struct Handle {
    tx: mpsc::Sender<Op>,
}

impl Handle {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut manager = EngineManager {
            engines: Vec::new(),
            rx,
        };
        tauri::async_runtime::spawn(async move { manager.run().await });
        Self { tx }
    }
}

#[tauri::command]
async fn start_engine(
    state: State<'_, AppState>,
    chan: tauri::ipc::Channel<Info>,
) -> Result<(), String> {
    println!("start engine");
    state.handle.tx.send(Op::Init(chan)).await.unwrap();
    Ok(())
}

#[tauri::command]
async fn go(fen: &str, state: State<'_, AppState>) -> Result<(), String> {
    let job = Go::new().fen(fen).depth(15);
    state.handle.tx.send(Op::Go(job)).await.unwrap();
    Ok(())
}

#[tauri::command]
fn test_what() -> &'static str {
    println!("test what");
    "Hello World!"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_engine, go, test_what])
        .setup(|app| {
            let state = AppState {
                handle: Handle::new(),
            };
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
