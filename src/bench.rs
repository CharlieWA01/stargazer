//! `bench`: search a fixed set of positions to a fixed depth and count the
//! positions looked at (DESIGN.md section 5).
//!
//! The search has no randomness and, at a fixed depth, no clock, so the
//! same code always gives the same count. That makes the count a
//! fingerprint: two builds with the same count search identically, and a
//! change that should only make the engine faster (not play differently)
//! must leave it unchanged. The speed (positions per second) shows whether
//! a change made things faster or slower.

use std::time::Instant;

use crate::board::Position;
use crate::search::{History, search};
use crate::tt::{self, Table};

/// How deep each position is searched, in half-moves.
pub const BENCH_DEPTH: u32 = 5;

/// A spread of positions: openings, middlegames full of tactics, endgames.
/// The first six are the standard perft positions.
const POSITIONS: [&str; 10] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
    "2r2rk1/pp1bqppp/2n1pn2/3p4/2PP4/2N1PN2/PPQ2PPP/R1B2RK1 w - - 0 12",
    "8/5pk1/6p1/3P4/1p3P2/1P4P1/5K2/8 w - - 0 40",
    "6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 30",
];

/// The result of a bench run.
pub struct Bench {
    pub nodes: u64,
    pub millis: u64,
}

impl Bench {
    pub fn nodes_per_second(&self) -> u64 {
        self.nodes * 1000 / self.millis.max(1)
    }
}

/// Search every bench position to `depth` and add up the positions looked at.
pub fn run(depth: u32) -> Bench {
    let started = Instant::now();
    let mut nodes = 0;
    // One table of the default size, emptied before each position, and a
    // fresh history for each, so each count depends only on that position.
    let mut table = Table::new(tt::DEFAULT_MB);
    for fen in POSITIONS {
        let position = Position::from_fen(fen).expect("bench positions are valid");
        table.clear();
        let mut last = 0;
        search(
            &[position],
            depth,
            &mut table,
            &mut History::new(),
            &mut || false,
            &mut |report| {
                last = report.nodes;
                true
            },
        );
        nodes += last;
    }
    Bench {
        nodes,
        millis: started.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_is_the_same_every_time() {
        // Depth 2 keeps the test quick; determinism doesn't depend on depth.
        let first = run(2).nodes;
        assert!(first > 0);
        assert_eq!(run(2).nodes, first);
    }

    #[test]
    fn deeper_searches_look_at_more_positions() {
        assert!(run(3).nodes > run(2).nodes);
    }
}
