# 2026-10-23: Position fingerprints (Zobrist hashing)

## Goal
Milestone 4, step 2a: give every position a 64-bit fingerprint, kept up
to date move by move, as the foundation for the transposition table
(step 2b). The search must not change at all: same moves, same bench
count (2457379).

## What was done
- **Design talked through with Charlie and approved** ("that all sounds
  good to me"): the table as the search's cache, keyed by a Zobrist
  fingerprint; two PRs (fingerprints, then the table); 16 MB with a UCI
  `Hash` option; newest result replaces the old one. Recorded in
  DESIGN.md section 3 and DECISIONS.md.
- `src/zobrist.rs`: the random numbers, made when the engine is compiled
  by SplitMix64 (a tiny, well-known generator, written here) from a fixed
  seed: 768 for pieces on squares, 16 for combinations of castling rights,
  8 for en passant files, 1 for Black to move.
- `src/board.rs`: `Position` has a private `hash`. `put_piece` and
  `remove_piece` XOR the piece's number in and out; `make_move` swaps the
  castling and en passant numbers and flips the side to move; `from_fen`
  works it out from scratch. `repeats` (the repetition rule) now compares
  fingerprints instead of all 64 squares. `is_consistent`, which the
  tests call after every change, also checks the fingerprint.

## Problems and fixes
- **The first version was 10 to 12% slower**, although the search was
  identical. The extra XORs weren't the cause: a build with the field but
  no updates was just as slow. The cause was size. `Position` was exactly
  256 bytes, and the 8-byte fingerprint pushed it to 264; it's copied for
  every move tried (copy-make, and every pseudo-legal move is played to
  check legality), and copies over 256 bytes are noticeably slower on
  this machine. A test build with 5 extra bytes (still 256) was full speed.
- **Fix:** castling rights were four `bool`s (4 bytes); they're now four
  bits of one byte (`CastlingRights` with `has`, `insert`, `remove`),
  which brings `Position` back to 256 bytes. It also makes the castling
  fingerprint a single table lookup.
- **Result:** about 4% slower than `main` (median of 8 bench runs each:
  471k vs 492k positions a second). So the plan's claim that 2a alone
  makes the engine faster was wrong: repetition checks look back only to
  the last capture or pawn move, so comparing whole positions was cheap
  already. The fingerprints are an investment that the table (2b) pays
  back many times over.

## How it was verified
- New tests: the step-by-step fingerprint equals one worked out from
  scratch for every position in the six perft trees, three moves deep
  (over 200,000 positions, with every kind of special move); the same
  position by two move orders has the same fingerprint (and a different
  one while the en passant right differs); side to move, each castling
  right and en passant each change it, the move counters don't; all 792
  random numbers are different and none is zero. Breaking the castling
  update in `make_move` makes the perft-tree test fail.
- 94 tests pass; the deep perft run (`cargo test --release -- --ignored`)
  passes; `bench` gives 2457379 nodes, unchanged.
- `tools/selfplay.py`: three games, no time losses or illegal moves.
  `cargo fmt`, clippy, WASM build.

## Impact
Nothing changes in play. Every position now carries a fingerprint, ready
for the transposition table, at a cost of about 4% of speed until then.
