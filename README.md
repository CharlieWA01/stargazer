# Stargazer

A chess engine written in Rust, named after an astronomer-diviner who reads the
future in the stars, which is roughly what a search algorithm does.

Stargazer speaks [UCI](https://www.chessprogramming.org/UCI), so it works with
any standard chess GUI (Arena, Cute Chess, En Croissant, BanksiaGUI) and,
eventually, as a Lichess bot.

## Status

**Pre-alpha: project skeleton.** The engine answers the UCI handshake but does
not play chess yet. Architecture decisions are being worked out in
[`docs/DESIGN.md`](docs/DESIGN.md).

## Building and running

You need [Rust](https://rustup.rs) installed.

```sh
cargo build --release          # build the engine
./target/release/stargazer     # run it, then type: uci
cargo test                     # run the tests
```

## Docs

- [`docs/DESIGN.md`](docs/DESIGN.md): architecture decisions, open and settled
- [`docs/ROADMAP.md`](docs/ROADMAP.md): milestones and the release naming scheme
- [`CLAUDE.md`](CLAUDE.md): how this project is developed, and context for AI sessions
