# Roadmap

Milestones are rough and will be refined during the architecture discussions.
Goal: about 2000 (raised from 1800 on 2026-10-24, once a first match
against Stockfish measured about 1900; DECISIONS.md). Design state: `docs/DESIGN.md`.

| # | Milestone | Done when |
|---|---|---|
| 0 | **Skeleton** | Builds, passes CI, answers the UCI handshake. *(done)* |
| 1 | **Foundation** | Board representation and legal move generation pass the perft suite. *(done 2026-10-20)* |
| 2 | **It plays** | Full UCI support and a basic search and evaluation; playable in a GUI. *(done 2026-10-20)* |
| 3 | **Testing rig** | SPRT pipeline in place; every strength change gets measured. *(done 2026-10-22)* |
| 4 | **Getting strong** | Search and evaluation improvements, each proven by SPRT. *(started 2026-10-23)* |
| 5 | **Going public** | Running as a Lichess bot with a real rating. |
| 6 | **Stretch** | NNUE evaluation, multithreading. |

### Milestone 1 steps

- [x] The board: hybrid bitboards plus 64-square array, FEN in and out,
      UCI `position` and `d` (PR #4)
- [x] Knight, king and pawn moves, including promotions and en passant
      (PR #5)
- [x] Sliding pieces (ray walking), castling, check detection, legal
      moves, making a move (copy-make), UCI `position ... moves` (PR #6)
- [x] Perft on the six standard positions: depth 4 in CI, full depth by
      hand before releases; `go perft N` in UCI (PR #6)

### Milestone 2 steps

- [x] Evaluation: material and tapered piece-square tables, our own values (#7)
- [x] Search: alpha-beta, iterative deepening, quiescence, captures first,
      mate and draw detection (repetition, 50-move rule, insufficient
      material); UCI `go depth N` and `eval` (#8)
- [x] Full UCI: clocks, `stop`, `info`, `bestmove`; plays in a GUI and
      against itself under a clock with no losses on time (#9)

### Milestone 3 steps

- [x] Design decided: Fargate Spot, fastchess, balanced book, 8+0.08,
      SPRT bounds, manual start, results on the PR and in S3 (#11)
- [x] `bench`: a fixed search whose node count fingerprints a build (#11)
- [x] AWS infrastructure in cloud-homelab (its own thread): image registry,
      Fargate Spot cluster, results bucket, GitHub OIDC role (cloud-homelab
      PR #20, applied 2026-10-21). The sandbox account's Fargate Spot
      limit was raised from 6 to 16 vCPUs the same day: one full-size test
      at a time.
- [x] The test container: fastchess, opening book, run script; tested
      locally, and a tiny match in the real image on every `rig/` change (#11)
- [x] GitHub Actions: build both engines for ARM, start the test, post the
      verdict ("Run SPRT", plus "SPRT report" for long tests) (#12)
- [x] First real test: the engine against itself must show no difference.
      Two 2,000-game self-matches on AWS: +7.3 ± 7.2 and +1.0 ± 7.5,
      with identical speed and depth for both copies (#14, #15)
- [x] Matches against Stockfish at a set strength, for a rough rating:
      a "Run match" button on the same rig (decided 2026-10-24; in review)

### Milestone 4 steps

Each step is one PR with its own SPRT test (journal 2026-10-23 has the plan).

- [x] Keep the best move from a depth that runs out of time: SPRT pass,
      +43.8 ± 22.1 Elo over 503 games (#16)
- [x] Transposition table (design decided 2026-10-23): SPRT pass,
      +135.7 ± 36.9 Elo over 235 games (2a and 2b together; #18, #19)
  - [x] 2a: Zobrist fingerprints; repetitions found by fingerprint
        (same search, bench 2457379; #18, no SPRT of its own)
  - [x] 2b: the table itself, with a UCI `Hash` option (bench 1275902;
        #19)
- [x] Killer and history moves (design decided 2026-10-24; two PRs):
  - [x] 3a: killer moves: SPRT pass, +45.3 ± 22.8 Elo over 588 games
        (bench 1162141; #21)
  - [x] 3b: history moves: SPRT pass, +25.1 ± 16.2 Elo over 1,095 games
        (bench 1153324; #22)
- [x] Principal variation search and aspiration windows (design decided
      2026-10-24; two PRs):
  - [x] 4a: principal variation search: SPRT pass, +18.8 ± 13.3 Elo over
        1,572 games (bench 987309; #23)
  - [x] 4b: aspiration windows: SPRT pass, +7.8 ± 6.0 Elo over 6,282
        games (bench 924579; #25)
- [x] Null-move pruning (R = 3 + depth/4, decided 2026-10-24): SPRT
      pass, +73.2 ± 29.1 Elo over 392 games (bench 942397; #28)
- [x] Late move reductions and check extensions (design decided
      2026-10-24; two PRs):
  - [x] 6a: check extensions: SPRT pass, +33.0 ± 19.0 Elo over 753
        games (bench 1013571; #30)
  - [x] 6b: late move reductions: SPRT pass, +113.4 ± 36.5 Elo over
        264 games (bench 343857; #31)
- [ ] Time-management rework, rule by rule (design decided 2026-10-25;
      background: the parking lot's position-aware idea, and Charlie's
      5+0 games on 2026-10-23 and 1. e4 taking 24 seconds on 2026-10-24):
  - [ ] 7a: one legal move played at once, `Move Overhead` option (no
        SPRT; in review)
  - [ ] 7b: best-move stability (150% after a change, 50% once settled
        for 4 depths; in review)
  - [ ] 7c: score drop
  - [ ] 7d: when to start a new depth
  - [ ] Sudden-death SPRT at 10 seconds + 0 for 7b to 7d together
- [ ] Evaluation terms: mobility, pawn structure, king safety, and a
      "mop-up" term for won endgames (drive the losing king to the edge,
      bring your own king close; Charlie's endgame point, 2026-10-24,
      after the queen-up shuffling game against PyChess) (design section
      first)
- [ ] Texel tuning (design section first)

## Release names

Releases are named after stars, starting dim and getting brighter as the
engine gets stronger, ending with Sirius, the brightest star in the night sky.
Magnitudes are apparent brightness as seen from Earth (lower is brighter).

| Star | Magnitude | Fits with |
|---|---|---|
| Polaris | 1.98 | First playable release (Milestone 2); the guide star. **Taken 2026-10-20** |
| Deneb | 1.25 | |
| Spica | 0.97 | |
| Aldebaran | 0.86 | |
| Altair | 0.77 | |
| Rigel | 0.13 | |
| Vega | 0.03 | |
| Arcturus | −0.05 | |
| Canopus | −0.74 | |
| Sirius | −1.46 | The big one |

Polaris is the first playable version (Milestone 2). Which milestone or
strength level earns each later name is still to be decided, and so is how
a release is tagged and built (DESIGN.md section 6).

## Parking lot

- NNUE evaluation (Milestone 6); the evaluation is built to be swappable.
- Magic bitboards for sliding pieces, and fully legal move generation, if
  `bench` shows either is worth it.
- Position-aware time management (Charlie's idea): use the search's own
  uncertainty as the measure of difficulty. Spend more time when the best
  move keeps changing between depths or the score suddenly drops; move
  early when one move is clearly best or forced. All inside a soft target
  and a hard ceiling. Each rule kept only if SPRT shows a gain. Scheduled
  as Milestone 4's time-management rework, after the pruning steps.
- Multithreading (Lazy SMP), native build only.
- A null-move "verification search" (after a pass cuts off, search the
  position again a little shallower to confirm), against middlegame
  zugzwang; and the milder R the design note described by mistake (3 up
  to depth 7). Each its own SPRT (journal 2026-10-24-null-move-pruning).
- Endgame tablebases (Syzygy): perfect play with 7 or fewer pieces, for
  the Lichess bot. Search extensions for endgames (passed pawn pushes,
  singular extensions) after check extensions.
- A board widget for the browser page: write one or vendor a library
  (check its licence).
- Don't throw away thinking time: on about 2% of moves the engine spends
  up to a second on a depth it can't finish (seen in the self-match game
  files). Using the unfinished depth's best move is done (#16, +44 Elo);
  not starting such a depth at all is part of the time rework.
- Give it a personality (Charlie's idea, 2026-10-21), after 2000 and each
  measured with SPRT so the strength cost is known:
  - a style knob, such as an attacking evaluation (king attacks, pawn
    storms, pawns given for activity), as a UCI option
  - contempt: treat a draw as slightly bad, so it keeps playing
  - a signature opening repertoire
  - Lichess chat in the star theme (Milestone 5)
  - "watch it think" in the browser: candidate moves drawn as a
    constellation of arrows, plus an evaluation bar
  - difficulty levels named after stars
- Regular Fargate as a fallback when Spot has no room or takes a test
  back. Charlie decided to stay on Spot for now (2026-10-23); revisit if
  Spot keeps failing (3 problems in the first 6 runs). AWS's reason is now
  printed and the start is retried (#17), so the next failure says why.
- Hide the AWS account ID in the SPRT workflow logs (task ARN, bucket
  name) before the repo goes public.

## Known limitations

- Depth 1 always finishes, even past the time limit, so there's always a
  move. It takes about a millisecond from the start position but about
  30 ms in a busy middlegame (Kiwipete), which matters only with a few
  milliseconds left on the clock.
- Time use is deliberately simple. Over a long game with an increment the
  clock settles at about 5 times the increment (half a second at +0.1s).
  The smarter version is in the parking lot.
- About 400,000 positions a second in `bench` and about 590,000 from the
  start position on the session machine (2026-10-24; bench was about
  560,000 before the fingerprints and table, and the start position about
  800,000 in Milestone 2). Every position generates all its legal moves, quiescence
  included, and tries each one in full (fingerprint included) to check it
  is legal; that, not the repetition check, is where the time goes. The
  fingerprints cost about 4% of speed; the transposition table (2b) more
  than pays it back by searching far fewer positions (a quarter as many
  at depth 7).
- After a two-square pawn move the en passant square is always recorded,
  even when no enemy pawn could capture, so that position never counts as
  a repeat. Rare and harmless (a draw claim might be missed).
- Insufficient material covers king vs king and a lone minor piece only.
- Zugzwang with pieces on the board can fool null-move pruning (the
  side to move "passes", which the rules don't allow). Example: the
  quiet mate in 2 1. Ra6! in `kbK5/pp6/1P6/8/8/8/8/R7 w`, missed since
  late move reductions (journal 2026-10-24-late-move-reductions). The
  parked verification search is the fix.
- The browser build, when it exists, will be single-threaded and so a little
  weaker than the Lichess bot.
