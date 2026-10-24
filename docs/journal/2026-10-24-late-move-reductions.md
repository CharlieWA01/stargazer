# 2026-10-24: Late move reductions

## Goal
Milestone 4, step 6b: late move reductions (LMR), with Charlie's choice
of a reduction that grows slowly with depth and with how late the move
comes (DECISIONS.md, 2026-10-24, "Late move reductions and check
extensions"). Builds on 6a (check extensions).

## What was done
- `src/search.rs`: from the 4th move tried in a position, a quiet move
  (not a capture, promotion or killer, not a check, and not when in
  check) with at least 3 half-moves left is first searched shallower,
  with the zero-width "is it better than the best so far?" window. Only
  if that quick look says yes is it searched again at full depth (and
  then, as before, with the full window if it might matter).
- How much shallower: a 64 × 64 table built at the start of each search
  from 0.75 + ln(depth) × ln(move number) / 2.25, rounded down: 1 for
  the 4th move at depth 3, 2 for the 10th at depth 8, 5 for the 40th at
  depth 20. One less in the main line, and never below one half-move.
  The two numbers (0.75 and 2.25) are the common starting point; Texel
  tuning or SPRTs can adjust them later.
- A `lmr` switch for tests, like the others.

## Problems and fixes
- **A quiet mate in 2 that the engine stopped finding.** In
  `kbK5/pp6/1P6/8/8/8/8/R7 w` the winning move is 1. Ra6!, a rook
  "sacrifice" that leaves Black in zugzwang: every Black move allows
  b7 mate. `main` finds it at depth 3; with LMR it wasn't found even at
  depth 10. Tracing the root showed why:
  - At depth 3, Ra6 is the 16th move, so it gets a 1-move-shallower
    look. That look sees only "the rook hangs", so the move is dropped
    (that's LMR working as designed).
  - From depth 4 on, the quick and the full looks both come back
    "exactly no better": the null move at Black's turn cuts them off.
    Black "passes", and with a pass Black really is fine. The trouble is
    that Black *can't* pass: it's zugzwang, which the king-and-pawns
    guard doesn't catch, because Black has a bishop.
  - In `main` the same blind spot exists, but there depth 3 (too shallow
    for null moves) finds the mate first and keeps it as the main line,
    which null moves never touch.

  So the mistake belongs to null-move pruning's zugzwang blind spot, and
  LMR only exposes it. It's a composed, study-like position, rare in
  games. The known fix is a null-move verification search, already
  parked as its own SPRT. It's left as is, and noted under the roadmap's
  known limitations.

## How it was verified
- New tests: the reduction table has the expected values and never
  shrinks as depth or move number grows; and LMR searches less than half
  as many positions on three positions at depth 6. Removing the
  reduction makes the second fail.
- Unit tests: 114 pass, including every mate test. `cargo fmt`,
  `clippy -D warnings`, the WASM build and selfplay (10+0.1, 3+0,
  1+0.01: no time losses or illegal moves) are clean.
- Bench (positions searched):

  | Depth | Before (6a) | With LMR | Change |
  |---|---|---|---|
  | 5 (bench) | 1,013,571 | 343,857 | −66% |
  | 6 | 2,293,225 | 634,860 | −72% |
  | 7 | 6,203,995 | 1,352,848 | −78% |

  New bench fingerprint **343857**.
- Depth reached in one second: start position 9 (was 8), a calm
  middlegame 8 (was 6), Kiwipete 6 (was 4).
- SPRT against `main` (0 to 10, 8+0.08, 16 vCPU): **pass**, +113.4 ±
  36.5 Elo over 264 games (136 wins, 74 draws, 54 losses), the second
  biggest gain after the transposition table.
- After the merge, a 400-game match against Stockfish set to 2100
  (10+0.1, 16 vCPU): 237 wins, 21 draws, 142 losses, 61.9%, +84 ± 35:
  an estimated **2184 ± 35**, up from 2027 ± 35 before check extensions
  and LMR (together +146 in the two SPRTs, about +157 here).

## Impact
The engine now spends its time on the moves that matter and reaches
about two half-moves deeper in the same time. The risk is a quiet move
that only works deep down; the full-depth re-search catches most, and
the case above shows the rest.
