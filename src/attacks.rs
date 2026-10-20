//! Ready-made tables of where knights, kings and pawns attack from each
//! square (DESIGN.md section 2).
//!
//! These pieces always reach the same squares from a given square, whatever
//! else is on the board, so the answers are worked out once, when Stargazer
//! is compiled, and looked up after that. (Bishops, rooks and queens depend
//! on what's in their way, so they're handled separately, by walking rays.)

use crate::bitboard::Bitboard;
use crate::board::{Color, Square};

/// The 8 L-shaped jumps a knight can make, as (files, ranks) steps.
const KNIGHT_JUMPS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

/// The 8 one-square steps a king can make.
const KING_STEPS: [(i8, i8); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

/// From `square`, take each step that stays on the board and collect the
/// squares reached. Steps that would fall off an edge are skipped.
const fn reachable(square: usize, steps: &[(i8, i8)]) -> Bitboard {
    let file = (square % 8) as i8;
    let rank = (square / 8) as i8;
    let mut set = 0u64;
    let mut i = 0;
    while i < steps.len() {
        let to_file = file + steps[i].0;
        let to_rank = rank + steps[i].1;
        if to_file >= 0 && to_file < 8 && to_rank >= 0 && to_rank < 8 {
            set |= 1 << (to_rank * 8 + to_file);
        }
        i += 1;
    }
    Bitboard(set)
}

/// Build a 64-entry table: for each square, the squares these steps reach.
const fn table(steps: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut result = [Bitboard::EMPTY; 64];
    let mut square = 0;
    while square < 64 {
        result[square] = reachable(square, steps);
        square += 1;
    }
    result
}

const KNIGHT_TABLE: [Bitboard; 64] = table(&KNIGHT_JUMPS);
const KING_TABLE: [Bitboard; 64] = table(&KING_STEPS);

/// Pawns capture one square diagonally forward: up the board for White,
/// down the board for Black.
const WHITE_PAWN_TABLE: [Bitboard; 64] = table(&[(-1, 1), (1, 1)]);
const BLACK_PAWN_TABLE: [Bitboard; 64] = table(&[(-1, -1), (1, -1)]);

/// Every square a knight on `square` can jump to (ignoring what's there).
pub fn knight_attacks(square: Square) -> Bitboard {
    KNIGHT_TABLE[square.index()]
}

/// Every square a king on `square` can step to (ignoring what's there).
pub fn king_attacks(square: Square) -> Bitboard {
    KING_TABLE[square.index()]
}

/// The two squares (one on the edge files) a pawn of this colour on
/// `square` attacks diagonally.
pub fn pawn_attacks(color: Color, square: Square) -> Bitboard {
    match color {
        Color::White => WHITE_PAWN_TABLE[square.index()],
        Color::Black => BLACK_PAWN_TABLE[square.index()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(name: &str) -> Square {
        Square::parse(name).unwrap()
    }

    fn names(set: Bitboard) -> Vec<String> {
        set.squares().map(|s| s.to_string()).collect()
    }

    #[test]
    fn knight_in_the_centre_has_eight_jumps() {
        assert_eq!(
            names(knight_attacks(sq("d4"))),
            ["c2", "e2", "b3", "f3", "b5", "f5", "c6", "e6"]
        );
    }

    #[test]
    fn knight_in_the_corner_has_two_jumps() {
        assert_eq!(names(knight_attacks(sq("a1"))), ["c2", "b3"]);
        assert_eq!(names(knight_attacks(sq("h8"))), ["g6", "f7"]);
    }

    #[test]
    fn knights_never_wrap_around_the_board_edge() {
        // From the h-file, a "jump right" would land on the a-file if the
        // edge weren't checked.
        let from_h = knight_attacks(sq("h4"));
        assert_eq!(names(from_h), ["g2", "f3", "f5", "g6"]);
    }

    #[test]
    fn king_steps() {
        assert_eq!(king_attacks(sq("e4")).count(), 8);
        assert_eq!(names(king_attacks(sq("a1"))), ["b1", "a2", "b2"]);
        assert_eq!(king_attacks(sq("h5")).count(), 5);
    }

    #[test]
    fn pawns_attack_diagonally_forward() {
        assert_eq!(names(pawn_attacks(Color::White, sq("e4"))), ["d5", "f5"]);
        assert_eq!(names(pawn_attacks(Color::Black, sq("e5"))), ["d4", "f4"]);
        // Edge pawns attack only one square.
        assert_eq!(names(pawn_attacks(Color::White, sq("a2"))), ["b3"]);
        assert_eq!(names(pawn_attacks(Color::Black, sq("h7"))), ["g6"]);
    }
}
