use anyhow::Result;
use escacs_lib::chess::{self, BestMove, Engine, Go, Info};
use tokio::sync::mpsc;

struct Visitor;

impl chess::Visitor for Visitor {
    fn info(&mut self, info: Info) {
        // self.chan.send(info).unwrap();
    }

    fn best(&mut self, best: BestMove) {}
}

#[tokio::main]
async fn main() -> Result<()> {
    let (stop_tx, stop_rx) = mpsc::channel(32);
    let mut engine = Engine::new("stockfish", stop_rx)?;
    engine.uci().await?;
    engine.isready().await?;

    let job = Go::new().moves(&["d2d4", "g8f6"]).depth(50);

    Ok(())
}
