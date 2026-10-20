# 2026-10-20: Full rules and perft (Milestone 1, steps 3 and 4)

## Goal
Finish the rules of chess: bishops, rooks and queens, castling, check,
legal moves and playing a move. Then prove it all with perft, which
completes Milestone 1.

## What was done
- **Sliding pieces** (`attacks.rs`): ray walking, as decided. Step along
  each direction until the edge or the first piece (which is included, as
  it may be captured). Exposed as `bishop_attacks`, `rook_attacks` and
  `queen_attacks`, so magic bitboards could replace the inside later.
- **Attacks and check** (`board.rs`): `is_attacked(square, by)` looks
  outward from the square ("if a knight stood here, would it see an enemy
  knight?"), which works because attacks are symmetrical. `in_check` uses it.
- **Playing a move** (`board.rs`): `make_move` returns a new position and
  leaves the old one untouched (copy-make). It handles captures,
  promotion, en passant (removing the pawn beside, not ahead), castling
  (moving the rook across), the en passant square after a double push,
  losing castling rights when a king or rook moves or a rook is captured
  in its corner, and both move counters.
- **Castling** (`movegen.rs`): the right must remain, the squares between
  must be empty, and the king may not be in check or cross an attacked
  square. A FEN that claims a right without the king and rook in place is
  ignored rather than trusted.
- **Legal moves:** `legal_moves` plays each pseudo-legal move on a copy
  and keeps it only if our king isn't attacked afterwards.
- **Perft** (`perft.rs`): `perft` and `divide`, with the six standard
  positions as tests: depth 4 on every PR, full depth (5, or 6 where
  published) as an ignored test for releases. Runbook:
  `docs/runbooks/perft.md`.
- **UCI:** `position ... moves e2e4 ...` now plays the moves (illegal ones
  are refused with an `info string`), and `go perft N` prints the divide
  list and total.

## Problems and fixes
- **A test with the wrong arithmetic.** The ray-walking test first
  expected 12 squares for a rook with two blockers; the right answer is 10
  (3 down, 2 up, 3 left, 2 right). The code was right; the test comment
  now spells out the count.
- **Depth 3 wasn't enough.** Breaking en passant on purpose passed every
  position at depth 3; only Kiwipete at depth 4 caught it. So the per-PR
  test runs depth 4 (see DECISIONS.md and the interview story).

## How it was verified
- All six standard positions match the published counts at every depth,
  including 119,060,324 at depth 6 from the start and 193,690,690 for
  Kiwipete at depth 5 (`cargo test --release -- --ignored`: 34 seconds).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` (55 tests plus the ignored deep one) and the WebAssembly
  build pass.
- Proved the perft tests can fail: letting the king castle through an
  attacked square failed Kiwipete at depth 2; leaving the en passant
  victim on the board failed Kiwipete at depth 4. Both restored.
- `go perft 6` from the start position in the release binary: the right
  total in about 7 seconds (about 16 million positions a second).

## Impact
Milestone 1 is done: Stargazer knows every rule, proven. Next is
Milestone 2 (it plays), which needs DESIGN.md sections 3 (search) and 4
(evaluation) decided first.
