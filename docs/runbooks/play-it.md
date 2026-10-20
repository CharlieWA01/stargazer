# Runbook: play Stargazer, or watch it play itself

## By hand, in a terminal
```sh
cargo run --release
```
Then type UCI commands, for example:
```
position startpos moves e2e4 e7e5
go movetime 2000      # think for 2 seconds
go depth 6            # or: search 6 half-moves deep
go infinite           # think until you type: stop
eval                  # the evaluation, from White's side
d                     # print the board
quit
```
Each `info` line is one finished depth: the score (`cp` = centipawns for
the side to move, or `mate N`), positions searched, speed, and the line it
expects (`pv`). `bestmove` is its choice.

## Against it in a GUI
Any UCI GUI works: Cute Chess, Arena, Banksia, En Croissant. Build with
`cargo build --release`, then add a new engine in the GUI pointing at
`target/release/stargazer` (no arguments, protocol UCI). Pick a time
control and play. The engine uses the clock the GUI sends (`wtime`,
`btime`, `winc`, `binc`).

## Self-play check (after any change to search, time or UCI)
```sh
cargo build --release
python3 tools/selfplay.py target/release/stargazer
```
Plays one game at each of 10s+0.1s, 3s and 1s+0.01s and prints how many
half-moves each lasted and the time left. Expected: exit code 0 and no
`PROBLEM` lines (a problem is an illegal move or a loss on time). Takes
about a minute. It checks that the engine behaves, not how strong it is.
