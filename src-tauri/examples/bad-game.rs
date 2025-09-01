use escacs_lib::chess::{Engine, Go};
use shakmaty::{san::San, uci::UciMove, Chess, Move, Position};

fn uci_to_san(chess: &mut Chess, uci_move: &str) -> anyhow::Result<String> {
    let m = uci_move.parse::<UciMove>()?.to_move(chess)?;
    chess.play_unchecked(m);
    Ok(San::from_move(chess, m).to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut chess = Chess::default();
    let uci_moves = ["f2f4", "d7d5", "g2g4"];
    let moves = dbg!(uci_moves
        .iter()
        .map(|uci_move| {
            let m = uci_move
                .parse::<UciMove>()
                .unwrap()
                .to_move(&chess)
                .unwrap();
            chess.play_unchecked(m);
            m
        })
        .collect::<Vec<_>>());

    let mut engine = Engine::new("stockfish")?;
    engine.uci().await?;
    engine.isready().await?;

    let job = Go::new().depth(26).moves(&uci_moves);
    let (info, _) = dbg!(engine.go(job).await?);

    let san_moves = info
        .pv
        .iter()
        .map(|uci_move| uci_to_san(&mut chess, uci_move))
        .collect::<anyhow::Result<String>>();
    Ok(())
}
