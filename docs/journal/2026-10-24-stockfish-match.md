# 2026-10-24: Matches against Stockfish

## Goal
Charlie wanted an easy way to test Stargazer against Stockfish at any
time, both to get a rough rating and to watch games. Charlie chose a "Run
match" button on the AWS test rig (DECISIONS.md, 2026-10-24).

## What was done
- `rig/run_test.py` has a match mode (`STARGAZER_MODE=match`). Only the
  `test` engine is fetched. Its opponent is Stockfish with
  `UCI_LimitStrength` on and `UCI_Elo` set to the chosen level, on one
  thread with a 16 MB table. It plays a set number of rounds with no
  SPRT, records Stockfish's exact version (its `id name`), and ends
  "done". Stockfish at a set strength plays a move other than the one
  its analysis ends with, on purpose, and fastchess prints a four-line
  warning each time. Those lines stay in the saved log but are left off
  the console.
- `rig/report.py` writes a match report: games, score, the Elo
  difference, and the estimated rating (Stockfish's level plus the
  difference).
- `rig/Dockerfile` installs Debian's `stockfish` package.
- `.github/workflows/sprt.yml` can now also be called by another
  workflow, with a `mode` and an `opponent_elo`. In match mode it builds
  only the one version and names the run `match-<number>-<attempt>`.
  The new `match.yml` is the button: it turns "games" into rounds and
  calls `sprt.yml`. "SPRT report" accepts match run ids.
- `rig.yml` (the container check on every `rig/` change) also plays a
  4-game match against the image's Stockfish.
- New runbook `docs/runbooks/stockfish-match.md`: the button, how to read
  the number, a results table, and how to watch games at home in
  PyChess with `apt install stockfish`.

## Problems and fixes
- The session machine can't reach Debian's package servers, so the
  container itself couldn't be built here. The match mode was tested
  outside the container with Stockfish 17.1's official x86 build and the
  same pinned fastchess and book. The container's own check (`rig.yml`)
  builds the real image with Debian's Stockfish on the PR.
- A test helper named `run` clashed with `unittest`'s own `run` method;
  it was renamed.

## How it was verified
- Rig unit tests: 21 pass (6 new: the match command plays Stockfish with
  its strength options and no SPRT, a finished match is "done", a
  stopped one "inconclusive", and the report's score and estimate).
  Breaking the strength option, the "done" verdict or the estimate's
  sign makes them fail.
- Local trial runs of `run_test.py` in match mode: 16 games at 2+0.02
  against Stockfish 1800 (8 wins, 8 losses), and 20 games against 1500
  (16 wins, 1 draw, 3 losses). Both reported correctly.
- A longer local preview (200 games at 10+0.1 against Stockfish 1800)
  was too slow on the 4-CPU session machine and stalled when the session
  went idle. Its first games went 15 wins, 7 losses (22 games): far too
  few to quote, but in line with a rating somewhere above 1800. The
  first real number will come from AWS.
- The workflows parse as YAML. They can only run on GitHub once merged
  (a new workflow's button appears only from `main`), so the first real
  match follows the merge.

- A games counter, at Charlie's suggestion: `docs/PORTFOLIO.md` now has
  a "Games played in testing" table, totalled from the journals and the
  roadmap: 6,619 games in Milestone 3 (317 and 502 in the first AWS
  runs, 200 and 2 × 800 locally, two 2,000-game self-matches) and
  10,275 in Milestone 4's six SPRTs, 16,894 in all. The SPRT and match
  runbooks say to add each new test's games.

## Impact
A yardstick outside our own versions, for a few cents a run. It gives
Charlie's "what would it be rated?" question a measured answer, and the
results table in the runbook shows the rating over time.
