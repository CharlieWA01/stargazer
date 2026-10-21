# 2026-10-21: The first real SPRT run

## Goal
Run the rig end to end on AWS for the first time: `main` against itself,
bounds 0 to 10 Elo, 16 vCPU. The right answer is "fail" (no gain).

## What was done
- Started "Run SPRT" (run id `sprt-39527492850-1`). Every step worked:
  - both engines were cross-built for ARM and uploaded through the OIDC role
  - the container image was built and pushed, and `current` was moved to it
  - the task started on Fargate Spot
  - the report was posted on the workflow summary
- Both engines' bench counts matched (2457379), as they should for the
  same commit.

## Problems and fixes
- **The ARM image took 31 minutes to build**, because fastchess (C++) was
  compiled under QEMU emulation. The Dockerfile now cross-compiles
  fastchess on the build machine's own CPU (`--platform=$BUILDPLATFORM`
  and Debian's `g++-aarch64-linux-gnu`), as the engines already were. Only
  the final stage's package install still runs under emulation. The "Test
  rig" check now also builds the ARM image and plays 2 games inside it
  under emulation, so a broken ARM build shows up on the PR, not during a
  real test.
- **AWS reclaimed the Spot capacity after 8.5 minutes** (`stopCode`
  `SpotInterruption`, checked by Charlie with `aws ecs describe-tasks`).
  The container caught the stop signal, saved 317 games and reported
  "inconclusive", as designed. What to do about interruptions (nothing,
  retry on Spot, or fall back to regular Fargate) is Charlie's call.
- **+25.5 ± 21.5 Elo against itself.** No time losses, crashes or illegal
  moves in the log. The pentanomial count was [4, 12, 103, 20, 11]: 103
  of 150 pairs were exactly 1-1. Stargazer has no randomness, so two
  identical copies on the same opening often play the same game twice with
  colours swapped. Those pairs carry no information, which leaves about 47
  that do. A result this far from 0 happens by chance about 2 times in 100.
  Most likely luck, but unproven: a self-test run to the end should give
  "fail".

## How it was verified
- The workflow log and report (summary page of the run).
- Charlie's `describe-tasks` output and greps of `fastchess.log` from S3.

## Impact
The rig works on AWS. Next: the faster image build, then a complete
self-test.
