# 2026-10-21: The Run SPRT workflow

## Goal
The last piece of the rig before its first real test: a button in GitHub
that builds two versions of the engine, starts the test container on AWS,
and posts the verdict (Milestone 3, step "GitHub Actions").

## What was done
- `.github/workflows/sprt.yml` ("Run SPRT"), started by hand with:
  - which two versions to compare
  - the SPRT bounds and the time control
  - the number of CPUs
  - the PR number to post on
  - how long to wait
  In order, it:
  - checks the inputs
  - cross-compiles both versions for ARM
  - signs in to AWS through GitHub's OIDC (no stored keys)
  - uploads the engines to `runs/<run id>/engines/`
  - rebuilds the container only if `rig/` changed, and points `current` at it
  - starts the Fargate Spot task with the settings as overrides
  - saves `request.json` (what was asked for) next to the results
  - waits up to 30 minutes, then reports
- `.github/workflows/sprt-report.yml` ("SPRT report"): given a run id,
  posts the verdict of a test that took longer.
- `rig/report.py`: shared by both. Reads `request.json` and `verdict.json`
  and checks the task on AWS. It writes a short report (verdict, games,
  wins/draws/losses, Elo, LLR, bench counts, the two commits) to the
  workflow summary and the PR. A task that stopped without writing a final
  verdict (killed by AWS, out of memory) is reported as an error, with
  AWS's reason.
- Inputs reach the scripts only through environment variables, never pasted
  into script text, so a branch name with odd characters can't run as a
  command (GitHub's guidance on script injection).
- The variable names come from cloud-homelab's Terraform outputs
  (`STARGAZER_RUNNER_ROLE_ARN`, `_CLUSTER`, `_TASK_DEFINITION_FAMILY`,
  `_IMAGE_REPOSITORY`, `_RESULTS_BUCKET`, `_SUBNETS`, `_SECURITY_GROUP`),
  agreed with the infra thread, which also confirmed the role allows every
  call the workflow makes.

## Problems and fixes
- **Waiting costs money on a private repo.** The plan was to wait out each
  test (up to about 5.5 hours). GitHub bills private repos for Actions
  minutes even while a job only waits, so that would use up the free
  2,000 minutes in about six tests. The workflow now waits 30 minutes by
  default, and the Report workflow covers the rest (DECISIONS.md). Charlie
  was asked to confirm.
- **Quota.** The sandbox allowed only 6 Fargate Spot vCPUs, so the
  workflow first defaulted to 4 vCPU. AWS raised it to 16 the same day,
  so the default is now 16 vCPU / 32 GB (2 GB per vCPU at any size). Only
  one full-size test fits at a time.
- **Moving the `current` tag** to the image it already points at can make
  ECR answer "already exists". The workflow treats that as success.

## How it was verified
- `actionlint` (with shellcheck) passes on both workflows.
- The "start the test" step was run here with a stand-in AWS command:
  - It built the right `run-task` call: the network settings, cpu 4096,
    memory 8192, and the container's settings as overrides.
  - It wrote a correct `request.json`.
  - A test branch named `feat"; echo pwned` came through as plain text.
- `rig/report.py` was run against a fake S3 and ECS (moto): "starting"
  before any results, the verdict once `verdict.json` existed, and an error
  for an unknown run id. 5 new unit tests (14 in `rig/` in all); breaking
  the "died without a verdict" rule makes one fail.
- The real end-to-end check is the first test on AWS, once this is merged:
  the engine against itself.

## Impact
Strength tests can be started from GitHub with no AWS keys anywhere. Each
test costs cents on AWS and 10 to 40 Actions minutes.
