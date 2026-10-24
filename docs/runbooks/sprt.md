# Run a strength test (SPRT)

An SPRT test plays the new version of the engine ("test") against the old
one ("base") until it can say, with 95% confidence either way, whether the
new one is stronger. Why it works this way: DESIGN.md section 5 and
DECISIONS.md (2026-10-21).

After each test, add its games to the "Games played in testing" table
in `docs/PORTFOLIO.md` (the same goes for Stockfish matches).

## On AWS (the normal way)

1. Push the change to a branch and open its PR.
2. GitHub, **Actions** tab, **Run SPRT**, **Run workflow**. Fill in:
   - **test**: the branch (or tag, or commit) with the change
   - **base**: what to compare against, normally `main`. A branch, a
     tag or a **full 40-character** commit ID (`git rev-parse <short id>`);
     a short ID like `f21fba3` is taken as a branch name and the run
     fails in its first minute
   - **elo0 / elo1**: `0` and `10` for a strength change; `-5` and `0`
     for a clean-up that should change nothing
   - **pr**: the PR's number, so the verdict is posted there
   - leave the rest: 8+0.08, 16 vCPU, 20000 rounds, wait 30 minutes. AWS allows 16
     Fargate Spot vCPUs at once in the sandbox, so run one full-size test
     at a time (a second one would be refused)
3. The workflow builds both versions for ARM (a few minutes, plus a few
   more if `rig/` changed and the container has to be rebuilt), starts the test on
   Fargate Spot, then waits up to 30 minutes. A clear result (often a
   change that's worse) arrives in that time and is posted on the PR.
4. Otherwise the PR gets a "still running" note with the run id
   (`sprt-<number>-<attempt>`, also on the workflow's summary page). Later,
   run **SPRT report** with that run id to post the verdict. Tests stop by
   themselves after 8 hours at most.

**If AWS won't start it:** sometimes AWS declines to start the task (most
likely no Spot room just then). The workflow prints AWS's reason in the
log and on the summary page, and tries again a minute later, three tries
in all. If all three fail, read the reason and press Run SPRT again later.

**If AWS interrupts it:** Spot capacity can be taken back at any time
(the first real test lost it after 8.5 minutes). The report then says
"inconclusive: the task was told to stop", with the games so far. Press
Run SPRT again; the test starts from scratch.

**Stopping a test early:** AWS console, ECS, cluster `cwa-stargazer-runner`,
Tasks, select it, Stop. It saves what it has and reports "inconclusive".

**If something goes wrong:** the workflow's log shows each step. The
container's own output is in CloudWatch Logs, group `/cwa/stargazer-runner`.
A task that stopped without a verdict (out of memory, say) is reported as
an error with AWS's stop reason.

**What it costs:** Fargate Spot ARM is roughly 20 cents an hour at 16
vCPU, so under $2 for an 8-hour test (most finish much sooner). GitHub bills private repos for
Actions time: each test uses about 10 to 40 minutes of the 2,000 free
minutes a month, and a report about 1.

## What the test container does

The image is `rig/Dockerfile`; the script it runs is `rig/run_test.py`.
The workflows are `.github/workflows/sprt.yml` (Run SPRT) and
`sprt-report.yml` (SPRT report); both use `rig/report.py` to write the
report. The container comes from the workflow's own version of the repo,
not from the versions being tested, so old commits can be tested too.

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
