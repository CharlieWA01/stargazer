# Run a strength test (SPRT)

An SPRT test plays the new version of the engine ("test") against the old
one ("base") until it can say, with 95% confidence either way, whether the
new one is stronger. Why it works this way: DESIGN.md section 5 and
DECISIONS.md (2026-10-21).

## On AWS (the normal way)

*Coming in the next PR: a "Run SPRT" button in GitHub Actions that builds
both versions, starts the test on Fargate Spot and posts the verdict on the
PR.* Until AWS raises the sandbox's Fargate Spot limit from 6 vCPUs to 16,
tests run at 4 vCPU / 8 GB.

## What the test container does

The image is `rig/Dockerfile`; the script it runs is `rig/run_test.py`.

1. Fetches `engines/base` and `engines/test` from
   `s3://<results bucket>/runs/<run id>/`.
2. Runs `bench` on each and records the node counts. Same number means
   same search, so a test of a speed-only change should show equal counts
   (and should then run with bounds -5 to 0).
3. Plays games with fastchess: random openings from the 8moves_v3 book,
   each opening twice with colours swapped, 8+0.08, one game per CPU.
4. Stops when the SPRT decides, after 8 hours, or when AWS sends its stop
   signal (Spot capacity reclaimed, or someone stopped the task). Every 10
   minutes it saves a progress report, in case it's killed without warning.
5. Writes to `runs/<run id>/`:
   - `verdict.json`: the verdict and the numbers
   - `games.pgn`: every game, viewable in any chess GUI
   - `fastchess.log`: the full match log

### Reading verdict.json

| Field | Meaning |
|---|---|
| `verdict` | `pass` (stronger), `fail` (not stronger), `inconclusive` (stopped early or ran out of games; play more), `error` (something broke, read the log), or `running` while a test is still going |
| `reason` | the verdict in words |
| `results.wins/draws/losses` | from the test engine's side |
| `results.elo`, `elo_error` | the estimated difference, plus or minus; positive means test is stronger |
| `results.llr`, `llr_bounds` | the SPRT's running total; it stops on reaching either bound |
| `results.pentanomial` | results of each pair of games on the same opening (0, 0.5, 1, 1.5, 2 points) |
| `bench` | each engine's bench node count |
| `settings` | the bounds, time control and so on that this test used |

## On your own machine (for checking the rig itself)

Needs Docker. From the repo root:

```sh
cargo build --release
docker build -t stargazer-rig rig
mkdir -p work/engines
cp target/release/stargazer work/engines/base   # or an older build
cp target/release/stargazer work/engines/test
docker run --rm -v "$PWD/work:/work" \
  -e STARGAZER_TC=1+0.01 -e STARGAZER_ROUNDS=4 stargazer-rig
cat work/results/verdict.json
```

With no `STARGAZER_RESULTS_BUCKET` set, it reads and writes `work/` and
never touches AWS. The settings it understands are listed at the top of
`rig/run_test.py`. Ctrl+C stops it cleanly.

The script's own tests: `python3 -m unittest discover -s rig -v`.
