# Stargazer design

This is the source of truth for architecture decisions. Each section is either
**Open** (still being discussed) or **Decided** (with the date and reasoning).
Code for a component isn't written until its section is Decided.

## Settled so far

| Decision | Choice | Why |
|---|---|---|
| Language | Rust | Fast, and its strict compiler catches many bugs before the code ever runs, which matters when the architect isn't reviewing code line by line. |
| Protocol | UCI | Standard for GUIs, testing tools and Lichess bots. |
| Origin | Written from scratch | The point is to build it, not wrap Stockfish. |
| Name | Stargazer | Named after an astronomer-diviner D&D character. |

## Open questions

These are the topics for the architecture discussions. The options listed are
starting points, not a complete survey.

### 1. Board representation
How the engine stores a position in memory. This choice shapes everything
built on top of it, so it comes first.
- **Bitboards:** one 64-bit number per piece type, one bit per square. The
  standard for strong engines; fast, but the low-level code is cryptic.
- **Mailbox:** an array of 64 (or 120) squares, each holding a piece. Easy to
  read and debug, noticeably slower.
- **Hybrid:** bitboards plus a mailbox array kept in sync, for fast "what's
  on this square?" lookups. Common in practice.

### 2. Move generation
- **Legal vs. pseudo-legal:** generate only legal moves, or generate all moves
  and discard the ones that leave the king in check?
- **Sliding pieces (bishops, rooks, queens):** magic bitboards, PEXT, or
  simpler ray loops?
- **Verification:** perft against published node counts; which test suite
  (standard positions plus tricky ones like castling and en passant edge cases).

### 3. Search
- Base algorithm: negamax with alpha-beta and iterative deepening (the
  near-universal starting point).
- What to add, and in what order: quiescence search, transposition table, move
  ordering (MVV-LVA, killer moves, history), null-move pruning, late move
  reductions, aspiration windows.
- Time management: how long to think per move under different time controls.
- Multithreading (Lazy SMP): now, later, or never?

### 4. Evaluation
- **Hand-crafted:** material, piece-square tables, pawn structure, king
  safety, mobility. Understandable and tunable; this is where chess knowledge
  shows up most directly.
- **NNUE:** a small neural network, the modern approach. Much stronger, but it
  needs training data and a training pipeline.
- Likely path: hand-crafted first, NNUE as a later milestone. Confirm.

### 5. Testing and infrastructure
- CI: GitHub Actions running format, lint, tests and a release build (in place).
- Perft suite: run in CI, or separately because it's slow?
- Strength testing: fastchess or cutechess-cli for SPRT matches; where it runs
  (local machine vs. a cloud box) and with what opening book.
- Benchmarking: a fixed `bench` command so speed regressions are visible.

### 6. Deployment
- Lichess bot via lichess-bot; where it's hosted (home server, cheap VPS).
- Release process: how and when versions get tagged and built.

### 7. Project housekeeping
- License: MIT, GPL-3.0 (what Stockfish uses), or none/private?
- Public or private repo long term?

## Decision log

Record decisions here as they're made, newest first.

<!--
### YYYY-MM-DD: Short title
**Decision:** what we chose.
**Why:** the reasoning and the alternatives we rejected.
-->
