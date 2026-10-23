//! Perft ("performance test"): count every possible sequence of moves to a
//! fixed depth. The correct totals for standard positions are published,
//! so a wrong count means a bug in the rules somewhere: a missed en passant,
//! an illegal castle, a wrong promotion. This is how move generation is
//! proven correct (DESIGN.md section 2).

use crate::board::Position;
use crate::movegen::{Move, legal_moves};

/// How many move sequences of exactly `depth` moves there are from here.
pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = legal_moves(position);
    if depth == 1 {
        return moves.len() as u64;
    }
    moves
        .iter()
        .map(|&mv| perft(&position.make_move(mv), depth - 1))
        .sum()
}

/// The perft count after each first move. When a total is wrong, comparing
/// this list with a trusted engine's shows which move goes wrong; then
/// repeat from that move one level deeper.
pub fn divide(position: &Position, depth: u32) -> Vec<(Move, u64)> {
    legal_moves(position)
        .into_iter()
        .map(|mv| {
            let count = perft(&position.make_move(mv), depth.saturating_sub(1));
            (mv, count)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The six standard test positions from the Chess Programming Wiki
    /// (https://www.chessprogramming.org/Perft_Results), each with its
    /// published counts at depths 1, 2, 3, ...
    const POSITIONS: [(&str, &str, &[u64]); 6] = [
        (
            "start position",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &[20, 400, 8_902, 197_281, 4_865_609, 119_060_324],
        ),
        (
            "Kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            &[48, 2_039, 97_862, 4_085_603, 193_690_690],
        ),
        (
            "position 3",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            &[14, 191, 2_812, 43_238, 674_624, 11_030_083],
        ),
        (
            "position 4",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[6, 264, 9_467, 422_333, 15_833_292],
        ),
        (
            "position 5",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            &[44, 1_486, 62_379, 2_103_487, 89_941_194],
        ),
        (
            "position 6",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            &[46, 2_079, 89_890, 3_894_594, 164_075_551],
        ),
    ];

    /// Check every position against its published counts, up to `max_depth`
    /// (or as deep as the counts go).
    fn check_up_to(max_depth: usize) {
        for (name, fen, expected) in POSITIONS {
            let position = Position::from_fen(fen).unwrap();
            for (i, &count) in expected.iter().enumerate().take(max_depth) {
                let depth = i as u32 + 1;
                assert_eq!(perft(&position, depth), count, "{name}, depth {depth}");
            }
        }
    }

    /// Runs on every PR: depths 1 to 4 of all six positions.
    #[test]
    fn perft_shallow() {
        check_up_to(4);
    }

    /// The deep run, before releases: depths up to 5 (and 6 where listed).
    /// Slow, so it only runs when asked:
    /// `cargo test --release -- --ignored`
    #[test]
    #[ignore]
    fn perft_deep() {
        check_up_to(6);
    }

    /// Every position reachable in `depth` moves keeps its fingerprint in
    /// step with its pieces and rights (`is_consistent` recomputes it).
    /// Returns how many positions were checked.
    fn check_fingerprints(position: &Position, depth: u32) -> u64 {
        assert!(position.is_consistent(), "{}", position.to_fen());
        if depth == 0 {
            return 1;
        }
        legal_moves(position)
            .into_iter()
            .map(|mv| check_fingerprints(&position.make_move(mv), depth - 1))
            .sum::<u64>()
            + 1
    }

    /// The step-by-step fingerprint equals one worked out from scratch,
    /// through every capture, castle, en passant and promotion in the six
    /// positions' trees, three moves deep.
    #[test]
    fn fingerprints_stay_right_after_every_move() {
        let checked: u64 = POSITIONS
            .iter()
            .map(|(_, fen, _)| check_fingerprints(&Position::from_fen(fen).unwrap(), 3))
            .sum();
        assert!(checked > 200_000, "{checked}");
    }

    #[test]
    fn divide_adds_up_to_perft() {
        let position = Position::startpos();
        let split = divide(&position, 3);
        assert_eq!(split.len(), 20);
        assert_eq!(split.iter().map(|(_, n)| n).sum::<u64>(), 8_902);
    }
}
