# 2026-11-08: Stockfish match after the evaluation terms

## Goal
Put a rating on the evaluation step (DECISIONS.md, 2026-11-07: "a
Stockfish match after all four"), the way the search step was measured
on 2026-10-24.

## What was done
- After #54 (king safety) merged, ran **Run match** on `main`
  (c499059, bench 287042): 400 games against Stockfish 17.1 at strength
  2300, 10+0.1, 16 vCPU on Fargate Spot (`match-42438078050-1`). The
  level was picked about 100 to 150 points above the last estimate
  (2184), expecting the engine to have gained roughly that much, so the
  score would land near 50% where the estimate is tightest.
- It took about 13 minutes of games.

## Problems and fixes
None.

## How it was verified
The match report (from the workflow's "Report" step):

| Games | Wins | Draws | Losses | Score | Elo vs Stockfish | Estimated rating |
|---|---|---|---|---|---|---|
| 400 | 225 | 27 | 148 | 59.6% | +68 ± 34 | **2368 ± 34** |

Recorded in the results table in `docs/runbooks/stockfish-match.md`.

## Impact
About **2370** on the CCRL-tied scale, up from about 2180 (2184 ± 35 on
2026-10-24). In between: the time-management rework (no-harm, so not
expected to add much at 10+0.1) and the three evaluation terms that
passed (+38, +74 and +71 Elo against the previous version each). SPRT
gains against the engine's own previous version don't simply add up
against a different opponent, so the match is the number to quote:
roughly +185 points. Next on the roadmap: Texel tuning (design section
first).
