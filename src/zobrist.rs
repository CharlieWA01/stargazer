//! Fingerprints for positions ("Zobrist hashing", DESIGN.md section 3).
//!
//! Every possible fact about a position gets its own fixed random 64-bit
//! number: "white knight on g1", "black may castle kingside", "en passant
//! possible on the e-file", "Black to move". A position's fingerprint is
//! all of its facts' numbers combined with XOR.
//!
//! XOR undoes itself (x ^ n ^ n == x), so when a knight goes from g1 to
//! f3 the fingerprint is updated by XORing out "knight on g1" and XORing
//! in "knight on f3", instead of looking at the whole board again. Two
//! positions with the same pieces, side to move, castling and en passant
//! rights get the same fingerprint, however they were reached. Two
//! different positions get the same one only by accident, about once in
//! 2^64 comparisons.
//!
//! The numbers come from a small random-number generator run when
//! Stargazer is compiled, from a fixed seed, so every build has the same
//! numbers and `bench` stays repeatable.

use crate::board::{CastlingRights, Color, Piece, Square};

/// SplitMix64: a tiny, well-tested generator. Returns the next state and
/// the random number it gives.
const fn split_mix(state: u64) -> (u64, u64) {
    let state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (state, z ^ (z >> 31))
}

/// All the random numbers, in one place.
struct Keys {
    /// One per colour, piece kind and square, in that order (see `piece`).
    pieces: [u64; 768],
    /// For each of the 16 combinations of castling rights (see `castling`).
    castling: [u64; 16],
    /// The file of the en passant square, when there is one.
    en_passant_file: [u64; 8],
    black_to_move: u64,
}

const fn make_keys() -> Keys {
    // Any fixed number works as a seed; this one is "Stargazer" in ASCII.
    let mut state = 0x5374_6172_6761_7A65;
    let mut keys = Keys {
        pieces: [0; 768],
        castling: [0; 16],
        en_passant_file: [0; 8],
        black_to_move: 0,
    };
    let mut i = 0;
    while i < 768 {
        let (next, key) = split_mix(state);
        state = next;
        keys.pieces[i] = key;
        i += 1;
    }
    // One number per right; a combination of rights gets the XOR of its
    // rights' numbers, worked out here so it's one lookup during a search.
    let mut rights = [0; 4];
    i = 0;
    while i < 4 {
        let (next, key) = split_mix(state);
        state = next;
        rights[i] = key;
        i += 1;
    }
    let mut combination = 0;
    while combination < 16 {
        let mut right = 0;
        while right < 4 {
            if combination & (1 << right) != 0 {
                keys.castling[combination] ^= rights[right];
            }
            right += 1;
        }
        combination += 1;
    }
    let mut file = 0;
    while file < 8 {
        let (next, key) = split_mix(state);
        state = next;
        keys.en_passant_file[file] = key;
        file += 1;
    }
    keys.black_to_move = split_mix(state).1;
    keys
}

static KEYS: Keys = make_keys();

/// The number for this piece standing on this square.
/// (One flat list rather than nested ones: this is looked up several
/// times per move, and one index is quicker to check than three.)
pub fn piece(piece: Piece, square: Square) -> u64 {
    let index = (piece.color as usize * 6 + piece.kind as usize) * 64 + square.index();
    KEYS.pieces[index]
}

/// The numbers for whichever castling rights are still held, combined.
pub fn castling(rights: CastlingRights) -> u64 {
    KEYS.castling[rights.bits()]
}

/// The number for an en passant square (only its file matters: the rank
/// follows from whose move it is), or 0 when there's none.
pub fn en_passant(square: Option<Square>) -> u64 {
    square.map_or(0, |s| KEYS.en_passant_file[s.file() as usize])
}

/// The number for whose move it is (White's adds nothing).
pub fn side_to_move(color: Color) -> u64 {
    match color {
        Color::White => 0,
        Color::Black => KEYS.black_to_move,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_numbers_are_all_different_and_none_is_zero() {
        let mut all = KEYS.pieces.to_vec();
        // Combination 0 (no rights) adds nothing; the other 15 must differ.
        assert_eq!(KEYS.castling[0], 0);
        all.extend(&KEYS.castling[1..]);
        all.extend(KEYS.en_passant_file);
        all.push(KEYS.black_to_move);
        assert_eq!(all.len(), 768 + 15 + 8 + 1);
        assert!(all.iter().all(|&key| key != 0));
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), 792);
    }
}
