# Runbook: prove move generation with perft

Perft counts every possible sequence of moves to a fixed depth. The
correct counts for six standard positions are published (Chess
Programming Wiki, "Perft Results"), so a wrong count means a rules bug.

## Every PR (automatic)
CI runs `cargo test`, which includes `perft_shallow`: all six positions to
depth 4 (about 10 seconds in a debug build).

## Before a release (by hand)
```sh
cargo test --release -- --ignored
```
Runs `perft_deep`: depth 5 for every position and depth 6 where the
published table has it (start position, position 3): about 600 million
positions, roughly 35 to 60 seconds. Expected: `1 passed`.

## Trying it yourself
```sh
cargo run --release
position startpos
go perft 5
```
Prints the count after each first move, then `Nodes searched: 4865609`.
`go perft 6` from the start position should give `119060324` (about 7
seconds on a cloud machine). For speed, use `stargazer bench` instead.

## When a count is wrong
1. Run the same position and depth in a trusted engine, used only as a
   reference (Stockfish: `position fen ...` then `go perft N`).
2. Compare the per-move lists. Find the first move whose count differs.
3. Play that move (`position fen ... moves <move>`), go one depth lower,
   and compare again. Repeat until depth 1: the difference is then a
   single missing or extra move, which names the bug.
