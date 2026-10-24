"""Run one SPRT test: the new engine ("test") against the old one ("base").
Or, in match mode, one match: the engine against Stockfish at a set
strength, to estimate its rating.

This is what the test container runs (see rig/Dockerfile,
docs/runbooks/sprt.md and docs/runbooks/stockfish-match.md). In order, it:

1. fetches the engines from S3 (or finds them in a local folder),
2. runs `bench` on each, so the results record exactly which builds played,
3. runs fastchess: games from a balanced opening book, each opening played
   twice with colours swapped, until the SPRT reaches a verdict (or, in
   match mode, for a set number of rounds),
4. writes verdict.json (plus the games and the full log) back to S3.

A verdict is one of:
- "pass": the test engine is stronger (SPRT accepted H1, "at least elo1 better")
- "fail": it isn't (SPRT accepted H0, "no better than elo0")
- "inconclusive": stopped before a verdict (deadline, AWS reclaimed the
  Spot capacity, or the game limit ran out); the numbers so far are kept
- "error": something broke (an engine crashed, fastchess failed)
- "done": a match played all its games (match mode has no pass or fail)

Every setting comes from an environment variable (all start STARGAZER_):

  RESULTS_BUCKET     S3 bucket for engines and results. Empty means local
                     mode: engines from WORK_DIR/engines, results written to
                     WORK_DIR/results, no AWS at all.
  RUNS_PREFIX        folder inside the bucket (default "runs/")
  RUN_ID             this test's name; files live under RUNS_PREFIX/RUN_ID/
  MODE               "sprt" (the default) or "match" (against Stockfish)
  OPPONENT_ELO       match mode: Stockfish's strength, 1320 to 3190
                     (its UCI_Elo setting; default 1800)
  STOCKFISH          match mode: the Stockfish program (set in the Dockerfile)
  DEADLINE_SECONDS   stop and report after this long (default 28800, 8 hours)
  ELO0, ELO1         the SPRT bounds (default 0 and 10; SPRT mode only)
  TC                 time control, seconds+increment (default "8+0.08")
  ROUNDS             most rounds to play; a round is one opening played
                     twice (default 20000, so at most 40,000 games)
  CONCURRENCY        games at once (default: one per CPU)
  BOOK               the opening book (set in the Dockerfile)
  WORK_DIR           local working folder (default the current folder)

Standard library only, so the image needs nothing but python3.
"""

import json
import os
import re
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

# The SPRT's error rates: a 5% chance of passing a change that isn't
# better, and a 5% chance of failing one that is (DECISIONS.md, 2026-10-21).
ALPHA = 0.05
BETA = 0.05

# How often to save a progress report while games are being played, so a
# test that's killed without warning still leaves its numbers behind.
PROGRESS_EVERY_SECONDS = 600

# Stockfish's strength setting (UCI_Elo) accepts these levels.
STOCKFISH_ELO_RANGE = (1320, 3190)


def setting(name, default=None):
    value = os.environ.get("STARGAZER_" + name, "").strip()
    return value if value else default


class Run:
    """Where this run's files are: S3, or a local folder."""

    def __init__(self):
        self.bucket = setting("RESULTS_BUCKET")
        self.run_id = setting("RUN_ID", None if self.bucket else "local")
        if not self.run_id:
            sys.exit("STARGAZER_RUN_ID is required when results go to S3")
        if not re.fullmatch(r"[A-Za-z0-9._-]+", self.run_id):
            sys.exit(f"STARGAZER_RUN_ID has unexpected characters: {self.run_id!r}")
        prefix = setting("RUNS_PREFIX", "runs/")
        self.s3_folder = f"s3://{self.bucket}/{prefix.rstrip('/')}/{self.run_id}"
        work = Path(setting("WORK_DIR", ".")).resolve()
        self.engines = work / "engines"
        self.results = work / "results"
        self.engines.mkdir(parents=True, exist_ok=True)
        self.results.mkdir(parents=True, exist_ok=True)

    def fetch_engines(self, names):
        for name in names:
            path = self.engines / name
            if self.bucket:
                aws("s3", "cp", f"{self.s3_folder}/engines/{name}", str(path))
            if not path.is_file():
                sys.exit(f"missing engine: {path}")
            path.chmod(0o755)

    def save(self, *names):
        """Upload these files from the results folder (local mode: nothing to do)."""
        if not self.bucket:
            return
        for name in names:
            path = self.results / name
            if path.is_file():
                try:
                    aws("s3", "cp", str(path), f"{self.s3_folder}/{name}")
                except subprocess.CalledProcessError as error:
                    # Keep going: losing one upload shouldn't lose the rest.
                    print(f"upload of {name} failed: {error}", flush=True)


def aws(*args):
    subprocess.run(["aws", *args, "--only-show-errors"], check=True)


def bench(engine):
    """The engine's bench fingerprint: nodes searched on a fixed set of
    positions. Same number means same search; None if it couldn't say."""
    try:
        output = subprocess.run(
            [str(engine), "bench"], capture_output=True, text=True, timeout=300
        ).stdout
    except (OSError, subprocess.TimeoutExpired):
        return None
    match = re.search(r"Bench: (\d+) nodes", output)
    return int(match.group(1)) if match else None


def fastchess_command(run, settings):
    # The test engine goes first: fastchess reports every number (Elo,
    # wins, the SPRT) from the first engine's point of view, so a positive
    # Elo means the new version is stronger.
    if settings["mode"] == "match":
        # Stockfish at a set strength, on one thread with a small table,
        # like Stargazer. No SPRT: a set number of games, then the score.
        opponent = [
            "-engine", f"cmd={settings['stockfish']}", "name=stockfish",
            "option.Threads=1", "option.Hash=16",
            "option.UCI_LimitStrength=true", f"option.UCI_Elo={settings['opponent_elo']}",
        ]
        stopping = []
    else:
        opponent = ["-engine", f"cmd={run.engines / 'base'}", "name=base"]
        stopping = [
            "-sprt", f"elo0={settings['elo0']}", f"elo1={settings['elo1']}",
            f"alpha={ALPHA}", f"beta={BETA}", "model=logistic",
        ]
    return [
        "fastchess",
        "-engine", f"cmd={run.engines / 'test'}", "name=test",
        *opponent,
        "-each", f"tc={settings['tc']}",
        "-openings", f"file={settings['book']}", "format=pgn", "order=random",
        "-rounds", str(settings["rounds"]), "-repeat",
        "-concurrency", str(settings["concurrency"]),
        *stopping,
        # Finish games that are clearly decided, as Stockfish's testing does:
        # resign when both engines agree one side is 6 pawns up for 3 moves
        # running; agree a draw after move 34 when both say within 0.2 of a
        # pawn for 8 moves. Saves time without changing results.
        "-resign", "movecount=3", "score=600",
        "-draw", "movenumber=34", "movecount=8", "score=20",
        # Each move also records the engine's node count, speed and how long
        # fastchess waited for its reply, to spot one engine being treated
        # differently (a slow start, a busy CPU).
        "-pgnout", f"file={run.results / 'games.pgn'}", "notation=san",
        "nodes=true", "nps=true", "latency=true",
        "-ratinginterval", "10",
    ]


def read_results(log):
    """Pull the numbers out of fastchess's output.

    Wins, draws and losses are counted here, game by game, from the test
    engine's side, because fastchess doesn't print a final summary when it's
    interrupted. Elo and the SPRT's LLR come from its latest summary
    (printed every 10 games), so they can trail by a few games.
    """
    numbers = {"games": 0, "wins": 0, "draws": 0, "losses": 0}
    for line in log.splitlines():
        if m := re.match(r"Finished game \d+ \((\w+) vs (\w+)\): (1-0|0-1|1/2-1/2)", line):
            white, _, result = m.groups()
            numbers["games"] += 1
            if result == "1/2-1/2":
                numbers["draws"] += 1
            elif (result == "1-0") == (white == "test"):
                numbers["wins"] += 1
            else:
                numbers["losses"] += 1
        elif m := re.search(r"^Games: (\d+),", line):
            numbers["summary_after_games"] = int(m[1])
        elif m := re.match(r"Elo: (\S+) \+/- (\S+)", line):
            numbers["elo"], numbers["elo_error"] = as_number(m[1]), as_number(m[2])
        elif m := re.match(r"Ptnml\(0-2\): \[([\d, ]+)\]", line):
            numbers["pentanomial"] = [int(n) for n in m[1].split(",")]
        elif m := re.match(r"LLR: (\S+) \(\S+\) \((\S+), (\S+)\)", line):
            numbers["llr"] = as_number(m[1])
            numbers["llr_bounds"] = [as_number(m[2]), as_number(m[3])]
    if "H1 was accepted" in log:
        numbers["sprt"] = "H1"
    elif "H0 was accepted" in log:
        numbers["sprt"] = "H0"
    return numbers


def as_number(text):
    """A float, or None for fastchess's 'nan' and 'inf' (not valid JSON)."""
    try:
        value = float(text.rstrip(","))
    except ValueError:
        return None
    return value if value == value and abs(value) != float("inf") else None


def verdict_from(numbers, exit_code, stopped_by, mode="sprt"):
    if numbers.get("sprt") == "H1":
        return "pass", "the test engine is stronger (SPRT accepted H1)"
    if numbers.get("sprt") == "H0":
        return "fail", "the test engine is not stronger (SPRT accepted H0)"
    if stopped_by:
        return "inconclusive", f"stopped before a verdict: {stopped_by}"
    if mode == "match" and exit_code == 0 and numbers.get("games"):
        return "done", "played every game"
    if exit_code == 0 and numbers.get("games"):
        return "inconclusive", "played every round without reaching a verdict"
    return "error", f"fastchess exited with code {exit_code} before a verdict"


def engine_name(program):
    """What a UCI engine calls itself (its "id name"), or None."""
    try:
        output = subprocess.run(
            [str(program)], input="uci\nquit\n", capture_output=True, text=True, timeout=30
        ).stdout
    except (OSError, subprocess.TimeoutExpired):
        return None
    match = re.search(r"^id name (.+)$", output, re.MULTILINE)
    return match.group(1).strip() if match else None


def main():
    run = Run()
    cpus = len(os.sched_getaffinity(0))
    mode = setting("MODE", "sprt")
    if mode not in ("sprt", "match"):
        sys.exit(f"STARGAZER_MODE must be sprt or match, not {mode!r}")
    settings = {
        "mode": mode,
        "elo0": float(setting("ELO0", "0")),
        "elo1": float(setting("ELO1", "10")),
        "tc": setting("TC", "8+0.08"),
        "rounds": int(setting("ROUNDS", "20000")),
        "concurrency": int(setting("CONCURRENCY", str(cpus))),
        "book": setting("BOOK", "8moves_v3.pgn"),
        "deadline_seconds": int(setting("DEADLINE_SECONDS", "28800")),
    }
    if mode == "match":
        low, high = STOCKFISH_ELO_RANGE
        elo = int(setting("OPPONENT_ELO", "1800"))
        if not low <= elo <= high:
            sys.exit(f"STARGAZER_OPPONENT_ELO must be {low} to {high}, not {elo}")
        settings["opponent_elo"] = elo
        settings["stockfish"] = setting("STOCKFISH", "stockfish")
        del settings["elo0"], settings["elo1"]
    if mode == "sprt":
        settings.update(alpha=ALPHA, beta=BETA)
    started = time.time()
    report = {
        "run_id": run.run_id,
        "verdict": "running",
        "settings": {**settings, "cpus": cpus},
        "started": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started)),
    }

    def write_report(**changes):
        report.update(changes)
        report["seconds"] = round(time.time() - started)
        (run.results / "verdict.json").write_text(json.dumps(report, indent=2) + "\n")
        run.save("verdict.json")

    # Two ways to stop early, both ending the same way: ask fastchess to
    # finish (SIGINT, like pressing Ctrl+C), which ends the games in
    # progress and prints the final numbers. A stop before the games start
    # just skips them.
    stopped_by = []
    games = None

    def stop(reason):
        if stopped_by:
            return
        print(f"stopping: {reason}", flush=True)
        stopped_by.append(reason)
        if games is not None and games.poll() is None:
            games.send_signal(signal.SIGINT)

    # 1. AWS's stop signal: sent when it reclaims Spot capacity or someone
    #    stops the task. AWS waits up to two minutes before killing us.
    signal.signal(signal.SIGTERM, lambda *_: stop("the task was told to stop (SIGTERM)"))
    signal.signal(signal.SIGINT, lambda *_: stop("interrupted (SIGINT)"))
    # 2. The deadline, so a test that never converges doesn't run forever.
    deadline = threading.Timer(
        settings["deadline_seconds"],
        stop,
        args=[f"deadline of {settings['deadline_seconds']} seconds reached"],
    )
    deadline.daemon = True
    deadline.start()

    names = ("test",) if mode == "match" else ("base", "test")
    run.fetch_engines(names)
    report["bench"] = {name: bench(run.engines / name) for name in names}
    print("bench:", report["bench"], flush=True)
    if mode == "match":
        report["opponent"] = engine_name(settings["stockfish"])
        print("opponent:", report["opponent"], flush=True)
        if not report["opponent"]:
            write_report(verdict="error", reason=f"Stockfish didn't start ({settings['stockfish']})")
            return 1
    write_report()

    log_path = run.results / "fastchess.log"
    if stopped_by:
        write_report(verdict="inconclusive", reason=f"stopped before any games: {stopped_by[0]}")
        return 0
    games = subprocess.Popen(
        fastchess_command(run, settings),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        cwd=run.results,
    )

    quiet = ("Started game", "Finished game")
    if mode == "match":
        quiet += ("Warning; Bestmove does not match", "Info;", "Position;", "Moves;")
    last_progress = time.time()
    with open(log_path, "w") as log:
        for line in games.stdout:
            log.write(line)
            # The console (CloudWatch on AWS) gets everything except the
            # two lines per game; the full log is saved with the results.
            # Stockfish at a set strength plays a move other than the one
            # its analysis ends with, on purpose, and fastchess warns about
            # it (four lines each time); those stay in the log only.
            if not line.startswith(quiet):
                print(line, end="", flush=True)
            if time.time() - last_progress > PROGRESS_EVERY_SECONDS:
                log.flush()
                write_report(results=read_results(log_path.read_text()))
                last_progress = time.time()
    exit_code = games.wait()
    deadline.cancel()

    numbers = read_results(log_path.read_text())
    verdict, reason = verdict_from(
        numbers, exit_code, stopped_by[0] if stopped_by else None, mode
    )
    run.save("games.pgn", "fastchess.log")
    write_report(verdict=verdict, reason=reason, results=numbers, fastchess_exit_code=exit_code)
    print(f"verdict: {verdict} ({reason})", flush=True)
    return 1 if verdict == "error" else 0


if __name__ == "__main__":
    sys.exit(main())
