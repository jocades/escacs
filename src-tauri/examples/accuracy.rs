use escacs_lib::chess::{Engine, Go, Score};
use pgn_reader::{Reader, SanPlus};
use shakmaty::{fen::Fen, uci::UciMove, Chess, Color, Position};
use std::ops::ControlFlow;
use tracing::{debug, info};

#[derive(Debug)]
struct Move {
    san: String,
    uci: String,
    color: Color,
    before: String,
    after: String,
}

#[derive(Default)]
struct Extractor {
    moves: Vec<Move>,
}

impl pgn_reader::Visitor for Extractor {
    type Tags = Option<Chess>;
    type Movetext = Chess;
    type Output = anyhow::Result<Chess>;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(None)
    }

    fn begin_movetext(&mut self, tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(tags.unwrap_or_default())
    }

    fn san(
        &mut self,
        movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        let before = Fen::from_position(movetext, shakmaty::EnPassantMode::Legal).to_string();
        let color = movetext.turn();
        match san_plus.san.to_move(movetext) {
            Ok(m) => {
                let uci = m.to_uci(shakmaty::CastlingMode::Standard).to_string();
                movetext.play_unchecked(m);
                self.moves.push(Move {
                    san: san_plus.to_string(),
                    uci,
                    color,
                    before,
                    after: Fen::from_position(movetext, shakmaty::EnPassantMode::Legal).to_string(),
                });
                ControlFlow::Continue(())
            }
            Err(e) => ControlFlow::Break(Err(e.into())),
        }
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        Ok(movetext)
    }
}

const CEILING: f32 = 10_000.0;

fn ceiled(cp: f32) -> f32 {
    cp.min(-CEILING).max(CEILING)
    // if cp > CEILING {
    //     CEILING
    // } else if cp < -CEILING {
    //     -CEILING
    // } else {
    //     cp
    // }
}

fn to_cp(score: Score, color: Color) -> i32 {
    match score {
        Score::Cp(n) => {
            if color == Color::White {
                n
            } else {
                -n
            }
        }
        Score::Mate(n) => {
            info!(mate = n, color = ?color);
            if color == Color::White {
                10_000
            } else {
                -10_000
            }
        }
    }
}

/// (-1 .. +1)
fn win_pct_from_cp(cp: f64) -> f64 {
    (50.0 + 50.0 * (2.0 / (1.0 + (-0.00368208 * cp).exp()) - 1.0))
        .min(-1.0)
        .max(1.0)
}

fn acc_from_win_pct(before: f64, after: f64) -> f64 {
    if after >= before {
        return 100.0;
    }

    {
        let delta = before - after;
        let raw = 103.1668100711649 * (-0.04354415386753951 * delta).exp() + -3.166924740191411;
        raw + 1.0
    }
    .min(0.0)
    .max(100.0)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let mut engine = Engine::new("stockfish")?;
    let opts = [("Threads", "8"), ("UCI_ShowWDL", "true")];
    engine.uci().await?;
    engine.opts(&opts).await?;
    engine.isready().await?;

    let pgn = std::fs::read_to_string("examples/game3.pgn")?;
    let mut reader = Reader::new(std::io::Cursor::new(&pgn));
    let mut extractor = Extractor::default();
    reader.read_game(&mut extractor)?;

    debug!(move_count = extractor.moves.len() / 2);

    let mut w = Vec::new();
    let mut b = Vec::new();

    for (i, m) in extractor.moves.iter().enumerate().take(16) {
        let (info_before, _) = Go::new()
            .fen(&m.before)
            .depth(15)
            .execute(&mut engine)
            .await?;

        let cp_before = to_cp(info_before.score, m.color) as f64;
        let win_pct_before = win_pct_from_cp(cp_before);

        let (info_after, _) = Go::new()
            .fen(&m.after)
            .depth(15)
            .execute(&mut engine)
            .await?;

        let cp_after = to_cp(info_after.score, m.color) as f64;
        let win_pct_after = win_pct_from_cp(cp_after);

        let acc_pct = acc_from_win_pct(win_pct_before, win_pct_after);

        debug!(index = i, color = ?m.color, ?m.san, cp_before, win_pct_before, cp_after, win_pct_after, acc_pct);

        match m.color {
            Color::White => w.push(acc_pct),
            Color::Black => b.push(acc_pct),
        };
    }

    let w_avg_acc = w.iter().sum::<f64>() / w.len() as f64;
    let b_avg_acc = b.iter().sum::<f64>() / b.len() as f64;

    println!("white = {w_avg_acc}");
    println!("black = {b_avg_acc}");

    Ok(())
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
