use anyhow::Result;
use escacs_lib::chess::{self, BestMove, Engine, Go, Info, Score};

struct Visitor;

impl chess::Visitor for Visitor {
    fn info(&mut self, info: chess::Info) {
        println!("INFO: {info:?}");
    }

    fn best(&mut self, best: chess::BestMove) {
        println!("BEST: {best:?}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveClassification {
    Best,       // Top engine move or within 10cp
    Excellent,  // 10-25cp loss
    Good,       // 25-50cp loss
    Inaccuracy, // 50-100cp loss
    Mistake,    // 100-200cp loss
    Blunder,    // 200+cp loss
    Missed,     // Missed forced mate
}

impl MoveClassification {
    pub fn from_centipawn_loss(cp_loss: i32, missed_mate: bool) -> Self {
        if missed_mate {
            return Self::Missed;
        }

        match cp_loss {
            0..=10 => Self::Best,
            11..=25 => Self::Excellent,
            26..=50 => Self::Good,
            51..=100 => Self::Inaccuracy,
            101..=200 => Self::Mistake,
            _ => Self::Blunder,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Best => "#00b050",
            Self::Excellent => "#92d050",
            Self::Good => "#ffc000",
            Self::Inaccuracy => "#ff9900",
            Self::Mistake => "#ff6600",
            Self::Blunder => "#ff0000",
            Self::Missed => "#cc0000",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Best => "✓",
            Self::Excellent => "◐",
            Self::Good => "◑",
            Self::Inaccuracy => "?!",
            Self::Mistake => "?",
            Self::Blunder => "??",
            Self::Missed => "??",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalyzedMove {
    pub move_san: String,
    pub move_uci: String,
    pub move_number: u32,
    pub is_white: bool,
    pub classification: MoveClassification,
    pub evaluation_before: Score,
    pub evaluation_after: Score,
    pub centipawn_loss: i32,
    pub best_move: String,
    pub principal_variation: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct PlayerStats {
    pub accuracy: f64,
    pub moves: u32,
    pub best_moves: u32,
    pub excellent_moves: u32,
    pub good_moves: u32,
    pub inaccuracies: u32,
    pub mistakes: u32,
    pub blunders: u32,
    pub missed_mates: u32,
    pub average_centipawn_loss: f64,
}

impl PlayerStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_move(&mut self, analyzed_move: &AnalyzedMove) {
        self.moves += 1;
        match analyzed_move.classification {
            MoveClassification::Best => self.best_moves += 1,
            MoveClassification::Excellent => self.excellent_moves += 1,
            MoveClassification::Good => self.good_moves += 1,
            MoveClassification::Inaccuracy => self.inaccuracies += 1,
            MoveClassification::Mistake => self.mistakes += 1,
            MoveClassification::Blunder => self.blunders += 1,
            MoveClassification::Missed => self.missed_mates += 1,
        }
    }

    pub fn calculate_accuracy(&mut self, total_centipawn_loss: f64) {
        if self.moves == 0 {
            self.accuracy = 0.0;
            return;
        }

        self.average_centipawn_loss = total_centipawn_loss / self.moves as f64;

        // Lichess-style accuracy calculation
        // accuracy = 103.1668 * exp(-0.04354 * (average_centipawn_loss + 1.72)) + 1.72
        self.accuracy = (103.1668 * (-0.04354 * (self.average_centipawn_loss + 1.72)).exp() + 1.72)
            .min(100.0)
            .max(0.0);
    }
}

#[derive(Debug)]
pub struct GameAnalysis {
    pub white_stats: PlayerStats,
    pub black_stats: PlayerStats,
    pub moves: Vec<AnalyzedMove>,
    pub opening_name: Option<String>,
    pub game_result: Option<String>,
}

impl GameAnalysis {
    pub fn new() -> Self {
        Self {
            white_stats: PlayerStats::new(),
            black_stats: PlayerStats::new(),
            moves: Vec::new(),
            opening_name: None,
            game_result: None,
        }
    }

    pub fn add_move(&mut self, analyzed_move: AnalyzedMove) {
        if analyzed_move.is_white {
            self.white_stats.add_move(&analyzed_move);
        } else {
            self.black_stats.add_move(&analyzed_move);
        }
        self.moves.push(analyzed_move);
    }

    pub fn finalize(&mut self) {
        let white_total_loss: f64 = self
            .moves
            .iter()
            .filter(|m| m.is_white)
            .map(|m| m.centipawn_loss as f64)
            .sum();
        let black_total_loss: f64 = self
            .moves
            .iter()
            .filter(|m| !m.is_white)
            .map(|m| m.centipawn_loss as f64)
            .sum();
        self.white_stats.calculate_accuracy(white_total_loss);
        self.black_stats.calculate_accuracy(black_total_loss);
    }
}

pub struct GameAnalyzer<'a> {
    engine: &'a mut Engine,
    depth: u32,
    skip_opening_moves: u32,
}

impl<'a> GameAnalyzer<'a> {
    pub fn new(engine: &'a mut Engine, depth: u32) -> Self {
        Self {
            engine,
            depth,
            skip_opening_moves: 8,
        }
    }

    async fn analyze_position(
        &mut self,
        moves: &[String],
        starting_fen: Option<&str>,
    ) -> Result<(Info, BestMove)> {
        let job = if let Some(fen) = starting_fen {
            Go::new().fen(fen).moves(moves).depth(self.depth)
        } else {
            Go::new().moves(moves).depth(self.depth)
        };
        self.engine.go(job).await
    }

    pub async fn analyze_game(
        &mut self,
        pgn_moves: &[String],
        starting_fen: Option<&str>,
    ) -> Result<GameAnalysis> {
        let mut analysis = GameAnalysis::new();
        let mut position_moves: Vec<String> = Vec::new();
        let mut move_number = 1;

        for (index, move_str) in pgn_moves.iter().enumerate() {
            let is_white = index % 2 == 0;

            // Skip opening moves
            if index < self.skip_opening_moves as usize {
                position_moves.push(move_str.to_string());
                if !is_white {
                    move_number += 1;
                }
                continue;
            }

            // Analyze position before the move
            let eval_before = self.analyze_position(&position_moves, starting_fen).await?;

            position_moves.push(move_str.to_string());

            let eval_after = self.analyze_position(&position_moves, starting_fen).await?;

            let cp_loss =
                self.calculate_centipawn_loss(&eval_before.0.score, &eval_after.0.score, is_white);

            let missed_mate = self.check_missed_mate(&eval_before.0.score, &eval_after.0.score);

            let classification = MoveClassification::from_centipawn_loss(cp_loss, missed_mate);

            let analyzed_move = AnalyzedMove {
                move_san: move_str.to_string(),
                move_uci: move_str.to_string(),
                move_number,
                is_white,
                classification,
                evaluation_before: eval_before.0.score,
                evaluation_after: eval_after.0.score,
                centipawn_loss: cp_loss,
                best_move: eval_after.1.best,
                principal_variation: eval_after.0.pv,
            };

            analysis.add_move(analyzed_move);

            if !is_white {
                move_number += 1;
            }
        }

        analysis.finalize();
        Ok(analysis)
    }

    fn calculate_centipawn_loss(
        &self,
        eval_before: &Score,
        eval_after: &Score,
        is_white_move: bool,
    ) -> i32 {
        let cp_before = self.score_to_centipawns(eval_before, is_white_move);
        let cp_after = self.score_to_centipawns(eval_after, !is_white_move);

        // Calculate loss from moving player's perspective
        let loss = cp_before - cp_after;
        loss.max(0) // Only count losses, not gains
    }

    fn score_to_centipawns(&self, score: &Score, from_white_perspective: bool) -> i32 {
        match score {
            Score::Cp(cp) => {
                if from_white_perspective {
                    *cp
                } else {
                    -*cp
                }
            }
            Score::Mate(mate_in) => {
                let mate_score = if *mate_in > 0 {
                    10_000 - mate_in.abs()
                } else {
                    -10_000 + mate_in.abs()
                };

                if from_white_perspective {
                    mate_score
                } else {
                    -mate_score
                }
            }
        }
    }

    fn check_missed_mate(&self, player_eval: &Score, best_eval: &Score) -> bool {
        match (player_eval, best_eval) {
            (_, Score::Mate(mate_in)) if *mate_in > 0 && *mate_in <= 5 => {
                !matches!(player_eval, Score::Mate(m) if *m > 0 && *m <= *mate_in + 1)
            }
            _ => false,
        }
    }
}

// Helper function to format the analysis report
impl GameAnalysis {
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Chess Game Analysis Report\n\n");

        // Player statistics
        report.push_str("## Player Statistics\n\n");
        report.push_str(&format!("### White Player\n"));
        report.push_str(&format!(
            "- **Accuracy:** {:.1}%\n",
            self.white_stats.accuracy
        ));
        report.push_str(&format!(
            "- **Average Centipawn Loss:** {:.1}\n",
            self.white_stats.average_centipawn_loss
        ));
        report.push_str(&format!(
            "- **Best Moves:** {}\n",
            self.white_stats.best_moves
        ));
        report.push_str(&format!(
            "- **Inaccuracies:** {}\n",
            self.white_stats.inaccuracies
        ));
        report.push_str(&format!("- **Mistakes:** {}\n", self.white_stats.mistakes));
        report.push_str(&format!("- **Blunders:** {}\n", self.white_stats.blunders));

        report.push_str(&format!("\n### Black Player\n"));
        report.push_str(&format!(
            "- **Accuracy:** {:.1}%\n",
            self.black_stats.accuracy
        ));
        report.push_str(&format!(
            "- **Average Centipawn Loss:** {:.1}\n",
            self.black_stats.average_centipawn_loss
        ));
        report.push_str(&format!(
            "- **Best Moves:** {}\n",
            self.black_stats.best_moves
        ));
        report.push_str(&format!(
            "- **Inaccuracies:** {}\n",
            self.black_stats.inaccuracies
        ));
        report.push_str(&format!("- **Mistakes:** {}\n", self.black_stats.mistakes));
        report.push_str(&format!("- **Blunders:** {}\n", self.black_stats.blunders));

        // Move-by-move analysis
        report.push_str("\n## Move Analysis\n\n");

        let mut current_move = 1;
        let mut white_move = true;

        for analyzed_move in &self.moves {
            if white_move {
                report.push_str(&format!("{}. ", current_move));
            }

            report.push_str(&format!(
                "{} {} ({}cp loss) ",
                analyzed_move.move_san,
                analyzed_move.classification.symbol(),
                analyzed_move.centipawn_loss
            ));

            if analyzed_move.centipawn_loss > 50 {
                report.push_str(&format!("[Best: {}] ", analyzed_move.best_move));
            }

            if white_move {
                report.push(' ');
                white_move = false;
            } else {
                report.push('\n');
                white_move = true;
                current_move += 1;
            }
        }

        report
    }
}

use pgn_reader::{Reader, SanPlus};
use shakmaty::{fen::Fen, Chess, Color, Position};
use std::ops::ControlFlow;

#[derive(Debug)]
struct Move {
    san: String,
    uci: String,
    color: Color,
    from: String,
    to: String,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pgn = r#"[Event "Weekly SuperBlitz Arena"]
[Site "https://lichess.org/vX1JWvhr"]
[Date "2025.09.02"]
[White "admin112"]
[Black "Ehrenstein"]
[Result "1-0"]
[GameId "vX1JWvhr"]
[UTCDate "2025.09.02"]
[UTCTime "18:02:28"]
[WhiteElo "2072"]
[BlackElo "2243"]
[WhiteRatingDiff "+9"]
[BlackRatingDiff "-9"]
[Variant "Standard"]
[TimeControl "180+0"]
[ECO "E10"]
[Opening "Indian Defense: Anti-Nimzo-Indian"]
[Termination "Normal"]
[Annotator "lichess.org"]

1. d4 Nf6 2. c4 e6 3. Nf3 { E10 Indian Defense: Anti-Nimzo-Indian } d5 4. g3 c6 5. Qc2 Be7 6. Bg2 O-O 7. O-O b6 8. Nbd2 Ba6 9. b3 c5 10. cxd5 exd5 11. dxc5 Bxe2 12. Re1 Bxf3 13. Nxf3 Bxc5 14. a3 Nc6 15. b4 Nd4 16. Qd3 Nxf3+ 17. Bxf3 Be7 18. Bb2 Ne4 19. Bxe4 dxe4 20. Qxe4 Bf6 21. Rad1 Qe8 22. Bxf6 Qxe4 23. Rxe4 gxf6 24. Rd7 Rfd8 25. Ree7 Rxd7 26. Rxd7 a5 27. Rb7 axb4 28. axb4 b5 29. Rxb5 Ra1+ 30. Kg2 Rb1 31. Rb8+ Kg7 32. b5 f5 33. b6 Kf6 34. b7 Rb2 35. h4 h5 36. Rh8 Rxb7 37. Rxh5 Kg6 38. Rg5+ Kf6 39. Kh3 Rb3 40. Rg8 Rc3 41. Ra8 Rc4 42. Ra6+ Kg7 43. h5 f6 44. f3 Rb4 45. Ra5 f4 46. g4 Rb6 47. Rf5 Rb4 48. Kh4 Rc4 49. g5 fxg5+ 50. Kxg5 Rc3 51. Rxf4 Rc5+ 52. Rf5 Rc3 53. f4 Rc4 54. Rd5 Rc2 55. Rd7+ Kh8 56. f5 Rg2+ 57. Kh6 Rg8 58. Rh7# { White wins by checkmate. } 1-0"#;

    let mut reader = Reader::new(std::io::Cursor::new(&pgn));
    let mut extractor = Extractor::default();
    reader.read_game(&mut extractor)?;

    let mut engine = Engine::new("stockfish")?;
    let opts = [("Threads", "8"), ("UCI_ShowWDL", "true"), ("MultiPV", "1")];

    engine.uci().await?;
    engine.opts(&opts).await?;
    engine.isready().await?;

    let mut analyzer = GameAnalyzer::new(&mut engine, 20);

    // Example game moves (in UCI format or you can convert from SAN)

    println!("Starting game analysis...");

    let moves = extractor
        .moves
        .iter()
        .map(|m| m.uci.clone())
        .collect::<Vec<_>>();

    println!("{moves:?}");

    // Analyze the game
    let analysis = analyzer.analyze_game(&moves, None).await?;

    // Generate and print the report
    let report = analysis.generate_report();
    println!("{}", report);

    // You can also access individual statistics
    println!("\n=== Detailed Statistics ===");
    println!("White accuracy: {:.1}%", analysis.white_stats.accuracy);
    println!("Black accuracy: {:.1}%", analysis.black_stats.accuracy);

    println!("\nWhite move breakdown:");
    println!("  Best: {}", analysis.white_stats.best_moves);
    println!("  Excellent: {}", analysis.white_stats.excellent_moves);
    println!("  Good: {}", analysis.white_stats.good_moves);
    println!("  Inaccuracies: {}", analysis.white_stats.inaccuracies);
    println!("  Mistakes: {}", analysis.white_stats.mistakes);
    println!("  Blunders: {}", analysis.white_stats.blunders);

    println!("\nBlack move breakdown:");
    println!("  Best: {}", analysis.black_stats.best_moves);
    println!("  Excellent: {}", analysis.black_stats.excellent_moves);
    println!("  Good: {}", analysis.black_stats.good_moves);
    println!("  Inaccuracies: {}", analysis.black_stats.inaccuracies);
    println!("  Mistakes: {}", analysis.black_stats.mistakes);
    println!("  Blunders: {}", analysis.black_stats.blunders);

    // Print significant moves
    println!("\n=== Significant Moves ===");
    for analyzed_move in analysis.moves.iter() {
        if analyzed_move.centipawn_loss > 100 {
            let player = if analyzed_move.is_white {
                "White"
            } else {
                "Black"
            };
            println!(
                "Move {}: {} {} - {} ({}cp loss, best: {})",
                analyzed_move.move_number,
                player,
                analyzed_move.move_san,
                match analyzed_move.classification {
                    crate::MoveClassification::Mistake => "Mistake",
                    crate::MoveClassification::Blunder => "Blunder",
                    crate::MoveClassification::Missed => "Missed Mate",
                    _ => "Significant",
                },
                analyzed_move.centipawn_loss,
                analyzed_move.best_move
            );
        }
    }

    // let mut visitor = Visitor;
    // let job = Go::new().depth(10).moves(&["d2", "d4"]);
    // engine.go_with(job, &mut visitor).await?;
    Ok(())
}
