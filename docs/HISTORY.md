# History

Every change merged into `main`, oldest first. The `docs-history` workflow
appends a row after each merged pull request; don't edit rows by hand. The
story behind each change is in `docs/journal/`. See `docs/DOCS-STANDARD.md`.

| Date | Change | Title |
|---|---|---|
| 2026-10-18 | [#1](https://github.com/CharlieWA01/stargazer/pull/1) | Add project skeleton |
| 2026-10-20 | [#2](https://github.com/CharlieWA01/stargazer/pull/2) | Record goals, browser target and board design decisions |
| 2026-10-20 | [#3](https://github.com/CharlieWA01/stargazer/pull/3) | Adopt the shared documentation standard |
| 2026-10-20 | [#4](https://github.com/CharlieWA01/stargazer/pull/4) | Record move generation decisions, and add the board (Milestone 1, step 1) |
| 2026-10-20 | [#5](https://github.com/CharlieWA01/stargazer/pull/5) | Add knight, king and pawn move generation (Milestone 1, step 2) |
| 2026-10-20 | [#6](https://github.com/CharlieWA01/stargazer/pull/6) | Complete the rules of chess and prove them with perft (Milestone 1 done) |
| 2026-10-20 | [#7](https://github.com/CharlieWA01/stargazer/pull/7) | Milestone 2 decisions and the evaluation function |
| 2026-10-20 | [#8](https://github.com/CharlieWA01/stargazer/pull/8) | Search: alpha-beta, iterative deepening, quiescence and the draw rules |
| 2026-10-20 | [#9](https://github.com/CharlieWA01/stargazer/pull/9) | Clocks and stop: the search runs on its own thread |
| 2026-10-20 | [#10](https://github.com/CharlieWA01/stargazer/pull/10) | docs: Milestone 2 done, the first version is Polaris |
| 2026-10-20 | [#11](https://github.com/CharlieWA01/stargazer/pull/11) | Milestone 3: design decisions, a bench command, and the test container |
| 2026-10-20 | [#12](https://github.com/CharlieWA01/stargazer/pull/12) | Run SPRT button and SPRT report workflows |
| 2026-10-20 | [#13](https://github.com/CharlieWA01/stargazer/pull/13) | Faster ARM test image (cross-compile fastchess), and an ARM build check |
| 2026-10-20 | [#14](https://github.com/CharlieWA01/stargazer/pull/14) | Run SPRT: a rounds input, and per-move nodes/speed/latency in game files |
| 2026-10-21 | [#15](https://github.com/CharlieWA01/stargazer/pull/15) | docs: the test rig is proven fair; Milestone 3 done |
| 2026-10-23 | [#16](https://github.com/CharlieWA01/stargazer/pull/16) | Search: keep the best move from a depth that runs out of time |
| 2026-10-23 | [#17](https://github.com/CharlieWA01/stargazer/pull/17) | Run SPRT: print AWS's reason and retry a refused start; cache the ARM cross-compiler |
| 2026-10-23 | [#18](https://github.com/CharlieWA01/stargazer/pull/18) | Zobrist fingerprints for positions (transposition table, step 2a) |
| 2026-10-23 | [#19](https://github.com/CharlieWA01/stargazer/pull/19) | Transposition table: the search remembers positions (step 2b) |
| 2026-10-23 | [#20](https://github.com/CharlieWA01/stargazer/pull/20) | Docs sweep: fix stale status, layout, speed and runbook notes |
| 2026-10-23 | [#21](https://github.com/CharlieWA01/stargazer/pull/21) | Killer moves: try last refutations right after the captures (step 3a) |
| 2026-10-23 | [#22](https://github.com/CharlieWA01/stargazer/pull/22) | History moves: order quiet moves by how well they have done (step 3b) |
| 2026-10-23 | [#23](https://github.com/CharlieWA01/stargazer/pull/23) | Principal variation search: prove later moves worse cheaply (step 4a) |
| 2026-10-23 | [#24](https://github.com/CharlieWA01/stargazer/pull/24) | Docs: record the PVS SPRT pass (+18.8 Elo) |
| 2026-10-24 | [#25](https://github.com/CharlieWA01/stargazer/pull/25) | Aspiration windows: search each depth near the last score |
| 2026-10-24 | [#26](https://github.com/CharlieWA01/stargazer/pull/26) | Run match: play Stockfish at a set strength for a rough rating |
| 2026-10-24 | [#27](https://github.com/CharlieWA01/stargazer/pull/27) | Docs: record the first Stockfish match (about 1900) |
| 2026-10-24 | [#28](https://github.com/CharlieWA01/stargazer/pull/28) | Null-move pruning (R = 3 + depth/4) |
| 2026-10-24 | [#29](https://github.com/CharlieWA01/stargazer/pull/29) | Docs: Stockfish match after null-move pruning (about 2030) |
| 2026-10-24 | [#30](https://github.com/CharlieWA01/stargazer/pull/30) | Check extensions (step 6a) |
| 2026-10-24 | [#31](https://github.com/CharlieWA01/stargazer/pull/31) | Late move reductions (step 6b) |
| 2026-10-24 | [#32](https://github.com/CharlieWA01/stargazer/pull/32) | Docs: Stockfish match after LMR (about 2180) |
| 2026-10-25 | [#33](https://github.com/CharlieWA01/stargazer/pull/33) | Time rule 1: lone legal move played at once, Move Overhead option (step 7a) |
| 2026-10-25 | [#34](https://github.com/CharlieWA01/stargazer/pull/34) | Time management: best-move stability dropped (SPRT -24.5), score drop kept (SPRT +10.9) |
| 2026-10-25 | [#36](https://github.com/CharlieWA01/stargazer/pull/36) | Lichess bot decisions, GPL-3.0 licence, and prepare the repo for going public |
