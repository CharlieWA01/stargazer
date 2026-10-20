//! Evaluation: how good is this position, without looking any further
//! ahead? (DESIGN.md section 4.)
//!
//! The answer is a score in centipawns (100 = one pawn), from the point of
//! view of the side to move: +150 means "I'm about a pawn and a half up".
//!
//! Two things count so far:
//! - **Material:** pawn 100, knight 320, bishop 330, rook 500, queen 900.
//! - **Where each piece stands** ("piece-square tables"): knights like the
//!   centre, pawns like to advance, the king hides in the middlegame and
//!   walks to the centre in the endgame.
//!
//! Every position gets two scores, a middlegame one and an endgame one, and
//! they're blended by how much material is left ("tapered evaluation").
//!
//! The square bonuses are our own: each comes from a simple chess rule
//! written below (for example "a knight loses points the nearer it is to
//! the edge"), not from someone else's tables. A tuner will refine the
//! numbers later (Milestone 4). Everything sits behind `evaluate`, so
//! the evaluation can grow or be replaced without touching the search.

use crate::board::{Color, Piece, PieceKind, Position, Square};

/// Material value of each piece kind, in centipawns, in the order of
/// `PieceKind::ALL` (pawn, knight, bishop, rook, queen, king). The king has
/// no material value: it can never be traded.
const MATERIAL: [i32; 6] = [100, 320, 330, 500, 900, 0];

/// How much each piece kind counts towards "the middlegame is still on".
/// With all pieces on the board the total is 24 (4 knights, 4 bishops,
/// 4 rooks x 2, 2 queens x 4); with none left it's 0, a pure endgame.
const PHASE_WEIGHT: [i32; 6] = [0, 1, 1, 2, 4, 0];
const FULL_PHASE: i32 = 24;

/// How far a square is from the centre, in rings: 0 for d4, e4, d5, e5;
/// 1 for the ring around them; 2 for the next; 3 for the edge of the board.
const fn centre_ring(file: i32, rank: i32) -> i32 {
    let file_distance = if file < 4 { 3 - file } else { file - 4 };
    let rank_distance = if rank < 4 { 3 - rank } else { rank - 4 };
    if file_distance > rank_distance {
        file_distance
    } else {
        rank_distance
    }
}

/// The bonus for a piece on a square, worked out from simple chess rules.
///
/// `rank` is counted from the piece's own side (0 = its back rank, 7 = the
/// far side), so the same rules work for both colours.
const fn square_bonus(kind: PieceKind, endgame: bool, file: i32, rank: i32) -> i32 {
    let ring = centre_ring(file, rank);
    let centre_file = file == 3 || file == 4;
    match kind {
        PieceKind::Pawn => {
            // Pawns gain value as they advance, much more in the endgame,
            // where they become queens.
            let advance = if endgame {
                [0, 0, 10, 20, 35, 60, 100, 0]
            } else {
                [0, 0, 5, 10, 20, 35, 60, 0]
            };
            let mut bonus = advance[rank as usize];
            if !endgame && centre_file {
                // In the middlegame, central pawns should leave home and
                // take the centre.
                if rank == 1 {
                    bonus -= 10;
                } else if rank == 3 || rank == 4 {
                    bonus += 10;
                }
            }
            bonus
        }
        // "A knight on the rim is dim": the further from the centre, the
        // fewer squares it reaches.
        PieceKind::Knight => [20, 10, -5, -25][ring as usize],
        // Bishops also prefer the centre, but suffer less on the edge.
        PieceKind::Bishop => [15, 10, 0, -10][ring as usize],
        PieceKind::Rook => {
            // Rooks love the seventh rank (attacking pawns, trapping the
            // king) and, in the middlegame, the central files.
            let mut bonus = if rank == 6 { 20 } else { 0 };
            if !endgame && centre_file {
                bonus += 5;
            }
            bonus
        }
        PieceKind::Queen => {
            if endgame {
                [10, 5, 0, -10][ring as usize]
            } else {
                [5, 5, 0, -5][ring as usize]
            }
        }
        PieceKind::King => {
            if endgame {
                // With few pieces left the king is a strong piece: bring
                // it to the centre.
                [30, 15, 0, -20][ring as usize]
            } else if rank == 0 {
                // Middlegame: safest tucked in a corner after castling,
                // on its back rank.
                if file <= 2 || file >= 6 { 20 } else { 0 }
            } else if rank == 1 {
                -10
            } else {
                // Out in the open with queens on the board: dangerous.
                let exposed = -20 * rank;
                if exposed < -60 { -60 } else { exposed }
            }
        }
    }
}

/// A 64-square table of bonuses for one piece kind and game phase, seen
/// from White's side (a1 = 0). Black uses the same table, flipped.
const fn table(kind: PieceKind, endgame: bool) -> [i32; 64] {
    let mut result = [0; 64];
    let mut square = 0;
    while square < 64 {
        result[square] = square_bonus(kind, endgame, (square % 8) as i32, (square / 8) as i32);
        square += 1;
    }
    result
}

/// All the tables, worked out when Stargazer is compiled:
/// `TABLES[kind][0]` is the middlegame table, `TABLES[kind][1]` the endgame one.
const TABLES: [[[i32; 64]; 2]; 6] = {
    let kinds = PieceKind::ALL;
    let mut result = [[[0; 64]; 2]; 6];
    let mut i = 0;
    while i < 6 {
        result[i] = [table(kinds[i], false), table(kinds[i], true)];
        i += 1;
    }
    result
};

/// Score the position for the side to move, in centipawns.
pub fn evaluate(position: &Position) -> i32 {
    // Totals from White's point of view: White's pieces add, Black's subtract.
    let mut middlegame = 0;
    let mut endgame = 0;
    let mut phase = 0;

    for color in Color::BOTH {
        let sign = match color {
            Color::White => 1,
            Color::Black => -1,
        };
        for (kind_index, kind) in PieceKind::ALL.into_iter().enumerate() {
            let tables = &TABLES[kind_index];
            for square in position.pieces(Piece { color, kind }).squares() {
                let seen_from_own_side = own_side_index(color, square);
                middlegame += sign * (MATERIAL[kind_index] + tables[0][seen_from_own_side]);
                endgame += sign * (MATERIAL[kind_index] + tables[1][seen_from_own_side]);
                phase += PHASE_WEIGHT[kind_index];
            }
        }
    }

    // Blend: all middlegame with a full board, all endgame with no pieces
    // left. (Phase can exceed 24 after promotions; cap it.)
    let phase = phase.min(FULL_PHASE);
    let blended = (middlegame * phase + endgame * (FULL_PHASE - phase)) / FULL_PHASE;

    match position.side_to_move {
        Color::White => blended,
        Color::Black => -blended,
    }
}

/// The square's index as seen from this colour's own side of the board:
/// unchanged for White, flipped top to bottom for Black (so a black pawn on
/// e7 reads like a white pawn on e2).
fn own_side_index(color: Color, square: Square) -> usize {
    match color {
        Color::White => square.index(),
        Color::Black => square.index() ^ 56,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(fen: &str) -> i32 {
        evaluate(&Position::from_fen(fen).unwrap())
    }

    /// The same position with the colours swapped and the board turned
    /// upside down: White's pieces become Black's and vice versa.
    fn mirror(fen: &str) -> String {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        let swap_case = |c: char| {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
            }
        };
        let board: Vec<String> = fields[0]
            .split('/')
            .rev()
            .map(|rank| rank.chars().map(swap_case).collect())
            .collect();
        let side = if fields[1] == "w" { "b" } else { "w" };
        let castling: String = if fields[2] == "-" {
            "-".into()
        } else {
            let mut swapped: Vec<char> = fields[2].chars().map(swap_case).collect();
            swapped.sort_by_key(|c| (c.is_ascii_lowercase(), *c != 'K' && *c != 'k'));
            swapped.into_iter().collect()
        };
        let en_passant = match fields[3] {
            "-" => "-".to_string(),
            square => {
                let file = &square[..1];
                let rank = if &square[1..] == "3" { "6" } else { "3" };
                format!("{file}{rank}")
            }
        };
        format!("{} {side} {castling} {en_passant} 0 1", board.join("/"))
    }

    #[test]
    fn the_start_position_is_level() {
        assert_eq!(eval(crate::board::START_FEN), 0);
    }

    #[test]
    fn mirrored_positions_score_the_same_for_the_side_to_move() {
        // If the evaluation treated the colours differently, a mirrored
        // position would score differently.
        for fen in [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        ] {
            assert_eq!(eval(fen), eval(&mirror(fen)), "{fen}");
        }
    }

    #[test]
    fn the_score_is_for_the_side_to_move() {
        // White is a queen up: good for White, bad for Black.
        let white_to_move = eval("4k3/8/8/8/8/8/8/3QK3 w - - 0 1");
        let black_to_move = eval("4k3/8/8/8/8/8/8/3QK3 b - - 0 1");
        assert!(white_to_move > 800);
        assert_eq!(black_to_move, -white_to_move);
    }

    #[test]
    fn material_counts_most() {
        // An extra knight outweighs any square bonus.
        let up_a_knight = eval("4k3/8/8/8/8/8/8/N3K3 w - - 0 1");
        assert!(up_a_knight > 250, "{up_a_knight}");
    }

    #[test]
    fn knights_prefer_the_centre() {
        let centre = eval("4k3/8/8/3N4/8/8/8/4K3 w - - 0 1");
        let corner = eval("4k3/8/8/8/8/8/8/N3K3 w - - 0 1");
        assert!(centre > corner + 30, "d5 {centre} vs a1 {corner}");
    }

    #[test]
    fn passed_pawns_grow_in_value_as_they_advance() {
        let on_e3 = eval("4k3/8/8/8/8/4P3/8/4K3 w - - 0 1");
        let on_e6 = eval("4k3/8/4P3/8/8/8/8/4K3 w - - 0 1");
        assert!(on_e6 > on_e3 + 40, "e6 {on_e6} vs e3 {on_e3}");
    }

    #[test]
    fn the_king_hides_in_the_middlegame_and_centralises_in_the_endgame() {
        // Middlegame (queens and rooks on): a castled king on g1 beats one on e3.
        let castled = eval("r2qk2r/8/8/8/8/8/8/R2Q1RK1 w - - 0 1");
        let exposed = eval("r2qk2r/8/8/8/8/4K3/8/R2Q1R2 w - - 0 1");
        assert!(castled > exposed, "g1 {castled} vs e3 {exposed}");
        // Endgame (kings and pawns only): a central king beats a corner one.
        let central = eval("4k3/p7/8/8/3K4/8/P7/8 w - - 0 1");
        let cornered = eval("4k3/p7/8/8/8/8/P7/7K w - - 0 1");
        assert!(central > cornered, "d4 {central} vs h1 {cornered}");
    }
}
