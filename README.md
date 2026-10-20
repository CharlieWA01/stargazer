# Stargazer

A chess engine written in Rust, named after an astronomer-diviner who reads the
future in the stars, which is roughly what a search algorithm does.

Stargazer speaks [UCI](https://www.chessprogramming.org/UCI), so it works with
any standard chess GUI (Arena, Cute Chess, En Croissant, BanksiaGUI) and,
eventually, as a Lichess bot.

## Status

**Pre-alpha: Milestone 1 (Foundation) done.** Stargazer knows every rule of
chess, proven by perft on six standard positions, but doesn't choose moves
yet; search and evaluation are Milestone 2. The architecture is in
[`docs/DESIGN.md`](docs/DESIGN.md).

## Building and running

You need [Rust](https://rustup.rs) installed.

```sh
cargo build --release          # build the engine
./target/release/stargazer     # run it, then type: uci, d, go perft 5
cargo test                     # run the tests
```

## Docs

- [`docs/DESIGN.md`](docs/DESIGN.md): architecture decisions, open and settled
- [`docs/DECISIONS.md`](docs/DECISIONS.md): every decision, with alternatives and reasons
- [`docs/ROADMAP.md`](docs/ROADMAP.md): milestones and the release naming scheme
- [`docs/PORTFOLIO.md`](docs/PORTFOLIO.md): one-page summary
- [`CLAUDE.md`](CLAUDE.md): how this project is developed, and context for AI sessions
