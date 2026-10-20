//! Move generation: given a position, list the moves the side to move can
//! make (DESIGN.md section 2).
//!
//! It works in two stages:
//! 1. `pseudo_legal_moves`: every move each piece can make by its own
//!    rules, without asking whether it leaves our own king in check.
//! 2. `legal_moves`: plays each of those on a copy of the board and keeps
//!    only the ones after which our king is not attacked.

use std::fmt;

use crate::attacks::{
    bishop_attacks, king_attacks, knight_attacks, pawn_attacks, queen_attacks, rook_attacks,
};
use crate::bitboard::Bitboard;
use crate::board::{Color, Piece, PieceKind, Position, Square};

/// A move, written the way UCI writes it: from-square, to-square, and the
/// piece a pawn becomes if it promotes (`e2e4`, `e7e8q`).
///
/// Special moves don't need extra labels: a king moving two squares is
/// castling, a pawn moving diagonally onto the en passant square is an en
/// passant capture, and a pawn moving two squares is a double push.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl Move {
    pub const fn new(from: Square, to: Square) -> Move {
        Move {
            from,
            to,
            promotion: None,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(kind) = self.promotion {
            let letter = Piece::new(Color::Black, kind).to_char();
            write!(f, "{letter}")?;
        }
        Ok(())
    }
}

/// The pieces a pawn can become, best first.
const PROMOTION_CHOICES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];

/// Every legal move for the side to move: the pseudo-legal moves, minus
/// any that would leave our own king in check.
pub fn legal_moves(position: &Position) -> Vec<Move> {
    let us = position.side_to_move;
    pseudo_legal_moves(position)
        .into_iter()
        .filter(|&mv| !position.make_move(mv).in_check(us))
        .collect()
}

/// Every pseudo-legal move for the side to move.
pub fn pseudo_legal_moves(position: &Position) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);
    knight_moves(position, &mut moves);
    king_moves(position, &mut moves);
    castling_moves(position, &mut moves);
    pawn_moves(position, &mut moves);
    slider_moves(position, PieceKind::Bishop, bishop_attacks, &mut moves);
    slider_moves(position, PieceKind::Rook, rook_attacks, &mut moves);
    slider_moves(position, PieceKind::Queen, queen_attacks, &mut moves);
    moves
}

/// Squares a piece of the side to move may land on: empty, or holding an
/// enemy piece (a capture). Never a square with one of its own pieces.
fn landing_squares(position: &Position) -> Bitboard {
    !position.occupied_by(position.side_to_move)
}

/// Knights jump to any square in their table that isn't one of our own pieces.
fn knight_moves(position: &Position, moves: &mut Vec<Move>) {
    let us = position.side_to_move;
    for from in position.pieces(Piece::new(us, PieceKind::Knight)).squares() {
        for to in (knight_attacks(from) & landing_squares(position)).squares() {
            moves.push(Move::new(from, to));
        }
    }
}

/// The king steps one square in any direction, onto an empty square or a
/// capture. (Castling is separate, below.)
fn king_moves(position: &Position, moves: &mut Vec<Move>) {
    let us = position.side_to_move;
    for from in position.pieces(Piece::new(us, PieceKind::King)).squares() {
        for to in (king_attacks(from) & landing_squares(position)).squares() {
            moves.push(Move::new(from, to));
        }
    }
}

/// Bishops, rooks and queens: every square along their lines up to the
/// first piece, which they may capture if it's an enemy.
fn slider_moves(
    position: &Position,
    kind: PieceKind,
    attacks: fn(Square, Bitboard) -> Bitboard,
    moves: &mut Vec<Move>,
) {
    let us = position.side_to_move;
    let blockers = position.occupied();
    for from in position.pieces(Piece::new(us, kind)).squares() {
        for to in (attacks(from, blockers) & landing_squares(position)).squares() {
            moves.push(Move::new(from, to));
        }
    }
}

/// Castling, written as the king moving two squares towards its rook.
/// Allowed when:
/// - the right hasn't been lost (king and that rook haven't moved);
/// - every square between king and rook is empty;
/// - the king isn't in check, and doesn't pass through an attacked square.
///
/// Landing in check is caught by the general legality test, like any move.
fn castling_moves(position: &Position, moves: &mut Vec<Move>) {
    let us = position.side_to_move;
    let them = us.opponent();
    let rank = match us {
        Color::White => 0,
        Color::Black => 7,
    };
    let (kingside, queenside) = match us {
        Color::White => (
            position.castling.white_kingside,
            position.castling.white_queenside,
        ),
        Color::Black => (
            position.castling.black_kingside,
            position.castling.black_queenside,
        ),
    };
    let king = Square::new(4, rank);
    let empty = |files: &[u8]| {
        files
            .iter()
            .all(|&file| position.piece_on(Square::new(file, rank)).is_none())
    };
    // A FEN can claim rights that the pieces don't match; trust the pieces.
    let has =
        |file: u8, kind| position.piece_on(Square::new(file, rank)) == Some(Piece::new(us, kind));
    let kingside = kingside && has(7, PieceKind::Rook);
    let queenside = queenside && has(0, PieceKind::Rook);
    if (!kingside && !queenside) || !has(4, PieceKind::King) || position.is_attacked(king, them) {
        return;
    }

    // Kingside: f and g empty; the king crosses f.
    if kingside && empty(&[5, 6]) && !position.is_attacked(Square::new(5, rank), them) {
        moves.push(Move::new(king, Square::new(6, rank)));
    }
    // Queenside: b, c and d empty; the king crosses d. (b only needs to be
    // empty, for the rook to pass; it may be attacked.)
    if queenside && empty(&[1, 2, 3]) && !position.is_attacked(Square::new(3, rank), them) {
        moves.push(Move::new(king, Square::new(2, rank)));
    }
}

/// Pawns: one square forward onto an empty square, two from their starting
/// rank if both squares are empty, diagonal captures, en passant, and
/// promotion on reaching the last rank.
fn pawn_moves(position: &Position, moves: &mut Vec<Move>) {
    let us = position.side_to_move;
    let enemies = position.occupied_by(us.opponent());
    let empty = !position.occupied();

    // Which way is "forward", which rank pawns start on, and which rank
    // they promote on.
    let (forward, start_rank, last_rank): (i8, u8, u8) = match us {
        Color::White => (1, 1, 7),
        Color::Black => (-1, 6, 0),
    };

    for from in position.pieces(Piece::new(us, PieceKind::Pawn)).squares() {
        // Pushes: straight ahead, only onto empty squares.
        let one_ahead = Square::new(from.file(), (from.rank() as i8 + forward) as u8);
        if empty.contains(one_ahead) {
            add_pawn_move(from, one_ahead, last_rank, moves);
            if from.rank() == start_rank {
                let two_ahead = Square::new(from.file(), (from.rank() as i8 + 2 * forward) as u8);
                if empty.contains(two_ahead) {
                    moves.push(Move::new(from, two_ahead));
                }
            }
        }

        // Captures: diagonally forward onto an enemy piece.
        for to in (pawn_attacks(us, from) & enemies).squares() {
            add_pawn_move(from, to, last_rank, moves);
        }

        // En passant: capture onto the square an enemy pawn just skipped over.
        if let Some(target) = position.en_passant
            && pawn_attacks(us, from).contains(target)
        {
            moves.push(Move::new(from, target));
        }
    }
}

/// Add a pawn move, or all four promotions if it reaches the last rank.
fn add_pawn_move(from: Square, to: Square, last_rank: u8, moves: &mut Vec<Move>) {
    if to.rank() == last_rank {
        for kind in PROMOTION_CHOICES {
            moves.push(Move {
                from,
                to,
                promotion: Some(kind),
            });
        }
    } else {
        moves.push(Move::new(from, to));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The moves in a position, as sorted UCI strings, for easy comparison.
    fn moves_in(fen: &str) -> Vec<String> {
        let position = Position::from_fen(fen).unwrap();
        let mut names: Vec<String> = pseudo_legal_moves(&position)
            .iter()
            .map(Move::to_string)
            .collect();
        names.sort();
        names
    }

    /// Only the moves starting from one square.
    fn moves_from(fen: &str, square: &str) -> Vec<String> {
        moves_in(fen)
            .into_iter()
            .filter(|m| m.starts_with(square))
            .collect()
    }

    #[test]
    fn twenty_moves_from_the_start() {
        // 16 pawn moves (each pawn one or two squares) and 4 knight moves.
        // Bishops, rooks and queens are all blocked in at the start.
        let moves = moves_in(crate::board::START_FEN);
        assert_eq!(moves.len(), 20);
        assert!(moves.contains(&"e2e4".to_string()));
        assert!(moves.contains(&"g1f3".to_string()));
        assert!(!moves.contains(&"g1e2".to_string())); // own pawn on e2
    }

    #[test]
    fn black_pawns_move_down_the_board() {
        let fen = "4k3/4p3/8/8/8/8/8/4K3 b - - 0 1";
        assert_eq!(moves_from(fen, "e7"), ["e7e5", "e7e6"]);
    }

    #[test]
    fn a_blocked_pawn_cannot_push() {
        // A black knight on e3 stands right in front of the e2 pawn.
        assert!(moves_from("4k3/8/8/8/8/4n3/4P3/4K3 w - - 0 1", "e2").is_empty());
        // A piece on e4 blocks only the double push.
        assert_eq!(
            moves_from("4k3/8/8/8/4n3/8/4P3/4K3 w - - 0 1", "e2"),
            ["e2e3"]
        );
    }

    #[test]
    fn pawns_capture_diagonally_but_not_straight_ahead() {
        // Black pieces on d5 (capturable), e5 (blocks the push) and f5 (capturable).
        let fen = "4k3/8/8/3nbn2/4P3/8/8/4K3 w - - 0 1";
        assert_eq!(moves_from(fen, "e4"), ["e4d5", "e4f5"]);
    }

    #[test]
    fn promotion_offers_four_pieces_for_each_target() {
        // The e7 pawn can push to e8 or capture the rook on d8.
        let fen = "3r4/4P3/8/8/8/8/8/k3K3 w - - 0 1";
        assert_eq!(
            moves_from(fen, "e7"),
            [
                "e7d8b", "e7d8n", "e7d8q", "e7d8r", "e7e8b", "e7e8n", "e7e8q", "e7e8r"
            ]
        );
    }

    #[test]
    fn black_promotes_on_rank_1() {
        let fen = "4k3/8/8/8/8/8/p7/4K3 b - - 0 1";
        assert_eq!(moves_from(fen, "a2"), ["a2a1b", "a2a1n", "a2a1q", "a2a1r"]);
    }

    #[test]
    fn en_passant_capture() {
        // Black just played d7-d5, skipping d6; the e5 pawn can capture onto d6.
        let fen = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";
        assert_eq!(moves_from(fen, "e5"), ["e5d6", "e5e6"]);
        // Without the en passant square, only the push remains.
        let fen = "4k3/8/8/3pP3/8/8/8/4K3 w - - 0 1";
        assert_eq!(moves_from(fen, "e5"), ["e5e6"]);
    }

    #[test]
    fn knights_capture_enemies_but_not_friends() {
        // White knight on d4; a black pawn on e6, a white pawn on c6.
        let fen = "4k3/8/2P1p3/8/3N4/8/8/4K3 w - - 0 1";
        let moves = moves_from(fen, "d4");
        assert_eq!(moves.len(), 7);
        assert!(moves.contains(&"d4e6".to_string()));
        assert!(!moves.contains(&"d4c6".to_string()));
    }

    #[test]
    fn king_steps_but_not_onto_its_own_pieces() {
        // King e1, own pawn e2: d1, f1, d2, f2.
        let fen = "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1";
        assert_eq!(moves_from(fen, "e1"), ["e1d1", "e1d2", "e1f1", "e1f2"]);
    }

    /// Only the legal moves starting from one square.
    fn legal_from(fen: &str, square: &str) -> Vec<String> {
        let position = Position::from_fen(fen).unwrap();
        let mut names: Vec<String> = legal_moves(&position)
            .iter()
            .map(Move::to_string)
            .filter(|m| m.starts_with(square))
            .collect();
        names.sort();
        names
    }

    #[test]
    fn castling_both_ways_when_the_path_is_clear() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let king = legal_from(fen, "e1");
        assert!(king.contains(&"e1g1".to_string()));
        assert!(king.contains(&"e1c1".to_string()));
    }

    #[test]
    fn no_castling_without_the_right_or_through_pieces() {
        // Rights lost (no K/Q in the FEN).
        let king = legal_from("r3k2r/8/8/8/8/8/8/R3K2R w kq - 0 1", "e1");
        assert!(!king.contains(&"e1g1".to_string()));
        // A knight on b1 blocks queenside castling even though the king
        // never crosses b1.
        let king = legal_from("r3k2r/8/8/8/8/8/8/RN2K2R w KQkq - 0 1", "e1");
        assert!(!king.contains(&"e1c1".to_string()));
        assert!(king.contains(&"e1g1".to_string()));
    }

    #[test]
    fn no_castling_out_of_through_or_into_check() {
        // A black rook on e8 gives check: no castling at all.
        let king = legal_from("4r1k1/8/8/8/8/8/8/R3K2R w KQ - 0 1", "e1");
        assert!(!king.iter().any(|m| m == "e1g1" || m == "e1c1"));
        // A rook on f8 attacks f1, which the king crosses going kingside.
        let king = legal_from("5rk1/8/8/8/8/8/8/R3K2R w KQ - 0 1", "e1");
        assert!(!king.contains(&"e1g1".to_string()));
        assert!(king.contains(&"e1c1".to_string()));
        // A rook on g8 attacks g1, where the king would land.
        let king = legal_from("6rk/8/8/8/8/8/8/R3K2R w KQ - 0 1", "e1");
        assert!(!king.contains(&"e1g1".to_string()));
    }

    #[test]
    fn a_pinned_piece_cannot_leave_the_line() {
        // The e2 knight stands between its king on e1 and a rook on e8.
        assert!(legal_from("4r1k1/8/8/8/8/8/4N3/4K3 w - - 0 1", "e2").is_empty());
    }

    #[test]
    fn in_check_only_escapes_are_legal() {
        // White king on e1, checked by a rook on e8; a white rook on a2 can
        // block on e2. Legal: king steps off the file, or block.
        let position = Position::from_fen("4r1k1/8/8/8/8/8/R7/4K3 w - - 0 1").unwrap();
        let mut moves: Vec<String> = legal_moves(&position).iter().map(Move::to_string).collect();
        moves.sort();
        assert_eq!(moves, ["a2e2", "e1d1", "e1d2", "e1f1", "e1f2"]);
    }

    #[test]
    fn checkmate_has_no_legal_moves() {
        // Fool's mate: 1. f3 e5 2. g4 Qh4#.
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        let position = Position::from_fen(fen).unwrap();
        assert!(position.in_check(Color::White));
        assert!(legal_moves(&position).is_empty());
    }

    #[test]
    fn moves_print_in_uci_form() {
        let e2 = Square::parse("e2").unwrap();
        let e4 = Square::parse("e4").unwrap();
        assert_eq!(Move::new(e2, e4).to_string(), "e2e4");
        let promotion = Move {
            from: Square::parse("e7").unwrap(),
            to: Square::parse("e8").unwrap(),
            promotion: Some(PieceKind::Knight),
        };
        assert_eq!(promotion.to_string(), "e7e8n");
    }
}
