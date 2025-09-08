#![allow(static_mut_refs)]

use std::time::Instant;

use shakmaty::{fen::Fen, san::San, Chess, EnPassantMode, Position};

const TSVS: [&str; 5] = [
    include_str!("../../openings/a.tsv"),
    include_str!("../../openings/b.tsv"),
    include_str!("../../openings/c.tsv"),
    include_str!("../../openings/d.tsv"),
    include_str!("../../openings/e.tsv"),
];

static mut OPENINGS: Vec<Opening> = Vec::new();

#[derive(Debug, serde::Serialize)]
pub struct Opening {
    eco: &'static str,
    name: &'static str,
    pgn: &'static str,
    fen: String,
}

impl Opening {
    pub fn from_tsv(line: &'static str) -> Self {
        let mut chess = Chess::new();
        let mut parts = line.split('\t');
        let eco = parts.next().unwrap();
        let name = parts.next().unwrap();
        let pgn = parts.next().unwrap();
        pgn.split_whitespace()
            .filter_map(|m| m.parse::<San>().ok())
            .for_each(|san| chess.play_unchecked(san.to_move(&chess).unwrap()));
        let fen = Fen::from_position(&chess, EnPassantMode::Legal).to_string();
        Self {
            eco,
            name,
            pgn,
            fen,
        }
    }
}

pub fn gather_openings() {
    let start = Instant::now();
    for tsv in TSVS {
        for line in tsv.lines().skip(1) {
            unsafe { OPENINGS.push(Opening::from_tsv(line)) };
        }
    }
    tracing::trace!(
        "gathered {} openings in {:?}",
        unsafe { OPENINGS.len() },
        start.elapsed()
    );
}

#[tauri::command]
pub fn find_opening(fen: &str) -> Option<&Opening> {
    unsafe { OPENINGS.iter().find(|o| o.fen == fen) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic() {
        gather_openings();

        let anti_nimzo = "rnbqkb1r/pppp1ppp/4pn2/8/2PP4/5N2/PP2PPPP/RNBQKB1R b KQkq - 1 3";
        assert_eq!(
            Some("Indian Defense: Anti-Nimzo-Indian"),
            find_opening(anti_nimzo).map(|o| o.name)
        );

        let catalan = "rnbqkb1r/pppp1ppp/4pn2/8/2PP4/6P1/PP2PP1P/RNBQKBNR b KQkq - 0 3";
        assert_eq!(
            Some("Catalan Opening"),
            find_opening(catalan).map(|o| o.name)
        );

        let open_catalan = "rnbqkb1r/ppp2ppp/4pn2/8/2pP4/5NP1/PP2PPBP/RNBQK2R b KQkq - 1 5";
        assert_eq!(
            Some("Catalan Opening: Open Defense"),
            find_opening(open_catalan).map(|o| o.name)
        );
    }
}
