# 2026-10-21: Milestone 3 design, and `bench`

## Goal
Decide how strength tests (SPRT matches between two versions of the
engine) will run, and build the first piece: a `bench` command.

## What was done
- Talked through where matches should run. Charlie's Chromebook would be
  tied up for hours. A cheap VPS is slow for long tests. On AWS, EC2 Spot is
  cheapest per hour but needs a machine image and a sign-off per launch.
  **ECS Fargate Spot** runs a container on spare capacity, paid per
  second, with nothing idle. Charlie chose Fargate Spot ("absolutely and
  overwhelmingly").
- Charlie asked whether Fargate also suits the Lichess bot. Not on Spot,
  since a reclaimed machine would abandon live games. Regular Fargate
  always-on is about $29 a month for one core, so Charlie chose a hybrid:
  Fargate Spot for testing, a steady host for the bot (decided in
  Milestone 5).
- Wrote a design doc with the seven remaining choices (match runner,
  openings, game length, SPRT bounds, how engines get in, how a test
  starts, where results go). Charlie said "yes to all". Doc:
  https://claude.ai/artifact/YHxE3Ym5nfamzJdtGSDKQK
- The AWS side was handed to its own thread in cloud-homelab. The two
  threads agreed on the interface:
  - One image runs one test and exits; inputs come as environment variables.
  - Engines are fetched from `s3://<bucket>/runs/<run id>/engines/`;
    results go back under `runs/<run id>/`, kept 90 days.
  - On SIGTERM (Spot's 2-minute warning) it saves what it has.
  - The run has an 8-hour deadline; no GitHub token in the container.
- `src/bench.rs`: searches 10 fixed positions to depth 5 and adds up the
  positions looked at. The search has no randomness and, at a fixed depth,
  no clock, so the count is a fingerprint of the build. UCI `bench [depth]`
  and `stargazer bench` print `Bench: <nodes> nodes <speed> nps`.

## Problems and fixes
- Bench depth: depth 6 takes 22 seconds, depth 5 about 4, depth 4 under
  one. Chose 5, long enough that the speed number is steady.

## How it was verified
- 3 new tests (89 pass): the count is identical run to run, deeper means
  more positions, and the UCI output format. Breaking the node total made
  all three fail.
- `stargazer bench` twice in release: `Bench: 2457379 nodes 561686 nps`,
  then the same count at 561173 nps.
- `cargo fmt`, clippy, WASM build.

## Impact
Every future change can be checked for "same play, different speed" in
four seconds. The rig itself comes next: the container, then the workflow.
