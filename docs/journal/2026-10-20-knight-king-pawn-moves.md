# 2026-10-20: Knight, king and pawn moves (Milestone 1, step 2)

## Goal
List the moves of the "fixed-pattern" pieces: knights, kings (without
castling) and pawns, with promotions and en passant. Bishops, rooks,
queens and castling are step 3.

## What was done
- **`src/attacks.rs`:** tables of the squares a knight, king or pawn
  attacks from each of the 64 squares. They're built from the list of
  steps ("a knight jumps one file and two ranks…"), skipping any step that
  would fall off the board, and the work happens at compile time.
- **`src/movegen.rs`:** the `Move` type (from, to, promotion; prints as
  UCI, `e7e8q`) and pseudo-legal generation:
  - knights and kings: every table square not holding one of our pieces;
  - pawns: one square forward if empty, two from the starting rank if both
    are empty, diagonal captures, en passant onto the skipped square, and
    four promotion choices (queen, rook, bishop, knight) on the last rank.
- Decision recorded: moves carry no special-move flags (DECISIONS.md).

## Problems and fixes
None this step.

## How it was verified
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` (35 tests) and the WebAssembly build pass.
- Tests cover: 20 moves from the start position (bishops, rooks and queens
  are blocked in there, so the count is already right), black pawns moving
  down, blocked pushes, captures vs. straight-ahead blocks, promotions for
  both colours including capture-promotions, en passant with and without
  the en passant square, knights not capturing their own pieces, and
  pieces on the edge not "wrapping around" to the other side of the board.
- Proved the tests can fail: letting steps wrap around the board edge
  failed 4 tests; switching off en passant failed 1. Both restored.

## Impact
Stargazer can list most of the moves in a position. Perft can't run until
sliding pieces, castling and move making exist (step 3).
