# 2026-10-20: Clocks and `stop` (Milestone 2, step 3)

## Goal
Make Stargazer play under a real clock, the way a GUI or Lichess runs it:
use the time it's given, answer `stop` and `isready` while thinking, and
never lose on time.

## What was done
- `src/uci.rs`: `go` starts the search on its own thread, so the command
  loop keeps reading. `stop` sets a flag the search checks every 2,048
  positions; `isready` is answered at once; `quit`, `position` and a new
  `go` finish any search first. Both threads print through one shared
  output, a line at a time.
- `go` reads `wtime btime winc binc movestogo`, `movetime`, `depth` and
  `infinite`. The plan for each move is a target and a ceiling. The target
  is a twentieth of the time left plus three quarters of the increment.
  The ceiling is three times that, and both stay 50 ms inside the clock.
  Between depths the engine only starts another if it's under half the
  target, because the next depth takes several times longer than this one.
- `go infinite` waits for `stop` before answering, even when the search
  runs out of depths (a bare-kings position does that instantly), as the
  UCI spec requires.
- `tools/selfplay.py` plays the engine against itself under a clock and
  reports illegal moves and losses on time. Runbook: `docs/runbooks/play-it.md`.

## Problems and fixes
- The first versions of two tests couldn't fail. "`go infinite` waits for
  `stop`" sent all its commands at once, so `stop` always arrived before
  the search finished, early answer or not. It now uses paced input that
  sends `isready` and `stop` after 300 ms, like a person typing. "The hard
  limit is respected" depended on how fast the machine was. The stop
  decisions are now two small functions (`out_of_time`,
  `worth_another_depth`) with direct tests.
- `cargo test` passes output to the engine through a buffer; with a second
  thread printing, it needed a shared, lockable buffer (`Captured`).
- `main.rs` now passes `stdout` itself instead of a lock on it, which can't
  move to another thread.

## How it was verified
- 6 new tests (86 pass): moving in time with a clock and with `movetime`,
  `go infinite` waiting for `stop` (with `isready` answered first), `stop`
  ending a long search quickly, reading the `go` command (negative times
  included), and the time budget numbers and stop decisions.
- Proved they can fail. Removing the ceiling, the between-depths rule or
  the safety margin each fails `the_time_budget`. Letting `go infinite`
  answer early fails `go_infinite_waits_for_stop`.
- Self-play in release, one game each at 10s+0.1s, 3s and 1s+0.01s: 240,
  239 and 300 half-moves, no illegal moves, no losses on time (the 3-second
  game ended with 48 ms left on each clock).
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, WASM build.

## Impact
Stargazer plays a whole game under a clock: it can be loaded into any UCI
GUI. Milestone 2's code is complete; it's done once Charlie has played it
in a GUI.
