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
setoption name Hash value 64   # memory for the transposition table, in MB (default 16)
setoption name Move Overhead value 200   # ms kept back each move for a slow GUI or connection (default 50)
eval                  # the evaluation, from White's side
d                     # print the board
quit
```
Each `info` line is one finished depth: the score (`cp` = centipawns for
the side to move, or `mate N`), positions searched, speed, and the line it
expects (`pv`). `bestmove` is its choice.

## Against it in a GUI
Any UCI GUI works. Build with `cargo build --release`, then add a new
engine in the GUI pointing at `target/release/stargazer` (no arguments,
protocol UCI). Pick a time control and play. The engine uses the clock the
GUI sends (`wtime`, `btime`, `winc`, `binc`).

**After every `git pull`, build again** (`cargo build --release`): the
pull only updates the source; the GUI runs whatever was last built. To
check which build it is, `./target/release/stargazer bench` prints a node
count that fingerprints the search (compare it with the latest entry in
`docs/HISTORY.md` or the roadmap), and a release build runs at a few
hundred thousand positions a second.

### On Charlie's Chromebook (Linux, Debian 12): PyChess
```sh
sudo apt install pychess
pychess          # start it from the terminal so errors show
```
Edit → Engines → New, pick `~/stargazer/target/release/stargazer`; then a
new game with Stargazer as the opponent. Fallback, plainer but dependable:
`sudo apt install xboard polyglot`, then
`xboard -fUCI -fcp ~/stargazer/target/release/stargazer` (Stargazer plays Black).

What didn't work there (2026-10-20):
- Cute Chess: not in Debian's package list.
- En Croissant 0.15.0 (`.deb` from its GitHub releases): installs, but won't
  start: `GLIBC_2.39 not found`. It was built for a newer Linux than
  Debian 12. Remove with `sudo apt remove en-croissant`.
- The Linux side of a Chromebook has no `~/Downloads`; download into the
  home folder (not into a repo, or git sees the file).

## Self-play check (after any change to search, time or UCI)
```sh
cargo build --release
python3 tools/selfplay.py target/release/stargazer
```
Plays one game at each of 10s+0.1s, 3s and 1s+0.01s and prints how many
half-moves each lasted and the time left. Expected: exit code 0 and no
`PROBLEM` lines (a problem is an illegal move or a loss on time). Takes
about a minute. It checks that the engine behaves, not how strong it is.
