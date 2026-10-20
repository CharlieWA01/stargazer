//! Bitboards: a set of squares packed into one 64-bit number.
//!
//! A chessboard has 64 squares and a `u64` has 64 bits, so bit 0 stands for
//! a1, bit 1 for b1, and so on up to bit 63 for h8. "Where are all of
//! White's pawns?" is then a single number, and questions about many pieces
//! at once become one quick operation.
//!
//! This is the one place where the low-level bit tricks live. Everything
//! else in the engine uses the chess-named helpers below.

use std::fmt;
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

use crate::board::Square;

/// A set of squares. Bit N is set when square N is in the set.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// No squares at all.
    pub const EMPTY: Bitboard = Bitboard(0);

    /// A set holding just this one square.
    pub const fn from_square(square: Square) -> Bitboard {
        Bitboard(1 << square.index())
    }

    /// Is this square in the set?
    pub const fn contains(self, square: Square) -> bool {
        self.0 & (1 << square.index()) != 0
    }

    /// Add a square to the set.
    pub fn insert(&mut self, square: Square) {
        self.0 |= 1 << square.index();
    }

    /// Take a square out of the set.
    pub fn remove(&mut self, square: Square) {
        self.0 &= !(1 << square.index());
    }

    /// How many squares are in the set (for example, how many pawns a side has).
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The squares in the set, from a1 towards h8.
    pub fn squares(self) -> Squares {
        Squares(self.0)
    }
}

/// Walks through the squares of a bitboard, lowest first.
pub struct Squares(u64);

impl Iterator for Squares {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }
        // The lowest set bit is the next square; then clear it.
        let index = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(Square::from_index(index))
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    fn bitand(self, other: Bitboard) -> Bitboard {
        Bitboard(self.0 & other.0)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    fn bitor(self, other: Bitboard) -> Bitboard {
        Bitboard(self.0 | other.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Bitboard) {
        self.0 |= other.0;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

/// Prints the set as an 8x8 grid, White's side at the bottom: `X` for a
/// square in the set, `.` for one that isn't. Handy when debugging.
impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let mark = if self.contains(Square::new(file, rank)) {
                    'X'
                } else {
                    '.'
                };
                write!(f, "{mark}")?;
                if file < 7 {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(name: &str) -> Square {
        Square::parse(name).unwrap()
    }

    #[test]
    fn a1_is_bit_0_and_h8_is_bit_63() {
        assert_eq!(Bitboard::from_square(sq("a1")).0, 1);
        assert_eq!(Bitboard::from_square(sq("h8")).0, 1 << 63);
    }

    #[test]
    fn insert_remove_and_contains() {
        let mut set = Bitboard::EMPTY;
        set.insert(sq("e4"));
        set.insert(sq("d5"));
        assert!(set.contains(sq("e4")));
        assert!(set.contains(sq("d5")));
        assert!(!set.contains(sq("e5")));
        assert_eq!(set.count(), 2);

        set.remove(sq("e4"));
        assert!(!set.contains(sq("e4")));
        assert_eq!(set.count(), 1);
    }

    #[test]
    fn squares_come_out_lowest_first() {
        let set = Bitboard::from_square(sq("h8"))
            | Bitboard::from_square(sq("a1"))
            | Bitboard::from_square(sq("e4"));
        let names: Vec<String> = set.squares().map(|s| s.to_string()).collect();
        assert_eq!(names, ["a1", "e4", "h8"]);
    }

    #[test]
    fn debug_grid_puts_rank_8_on_top() {
        let grid = format!("{:?}", Bitboard::from_square(sq("a8")));
        let first_row = grid.lines().nth(1).unwrap();
        assert_eq!(first_row, "X . . . . . . .");
    }
}
