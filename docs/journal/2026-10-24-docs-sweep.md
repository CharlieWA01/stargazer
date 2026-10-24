# 2026-10-24: Documentation sweep

## Goal
Charlie asked for a housekeeping pass: read the living docs (README,
CLAUDE.md, DESIGN, DECISIONS, ROADMAP, PORTFOLIO, runbooks) and fix
anything stale. Journals and HISTORY.md are records of their day, so they
were left alone.

## What was done
- **README.md:** the status still said Milestone 1 and "doesn't choose
  moves yet". It now says Milestone 4 is under way, shows a `go` and a
  `bench` command, and links the runbooks.
- **CLAUDE.md layout:** `search.rs` mentions the transposition table;
  `uci.rs` lists `setoption name Hash` and `ucinewgame`.
- **DESIGN.md:** copy-make copies 256 bytes (it said about 200); the
  Milestone 4 list marks the table as built, with its SPRT result; the
  perft section no longer says the deep perft run "serves as `bench`"
  (`bench` is its own command since #11).
- **DECISIONS.md:** the 2026-10-20 time-budget entry is marked amended by
  the 2026-10-23 unfinished-depth change, instead of reading as if a cut
  depth is still thrown away.
- **ROADMAP.md:** the speed figure was Milestone 2's 800,000 positions a
  second; it now gives today's bench speed and says where the old number
  came from. The parking-lot note on wasted thinking time says the first
  half is done (#16).
- **PORTFOLIO.md:** status says Milestone 4 is under way.
- **Runbooks:** `perft.md` no longer says `bench` doesn't exist yet;
  `play-it.md` shows the new `Hash` option.

Checked and left as they are: `DOCS-STANDARD.md` matches the copies in
maplenote, cloud-homelab and portfolio; the repo is still private, so
"private for now" and the account-ID parking-lot item still apply; the
browser build still doesn't exist (the portfolio tile says "soon").

## Problems and fixes
None.

## How it was verified
- `stargazer bench` three times on the session machine: 1275902 nodes at
  391,000 to 441,000 nps; `go depth 7` from the start position at about
  587,000 nps. The machine is noisy, hence a range.
- The README's commands run as written.
- The repo's visibility checked on GitHub (private), and the four
  `DOCS-STANDARD.md` copies compared byte for byte.

## Impact
Docs only; nothing in the engine changed.
