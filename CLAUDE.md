# Stargazer: project context

Stargazer is a UCI chess engine written in Rust, built from scratch (no
borrowed engine code).

## How we work

- **Charlie is the architect and infrastructure engineer.** Charlie makes the
  design decisions, owns the repo, CI, testing harness and deployment, and
  decides what gets merged. Charlie knows some basic Python, not Rust.
- **Claude writes the code.** Explain choices in plain language and chess
  terms, not jargon. When there's a real design choice, lay out the options
  and tradeoffs and give a recommendation, but let Charlie decide.
- **Architecture before code.** Don't implement a component until its section
  in `docs/DESIGN.md` is marked **Decided**. If a decision is needed mid-task,
  stop and ask instead of picking one silently.
- **Prove every change.** Correctness is proven with perft (move generation)
  and unit tests. Strength changes are proven with SPRT matches against the
  previous version. "It seems stronger" isn't evidence.

## Code conventions

- Comment in chess terms so a non-Rust reader can skim a file and know what it
  does ("knights can jump to these squares", not "shift and mask").
- Keep high-level code (search, evaluation) readable. Low-level speed tricks
  (bitboards, magic numbers) are fine, but isolate and test them.
- No external crates without asking. The engine core should stay
  dependency-free; dev tooling can be more relaxed.
- `cargo fmt` and `cargo clippy -- -D warnings` must pass. CI enforces both.

## Commands

```sh
cargo build --release              # optimized build (always benchmark this one)
cargo test                         # unit tests
cargo fmt                          # format
cargo clippy --all-targets -- -D warnings
```

## Layout

- `src/main.rs`: entry point, hands stdin/stdout to the UCI loop
- `src/lib.rs`: library root (engine name, version, modules)
- `src/uci.rs`: UCI protocol handling (handshake only, so far)
- `docs/DESIGN.md`: architecture decisions; the source of truth
- `docs/ROADMAP.md`: milestones and release names
