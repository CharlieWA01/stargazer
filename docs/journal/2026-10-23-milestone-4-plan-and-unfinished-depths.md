# 2026-10-23: Milestone 4 plan, and keeping an unfinished depth's move

## Goal
Plan Milestone 4 ("Getting strong") and make its first change: stop
throwing away the work done in a depth that runs out of time.

## What was done
- Drafted a Milestone 4 plan: one PR and one SPRT test per step, in
  this order: (1) keep an unfinished depth's move, (2) transposition
  table with Zobrist hashing, (3) killer and history moves, (4) principal
  variation search and aspiration windows, (5) null-move pruning,
  (6) late move reductions and check extensions, (7) a time-management
  rework, (8) evaluation terms (mobility, pawn structure, king safety),
  (9) Texel tuning. The full draft, with rough expected gains and costs,
  is in the project's shared files (`stargazer/milestone-4-plan.md`).
- **Why time was being wasted.** Charlie asked for the root cause and
  whether time management should be reworked instead. The budget gives
  each move a target (about 1/20 of the clock plus 3/4 of the increment)
  and a ceiling of 3 times that. A new depth starts only while under half
  the target, which assumes the next depth takes about twice as long. Ours
  takes 5 to 6 times as long (no pruning or position memory yet). Early in
  an 8+0.08 game: target about 460 ms, ceiling 1,380 ms; depth 5 is done
  at 200 ms, so depth 6 starts, needs over a second, hits the ceiling and
  used to be thrown away.
- **Why not rework time management first.** Good time rules are tuned to
  how fast each depth grows, and steps 2 to 6 should bring that from about
  5 times down to about 2. A rework now would be tuned for an engine that
  won't exist for long, and bundling several rules into one test hides
  which one helped. So the rework moved up from last to step 7, right
  after the pruning work, and is tested rule by rule. Charlie agreed.
- **The fix (Charlie chose option A):** `search()` in `src/search.rs` now
  keeps the best move from an unfinished depth. Last depth's best move is
  always searched first; once it's finished, the search has a real score
  for it at the new depth, and any move that beats that score is genuinely
  better. A move that was still being searched when time ran out is still
  ignored. Option B (don't start a depth that can't finish) changes how
  the clock is spent, so it belongs to the time rework.

## Problems and fixes
- Writing a test for "time ran out partway through a depth" without a
  clock. The search asks `should_stop` every 2,048 positions and is
  fully repeatable, so the test first runs unstopped and counts the asks,
  then reruns and says "stop" at a chosen ask. In the fourth perft
  position depth 4 picks d2-d4 and depth 5 changes to c4-c5. Stopping at
  the last ask of depth 5 must play c4-c5; stopping at its first ask
  (d2-d4 still being searched) must keep d2-d4.

## How it was verified
- New test `a_better_move_found_in_an_unfinished_depth_is_played`. With
  the fix removed it fails (plays d2-d4 instead of c4-c5); restored, all
  90 tests pass.
- `stargazer bench` still gives 2457379 nodes: bench searches to a fixed
  depth and never stops early, so the change can't show there. It only
  changes moves played under a clock, which is what the SPRT test measures.
- `tools/selfplay.py`: three games at 10+0.1, 3+0 and 1+0.01, no time
  losses or illegal moves.
- `cargo fmt`, clippy, WASM build.
- SPRT: to come (bounds 0 to 10, 8+0.08). Agreed beforehand: if it
  fails, it's parked and not rerun until it passes.

## Impact
On the roughly 2% of moves that used to end with a binned depth, the
engine now plays what it learned in that time. Time use is unchanged.
