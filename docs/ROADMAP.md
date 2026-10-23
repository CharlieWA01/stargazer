# Roadmap

Milestones are rough and will be refined during the architecture discussions.
Goal: about 1800 on Lichess (2000 maybe later). Design state: `docs/DESIGN.md`.

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

### Milestone 4 steps

Each step is one PR with its own SPRT test (journal 2026-10-23 has the plan).

- [ ] Keep the best move from a depth that runs out of time (in review)
- [ ] Transposition table with Zobrist hashing (table size, replacement
      and the browser build's size to decide first)
- [ ] Killer and history moves
- [ ] Principal variation search and aspiration windows
- [ ] Null-move pruning
- [ ] Late move reductions and check extensions
- [ ] Time-management rework, rule by rule, once the search has settled
      (see the parking lot's position-aware idea; also "don't start a
      depth that can't finish")
- [ ] Evaluation terms: mobility, pawn structure, king safety (design
      section first)
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
- Push from 1800 to 2000: more search tricks, a richer evaluation, Texel
  tuning (see `docs/DECISIONS.md`, 2026-10-20).
- Multithreading (Lazy SMP), native build only.
- A board widget for the browser page: write one or vendor a library
  (check its licence).
- Don't throw away thinking time: on about 2% of moves the engine spends
  up to a second on a depth it can't finish (seen in the self-match game
  files). Using the unfinished depth's best move is Milestone 4's first
  step; not starting such a depth at all is part of the time rework.
- Give it a personality (Charlie's idea, 2026-10-21), after 1800 and each
  measured with SPRT so the strength cost is known:
  - a style knob, such as an attacking evaluation (king attacks, pawn
    storms, pawns given for activity), as a UCI option
  - contempt: treat a draw as slightly bad, so it keeps playing
  - a signature opening repertoire
  - Lichess chat in the star theme (Milestone 5)
  - "watch it think" in the browser: candidate moves drawn as a
    constellation of arrows, plus an evaluation bar
  - difficulty levels named after stars
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
- About 800,000 positions a second. Every position generates all its legal
  moves, quiescence included, and repetitions are found by comparing whole
  positions; Zobrist hashing (with the transposition table) will speed both.
- After a two-square pawn move the en passant square is always recorded,
  even when no enemy pawn could capture, so that position never counts as
  a repeat. Rare and harmless (a draw claim might be missed).
- Insufficient material covers king vs king and a lone minor piece only.
- The browser build, when it exists, will be single-threaded and so a little
  weaker than the Lichess bot.
