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
| Goal | A Lichess rating around 1800, with code Charlie understands | Reachable with a classic search and hand-written evaluation, no neural network. Understanding comes first, strength second. 2000 is a possible later goal: more search and evaluation work, no change of design. |
| Targets | Native (GUIs, Lichess bot) and the browser (WebAssembly) | A free "play it here" tile on the portfolio site. See "Runs in a browser" below. |
| Board | Hybrid: bitboards plus a 64-square board | See section 1. |
| Repo | Private for now | Licence decided before it goes public (section 7). |

## Open questions

These are the topics for the architecture discussions. The options listed are
starting points, not a complete survey.

### 1. Board representation: **Decided** (2026-10-20)
How the engine stores a position in memory. This choice shapes everything
built on top of it, so it came first.

**Choice: hybrid.** Twelve bitboards (one 64-bit number per piece type and
colour, one bit per square) plus a 64-square array saying what stands on
each square. Both are updated on every move and must always agree.
- Questions about groups of pieces ("all of White's pawns", "is this pawn
  passed?", "what attacks the king's zone?") use the bitboards.
- Questions about one square ("what's on e4?") use the array.
- All shift-and-mask code lives in one small module whose functions have
  chess names (`knight_attacks(square)`, `pawn_pushes(pawns)`), each with its
  own tests. Search and evaluation call those and read like chess.
- The risk of the two copies disagreeing is caught by perft.

Considered and rejected:
- **Mailbox only:** easiest to read, but a few times slower (rough estimate)
  and evaluation ideas become loops over squares. Switching later would mean
  rewriting move generation.
- **Bitboards only:** just as fast, but "what's on this square?" means
  checking twelve numbers, which makes the readable code less readable.

How sliding pieces (bishops, rooks, queens) find their moves on bitboards is
section 2.

### 2. Move generation
- **Legal vs. pseudo-legal:** generate only legal moves, or generate all moves
  and discard the ones that leave the king in check?
- **Sliding pieces (bishops, rooks, queens):** magic bitboards, PEXT, or
  simpler ray loops?
- **Verification:** perft against published node counts; which test suite
  (standard positions plus tricky ones like castling and en passant edge cases).

### Runs in a browser (constraint, decided 2026-10-20)
Stargazer must also compile to WebAssembly so visitors can play it on the
portfolio site, with the engine running on their computer (no server, about
$0). So:
- The engine core never reads input, prints output, or reads the system
  clock itself. A thin UCI layer (native) or browser layer does that and
  passes the time in.
- The browser build is single-threaded. Multithreading, if it comes, is for
  the native build only.
- Any crate needed to talk to JavaScript (such as `wasm-bindgen`) lives in a
  separate small browser package, never in the engine core, and is asked
  about first.

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
- Likely path: hand-crafted first, NNUE as a later milestone. **Confirmed
  2026-10-20:** hand-crafted is enough for the 1800 goal. NNUE stays a
  stretch goal, so the evaluation is designed as a swappable piece that a
  network can replace later without touching the board or search.

### 5. Testing and infrastructure
- CI: GitHub Actions running format, lint, tests and a release build (in place).
- Perft suite: run in CI, or separately because it's slow?
- Strength testing: fastchess or cutechess-cli for SPRT matches; where it runs
  (local machine vs. a cloud box) and with what opening book.
- Benchmarking: a fixed `bench` command so speed regressions are visible.

### 6. Deployment
- Lichess bot via lichess-bot; where it's hosted (home server, cheap VPS).
  The bot account is registered (an account becomes a BOT only if it has
  played no games, and the upgrade can't be undone, so it stays unused until
  then). The bot only connects out to Lichess, so nothing is exposed. Likely
  host: the planned homelab mini PC (no extra cost); a small always-on cloud
  server would be about $5 to $15 a month (Lambda can't hold the connection).
- Portfolio tile (decided 2026-10-20): **both** a "play it here" page on the
  portfolio site, with the engine running in the visitor's browser (see
  "Runs in a browser"), and a link to the Lichess bot's profile to show its
  real rating. The page says the browser version is single-threaded and so
  a little weaker than the rated one. The tile links to Lichess rather than
  fetching the rating live, so the site makes no calls to other servers.
- Release process: how and when versions get tagged and built.

### 7. Project housekeeping
- License: MIT, GPL-3.0 (what Stockfish uses), or none/private?
- Public or private repo long term?

## Decision log

Record decisions here as they're made, newest first.

### 2026-10-20: Goals, targets, and the board
**Decision:** keep Rust, UCI and "from scratch". Aim for about 1800 on
Lichess with code Charlie understands; NNUE is a stretch goal. Stargazer must
also run in a browser. The board is a hybrid of bitboards and a 64-square
array. The repo stays private for now.
**Why:** Rust compiles cleanly to WebAssembly and its compiler catches bugs
early. UCI plugs into GUIs, lichess-bot, fastchess and the browser tile with
one interface. 1800 doesn't need a neural network. The hybrid gives
bitboard speed while keeping the code readable outside one tested module.
Rejected: a mailbox-only board (slower, hard to switch away from later) and
bitboards only (less readable).

<!--
### YYYY-MM-DD: Short title
**Decision:** what we chose.
**Why:** the reasoning and the alternatives we rejected.
-->
