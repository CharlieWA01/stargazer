# 2026-10-23: Two small fixes to the test rig

## Goal
Fix the two problems the first Milestone 4 test ran into (journal
2026-10-23, Milestone 4 plan): a test that AWS silently refused to
start, and 20+ minutes spent installing the ARM cross-compiler.

## What was done
- **Staying on Fargate Spot.** Charlie first thought the failed run was
  a Spot reclaim and suggested moving to regular Fargate. It wasn't a
  reclaim (the task never started), and reclaims cost only a rerun, so
  Charlie decided to stay on Spot. Regular Fargate as a fallback is
  parked in the roadmap.
- **AWS's reason, and retries** (`.github/workflows/sprt.yml`, "Start the
  test"). When AWS can't start a task, `aws ecs run-task` doesn't fail:
  it returns no task and a list of failures with a reason. The workflow
  used to print only "AWS didn't start the task". It now prints each
  failure's reason and detail (not its ARN, which holds the account ID),
  adds them to the run's summary page, and tries again a minute later,
  three tries in all, since "no Spot room" often clears quickly.
- **The cross-compiler's packages are cached** (`sprt.yml` and
  `rig.yml`). The log of the second run showed the package list arriving
  in 10 seconds but the 45 MB of packages at 40 kB/s: 18.5 minutes, from
  GitHub's Azure-hosted Ubuntu mirror. apt now keeps its downloads in
  `~/apt-archives`, which `actions/cache` saves after each run and
  restores before the next; apt then fetches only packages that changed.

## Problems and fixes
- A cache entry is saved under a key that includes the run id, and the
  newest one is restored by prefix. A fixed key would be saved once and
  never updated, so it would slowly fall behind Ubuntu's package updates.
  About 45 MB per entry; GitHub keeps 10 GB per repo and drops entries
  unused for 7 days.

## How it was verified
- The retry loop was run locally against a fake `aws` command: a refusal
  then a start (prints the reason once, carries on with the task), and
  three refusals (prints all three reasons, fails the step).
- Both workflow files parse as YAML. The `Test rig` workflow runs on this
  PR (it watches `rig.yml`), which exercises the cached install for real;
  the next Run SPRT exercises `sprt.yml`.

## Impact
A refused start now says why and usually recovers on its own, and a slow
mirror costs one slow run instead of every run. Cost: nothing on AWS;
fewer GitHub Actions minutes (the two runs on 2026-10-23 spent about 45
of the 2,000 free minutes on the install).
