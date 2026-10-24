# 2026-10-24: Principal variation search (Milestone 4, step 4a)

## Goal
Use the better move ordering from the table, killers and history to
search less: principal variation search (PVS). Charlie chose two PRs,
PVS first and aspiration windows second (DECISIONS.md, 2026-10-24).

## What was done
- `src/search.rs`: in each position the first move is searched as
  before. Every later move is first asked only "are you better than the
  best so far?", with a zero-width window (alpha, alpha + 1). A "no" is
  cheap to prove: one good reply is enough. A "yes" that could change
  the result here is searched again with the full window, to find out by
  how much.
- At the root, if time runs out during that second search, the move is
  still played: the quick test already proved it beats the old best.
  This is the same idea as keeping an unfinished depth's finished moves
  (#16).
- `search_with` carries a switch for PVS. `search` always turns it on;
  only a test turns it off.

## Problems and fixes
- The first version broke `a_better_move_found_in_an_unfinished_depth_is_played`.
  With PVS, c4-c5 at depth 5 was in its second, full search when time
  ran out, so it wasn't credited and d2-d4 was played. Since the quick
  test had already shown c4-c5 is better, the root now plays such a
  move. The test passes again unchanged.

## How it was verified
- New test: on five positions (the start position, Kiwipete, a Queen's
  Gambit middlegame, perft position 3, a mate in two), with no
  transposition table, every depth's score is the same with and without
  PVS, and PVS searches fewer positions in total. It fails if the second
  search is left out (Kiwipete depth 3: −7 vs −8) and if PVS is switched
  off everywhere (same count). 107 pass.
- Positions searched by bench (main → PVS):

  | Depth | Before | After | Saved |
  |---|---|---|---|
  | 5 | 1,153,324 | 987,309 | 14% |
  | 6 | 3,730,681 | 3,052,912 | 18% |
  | 7 | 16,819,905 | 12,949,314 | 23% |

  New bench fingerprint: **987309**. Speed per position unchanged
  (median of five runs: 390,000 main, 398,000 PVS; noise).
- `cargo fmt`, clippy, WASM build, deep perft; `tools/selfplay.py` three
  games, no problems.
- SPRT (2026-10-24), bounds 0 to 10 at 8+0.08 against `main`: **pass,
  +18.8 ± 13.3 Elo over 1,572 games** (618 wins, 421 draws, 533 losses;
  run `sprt-40054595706-1`, verdict collected with SPRT report after the
  30-minute wait). #23 was merged by mistake about 20 minutes before the
  verdict. Both engines had already been built, so the test still
  compared PVS against the `main` before it, and the pass makes the
  merge good. The result is recorded here in a follow-up docs PR.

## Impact
About a fifth less work to reach the same depth, more the deeper it
goes. No memory added. The test-only switch is a few lines.
