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
- SPRT against main: **passed**, +10.9 ± 8.7 Elo over 2,651 games (883
  wins, 969 draws, 799 losses; LLR 3.0). It ran past the workflow's
  wait, so the verdict came from the SPRT report workflow.

### Impact
A small, real gain, in line with the +10 to +30 expected for the whole
rework. Unlike rule 2, it adds time only in the few positions where the
search has just found a problem, instead of taking time from every
move.

## Rule 4: when to start a new depth

### What was done
- `src/uci.rs`: another depth starts only while under 60% of the target
  (`NEW_DEPTH_PERCENT`), instead of 50%. The score-drop stretch and the
  ceiling apply as before.

### Why 60%
Measured with `go depth 13`: the time used so far grows by about 1.2 to
2 times per depth (start position, depths 10 to 13: 1.3, 1.6, 2.4, 4.0
seconds; Kiwipete, depths 8 to 12: 2.7, 5.6, 10.0, 15.3, 25.0). Before
late move reductions it was 3 to 5 times, which is what "half the
target" was set for. A depth started at 60% now usually finishes around
1.2 times the target, and the ceiling (3 times) still stops one that
runs long.

### How it was verified
- The budget test moves its edges to 60% (599 ms starts another depth
  at a 1,000 ms target, 600 doesn't; 1,199 and 1,200 at 200%; the
  ceiling caps a 1,000% stretch at 1,799 and 1,800). Setting the
  share back to 50% makes it fail.
- 117 unit tests pass; `cargo fmt`, `clippy -D warnings` and the WASM
  build are clean. Bench unchanged at 343857.
- Selfplay: no time losses or illegal moves. The clocks ran lower than
  before (10s+0.1: 0.4 and 0.2 seconds left after 140 half-moves; 3s+0:
  about 50 ms each after 224), as expected when each move thinks a
  little longer. The sudden-death SPRT is the real check on that.
- SPRT at 8+0.08: **inconclusive**, +3.3 ± 6.7 Elo over 5,827 games
  (LLR -1.46, drifting slowly toward "not stronger"). Too small an effect
  for this test to settle quickly, and it was holding the rig, so Charlie
  stopped it in ECS after about 2.3 hours and chose to judge the rule at
  60 seconds + 0.6 instead, where deeper searches should show whether
  the per-depth cost matters.
- SPRT at 60 seconds + 0.6: **inconclusive**, +1.7 ± 9.6 Elo over 2,695
  games (LLR -1.39), stopped at the 8-hour limit (2026-10-26, about
  04:00 UTC). Neutral at both speeds, so by the rework's rule (kept only
  if it gains) its code came out again on the same PR, before the
  pacing rule was tested against main.

### What the two tests say
The cost of each new depth did fall with late move reductions, but
"start under 50% or 60%" changes when the engine stops by only a sliver
of the clock, and the two choices play about the same. Not worth a
number we can't show helps.

### Pondering (a note from the Lichess bot, 2026-10-26)
Many Lichess bots ponder (think on Stargazer's time); Stargazer doesn't
yet. Charlie's point: every second Stargazer overspends is also free
thinking time for such an opponent, so wasted time costs it twice in
real games. Self-play SPRTs can't see that, since neither side ponders;
a Stockfish match with pondering switched on could. The real-game value
of the time rules is probably larger than the SPRTs show, which also
argues for retrying best-move stability (the milder version is built).

## Charlie's 5+0 game against Stockfish (2026-10-25)

Charlie played Stargazer against Stockfish 15.1 in a GUI at 5 minutes, no
increment: 84 moves, lost (mated as the clock ran out). Replayed with
Stargazer at depth 12 on each position: White about +1.5 to +2 pawns
from move 20 to 44, then a slow slide (46. Kd4, where Kf4 was preferred;
0.0 by 54. a5; lost by 58. Nc2; 64. d6 the last blunder). With no
increment the engine spends a twentieth of what's left on each move:
about 15 seconds on move 1, 1.5 by move 46, 0.6 by move 64, just when
the endgame needed thinking. That's the case for a sudden-death pacing
rule (roadmap). By that arithmetic it should still have had a few
seconds at the end, so the GUI probably takes more than the 50 ms
`Move Overhead` keeps back; 200 was suggested.
It then turned out the game was played on an older build: Charlie had
pulled but not rebuilt, so the GUI ran whatever was built before. The
pacing problem stands, but the strength in that game says little about
the current engine. The play-it runbook now says to rebuild after every
pull, and how to check a build with `bench`.

## Sudden-death pacing (option D, Charlie's choice)

### What was done
- `src/eval.rs`: `game_phase`, the evaluation's count of pieces left (24
  with all on, 0 with only kings and pawns), now its own tested
  function. `evaluate` is unchanged, so bench is too.
- `src/uci.rs`: with no increment and no `movestogo`, the clock is
  shared over `sudden_death_moves_left(phase)` = 25 + phase, at most 45,
  instead of 20; the ceiling is held to a tenth of the clock.

### The choice
Claude first recommended a fixed fortieth (A). Charlie asked how time
should really be spread (middlegame most, then endgame, then opening),
which favoured an estimate of the moves left. Two versions were drafted
and compared on Charlie's 5+0 game: by move number (B) and by material
(D). Replayed on that game, both give about 3 to 5 seconds a move in the
part that was lost (moves 46 to 64) instead of 0.6 to 2.6; D spends
most as the trades happen (5.4 s at move 30, 5.1 s at move 40), B keeps
a little more for a very long ending (1.8 s against 1.3 s at move 84).
Charlie chose D. B is kept on a local branch for a head-to-head test.

### How it was verified
- New tests: the phase count (full board 24, a queen and rook each 12,
  knight against bishop 2, kings and pawns 0, extra queens capped at
  24); the moves-left estimate (45, 37, 27, 25); the budget with every
  piece on (1,333 ms of a minute, ceiling 3,999) and with only pawns
  (2,400 ms, ceiling held to 6,000, a tenth). Each fails when its rule
  is broken.
- 119 unit tests pass; `cargo fmt`, `clippy -D warnings` and the WASM
  build are clean. Bench unchanged at 343857.
- Selfplay: no time losses. At 3 seconds + 0 a 300-ply game (the
  selfplay limit) ended with about 0.3 seconds on each clock on its own,
  and with 37 and 49 ms on top of rule 4. Any "share of what's left"
  rule shrinks the clock geometrically in a very long game; the 50 ms
  move overhead is what keeps it from flagging, and the 10 seconds + 0
  SPRT counts any time loss as a loss.
- SPRT at 10 seconds + 0 against main (rule 4 had been taken out by
  then, see below): **failed**, -8.2 ± 13.1 Elo over 1,815 games (575
  wins, 622 draws, 618 losses; LLR -2.97), 2026-10-26.

### Why it might have lost (not yet checked)
At 10 seconds a game, move 1 now gets about 0.22 seconds instead of
0.5: a full board expects 45 moves left instead of 20. In self-play
both sides then think less in the opening and middlegame, and the old
version, thinking 2.25 times longer there, makes better early
decisions, which engines convert. The endgame time the rule buys came
too late to pay for that. Charlie's lost 5+0 game was also played on an
older build, so it overstated how badly the endgame was starved.

## A new bar for time management (2026-10-26)
Charlie's point: rule 4 and D are both close to zero at the short
controls we test, but the engine is meant to be good at Lichess time
controls (minutes), where the advantage might show and which we can't
afford to test (a 5+0 SPRT would take about 30 hours). Agreed: a
time-management change now only has to show no harm ([-10, 0] Elo),
then pass a 60-second confirmation, and a change that only ties needs a
reason we can articulate (Charlie broadened Claude's stricter draft;
DECISIONS.md, 2026-10-26).

- D re-run as a no-harm test at 60 seconds + 0 against main, PR #35:
  **passed**, +0.9 ± 8.7 Elo over 3,707 games (1,115 wins, 1,484 draws,
  1,108 losses; LLR 3.01). D stays. The report still said "stronger",
  since it ran from main before the wording fix; read it as "no weaker".
- `rig/report.py` worded every pass as "stronger"; a test with an upper
  bound of 0 or less now reads "no weaker" / "weaker". New test
  `test_a_no_harm_test_says_no_weaker`, seen failing with the wording
  switched off, then passing (22 rig tests). The report workflow runs
  from main, so the new wording shows only after #35 merges.

## The milder stability rule (2026-10-27)
With #35 merged, the parked milder rule 2 was rebuilt on main (PR #44):
`Stability` in `src/uci.rs` ignores depths below 6, gives 75% once the
best move has held for 4 more depths and 150% right after it changes;
`ScoreWatch` (rule 3) wins when the score has just fallen. New test
`a_settled_best_move_gets_less_time_and_a_new_one_more`, seen failing
with depths 1 to 5 counted again, then passing (121 tests). Bench is
unchanged (343857): this only decides when to stop, which bench, a
fixed-depth search, never asks.

SPRTs under the no-harm bar, [-10, 0] at 8+0.08:
- First launch: AWS had no Fargate Spot capacity; the workflow gave up
  after 3 tries and posted nothing on the PR, so it went unnoticed for
  six hours. Lesson: check that a test started, not just that it was
  launched.
- Second: stopped by AWS (SIGTERM, almost certainly the Spot machine
  taken back) at 1,692 games, -4.5 ± 11.2 Elo, no verdict.
- Third: **passed**, +6.2 ± 12.0 over 1,731 games (559 wins, 645 draws,
  527 losses; LLR 2.97). The first report with the "no weaker" wording.
- Then the 60 seconds + 0.6 confirmation: see the PR and the roadmap.
