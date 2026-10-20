# 2026-10-20: Milestone 2 design and the evaluation

## Goal
Record the Milestone 2 decisions Charlie made (search, evaluation, time),
then build step 1: a function that scores a position without looking ahead.

## What was done
- DESIGN.md sections 3 (search) and 4 (evaluation) marked **Decided**, with
  three matching DECISIONS.md entries. Charlie's idea of spending more time
  on harder positions went to the roadmap's parking lot.
- `src/eval.rs`: `evaluate(position)` returns centipawns for the side to
  move. Material (P 100, N 320, B 330, R 500, Q 900) plus piece-square
  bonuses, scored twice (middlegame and endgame) and blended by how much
  material is left (24 = full board, 0 = only kings and pawns).
- The square bonuses are not copied from anyone's tables. Each comes from a
  rule written in the code: knights lose points towards the rim, pawns gain
  points as they advance (more in the endgame), rooks like the seventh rank,
  the king hides in the corner in the middlegame and walks to the centre in
  the endgame. The tables are built from those rules when the engine is
  compiled, like the attack tables, so a lookup costs nothing at runtime.
  Black uses White's tables flipped top to bottom.
- The numbers are deliberately rough; the tuner in Milestone 4 will refine
  them against real games.

## Problems and fixes
- The "pawns grow in value" test first asked for more than a 50-point gain
  from e3 to e6; the real gain is exactly 50. The rule was right and the
  threshold too tight, so the test now asks for more than 40.

## How it was verified
- 7 new tests: the start position scores 0; four mirrored positions (colours
  swapped, board flipped) score the same; the score flips sign with the side
  to move; an extra knight is worth > 250; a knight on d5 beats one on a1;
  e6 pawn beats e3; castled king beats exposed king with queens on, central
  king beats cornered king in a pawn ending.
- Proved the tests can fail: forgetting to flip Black's tables broke the
  mirror and start-position tests; always using the middlegame score broke
  the pawn and king tests. Restored, all 62 tests pass.
- `cargo fmt`, `cargo clippy -- -D warnings` and the WASM build pass.

## Impact
The engine can now say "who's better here" in a position. Nothing plays yet:
the search (step 2) is what turns this score into a move choice.
