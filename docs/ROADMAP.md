# Roadmap

Milestones are rough and will be refined during the architecture discussions.
Goal: about 1800 on Lichess (2000 maybe later). Design state: `docs/DESIGN.md`.

| # | Milestone | Done when |
|---|---|---|
| 0 | **Skeleton** | Builds, passes CI, answers the UCI handshake. *(done)* |
| 1 | **Foundation** | Board representation and legal move generation pass the perft suite. *(current; board and move generation designs decided)* |
| 2 | **It plays** | Full UCI support and a basic search and evaluation; playable in a GUI. |
| 3 | **Testing rig** | SPRT pipeline in place; every strength change gets measured. |
| 4 | **Getting strong** | Search and evaluation improvements, each proven by SPRT. |
| 5 | **Going public** | Running as a Lichess bot with a real rating. |
| 6 | **Stretch** | NNUE evaluation, multithreading. |

### Milestone 1 steps

- [x] The board: hybrid bitboards plus 64-square array, FEN in and out,
      UCI `position` and `d` (PR #4)
- [ ] Knight, king and pawn moves
- [ ] Sliding pieces (ray walking), castling, en passant, promotion; making
      a move (copy-make)
- [ ] Perft on the six standard positions, shallow in CI, deep as `bench`

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
- Push from 1800 to 2000: more search tricks, a richer evaluation, Texel
  tuning (see `docs/DECISIONS.md`, 2026-10-20).
- Multithreading (Lazy SMP), native build only.
- A board widget for the browser page: write one or vendor a library
  (check its licence).

## Known limitations

- Plays no chess yet: it can load and print a position, but not move.
- `position ... moves ...` is refused until move making exists.
- The browser build, when it exists, will be single-threaded and so a little
  weaker than the Lichess bot.
