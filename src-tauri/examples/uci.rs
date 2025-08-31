use std::ops::ControlFlow;

use escacs_lib::chess::{Engine, Go, Score, Visitor};
use pgn_reader::{Reader, SanPlus};
use shakmaty::{fen::Fen, Chess, Color, Position};

#[derive(Debug)]
struct Move {
    san: String,
    color: Color,
    from: String,
    to: String,
    before: String,
    after: String,
}

#[derive(Default)]
struct Extractor {
    pos: Chess,
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
                movetext.play_unchecked(m);
                self.moves.push(Move {
                    san: san_plus.to_string(),
                    color,
                    from: m.from().unwrap().to_string(),
                    to: m.to().to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveCategory {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

fn classify_move(loss: i32) -> MoveCategory {
    match loss {
        0..=20 => MoveCategory::Best,
        21..=50 => MoveCategory::Excellent,
        51..=100 => MoveCategory::Good,
        101..=300 => MoveCategory::Inaccuracy,
        301..=600 => MoveCategory::Mistake,
        _ => MoveCategory::Blunder,
    }
}

fn score_to_cp(score: Score) -> i32 {
    match score {
        Score::Cp(cp) => cp,
        Score::Mate(n) => {
            if n > 0 {
                10_000
            } else {
                -10_000
            }
        }
    }
}

#[derive(Debug, Clone)]
struct AnalyzedMove {
    san: String,
    color: Color,
    eval_before: i32,
    eval_after: i32,
    loss: i32,
    category: MoveCategory,
}

#[derive(Debug, Default, serde::Serialize)]
struct GameSummary {
    best: usize,
    excellent: usize,
    good: usize,
    inaccuracy: usize,
    mistake: usize,
    blunder: usize,
    move_count: usize,
    total_loss: i32,
    accuracy: f64, // percentage
}

impl GameSummary {
    fn update(&mut self, category: MoveCategory, loss: i32) {
        self.move_count += 1;
        self.total_loss += loss;
        match category {
            MoveCategory::Best => self.best += 1,
            MoveCategory::Excellent => self.excellent += 1,
            MoveCategory::Good => self.good += 1,
            MoveCategory::Inaccuracy => self.inaccuracy += 1,
            MoveCategory::Mistake => self.mistake += 1,
            MoveCategory::Blunder => self.blunder += 1,
        }
    }

    fn digest(&mut self) {
        if self.move_count > 0 {
            let acpl = self.total_loss as f64 / self.move_count as f64;
            self.accuracy = 100.0 / (1.0 + (acpl / 80.0).powi(2));
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct GameReport {
    white: GameSummary,
    black: GameSummary,
}

async fn analyze(engine: &mut Engine, game: &[Move]) -> anyhow::Result<Vec<AnalyzedMove>> {
    let mut moves = Vec::new();
    for (i, m) in game.iter().enumerate() {
        let (info_before, _) = Go::new().fen(&m.before).depth(15).execute(engine).await?;
        let (info_after, _) = Go::new().fen(&m.after).depth(15).execute(engine).await?;

        let cp_before = score_to_cp(info_before.score);
        let cp_after = score_to_cp(info_after.score);

        let loss = (cp_before - cp_after).max(0);

        let category = classify_move(loss);
        moves.push(AnalyzedMove {
            san: m.san.clone(),
            color: m.color,
            eval_before: cp_before,
            eval_after: cp_after,
            loss,
            category,
        });

        println!(
            "{}. {} -> {:?} (best={}cp, played={}cp, loss={})",
            i + 1,
            m.san,
            category,
            cp_before,
            cp_after,
            loss
        );
    }

    Ok(moves)
}

fn compute_report(moves: &[AnalyzedMove]) -> GameReport {
    let mut white = GameSummary::default();
    let mut black = GameSummary::default();

    for m in moves {
        match m.color {
            Color::White => white.update(m.category, m.loss),
            Color::Black => black.update(m.category, m.loss),
        }
    }

    white.digest();
    black.digest();

    GameReport { white, black }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let pgn = "1.f3 e5 2. g4";
    let pgn = std::fs::read_to_string("./examples/game1.pgn")?;
    println!("{pgn}");

    let mut reader = Reader::new(std::io::Cursor::new(&pgn));
    let mut extractor = Extractor::default();
    reader.read_game(&mut extractor)?;

    let mut engine = Engine::new("stockfish")?;
    let opts = [("Threads", "8"), ("UCI_ShowWDL", "true")];

    engine.uci().await?;
    engine.opts(&opts).await?;
    engine.isready().await?;

    let analyzed_moves = analyze(&mut engine, &extractor.moves).await?;
    let report = compute_report(&analyzed_moves);

    println!("Report: {:#?}", report);

    Ok(())
}
