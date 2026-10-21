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
  A 200-game match in the cloud session also favoured the first-listed
  engine (+22.6 ± 38), so the next check was whether fastchess favours
  whichever engine is listed first. With 800 games of the same engine
  against itself it didn't: the first-listed copy scored -5.2 ± 17.4
  (313 wins, 325 losses, 162 draws).
- **The rerun wrongly passed.** `sprt-39532116964-1` ran to the end:
  +25.6 ± 16.7 over 502 games, SPRT accepted H1 for the engine against
  itself. Checks since then:
  - 800 local games at full load (4 games on 4 cores): +1.3 ± 16.8. Load
    alone doesn't cause it.
  - The two AWS game files (fetched by Charlie) show the two copies
    thinking identically: average depth 4.83 vs 4.83 and 4.85 vs 4.87,
    0.242 seconds a move each, and the edge split evenly between White
    and Black.
  - Together the two AWS runs give "test" 53.5% over 819 games, about a
    1-in-60 chance for equal engines. The first run sat near 0 for 200
    games before drifting up, and the second stopped the moment it crossed
    the line, which inflates the estimate.
  So far this looks like bad luck, not a bug, but it isn't settled. Next:
  a fixed 2,000-game self-match on AWS (new `rounds` input) with nodes,
  speed and latency recorded for every move.

## How it was verified
- The workflow log and report (summary page of the run).
- Charlie's `describe-tasks` output and greps of `fastchess.log` from S3.
- 800 local games (1+0.01, 3 at a time) for the first-listed check: no
  time losses.

## What changed in the rig
- "Run SPRT" has a `rounds` input (default 20,000, as before), for a
  fixed-length match such as a long self-test.
- Game files record nodes, speed (nps) and fastchess's latency for each
  move, so the two engines' conditions can be compared afterwards.

## Impact
The rig works on AWS. Next: the faster image build, then a complete
self-test.
