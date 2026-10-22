# 2026-10-22: Proving the rig is fair

## Goal
Settle whether the test rig favours one engine. Both earlier AWS runs of
the engine against itself came out at about +25 Elo for "test", and the
second one wrongly passed (journal 2026-10-21, first SPRT run).

## What was done
- Added a `rounds` input to Run SPRT and per-move nodes, speed and
  latency to the game files (#14), so a fixed-length match could be played
  and the two engines' conditions compared afterwards.
- **Self-match 1** (`sprt-39539285000-1`): `main` against `main`, 1,000
  rounds (2,000 games), bounds -0.5 to 0.5 so the SPRT never stops early,
  8+0.08, 16 vCPU. Result: **+7.3 ± 7.2** (793 wins, 456 draws, 751
  losses; pentanomial [29, 73, 765, 93, 40]). Borderline: the range only
  just misses 0, which happens about 1 time in 20 by chance.
- Compared the two copies move by move from the game file (about 92,000
  moves each, `pgnstats`-style script in the session scratchpad):

  | | test | base |
  |---|---|---|
  | median speed (positions a second) | 618,090 | 618,172 |
  | average depth | 5.036 | 5.034 |
  | average time per move | 0.2404 s | 0.2402 s |
  | average latency | 0.030 s | 0.030 s |

  No time losses, crashes or illegal moves. Neither copy got more CPU
  time, so the guess that the first-listed engine is favoured when all 16
  CPUs are busy was wrong.
- **Self-match 2** (`sprt-39693980805-2`, after the first attempt lost its
  Spot capacity at 4 minutes and 99 games): **+1.0 ± 7.5** over 2,000
  games (793 wins, 420 draws, 787 losses).
- Together the two self-matches give about +4 ± 5 over 4,000 games, which
  includes 0. The rig is fair; the +25s were bad luck, and the second one
  was made worse by stopping the moment it crossed the line.

## Problems and fixes
- **Another Spot interruption**, 4 minutes in. Rerun, as decided
  (2026-10-21: no automatic retry for now). Two interruptions in four
  runs so far.
- **What "latency" turned out to be.** About 2% of moves had a latency of
  0.5 to 1.3 seconds. That is not the machine stalling: it's the engine
  reporting its last finished depth early, then spending up to a second
  on a deeper search that runs out of time and is thrown away. Equal for
  both copies, so harmless for fairness, but wasted thinking time. Parked
  in the roadmap as a first strength change to test.
- The workflow logs show the full AWS account ID (in the task ARN and the
  bucket name). Fine while the repo is private; parked so it's fixed
  before the repo goes public.

## How it was verified
- The workflow reports for both runs, and both bench counts equal
  (2457379).
- `games.pgn` and `fastchess.log` of self-match 1, fetched from S3 by
  Charlie, analysed per engine as above.

## Impact
Milestone 3 is done: a strength test can now be trusted to tell a real
gain from noise, within its stated error. Cost of this check: about 50
minutes of Fargate Spot at 16 vCPU per self-match, roughly 25 cents each.
