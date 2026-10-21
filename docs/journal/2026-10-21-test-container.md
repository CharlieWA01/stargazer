# 2026-10-21: The test container

## Goal
Build the container that runs one SPRT test, and prove it works before it
ever runs on AWS (Milestone 3, step "the test container").

## What was done
- `rig/Dockerfile`: three stages. One compiles fastchess from its
  v1.8.2-alpha release (checked against the release's commit) without
  `-march=native`, because the image is built on one machine and runs on
  another. One downloads the 8moves_v3 opening book from a pinned commit of
  official-stockfish/books (public domain) and checks its hash. The final
  image adds Python, the AWS CLI and the run script.
- `rig/run_test.py`: fetches the two engines from S3 (or a local folder),
  records each one's `bench` count, runs fastchess, and writes
  `verdict.json`, the games and the log back. Its settings come from
  environment variables, which is how the GitHub workflow will pass them to
  the AWS task. Standard library only.
- How it stops early. AWS sends a stop signal (SIGTERM) when it takes Spot
  capacity back, then waits up to two minutes. The script catches it and
  asks fastchess to finish (as if Ctrl+C was pressed), then uploads what it
  has. The 8-hour deadline goes through the same path.
- The new engine is listed first, because fastchess reports everything
  from the first engine's side: positive Elo means the new one is stronger.
- Games that are clearly decided are adjudicated (Stockfish's rules).
- `rig/test_run_test.py`: unit tests for reading fastchess's output and
  choosing the verdict.
- `.github/workflows/rig.yml` ("Test rig"): on every change to `rig/`,
  runs those tests, builds the image and plays an 8-game match inside it.
- `docs/runbooks/sprt.md`: what the container does, how to read its
  verdict, and how to run it locally.

## Problems and fixes
- **Engine paths.** fastchess runs inside the results folder, so relative
  engine paths broke ("posix_spawn failed"). Paths are now absolute. The
  error path worked as intended: verdict `error`, exit code 1.
- **A stop during `bench` killed the script** instead of saving a result,
  because the signal handler was set up after bench. It's now set up first;
  a stop before the games start gives `inconclusive` with no games.
- **fastchess prints no final summary when interrupted**, so a stopped
  test lost its win/draw/loss count. The script now counts results itself
  from the line fastchess prints after each game, and takes Elo and the
  SPRT's LLR from fastchess's latest summary (now every 10 games).
- **The image couldn't be built in the cloud session**: its network rules
  don't allow Debian's package servers. Rather than work around it, the
  real build runs on GitHub (the "Test rig" check).
- **Static engines are slow.** A fully static (musl) build runs on any
  Linux but searched about 30% slower (390k vs 560k positions a second).
  Engines built normally on GitHub's Ubuntu 24.04 machines need a newer C
  library than Debian 12 has, so the image uses Debian 13 instead
  (DECISIONS.md).

## How it was verified
In the cloud session, with fastchess built from the same release and two
copies of the release engine:
- 4 rounds (8 games): `inconclusive`, "played every round without reaching
  a verdict"; the counts matched fastchess's own summary.
- Bounds 0 to 200 Elo (the engine against itself is nowhere near 200
  better): `fail` after 10 games.
- Bounds -300 to -100 (it's certainly better than 100 Elo worse): `pass`
  after 8 games.
- Stop signal after 2 seconds (during bench) and after 25 seconds (during
  games): `inconclusive`, exit code 0, files saved.
- 5-second deadline: `inconclusive`, "deadline of 5 seconds reached".
- S3 mode against a fake S3 (moto): engines fetched, and `verdict.json`,
  `games.pgn` and `fastchess.log` uploaded under `runs/<run id>/`. With no
  run id it refuses to start.
- Unit tests: 9 pass. Flipping the win/loss logic makes the counting test
  fail.
- The image build and a match inside it: the "Test rig" check on the PR.

## Impact
Nothing changes in the engine. The rig is ready for the GitHub workflow
(next PR), which will build it for ARM and start it on Fargate Spot. The
infra thread reports the sandbox allows 6 Fargate Spot vCPUs until AWS
approves 16, so the first tests will run at 4 vCPU / 8 GB.
