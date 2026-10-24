# 2026-10-24: Check extensions

## Goal
Milestone 4, step 6a: search every move that gives check one half-move
deeper. Charlie's choices for step 6 (DECISIONS.md, 2026-10-24, "Late
move reductions and check extensions"): extend every check, with a cap;
two PRs, check extensions first, then late move reductions.

## What was done
- `src/search.rs`: after each move is made, the search asks whether it
  gives check. If so, the reply is searched at the same depth instead of
  one less. Checks are forcing (the side in check has few replies), so
  following them costs little, and mates and perpetual checks hide in
  them just past the horizon.
- The cap: no extension once the line is twice as long as the depth the
  root started with (depth 6 extends checks up to 12 half-moves in).
  Repetitions already end a perpetual check, and the search's hard limit
  (128 half-moves) stays as the last stop.
- A `check_extensions` switch for tests, like the others.

## Problems and fixes
None this time. The test position was picked so that the mate is found
only through the extensions: the end-of-line capture search can't see a
quiet knight mate.

## How it was verified
- New test: Philidor's smothered mate (1. Qg8+ Rxg8 2. Nf7#) is found at
  depth 2 with extensions and not without. Switching the extensions off
  in the code makes it fail.
- Unit tests: 112 pass. `cargo fmt`, `clippy -D warnings`, the WASM build
  and selfplay (10+0.1, 3+0, 1+0.01: no time losses or illegal moves)
  are clean.
- Bench (positions searched):

  | Depth | Before (main) | With check extensions | Change |
  |---|---|---|---|
  | 5 (bench) | 942,397 | 1,013,571 | +8% |
  | 6 | 2,019,859 | 2,293,225 | +14% |
  | 7 | 5,087,917 | 6,203,995 | +22% |

  More work is expected: the extensions look deeper along forcing lines.
  Whether that's worth it is the SPRT's question. New bench fingerprint
  **1013571**.
- SPRT: see the PR and the roadmap.

## Impact
The engine sees mating attacks and checking sequences further ahead, at
the cost of about 10 to 20% more positions per depth. Late move
reductions (6b) will win that back and more by searching the dull moves
shallower.
