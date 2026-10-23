//! The transposition table: the search's memory of positions it has
//! already looked at (DESIGN.md section 3).
//!
//! The same position is often reached by different move orders
//! (1. Nf3 d5 2. d4 and 1. d4 d5 2. Nf3). For each position searched, the
//! table writes down how deep it looked, the score it found and the best
//! move. Next time the position comes up, the search can reuse the score
//! (if it looked deep enough before) or at least try the remembered best
//! move first, which makes alpha-beta cut sooner.
//!
//! The table is one big list of slots. A position's fingerprint (see
//! `zobrist`) picks its slot, and the full fingerprint is kept in the
//! slot to check it really is the same position. When two positions want
//! the same slot, the newer one wins ("always replace").

use crate::movegen::Move;
use crate::search::{MATE, MAX_PLY};

/// What a stored score means. Alpha-beta often only proves that a score
/// is at least or at most some value, not exactly what it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    /// The true score.
    Exact,
    /// At least this: a move was good enough that the opponent avoids
    /// this position (a beta cut-off), so the rest weren't looked at.
    Lower,
    /// At most this: nothing here reached what we already had elsewhere.
    Upper,
}

/// One slot: 16 bytes.
#[derive(Clone, Copy, Debug)]
struct Entry {
    /// The whole fingerprint, to tell apart positions sharing a slot.
    /// 0 marks an empty slot (a real position gets 0 about once in 2^64).
    key: u64,
    best_move: Option<Move>,
    score: i16,
    depth: u8,
    bound: Bound,
}

const EMPTY: Entry = Entry {
    key: 0,
    best_move: None,
    score: 0,
    depth: 0,
    bound: Bound::Upper,
};

/// What the table remembers about a position.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Found {
    pub best_move: Option<Move>,
    /// From the side to move's point of view, mates counted from here.
    pub score: i32,
    pub depth: u32,
    pub bound: Bound,
}

pub struct Table {
    slots: Vec<Entry>,
}

/// The size used unless the UCI `Hash` option says otherwise.
pub const DEFAULT_MB: usize = 16;

impl Table {
    /// A table using about `megabytes` of memory (at least one slot).
    pub fn new(megabytes: usize) -> Table {
        let count = (megabytes * 1024 * 1024 / std::mem::size_of::<Entry>()).max(1);
        Table {
            slots: vec![EMPTY; count],
        }
    }

    /// Forget everything (a new game).
    pub fn clear(&mut self) {
        self.slots.fill(EMPTY);
    }

    /// Which slot a fingerprint goes in: the fingerprint scaled down to
    /// the number of slots (works for any table size, unlike masking).
    fn index(&self, key: u64) -> usize {
        ((u128::from(key) * self.slots.len() as u128) >> 64) as usize
    }

    /// What's remembered about the position with this fingerprint, if
    /// anything. `ply` is how far the position is from the root, for
    /// mate scores.
    pub fn probe(&self, key: u64, ply: usize) -> Option<Found> {
        let entry = self.slots[self.index(key)];
        (entry.key == key && key != 0).then(|| Found {
            best_move: entry.best_move,
            score: score_from_table(i32::from(entry.score), ply),
            depth: u32::from(entry.depth),
            bound: entry.bound,
        })
    }

    /// Remember a search result, replacing whatever was in the slot.
    pub fn store(
        &mut self,
        key: u64,
        ply: usize,
        depth: u32,
        score: i32,
        bound: Bound,
        best_move: Option<Move>,
    ) {
        let index = self.index(key);
        self.slots[index] = Entry {
            key,
            best_move,
            score: score_to_table(score, ply) as i16,
            depth: depth.min(u32::from(u8::MAX)) as u8,
            bound,
        };
    }
}

/// Is this score a forced mate (for either side)?
fn is_mate(score: i32) -> bool {
    score.abs() >= MATE - MAX_PLY as i32
}

/// Mate scores count half-moves from the root ("mate in 5 from where the
/// search started"). The same position can be reached at different
/// distances from the root, so the table stores mates counted from the
/// position itself instead, and converts back when reading.
fn score_to_table(score: i32, ply: usize) -> i32 {
    match score {
        s if is_mate(s) && s > 0 => s + ply as i32,
        s if is_mate(s) => s - ply as i32,
        s => s,
    }
}

fn score_from_table(score: i32, ply: usize) -> i32 {
    match score {
        s if is_mate(s) && s > 0 => s - ply as i32,
        s if is_mate(s) => s + ply as i32,
        s => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;

    fn a_move() -> Move {
        Move::new(Square::parse("e2").unwrap(), Square::parse("e4").unwrap())
    }

    #[test]
    fn a_slot_is_16_bytes() {
        assert_eq!(std::mem::size_of::<Entry>(), 16);
        assert_eq!(Table::new(1).slots.len(), 65_536);
    }

    #[test]
    fn remembers_and_forgets() {
        let mut table = Table::new(1);
        let key = 0x1234_5678_9ABC_DEF0;
        assert_eq!(table.probe(key, 0), None);
        table.store(key, 0, 5, 42, Bound::Exact, Some(a_move()));
        let found = table.probe(key, 0).unwrap();
        assert_eq!(
            (found.score, found.depth, found.bound),
            (42, 5, Bound::Exact)
        );
        assert_eq!(found.best_move, Some(a_move()));
        // A different position in the same slot isn't mistaken for it.
        let other = key ^ 1;
        assert_eq!(table.index(other), table.index(key));
        assert_eq!(table.probe(other, 0), None);
        table.clear();
        assert_eq!(table.probe(key, 0), None);
    }

    #[test]
    fn the_newest_result_replaces_the_old_one() {
        let mut table = Table::new(1);
        let key = 99;
        table.store(key, 0, 9, 10, Bound::Exact, None);
        table.store(key, 0, 2, -30, Bound::Upper, None);
        let found = table.probe(key, 0).unwrap();
        assert_eq!((found.score, found.depth), (-30, 2));
    }

    #[test]
    fn mates_are_counted_from_wherever_the_position_is_reached() {
        // Found 4 half-moves from the root: we mate 3 half-moves later,
        // so 7 from the root.
        let mut table = Table::new(1);
        table.store(7, 4, 3, MATE - 7, Bound::Exact, None);
        // Reached again 2 half-moves from the root: mate is 5 from there.
        assert_eq!(table.probe(7, 2).unwrap().score, MATE - 5);
        // The same for being mated.
        table.store(8, 4, 3, -(MATE - 7), Bound::Exact, None);
        assert_eq!(table.probe(8, 2).unwrap().score, -(MATE - 5));
        // Ordinary scores are left alone.
        table.store(9, 4, 3, 150, Bound::Exact, None);
        assert_eq!(table.probe(9, 2).unwrap().score, 150);
    }
}
