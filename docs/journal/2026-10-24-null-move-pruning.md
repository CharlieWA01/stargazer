# 2026-10-24: Null-move pruning

## Goal
Milestone 4, step 5: null-move pruning, with the settings Charlie chose
(DECISIONS.md, 2026-10-24, "Null-move pruning"): the pass is searched
R = 3 + depth/4 half-moves shallower, and it's skipped when the side to
move has only a king and pawns.

## What was done
- `src/board.rs`: `make_null_move` hands the move to the other side
  without moving anything. It clears any en passant square (a pass ends
  that chance) and updates the fingerprint to match. It also resets the
  50-move counter, so the repetition check never looks back past the
  pass: a position from before a pass isn't a real repeat.
  `has_pieces_besides_pawns` tells the search when to skip it.
- `src/search.rs`, before trying any moves: if the position already
  looks good enough (its evaluation is at least beta), the side to move
  passes, and the opponent's reply is searched 1 + R half-moves less
  deep with a zero-width window. If the side is *still* good enough, the
  opponent would never allow this position, so the search stops here and
  returns beta.
- It's skipped: at the root, below depth 3, in the main line (only
  zero-width windows, as PVS gives every later move), right after another
  pass, in check, when beta is a mate score, and with only king and
  pawns.
- A cutoff by a pass is remembered in the transposition table as "at
  least beta", so the next depth of iterative deepening doesn't have to
  ask again. This came out of the measurements below: it saved about 2%
  more positions.

## Problems and fixes
- **R in the design note was described wrongly.** The note sent to
  Charlie said R = 3 + depth/4 meant "3 at depths 3 to 7 and 4 at 8 to
  11". The formula (rounding down) actually gives 3 at depth 3, 4 at
  depths 4 to 7 and 5 at 8 to 11. The build uses the formula Charlie
  picked; the milder version the note described (R = 2 + depth/4, at
  least 3) could be tried as its own SPRT.
- **The first version of the "saves work" test failed**: at depth 6 its
  three positions searched slightly *more* with null moves. One of them
  was Kiwipete, a position packed with tactics where passing usually
  fails (the opponent's threats are real), so the pass searches are pure
  cost: 1.2 million positions instead of 0.8 million. In calm positions
  the saving is large (3 to 3.5 times fewer). The test now uses the
  start position and two calm middlegames, and the comment says why.
- At bench's own depth (5) the build searches about 2% *more* positions
  (942,397 against 924,579): at shallow depth the pass searches cost
  about as much as they save. The saving grows fast with depth (table
  below), and real games reach depth 7 to 10.
- The first saving test ran at depth 7, which took six minutes in a
  debug build. Depth 6 gives a clear answer in under a minute.

## How it was verified
- Unit tests: 111 pass. New ones: a pass only hands the move over (the
  FEN after it, its fingerprint matches one computed from scratch, the
  board stays consistent); "pieces besides pawns" for king and pawns
  with and without a rook; and null moves search at most 70% as many
  positions on the three calm positions at depth 6. Each fails when the
  code it protects is broken (null moves switched off; the fingerprint's
  side-to-move change removed; the king counted as a piece).
- Bench (positions searched, from a fresh table each position):

  | Depth | Before (main) | With null moves | Change |
  |---|---|---|---|
  | 5 (bench) | 924,579 | 942,397 | +2% |
  | 6 | 2,994,075 | 2,019,859 | −33% |
  | 7 | 12,914,603 | 5,087,917 | −61% |

  The new bench fingerprint is **942397**.
- Depth reached in one second: start position 8 (was 7), a calm
  middlegame 6 (was 5), Kiwipete 4 (unchanged).
- The zugzwang guard: on bench's king-and-pawn ending the node count
  is identical with and without null moves (the guard switches it
  off). Ten king-and-pawn endgames searched to depth 12 with and
  without the guard all chose sensible moves, so this didn't show a
  case where it matters; it's a safety net, as in other engines.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, the WASM
  build, and `tools/selfplay.py` (three games under 10+0.1, 3+0 and
  1+0.01: no time losses, no illegal moves) all pass.
- SPRT against `main` (0 to 10, 8+0.08, 16 vCPU): **pass**, +73.2 ±
  29.1 Elo over 392 games (188 wins, 98 draws, 106 losses), the biggest
  gain since the transposition table.

## Impact
The engine skips most of the hopeless positions in its search, reaching
about one depth further in the same time in ordinary positions. The
risk is zugzwang (where passing would be best): covered for king-and-pawn
endings, not for rare middlegame cases. A "verification search" is the
known fix, parked for later.
