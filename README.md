# Stargazer

A chess engine written in Rust, named after an astronomer-diviner who reads the
future in the stars, which is roughly what a search algorithm does.

Stargazer speaks [UCI](https://www.chessprogramming.org/UCI), so it works with
any standard chess GUI (Arena, Cute Chess, En Croissant, BanksiaGUI) and,
eventually, as a Lichess bot.

## Status

**Milestone 4 (Getting strong) under way.** Stargazer plays full games in
any UCI GUI under a real clock (first playable version: Polaris). Every
rule of chess is proven by perft, and every strength change is measured by
an SPRT match on AWS before it merges. The architecture is in
[`docs/DESIGN.md`](docs/DESIGN.md); progress is in
[`docs/ROADMAP.md`](docs/ROADMAP.md).

## Building and running

You need [Rust](https://rustup.rs) installed.

```sh
cargo build --release          # build the engine
./target/release/stargazer     # run it, then type: uci, position startpos, go movetime 2000
./target/release/stargazer bench   # fixed search; prints its node count and speed
cargo test                     # run the tests
```

## Docs

- [`docs/DESIGN.md`](docs/DESIGN.md): architecture decisions, open and settled
- [`docs/DECISIONS.md`](docs/DECISIONS.md): every decision, with alternatives and reasons
- [`docs/ROADMAP.md`](docs/ROADMAP.md): milestones and the release naming scheme
- [`docs/PORTFOLIO.md`](docs/PORTFOLIO.md): one-page summary
- [`docs/runbooks/`](docs/runbooks/): how to play it, run perft and run a strength test
- [`CLAUDE.md`](CLAUDE.md): how this project is developed, and context for AI sessions

## Licence

Copyright (C) 2026 Charlie (CharlieWA01).

Stargazer is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option)
any later version. It is distributed in the hope that it will be useful,
but without any warranty; see [`LICENSE`](LICENSE) for the full terms.
