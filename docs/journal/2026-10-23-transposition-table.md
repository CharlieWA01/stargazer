# 2026-10-23: The transposition table

## Goal
Milestone 4, step 2b: give the search a memory of positions it has
already searched, using the fingerprints from step 2a (#18), as designed
and approved earlier the same day (DECISIONS.md 2026-10-23).

## What was done
- `src/tt.rs`: the table. One slot is 16 bytes: the full fingerprint,
  the best move, the score (as a 16-bit number; scores never go past
  ±32,000), the depth and whether the score is exact, a lower bound or an
  upper bound. The fingerprint picks the slot by scaling it down to the
  number of slots, which works for any table size. Always replace. Mate
  scores are stored counted from the position itself and converted back
  when read, so a mate found by one route is right when reached by
  another, nearer or further from the root.
- `src/search.rs`: before searching a position's moves, look it up. If it
  was searched at least as deep before and the stored score settles the
  question (exact, or a bound on the right side of alpha or beta), return
  it; never at the root. Otherwise try its remembered best move first
  (after last iteration's best line, which is usually the same move).
  After searching, store the result with the right bound, unless the
  search was stopped partway (that result would be wrong). Not used in
  quiescence for now.
- `src/uci.rs`: the table lives in the UCI loop and is lent to each
  search, so what one move learned helps the next. `ucinewgame` clears
  it. `uci` now lists `option name Hash type spin default 16 min 1 max
  1024`, and `setoption name Hash value N` makes a new table of N MB.
- `src/bench.rs`: a 16 MB table, cleared before each of the ten
  positions so each count depends only on its position.

## Problems and fixes
- The new "the table saves work" test first expected half the positions
  at depth 4 in Kiwipete; the real saving there is about a quarter
  (126,895 against 173,474 with a one-slot table). The saving grows with
  depth (below), so the test now asks only for a clear saving.
- Two known shortcuts, both standard: a stored score can hide a draw by
  repetition that only exists along one route to the position, and the
  best line shown in `info` stops where a stored exact score was used.
  Neither affects the move chosen at the root in practice.

## How it was verified
- New tests (100 pass): a slot is 16 bytes; remember, miss on a
  different position in the same slot, clear; the newest result
  replaces the old one; mate scores are converted correctly in both
  directions; a real table searches fewer positions than a one-slot one,
  and fewer again on a second search of the same position; the `Hash`
  option is listed, parsed and clamped, and a search works after
  resizing. Breaking the mate conversion fails two tests. All the
  earlier mate, draw and stop tests still pass with the table in use.
- Bench (positions searched, all ten positions):

  | depth | before | with the table | fewer |
  |---|---|---|---|
  | 5 | 2,457,379 | 1,275,902 | 48% |
  | 6 | 12,544,532 | 4,562,526 | 64% |
  | 7 | 73,875,803 | 19,554,844 | 74% |

  Depth 7 takes about 41 seconds instead of 145 on the session machine.
  The new bench fingerprint is **1275902**.
- `tools/selfplay.py`: three games, no time losses or illegal moves.
  Deep perft, `cargo fmt`, clippy, WASM build.
- SPRT (2026-10-24), bounds 0 to 10, against `main` from before #18
  (`f21fba3`) so it measures fingerprints and table together (Charlie's
  call): **pass, +135.7 ± 36.9 Elo over 235 games** (132 wins, 57 draws,
  46 losses; run `sprt-40043985301-1`). The biggest gain so far, decided
  in under a quarter of an hour.
- The first try at that SPRT failed in 45 seconds without reaching AWS:
  the base was given as the short commit ID `f21fba3`, and the checkout step
  treats a short ID as a branch name. Rerun with the full 40-character
  ID; the SPRT runbook now says so.

## Impact
The engine reaches the same depth three to four times faster, so in the
same time it looks about one half-move deeper. Memory: 16 MB per engine
by default.
