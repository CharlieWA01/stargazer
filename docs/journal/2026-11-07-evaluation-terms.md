# 2026-11-07: Evaluation terms, and mop-up

## Goal
Decide which evaluation terms come next and in what order, then build
the first one, mop-up.

## What was done
- After the time-management rework (journal 2026-10-25), Claude drafted
  a design doc for four evaluation terms: mop-up, pawn structure,
  mobility and king safety, with starting numbers, order, testing and
  five decisions. Charlie agreed to all five recommendations
  (DECISIONS.md, 2026-11-07; DESIGN.md section 4 now **Decided** for
  the terms).
- Mop-up (PR #46), in `src/eval.rs`: `mop_up()` applies when one side's
  pieces (pawns not counted) are worth at least a rook (500) more and
  the other side has no pawns. It adds to the endgame score +10 for
  each ring the losing king is from the centre (0 to 3) and +4 for each
  square the kings are closer than 14 apart, counting along files and
  ranks. Mate needs both: the losing king on the edge and the winning
  king next to it. Without this, every king move in such an ending
  scored the same, which is why the engine shuffled a queen up against
  PyChess.
- It reuses `centre_ring`, the rule the piece-square tables already use.

## Problems and fixes
- The first versions of the tests compared whole evaluations, and they
  still passed with mop-up switched off: the king's endgame square
  table already likes the centre, so the defending king on the edge
  scores worse anyway. The tests now check the mop-up bonus itself.

## How it was verified
- New tests: `mop_up_drives_a_bare_king_to_the_edge`,
  `mop_up_brings_the_winning_king_closer`,
  `mop_up_stays_out_of_ordinary_positions` (not a rook up; the
  defender still has a pawn; works for either colour), and a mop-up
  position added to the mirrored-positions test. Each of the first two
  was seen failing with its number set to 0, then passing. 122 tests
  pass; `cargo clippy` and the wasm build are clean.
- Bench is unchanged (343857): none of its positions is a won ending
  against a bare king, so the term never applies there.
- A quick check with 0.1 seconds a move from four rook- or queen-against-
  king positions: both main and mop-up finished every game, in 15 to
  33 half-moves, with no clear difference at this depth (the search
  already sees these short mates). Mop-up should matter most in longer
  endings with more material; the SPRT decides.
- SPRT [0, 10] at 8+0.08 against main: see the PR and the roadmap.

## Impact
The engine has a plan in won endings against a bare king. The cost is a
few additions per evaluation, only when the term applies.
