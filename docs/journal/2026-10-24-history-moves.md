# 2026-10-24: History moves (Milestone 4, step 3b)

## Goal
The second half of step 3: order the quiet moves that come after the
killers by how well each has done so far in the game. Charlie chose the
details: halve the scores between moves, and lower the scores of quiet
moves that failed (DECISIONS.md, 2026-10-24).

## What was done
- `src/search.rs`: a `History` table, one score per quiet move by side
  to move, from-square and to-square (2 × 64 × 64 numbers, 32 KB).
- When a quiet move refutes a position with `depth` half-moves to go,
  its score rises by depth × depth, and each quiet move tried before it
  there falls by the same amount. Deeper verdicts count for more. Each
  change shrinks as a score nears ±16,384, so no score can pass it.
- Move order: last depth's or the table's move, captures and
  promotions, killers, then the other quiet moves by history score,
  highest first. Quiescence doesn't use it.
- `search` takes the history from its caller and halves it before
  starting, so it carries from move to move but older lessons fade.
  The UCI layer keeps it with the transposition table and clears both
  on `ucinewgame`. Bench gives each position a fresh history, so each
  count still depends only on that position.
- The list of quiet moves tried at each node is a reusable list per ply.
  A fresh list at every node made the engine about 4% slower per
  position; reusing them brought it to about 1.5%.

## Problems and fixes
- First version about 4% slower per position, for only 1 to 4% fewer
  positions. The cost was making a new list at every node; fixed as
  above.

## How it was verified
- Three new tests: scores rise and fall, stay within the limit (and grow
  more slowly near it), are kept per side, halve and clear; quiet moves
  after the killers follow their scores, with a failed move at the back;
  a quiet refutation gains and the quiet moves before it lose, while a
  capturing refutation changes nothing. Breaking each rule in turn (no
  lowering, no ordering by score, no limit) fails its test; restored,
  106 pass.
- Positions searched by bench (main → history):

  | Depth | Before | After | Saved |
  |---|---|---|---|
  | 5 | 1,162,141 | 1,153,324 | 1% |
  | 6 | 3,893,639 | 3,730,681 | 4% |
  | 7 | 17,326,568 | 16,819,905 | 3% |

  New bench fingerprint: **1153324**. Bench starts every position with
  no history, so it can't show the carry-over between moves.
- Speed per position: median of five bench runs 405,000 (main) against
  399,000 (history).
- `cargo fmt`, clippy, WASM build, deep perft; `tools/selfplay.py` three
  games, no problems; a UCI `ucinewgame`, `position`, `go` by hand.
- SPRT (2026-10-24), bounds 0 to 10 at 8+0.08 against `main`: **pass,
  +25.1 ± 16.2 Elo over 1,095 games** (462 wins, 251 draws, 382 losses;
  run `sprt-40051096528-1`). More than bench suggested: in real games
  the scores carry over from move to move.

## Impact
A small saving in bench but a clear gain in games (+25 Elo), and it will
matter again once late move reductions (a later step) use it to decide
which quiet moves to search less deeply. Memory: 32 KB.
