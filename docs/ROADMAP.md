# Roadmap

Milestones are rough and will be refined during the architecture discussions.

| # | Milestone | Done when |
|---|---|---|
| 0 | **Skeleton** | Builds, passes CI, answers the UCI handshake. *(current)* |
| 1 | **Foundation** | Board representation and legal move generation pass the perft suite. |
| 2 | **It plays** | Full UCI support and a basic search and evaluation; playable in a GUI. |
| 3 | **Testing rig** | SPRT pipeline in place; every strength change gets measured. |
| 4 | **Getting strong** | Search and evaluation improvements, each proven by SPRT. |
| 5 | **Going public** | Running as a Lichess bot with a real rating. |
| 6 | **Stretch** | NNUE evaluation, multithreading. |

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
