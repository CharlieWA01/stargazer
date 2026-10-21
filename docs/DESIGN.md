# Stargazer design

This is the source of truth for the architecture. Each section is either
**Open** (still being discussed) or **Decided** (with the date). Code for a
component isn't written until its section is Decided. The dated record of
each decision, with alternatives and reasons, is in `DECISIONS.md`.

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

### 2. Move generation: **Decided** (2026-10-20)
Given a position, list every move. Search runs this millions of times a
second, so it must be both exactly right and fast.

- **Pseudo-legal, checked afterwards.** List each piece's moves as if the
  king didn't matter; before a move is played for real, ask "after this,
  is my own king attacked?" and skip it if so. Simplest to write and
  explain. Rejected for now: fully legal generation (work out pins and
  checks first): faster in perft and in-check positions, but notorious edge
  cases (an en passant capture that uncovers a check along the rank). The
  switch would stay inside move generation if profiling ever asks for it.
- **Sliding pieces by ray walking.** From a bishop, rook or queen, step
  along each direction until a piece blocks. The rest of the engine only
  calls `bishop_attacks(square, blockers)` and `rook_attacks(square,
  blockers)`, so magic bitboards can replace the inside later, if `bench`
  shows it's worth it, with perft proving identical answers. PEXT is ruled
  out: browsers can't use the instruction.
- **Knights, kings and pawns** use ready-made tables of their moves from
  each square.
- **Copy-make.** To try a move, copy the position (about 200 bytes) and play
  the move on the copy; to take it back, discard the copy. No undo code, so
  no undo bugs. Rejected: make/unmake (reverse each move in place), slightly
  faster in theory but every special move needs its own undo logic.
- **Proof: perft** on the six standard positions from the Chess Programming
  Wiki (start position, "Kiwipete" and positions 3 to 6), compared with
  published node counts. When a count is wrong, "perft divide" (the count
  after each first move) finds the move at fault.

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

### 3. Search: **Decided** (2026-10-20)
- **Core:** negamax with alpha-beta and iterative deepening. Scores are in
  centipawns (100 = one pawn) from the side to move's point of view; the
  search reports `info depth ... score cp ... pv ...` over UCI.
- **In Milestone 2:** quiescence search (play out captures before judging,
  so it never stops mid-exchange), captures first ordered best-first
  (MVV-LVA), checkmate and stalemate scoring (quicker mates score higher),
  and draws by repetition, the 50-move rule and insufficient material. Inside
  the search a single repeat counts as a draw. Built in PR #8.
- **Later (Milestone 4), each only if SPRT shows a gain:** transposition
  table (with Zobrist hashing), killer and history moves, null-move
  pruning, late move reductions, check extensions, aspiration windows,
  principal variation search.
- **Time:** a target of one twentieth of the remaining time (or a share
  until the next time control, with `movestogo`) plus three quarters of the
  increment, and a ceiling of three times the target. Both stay 50 ms
  inside the clock. Inside a depth the search stops at the ceiling; between
  depths it starts another only if under half the target, because each
  depth takes several times longer than the last. Also `go depth`,
  `go movetime`, `go infinite`. Built in PR #9. Smarter, position-aware
  time use is parked (ROADMAP.md).
- **`stop` and threads:** the core stays single-threaded and clock-free; it
  is handed a stop signal and a time limit and checks them every few
  thousand positions. The native UCI layer runs the search on a second
  thread so it can still read `stop`; the browser page uses a background
  worker. Multithreaded search (Lazy SMP) stays a stretch goal, native only.

### 4. Evaluation: **Decided** (2026-10-20)
- **Hand-crafted first; NNUE a stretch goal** (confirmed 2026-10-20). The
  evaluation sits behind one function, `evaluate(position) -> score`, so it
  can be extended or replaced without touching search.
- **Milestone 2 terms:** material (pawn 100, knight 320, bishop 330, rook
  500, queen 900) plus piece-square tables, with separate middlegame and
  endgame tables blended by the material left on the board ("tapered").
- **The numbers are our own:** written by hand from chess principles, then
  fitted to game results by our own tuner (Texel tuning) in Milestone 4.
  Rejected: the public "Simplified Evaluation Function" tables and PeSTO's
  machine-tuned tables, which would start stronger but aren't ours.
- **Later, each by SPRT:** pawn structure, king safety, mobility.

### 5. Testing and infrastructure
- CI: GitHub Actions running format, lint, tests and a release build (in place).
- Perft suite: **decided 2026-10-20: both.** Shallow depths (about 4) of
  the six standard positions run in CI on every PR; deep runs (5 to 6) are a
  separate command run before each release, which also serves as `bench`.
- Strength testing: **decided 2026-10-21.**
  - **Where:** AWS ECS Fargate Spot on ARM (Graviton), us-east-2. One
    container per test, up to 16 vCPU, paid per second only while it runs.
    Infrastructure is Terraform in cloud-homelab.
  - **Match runner:** fastchess. **Openings:** a balanced book of short
    openings, each played with both colours. UHO (openings where one side
    starts better) later, once draws become common.
  - **Games:** 8 seconds + 0.08 per move; 40 + 0.4 only to double-check a
    big change.
  - **SPRT:** 95% confidence both ways (alpha = beta = 0.05). Bounds 0 to
    10 Elo while gains are big, 0 to 5 later, and -5 to 0 for clean-ups
    that should change nothing.
  - **Flow:** a manual GitHub Actions workflow builds both engine versions
    for ARM, uploads them to S3 and starts the task. The task runs the match
    and writes games, logs and a verdict to S3 (kept 90 days). The workflow
    posts the verdict on the PR. No GitHub token in the container; no stored
    AWS keys (OIDC).
  - **Rule:** a strength change merges only with a passing SPRT result
    linked in its PR.
  - **The test container: built 2026-10-21 (PR #11).** `rig/Dockerfile`
    holds fastchess (pinned release, built for whatever CPU runs it), the
    8moves_v3 book (pinned, hash-checked) and `rig/run_test.py`, which
    fetches the two engines, records each one's `bench`, plays until the
    SPRT decides and writes `verdict.json` (pass, fail, inconclusive or
    error), the games and the log. It stops cleanly on AWS's stop signal or
    an 8-hour deadline, keeping the games played so far. The test engine
    is listed first, so a positive Elo means the new version is stronger.
    Clearly decided games end early (resign at 6 pawns for 3 moves, draw
    after move 34 within 0.2 pawns for 8 moves), as Stockfish's testing
    does. The `Test rig` workflow builds the image and plays a tiny match
    in it whenever `rig/` changes.
  - **The workflows: built 2026-10-21 (PR #12).** "Run SPRT" cross-builds
    both engines for ARM on an x86 GitHub machine, uploads them, rebuilds
    the container only when `rig/` has changed (tagged with a fingerprint
    of `rig/`), starts the task at 4 vCPU (adjustable) and waits up to 30
    minutes. "SPRT report" posts a longer test's verdict later. Both write
    the same report (`rig/report.py`) to the workflow summary and the PR.
- Benchmarking: **built 2026-10-21 (PR #11).** `bench` (a UCI command, or
  `stargazer bench`) searches 10 fixed positions to depth 5 and prints
  `Bench: <nodes> nodes <speed> nps`. The node count fingerprints a build: a
  speed-only change must leave it unchanged.

### 6. Deployment
- Lichess bot via lichess-bot; where it's hosted (home server, cheap VPS).
  The bot account is registered (an account becomes a BOT only if it has
  played no games, and the upgrade can't be undone, so it stays unused until
  then). The bot only connects out to Lichess, so nothing is exposed. Likely
  host: the planned homelab mini PC (no extra cost); a small always-on cloud
  server would be about $5 to $15 a month (Lambda can't hold the connection).
  **Not Fargate Spot** (decided 2026-10-21): a reclaimed machine would
  abandon live games. Regular Fargate works but costs about $29 a month
  always-on for 1 vCPU (N. Virginia rates), so if it's Fargate it runs on a
  schedule (about $4 a month for 4 hours an evening). Decided in Milestone 5.
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

Dated decisions, with the alternatives and the reasons, live in
[`DECISIONS.md`](DECISIONS.md). This file holds the current state of each
part of the design; `DECISIONS.md` holds how it got there.
