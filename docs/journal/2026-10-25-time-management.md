# 2026-10-25: Time-management rework

## Goal
Milestone 4, step 7: spend thinking time where the position needs it.
Charlie's choices (DECISIONS.md, 2026-10-25, "Time-management rework"):
four rules in order, each its own PR and SPRT, starting from common
values; and one extra SPRT at 10 seconds + 0 at the end for sudden
death. The trigger was Charlie's game against Stockfish 2000 in PyChess
on 2026-10-24, where 1. e4 took 24 seconds.

## Rule 1: the basics (no SPRT; unit tests)

### What was done
- `src/uci.rs`: with a clock running and only one legal move, the
  engine answers `bestmove` straight away, without searching. `go depth`
  and `go infinite` still search, for anyone who wants the analysis.
- A UCI option, `Move Overhead` (0 to 5,000 ms, default 50): the time
  kept back on every move for the GUI or server to pass the move on. It
  replaces the fixed 50 ms. On Lichess (Milestone 5), network lag can be
  allowed for without a rebuild.
- Option parsing now handles names with spaces (`setoption name Move
  Overhead value 100`).

### Problems and fixes
- The first test position for "only one legal move" had two: the
  checked king could also take the unguarded bishop. A black pawn now
  guards it.

### How it was verified
- New tests: a lone legal move is answered in under half a second with
  a minute on the clock (no `info` lines), while `go depth 3` still
  searches; the option is offered, read with any capitalisation, capped
  at 5,000, and rejected when missing or negative; with 900 of a 1,000 ms
  move time kept back, the answer comes in under 500 ms. Each test fails
  when its rule is switched off or the option isn't passed on.
- 116 unit tests pass; `cargo fmt`, `clippy -D warnings`, the WASM build
  and selfplay are clean. Bench unchanged at 343857 (the search itself
  didn't change).

### Impact
Forced moves (a king with one escape, the only recapture) no longer use
any clock. No strength test: in rig games such positions are rare, and
the option only matters with a slow connection.

## Rule 2: best-move stability

### What was done
- `src/uci.rs`: after each depth, the UCI layer notes the search's best
  move. If it has just changed, the target for this move is stretched
  to 150% (the search is still making up its mind); if it has been the
  same for 4 depths in a row after the first, the target shrinks to 50%
  (it's very likely right); otherwise 100%. The target only decides
  whether to start another depth, and never goes past the ceiling, which
  still stops a depth in the middle.

### How it was verified
- New tests: the stretched and shrunk targets (and the ceiling as a
  cap), and the 100/50/150 sequence as the best move settles and then
  changes. Turning the 50% into 100% makes the second fail.
- 117 unit tests pass; `cargo fmt`, `clippy -D warnings`, the WASM build
  and selfplay are clean. Bench unchanged at 343857.
- First move with 5 minutes each: from the start position, depth 14 in
  9.2 seconds either way (the best move wasn't settled long enough); in
  a calm middlegame where Nd5 stayed best, depth 11 in 4.1 seconds
  instead of depth 12 in 12.5 seconds.
- SPRT against main: **failed**, -24.5 ± 19.4 Elo over 626 games (178
  wins, 226 draws, 222 losses; LLR -2.98).

### Problems and fixes
- The rule lost. The best guess at why: the first few depths take
  microseconds and nearly always agree, so "the same move for 4 depths"
  was true on almost every move, and the engine started no new depth
  past a quarter of its planned time. The saving at the clock was real
  (the 12.5 to 4.1 seconds above), but thinking less on every move cost
  more than the saved time won back. Charlie chose to drop it
  (DECISIONS.md, "Best-move stability dropped"); a milder version is in
  the parking lot.

### Impact
None in the engine: the code came out again in the same PR. What it
taught: a time rule has to tell a truly settled move from one that is
only settled at depths too shallow to matter.

## Rule 3: score drop

### What was done
- `src/uci.rs`: a small `ScoreWatch` remembers the score each depth
  gave. If the new depth's score is more than 30 centipawns (about a
  third of a pawn) below the last one, the search has just seen trouble
  coming, so the target for starting another depth becomes 200%: twice
  the time, to look for a way out. A rising score changes nothing. The
  ceiling still caps it all, so it can't lose on time.
- `TimeBudget::worth_another_depth` keeps the `percent` argument from
  rule 2, which now carries only the score-drop stretch.

### How it was verified
- New test: a fall of exactly 30 is ignored, a fall of 31 or more gives
  200%, a steady score goes back to 100%, and a rise does nothing.
  Making a drop give 100% makes it fail.
- 117 unit tests pass; `cargo fmt`, `clippy -D warnings`, the WASM build
  and selfplay are clean. Bench unchanged at 343857.
- SPRT: see the PR and the roadmap.
