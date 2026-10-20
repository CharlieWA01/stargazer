# Roadmap

Milestones are rough and will be refined during the architecture discussions.
Goal: about 1800 on Lichess (2000 maybe later). Design state: `docs/DESIGN.md`.

| # | Milestone | Done when |
|---|---|---|
| 0 | **Skeleton** | Builds, passes CI, answers the UCI handshake. *(done)* |
| 1 | **Foundation** | Board representation and legal move generation pass the perft suite. *(done 2026-10-20)* |
| 2 | **It plays** | Full UCI support and a basic search and evaluation; playable in a GUI. *(current; design decided 2026-10-20)* |
| 3 | **Testing rig** | SPRT pipeline in place; every strength change gets measured. |
| 4 | **Getting strong** | Search and evaluation improvements, each proven by SPRT. |
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

## Release names (proposal)

Releases are named after stars, starting dim and getting brighter as the
engine gets stronger, ending with Sirius, the brightest star in the night sky.
Magnitudes are apparent brightness as seen from Earth (lower is brighter).

| Star | Magnitude | Fits with |
|---|---|---|
| Polaris | 1.98 | First playable release; the guide star |
| Deneb | 1.25 | |
| Spica | 0.97 | |
| Aldebaran | 0.86 | |
| Altair | 0.77 | |
| Rigel | 0.13 | |
| Vega | 0.03 | |
| Arcturus | −0.05 | |
| Canopus | −0.74 | |
| Sirius | −1.46 | The big one |

Which milestone or strength level earns each name is still to be decided.

## Parking lot

- NNUE evaluation (Milestone 6); the evaluation is built to be swappable.
- Magic bitboards for sliding pieces, and fully legal move generation, if
  `bench` shows either is worth it.
- Position-aware time management (Charlie's idea): use the search's own
  uncertainty as the measure of difficulty. Spend more time when the best
  move keeps changing between depths or the score suddenly drops; move
  early when one move is clearly best or forced. All inside a soft target
  and a hard ceiling. Each rule kept only if SPRT shows a gain (Milestone 4).
- Push from 1800 to 2000: more search tricks, a richer evaluation, Texel
  tuning (see `docs/DECISIONS.md`, 2026-10-20).
- Multithreading (Lazy SMP), native build only.
- A board widget for the browser page: write one or vendor a library
  (check its licence).

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
- No `bench` command yet; `go perft 6` is the rough speed check.
- The browser build, when it exists, will be single-threaded and so a little
  weaker than the Lichess bot.
