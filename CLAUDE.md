# Stargazer: project context

Stargazer is a UCI chess engine written in Rust, built from scratch (no
borrowed engine code).

Start of a session: read `docs/ROADMAP.md`, `docs/DESIGN.md` and
`docs/DECISIONS.md` before proposing work.

## Documenting is part of every task

Claude's memory does not carry over between sessions, so this repo is the
memory. Follow `docs/DOCS-STANDARD.md` (shared with Charlie's other project
repos). A task isn't done until its docs are updated **in the same PR**:

1. **Journal:** add or extend `docs/journal/YYYY-MM-DD-<topic>.md` using the
   template in `docs/journal/README.md`.
2. **Decisions:** real choices go in `docs/DECISIONS.md` (newest at the
   bottom; correct old entries by marking them superseded), and the matching
   `docs/DESIGN.md` section is updated.
3. **Roadmap:** tick off finished items; park side ideas; note limitations.
4. **Interview stories:** add or extend one in `docs/interview/README.md`
   when the work makes a good story.
5. **Runbooks:** repeatable procedures (perft, SPRT, releases) go in
   `docs/runbooks/`.
6. **Portfolio:** update `docs/PORTFOLIO.md` when the headline facts change.

Never edit `docs/HISTORY.md` by hand; a workflow appends to it after each
merge. The `docs-check` workflow fails PRs that change code without touching
`docs/`; fill in the PR template's checklist. End every GitHub comment Claude
posts with `— Claude` (comments appear under Charlie's account).

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
cargo build --lib --target wasm32-unknown-unknown   # the browser build must keep compiling
python3 tools/selfplay.py target/release/stargazer  # after search, time or UCI changes
python3 -m unittest discover -s rig -v              # after changing the test rig
./target/release/stargazer bench   # node count must not change for speed-only changes
```

## Layout

- `src/main.rs`: entry point, hands stdin/stdout to the UCI loop
- `src/lib.rs`: library root (engine name, version, modules)
- `src/bench.rs`: `bench`, a fixed search whose node count fingerprints a
  build (`stargazer bench` or the UCI command)
- `src/bitboard.rs`: sets of squares as 64-bit numbers; the bit tricks live here
- `src/board.rs`: squares, pieces and a position (hybrid: bitboards plus a
  64-square array), read from and written as FEN
- `src/attacks.rs`: ready-made tables of knight, king and pawn attacks
  (built at compile time) and ray walking for bishops, rooks and queens
- `src/movegen.rs`: the `Move` type, pseudo-legal moves for every piece
  (castling included), and `legal_moves`
- `src/eval.rs`: scores a position (material plus where each piece stands,
  blended between middlegame and endgame); tables built from chess rules
- `src/perft.rs`: perft and divide, with the six standard positions as tests
  (deep run: `cargo test --release -- --ignored`; see `docs/runbooks/perft.md`)
- `src/zobrist.rs`: position fingerprints (Zobrist hashing), for repeats
  and the transposition table
- `src/tt.rs`: the transposition table (the search's memory of positions)
- `src/search.rs`: choosing a move (alpha-beta, iterative deepening,
  quiescence, mate and draw rules, the transposition table, killer and
  history moves, principal variation search, aspiration windows,
  null-move pruning); clock-free,
  asks `should_stop`
- `src/uci.rs`: UCI protocol handling (handshake, `position ... moves`,
  `go` with clocks, `movetime`, `depth`, `infinite`, `stop`; `go perft N`,
  `d` to print the board, `eval`, `setoption name Hash`, `ucinewgame`);
  the search runs on its own thread
- `rig/`: the SPRT test container (`Dockerfile`, `run_test.py`), the
  report the workflows post (`report.py`), and their tests; started by the
  "Run SPRT" and "SPRT report" workflows; see `docs/runbooks/sprt.md`.
  Its match mode plays Stockfish at a set strength ("Run match";
  `docs/runbooks/stockfish-match.md`)
- `tools/selfplay.py`: engine-vs-itself games under a clock (no time
  losses, no illegal moves); see `docs/runbooks/play-it.md`
- `docs/DESIGN.md`: the architecture, section by section; the source of truth
- `docs/DECISIONS.md`: dated decisions with alternatives and reasons
- `docs/ROADMAP.md`: milestones, release names, parking lot
- `docs/journal/`, `docs/interview/`, `docs/PORTFOLIO.md`, `docs/HISTORY.md`:
  the record (see `docs/DOCS-STANDARD.md`)
