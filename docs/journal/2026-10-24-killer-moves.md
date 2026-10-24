# 2026-10-24: Killer moves (Milestone 4, step 3a)

## Goal
Sort the quiet moves better, starting with killer moves. Charlie chose
two PRs (killers, then history), each with its own SPRT; the design is
in DESIGN.md section 3 and DECISIONS.md (2026-10-24).

## What was done
- `src/search.rs`: the searcher keeps two killer slots per ply. When a
  quiet move refutes a position (the alpha-beta cut-off), it becomes
  that ply's newest killer and the older one moves to the second slot;
  a move already in the first slot isn't stored twice. Captures and
  promotions are never stored, since they are already tried early.
- Move order is now: last depth's best move or the table's, captures and
  promotions (MVV-LVA), killer 1, killer 2, then the other quiet moves.
  A killer that happens to capture in the new position keeps its place
  among the captures.
- Killers live for one `go` (kept across its depths); quiescence doesn't
  use them.

## Problems and fixes
- `tools/selfplay.py` lost one game on time, at 3 s with no increment,
  at half-move 255 (36 ms against −1 ms). The same build passed all
  three games on the next run, and `main` passes too, but in shorter
  games. This is the known sudden-death drain, not the killers: with no
  increment the clock shrinks about 5% a move, so by move 120 there are
  a few milliseconds left, and depth 1 always finishes even past the
  limit (ROADMAP.md, known limitations). The killers changed which game
  was played, and this one ran long. Left for the time rework, which
  already lists sudden death.

## How it was verified
- Three new tests: killers are tried after the captures and before other
  quiet moves; a killer that captures here is ranked as a capture; only
  quiet refutations become killers, newest first, no duplicates, each
  ply separate. Breaking each rule in turn (storing captures; ranking a
  capturing killer as a killer) fails its test; restored, 103 pass. The
  capture test first used a position with only two captures, where the
  broken version still passed, so a third capture (Rxa5) was added.
- Positions searched by bench (main → killers):

  | Depth | Before | After | Saved |
  |---|---|---|---|
  | 5 | 1,275,902 | 1,162,141 | 9% |
  | 6 | 4,562,526 | 3,893,639 | 15% |
  | 7 | 19,554,844 | 17,326,568 | 11% |

  New bench fingerprint: **1162141**.
- Speed per position: median of five bench runs 405,000 (main) against
  397,000 (killers), about 2% slower; the saving in positions is larger.
- `cargo fmt`, clippy, WASM build, deep perft unchanged (move generation
  untouched).
- SPRT: to come, bounds 0 to 10 at 8+0.08 against `main`.

## Impact
The search wastes less time on quiet moves that a sibling position has
already shown to be bad: about a tenth fewer positions to reach the same
depth. Memory: about 1 KB.
