"""Report on one SPRT test or Stockfish match: read its results from S3
and post them.

Used by the GitHub workflows: "Run SPRT" and "Run match" (after waiting a
while) and "SPRT report" (pressed later, for a run that took longer).

    python3 rig/report.py <run id>

It reads runs/<run id>/request.json (what was asked for, written when the
test started) and runs/<run id>/verdict.json (written by the container),
checks the task on AWS, then writes the report to the workflow's summary
page and, if the test was started for a pull request, comments on it.

Needs: the AWS CLI signed in, and the environment variables BUCKET and
CLUSTER; GH_TOKEN and GH_REPO for the PR comment (the gh command).
"""

import json
import os
import subprocess
import sys


def aws_json(*args):
    """Run an AWS CLI command and return its JSON output, or None if it failed."""
    result = subprocess.run(["aws", *args, "--output", "json"], capture_output=True, text=True)
    if result.returncode != 0:
        return None
    return json.loads(result.stdout) if result.stdout.strip() else None


def read_s3_json(bucket, key):
    result = subprocess.run(
        ["aws", "s3", "cp", f"s3://{bucket}/{key}", "-"], capture_output=True, text=True
    )
    return json.loads(result.stdout) if result.returncode == 0 else None


def task_state(cluster, task_arn):
    """The task's status on AWS (e.g. RUNNING, STOPPED) and why it stopped."""
    found = aws_json("ecs", "describe-tasks", "--cluster", cluster, "--tasks", task_arn)
    tasks = (found or {}).get("tasks") or []
    if not tasks:
        return "UNKNOWN", ""
    return tasks[0].get("lastStatus", "UNKNOWN"), tasks[0].get("stoppedReason", "")


HEADLINES = {
    "pass": "✅ **Pass**: the new version is stronger.",
    "fail": "❌ **Fail**: the new version is not stronger.",
    "inconclusive": "⚪ **Inconclusive**: stopped before the test could decide.",
    "error": "⚠️ **Error**: the test broke; see the log.",
    "running": "⏳ **Still running.**",
    "done": "🏁 **Done**: every game played.",
}

# A "no harm" test (upper bound 0 or below, used for time management)
# asks whether the new version is no weaker, not whether it's stronger.
NO_HARM_HEADLINES = {
    "pass": "✅ **Pass**: the new version is no weaker (a no-harm test).",
    "fail": "❌ **Fail**: the new version is weaker (a no-harm test).",
}


def rounded_elo(elo, error):
    if elo is None:
        return "not yet known"
    return f"{elo:+.0f}" + (f" ± {error:.0f}" if error is not None else "")


def match_lines(request, results):
    """The table for a match against Stockfish: the score, and the rating
    it suggests (Stockfish's level plus the Elo difference)."""
    games = results["games"]
    score = (results["wins"] + results["draws"] / 2) / games
    elo, error = results.get("elo"), results.get("elo_error")
    level = int(request["opponent_elo"])
    rating = "not yet known" if elo is None else rounded_elo(level + elo, error).lstrip("+")
    return [
        "",
        "| Games | Wins | Draws | Losses | Score | Elo vs Stockfish | Estimated rating |",
        "|---|---|---|---|---|---|---|",
        f"| {games} | {results['wins']} | {results['draws']} | {results['losses']} | "
        f"{score:.1%} | {rounded_elo(elo, error)} | {rating} |",
    ]


def describe(run_id, request, verdict, task_status, stopped_reason):
    """The report, as Markdown."""
    match = (request or {}).get("mode") == "match"
    lines = [f"### {'Match' if match else 'SPRT'} `{run_id}`", ""]
    state = verdict.get("verdict") if verdict else None
    if state == "running" and task_status == "STOPPED":
        # The container never wrote its final verdict: killed without
        # warning (for example out of memory), so only progress so far.
        state = "error"
        verdict["reason"] = f"the task stopped without a final verdict ({stopped_reason})"
    if not verdict:
        if task_status == "STOPPED":
            lines.append(f"⚠️ **Error**: the task stopped before writing any results ({stopped_reason}).")
        else:
            lines.append(f"⏳ **Starting** (task is {task_status}); no results yet.")
    else:
        no_harm = "elo1" in request and float(request["elo1"]) <= 0
        headline = NO_HARM_HEADLINES.get(state) if no_harm else None
        lines.append(headline or HEADLINES.get(state, state))
        if verdict.get("reason") and state != "running":
            lines.append(f"Reason: {verdict['reason']}.")
        results = verdict.get("results") or {}
        if results.get("games") and match:
            lines += match_lines(request, results)
        elif results.get("games"):
            elo = results.get("elo")
            error = results.get("elo_error")
            elo_text = "not yet known" if elo is None else f"{elo:+.1f}" + (
                f" ± {error:.1f}" if error is not None else ""
            )
            lines += [
                "",
                "| Games | Wins | Draws | Losses | Elo (new vs old) | LLR |",
                "|---|---|---|---|---|---|",
                f"| {results['games']} | {results['wins']} | {results['draws']} | "
                f"{results['losses']} | {elo_text} | "
                f"{results.get('llr', '?')} of {results.get('llr_bounds', '?')} |",
            ]
        bench = verdict.get("bench") or {}
        if bench and match:
            lines.append(f"\nBench: {bench.get('test')}. Opponent: {verdict.get('opponent')}.")
        elif bench:
            same = " (same search)" if bench.get("base") == bench.get("test") else ""
            lines.append(f"\nBench: base {bench.get('base')}, test {bench.get('test')}{same}.")
    if request and match:
        lines += [
            "",
            f"Stargazer `{request['test']}` ({request['test_sha'][:8]}) against Stockfish "
            f"at strength {request['opponent_elo']}; {request['tc']}; {request['vcpus']} vCPU. "
            f"[Workflow run]({request['workflow_run']}).",
            "",
            "The estimate is rough: Stockfish's levels are tied to the CCRL rating "
            "list but were set at longer games.",
        ]
    elif request:
        lines += [
            "",
            f"Test `{request['test']}` ({request['test_sha'][:8]}) against base "
            f"`{request['base']}` ({request['base_sha'][:8]}); bounds "
            f"[{request['elo0']}, {request['elo1']}] Elo; {request['tc']}; "
            f"{request['vcpus']} vCPU. [Workflow run]({request['workflow_run']}).",
        ]
    lines += ["", f"Games and log: `s3://<results bucket>/runs/{run_id}/`."]
    if state in (None, "running"):
        lines.append(f"Run the **SPRT report** workflow with run id `{run_id}` for the verdict later.")
    return "\n".join(lines) + "\n", state


def main(run_id):
    bucket = os.environ["BUCKET"]
    cluster = os.environ["CLUSTER"]
    request = read_s3_json(bucket, f"runs/{run_id}/request.json")
    verdict = read_s3_json(bucket, f"runs/{run_id}/verdict.json")
    if request is None:
        sys.exit(f"no test found with run id {run_id}")
    task_status, stopped_reason = task_state(cluster, request["task_arn"])
    report, state = describe(run_id, request, verdict, task_status, stopped_reason)
    print(report)

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a") as page:
            page.write(report)
    if request.get("pr"):
        subprocess.run(["gh", "pr", "comment", request["pr"], "--body", report], check=True)
    # A broken test fails the workflow, so it shows red.
    return 1 if state == "error" else 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: report.py <run id>")
    sys.exit(main(sys.argv[1]))
