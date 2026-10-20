//! Ready-made tables of where knights, kings and pawns attack from each
//! square (DESIGN.md section 2).
//!
//! These pieces always reach the same squares from a given square, whatever
//! else is on the board, so the answers are worked out once, when Stargazer
//! is compiled, and looked up after that.
//!
//! Bishops, rooks and queens depend on what's in their way, so their
//! attacks are found by "walking rays": step along each direction until
//! the edge of the board or the first piece, which is attacked too (it may
//! be an enemy to capture). The rest of the engine only calls
//! `bishop_attacks` and `rook_attacks`, so a faster method (magic
//! bitboards) could replace the inside later without touching anything else.

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

/// The 4 diagonal directions a bishop slides in.
const BISHOP_DIRECTIONS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

/// The 4 straight directions a rook slides in.
const ROOK_DIRECTIONS: [(i8, i8); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

/// From `square`, slide along each direction, one square at a time, until
/// leaving the board or reaching a square in `blockers` (included: a piece
/// standing there is attacked).
fn slide(square: Square, directions: &[(i8, i8)], blockers: Bitboard) -> Bitboard {
    let mut attacked = Bitboard::EMPTY;
    for &(file_step, rank_step) in directions {
        let mut file = square.file() as i8;
        let mut rank = square.rank() as i8;
        loop {
            file += file_step;
            rank += rank_step;
            if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                break;
            }
            let reached = Square::new(file as u8, rank as u8);
            attacked.insert(reached);
            if blockers.contains(reached) {
                break;
            }
        }
    }
    attacked
}

/// Every square a bishop on `square` attacks, given the pieces in the way.
pub fn bishop_attacks(square: Square, blockers: Bitboard) -> Bitboard {
    slide(square, &BISHOP_DIRECTIONS, blockers)
}

/// Every square a rook on `square` attacks, given the pieces in the way.
pub fn rook_attacks(square: Square, blockers: Bitboard) -> Bitboard {
    slide(square, &ROOK_DIRECTIONS, blockers)
}

/// A queen moves like a rook and a bishop combined.
pub fn queen_attacks(square: Square, blockers: Bitboard) -> Bitboard {
    bishop_attacks(square, blockers) | rook_attacks(square, blockers)
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

    fn set_of(names: &[&str]) -> Bitboard {
        let mut set = Bitboard::EMPTY;
        for name in names {
            set.insert(sq(name));
        }
        set
    }

    #[test]
    fn rook_on_an_empty_board_sees_its_whole_file_and_rank() {
        assert_eq!(rook_attacks(sq("d4"), Bitboard::EMPTY).count(), 14);
        assert_eq!(rook_attacks(sq("a1"), Bitboard::EMPTY).count(), 14);
    }

    #[test]
    fn bishop_on_an_empty_board() {
        assert_eq!(bishop_attacks(sq("d4"), Bitboard::EMPTY).count(), 13);
        assert_eq!(bishop_attacks(sq("a1"), Bitboard::EMPTY).count(), 7);
    }

    #[test]
    fn sliders_stop_at_the_first_piece_and_include_it() {
        // Rook on d4, pieces on d6 and f4: it sees d5 and d6 but not d7;
        // e4 and f4 but not g4.
        let blockers = set_of(&["d6", "f4"]);
        let attacks = rook_attacks(sq("d4"), blockers);
        assert!(attacks.contains(sq("d5")) && attacks.contains(sq("d6")));
        assert!(!attacks.contains(sq("d7")));
        assert!(attacks.contains(sq("f4")) && !attacks.contains(sq("g4")));
        // Down 3 (d3-d1), up 2 (d5, d6), left 3 (c4-a4), right 2 (e4, f4).
        assert_eq!(attacks.count(), 3 + 2 + 3 + 2);

        let attacks = bishop_attacks(sq("c1"), set_of(&["e3"]));
        assert_eq!(names(attacks), ["b2", "d2", "a3", "e3"]);
    }

    #[test]
    fn queen_is_rook_plus_bishop() {
        assert_eq!(queen_attacks(sq("d4"), Bitboard::EMPTY).count(), 27);
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
