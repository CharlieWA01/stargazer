# 2026-10-24: Aspiration windows (Milestone 4, step 4b)

## Goal
The second half of step 4: search each depth with a narrow window
around the last depth's score, since it rarely moves much. Charlie chose
±25 centipawns, doubling on a miss (DECISIONS.md, 2026-10-24).

## What was done
- `src/search.rs`: from depth 5, unless the last score was a mate, the
  root is searched only for scores between (last − 25) and (last + 25).
  A narrow window lets more lines be cut off early.
- If the score lands outside, that's a miss: "worse than expected" (fail
  low) or "better than expected" (fail high). The window on that side
  doubles (50, 100, 200, 400) and the depth is searched again. Past 400
  the full window is used.
- Time running out mid-way: after a fail high, the move that failed high
  is played, since it already beat the last best (like PVS at the root,
  #23). After a fail low, the last finished depth's move stands.
- The test-only PVS switch is now a small `Features` set (PVS,
  aspiration), all on outside tests.

## Problems and fixes
- The new test first also asserted that aspiration windows search fewer
  positions. With no transposition table (needed so scores can be
  compared exactly), every miss redoes the whole depth from scratch,
  and the total came out slightly higher (3,061,442 vs 3,044,136). The
  saving depends on the table, so the test now checks only that the
  answers match, and the saving is measured with bench below.

## How it was verified
- New test: on five positions (as in the PVS test) at depth 6, with no
  table, every depth's score is the same with and without aspiration
  windows. It fails if a miss is accepted instead of re-searched. 108
  pass.
- Positions searched by bench (main → aspiration):

  | Depth | Before | After | Saved |
  |---|---|---|---|
  | 5 | 987,309 | 924,579 | 6% |
  | 6 | 3,052,912 | 2,994,075 | 2% |
  | 7 | 12,949,314 | 12,914,603 | 0.3% |

  New bench fingerprint: **924579**. Speed per position unchanged
  (noise). From the start position after 1. e4 e5 2. Nf3 Nc6, `go
  depth 7` searched 1,089,245 positions against 1,140,414 on main.
- `cargo fmt`, clippy, WASM build, deep perft; `tools/selfplay.py`, no
  problems.
- SPRT (2026-10-24), bounds 0 to 10 at 8+0.08 against `main`: **pass,
  +7.8 ± 6.0 Elo over 6,282 games** (2,457 wins, 1,509 draws, 2,316
  losses; run `sprt-40059907237-1`). It took four times as many games
  as PVS, as a small gain does: after 1,301 games it read −4.0 and was
  leaning toward a fail, then climbed back.

## Impact
A small saving in bench, smaller the deeper it goes, but a real gain
in games: +8 Elo, in the usual +5 to +15 range. Milestone 4 so far:
about +277 Elo over six changes.
