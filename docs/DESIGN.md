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
| Goal | A rating around 2000 (raised from 1800 on 2026-10-24), with code Charlie understands | Reachable with a classic search and hand-written evaluation, no neural network. Understanding comes first, strength second. 2000 needs more search and evaluation work and tuning, no change of design. |
| Targets | Native (GUIs, Lichess bot) and the browser (WebAssembly) | A free "play it here" tile on the portfolio site. See "Runs in a browser" below. |
| Board | Hybrid: bitboards plus a 64-square board | See section 1. |
| Repo | Going public, GPL-3.0-or-later (decided 2026-10-25) | Open source, and nobody can ship a closed fork (section 7). |

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
- Each position also carries its fingerprint (section 3, transposition
  table), kept up to date by the same two functions that place and lift
  pieces.

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
- **Copy-make.** To try a move, copy the position (256 bytes) and play
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
  table (with Zobrist hashing; built, below), killer and history moves, null-move
  pruning (built, below), late move reductions, check extensions (decided,
  below), aspiration windows,
  principal variation search.
- **Time:** a target of one twentieth of the remaining time (or a share
  until the next time control, with `movestogo`) plus three quarters of the
  increment, and a ceiling of three times the target. Both stay inside
  the clock by the `Move Overhead` option (50 ms unless set; added
  2026-10-25). With only one legal move and a clock running, the move is
  played at once. Inside a depth the search stops at the ceiling; between
  depths it starts another only if under half the target, because each
  depth takes several times longer than the last. A depth cut short by the
  ceiling still counts for the moves it finished: last depth's best move is
  searched first, so if another finished move beats it, that move is
  played (Milestone 4, 2026-10-23). Also `go depth`,
  `go movetime`, `go infinite`. Built in PR #9.
  **Rework decided 2026-10-25** (DECISIONS.md), rule by rule:
  - **Best-move stability (7b): tried and dropped** (150% after a
    change, 50% once the best move had held for 4 depths): it lost
    24.5 ± 19.4 Elo, because the shallow depths nearly always agree and
    the engine stopped after a quarter of its planned time.
  - **Score drop (7c):** if a depth's score is more than 30 centipawns
    below the last depth's, the target for starting another depth is
    200%; a rising score changes nothing. It decides only whether to
    start another depth, never past the ceiling.
  - **Starting a new depth (7d): tried and dropped** (neutral at 8+0.08
    and at 60 seconds + 0.6, 2026-10-26); it stays at 50%. The idea:
    another depth starts only while under 60% of the target. With late move reductions, the
    time used so far grows by about 1.5 to 2 times per depth (measured
    2026-10-25), so a depth started at 60% usually ends near 1.2 times
    the target, well inside the ceiling.
  - **Sudden-death pacing (decided 2026-10-25, Charlie's choice):** with
    no increment and no `movestogo`, the clock is shared over an estimate
    of the moves left instead of a fixed twentieth: 25 plus the game
    phase (the evaluation's count of pieces left, 24 with all on), at
    most 45. The ceiling is also held to a tenth of the clock. So the
    engine keeps a reserve while the board is full and thinks hardest as
    the pieces come off. Increment games are unchanged.
- **Transposition table: decided 2026-10-23** (built in two steps).
  - **Fingerprints (Zobrist hashing), step 2a, built:** a fixed random
    64-bit number for each piece on each square, for each combination of
    castling rights, for each en passant file and for Black to move,
    generated when the engine is compiled from a fixed seed (our own
    SplitMix64, no crate). A position's fingerprint is their XOR, updated
    step by step as pieces move. Repetitions are spotted by comparing
    fingerprints. Castling rights are four bits of one byte so `Position`
    stays at 256 bytes (it's copied for every move tried).
  - **The table, step 2b, built 2026-10-23** (SPRT with 2a: +136 Elo): 16 MB by default, set by a UCI `Hash` option
    (1 to 1024 MB), the same size in the browser build. Each slot holds
    the fingerprint, best move, score, depth and whether the score is
    exact or a bound; mate scores are stored relative to the position.
    When full, the newest result replaces the old one. The stored move is
    tried first everywhere; the stored score ends a search early only when
    deep enough and its bound allows, never at the root. Kept between
    moves, cleared on `ucinewgame`.
- **Killer and history moves: decided 2026-10-24** (two PRs, each with
  its own SPRT).
  - **Killers, step 3a, built 2026-10-24** (SPRT: +45 Elo): at each ply the search keeps
    the last two quiet moves (not captures or promotions) that refuted a
    position there, newest first, and tries them right after the
    captures. A killer that happens to capture in the new position is
    ranked as a capture. Kept across the depths of one search, cleared
    at each `go`. Not used in quiescence (captures only there).
  - **History, step 3b, built 2026-10-24** (SPRT: +25 Elo): a score for each quiet move
    (by side, from and to square) that rises by depth × depth when it
    refutes a position, and falls by the same for each quiet move tried
    before it there that failed. Scores stay within ±16,384: the nearer
    a score is to the limit, the less it moves. The quiet moves after
    the killers are tried highest score first. Halved at the start of
    each search (so between moves), cleared on `ucinewgame`; the UCI
    layer keeps it next to the transposition table. Bench starts each
    position with a fresh history.
  - Move order then: last depth's or the table's move, captures and
    promotions, killers, then the other quiet moves.
- **PVS and aspiration windows: decided 2026-10-24** (two PRs, each
  with its own SPRT).
  - **Principal variation search, step 4a, built 2026-10-24** (SPRT:
    +19 Elo): in every
    position the first move is searched with the full window; each later
    move gets a zero-width window ("is it better than the best so
    far?"), and only a "yes" that could matter is searched again
    properly. Quiescence is unchanged. If time runs out during that
    second search at the root, the move is still played: it's already
    proven better than the old best.
  - **Aspiration windows, step 4b, built 2026-10-24** (SPRT: +8 Elo):
    each depth from 5 on first searches the root only for scores within
    ±25 centipawns of the last depth's score. On a miss the window is
    doubled on the side that missed (25, 50, 100, ...) and that depth
    searched again; past 400 the full window is used. A mate score always
    gets the full window. If time runs out after a "better than
    expected" miss, that move is played (it beat the last best); after a
    "worse than expected" miss, the last finished depth's move stands.
- **Null-move pruning: decided 2026-10-24, step 5, built 2026-10-24** (SPRT:
  +73 Elo):
  if a position already looks good enough (evaluation at least beta),
  the side to move passes and the reply is searched R = 3 + depth/4
  half-moves shallower with a zero-width window; if still good enough,
  the search stops there and returns beta (stored in the table as a
  lower bound). Only at depth 3 or more, away from the root and the main
  line, not twice in a row, not in check, not against a mate score, and
  not with only king and pawns (zugzwang). A pass clears en passant and
  resets the 50-move counter, so repetitions never look back past it.
- **Late move reductions and check extensions: decided 2026-10-24**
  (two PRs, each with its own SPRT).
  - **Check extensions, step 6a, built 2026-10-24** (SPRT: +33 Elo): a move that gives
    check is searched one half-move deeper (its reply gets the same
    depth, not one less), but not once a line is twice as long as the
    depth the root started with.
  - **Late move reductions, step 6b, built 2026-10-24** (SPRT:
    +113 Elo): quiet moves late in the order are
    searched shallower, by a slowly growing formula of depth and move
    number, 0.75 + ln(depth) × ln(move number) / 2.25 rounded down,
    from the 4th move (one less in the main line); not the first few moves,
    captures, promotions, checks, or when in check, and not below depth
    3. A reduced move that beats the best so far is searched again at
    full depth.
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
  separate command run before each release. Speed is measured with
  `bench` (below), not perft.
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
    of `rig/`), starts the task at 16 vCPU (adjustable) and waits up to 30
    minutes. "SPRT report" posts a longer test's verdict later. Both write
    the same report (`rig/report.py`) to the workflow summary and the PR.
  - **Matches against Stockfish: decided and built 2026-10-24.** A "Run
    match" button (`match.yml`) runs "Run SPRT" in match mode: only one
    Stargazer is built, and it plays Debian's Stockfish (in the same
    container) at a set `UCI_Elo` for a fixed number of games (default
    400 at 10+0.1). The report gives the score, the Elo difference and the
    estimated rating (Stockfish's level plus the difference). It is a
    rough yardstick, not a test: no pass or fail. See
    `docs/runbooks/stockfish-match.md`.
- Benchmarking: **built 2026-10-21 (PR #11).** `bench` (a UCI command, or
  `stargazer bench`) searches 10 fixed positions to depth 5 and prints
  `Bench: <nodes> nodes <speed> nps`. The node count fingerprints a build: a
  speed-only change must leave it unchanged.

### 6. Deployment
- Lichess bot via lichess-bot. The bot account is registered (an account
  becomes a BOT only if it has played no games, and the upgrade can't be
  undone, so it stays unused until then). The bot only connects out to
  Lichess, so nothing is exposed. **Not Fargate Spot** (decided
  2026-10-21): a reclaimed machine would abandon live games.
- **Host: decided 2026-10-25.** One EC2 t4g.small (ARM, 2 burstable
  vCPUs, 2 GB, "unlimited" CPU credits) in workloads-prod, Ubuntu 24.04
  (see "The bot's software" below), no inbound ports, no SSH (Session Manager), rebuilt by Terraform
  (cloud-homelab) rather than hand-tended. It runs **10am to 10pm Central**
  (America/Chicago, so it follows daylight saving), 12 hours to start:
  EventBridge Scheduler starts it, tells the bot to drain at 21:15 (no new
  games, finish the current one) and stops it once no game is running, or
  at 22:45 at the latest. About $9 to $10 a month (always on would be $17
  to $20; always-on Fargate at 1 vCPU $32.50). lichess-bot stops cleanly
  only on SIGINT, so its service uses `KillSignal=SIGINT`.
- Lichess rules that shape it (checked 2026-10-25): at most 100 games a
  day against other bots (lichess-bot reads the limit and waits); challenge
  games only, no pools or tournaments except bot events; no UltraBullet.
- **Rated play (decided 2026-10-26):** it challenges other bots to rated
  games and accepts rated and casual challenges (DECISIONS.md).
- **Time controls (decided 2026-10-25):** it
  accepts every standard bullet and blitz control, with or without
  increment, and rapid up to 10+5 (lichess-bot: `time_controls: [bullet,
  blitz, rapid]`, `max_base: 600`, `max_increment: 5`; the increment cap
  also turns down rare blitz controls like 3+10). No classical or
  correspondence: long games would eat playing time and could run past
  the evening drain. One game at a time, no pondering (the engine doesn't
  support it yet; parked in the roadmap, 2026-10-26).
- **Open (Milestone 5 design, journal 2026-10-25-lichess-bot-design):**
  the SRE layer (SLOs, burn-rate alerts, an observer Lambda, DynamoDB for
  live records, S3 + Athena for history, a status page on the portfolio
  site) and deploys gated on SPRT that let games finish. Prod's budget
  rose to $25 for the server (cloud-homelab PR #28).
- Portfolio tile (decided 2026-10-20): **both** a "play it here" page on the
  portfolio site, with the engine running in the visitor's browser (see
  "Runs in a browser"), and a link to the Lichess bot's profile to show its
  real rating. The page says the browser version is single-threaded and so
  a little weaker than the rated one. The tile links to Lichess rather than
  fetching the rating live, so the site makes no calls to other servers.
  Until the browser version exists, the tile links straight to the
  Lichess profile (portfolio DECISIONS.md, 2026-10-26).
- Release process: **decided 2026-10-25.** A version is released by
  pushing a tag `vX.Y.Z` that matches `Cargo.toml`'s version. The
  `Release` workflow tests it, builds it for ARM Linux on Ubuntu 24.04,
  and publishes a GitHub release with the binary
  (`stargazer-aarch64-linux`), its SHA-256 and its `bench` node count.
  Star names (the roadmap's table) can be given to releases at
  milestones; which ones is still open.
- **The bot's software (decided 2026-10-25):** `bot/` holds lichess-bot's
  config, the lichess-bot commit to use (`LICHESS_BOT_COMMIT`), the engine
  release to run (`ENGINE`: tag and checksum), the installer (`setup.sh`),
  the start script (`run.sh`, which reads the Lichess token from SSM) and
  the systemd service. A deploy is a PR that changes `ENGINE` (or the
  config), then `setup.sh` at the merged commit through Session Manager;
  the restart lets the current game finish (`KillSignal=SIGINT`,
  `KillMode=mixed`). The server runs Ubuntu 24.04 (security updates are
  automatic, and it matches the build machine), not Amazon Linux 2023 as
  first sketched. Runbook: `docs/runbooks/lichess-bot.md`.

### 7. Project housekeeping
- License: **decided 2026-10-25: GPL-3.0-or-later** (what Stockfish
  uses). `LICENSE` holds the full text, `Cargo.toml` names it, and the
  README carries the notice.
- Public or private repo long term? **Decided 2026-10-25: public**, for
  the Lichess bot's deploy approvals and the portfolio. First: the three
  AWS settings that contain the account ID are secrets, and the account ID
  is masked in the Actions logs (done in this PR); old SPRT and match run
  logs, which show it, are deleted (Charlie's OK needed).

## Decision log

Dated decisions, with the alternatives and the reasons, live in
[`DECISIONS.md`](DECISIONS.md). This file holds the current state of each
part of the design; `DECISIONS.md` holds how it got there.
