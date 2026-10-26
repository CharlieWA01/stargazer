# Stargazer: project summary

*The one page to read before a presentation, resume update or interview.
Details: `docs/journal/` (what happened), `docs/DECISIONS.md` (why),
`docs/DESIGN.md` (the architecture), `docs/interview/` (stories),
`docs/HISTORY.md` (timeline).*

## In one sentence
A chess engine written from scratch in Rust, aiming for about 2000
(first goal 1800, passed) as a Lichess bot, and playable in any visitor's browser on my portfolio site
through WebAssembly.

## Status
Milestone 4 of 6 under way (started 2026-10-23). Milestone 2 (2026-10-20) was the
first playable version, named Polaris: it plays full games in any chess
GUI under a real clock. Every rule of chess is proven by perft (six
standard test positions, up to 119 million positions deep, all exact). It
looks ahead with alpha-beta search (depth 6 from the start in under a
second, about 800,000 positions a second) and judges positions with an
evaluation whose every number is our own. Its first opponent, me (about
1000 on Lichess), lost. Milestone 3 is the testing rig: one button in
GitHub builds two versions, plays them against each other on AWS Fargate
Spot (16 ARM CPUs, about 2,000 games in 50 minutes for roughly 25 cents)
and posts a statistical verdict (SPRT) on the pull request. It was proven
fair by playing the engine against itself: +1.0 ± 7.5 Elo over 2,000
games. Milestone 4 makes it stronger one measured change at a time: the
first nine (keeping the best move from an unfinished search, a
transposition table that remembers searched positions, killer moves,
history moves, principal variation search, aspiration windows, null-move
pruning, check extensions, late move reductions, and thinking longer
when the score drops) passed at +44, +136, +45, +25, +19, +8, +73, +33,
+113 and +11 Elo; one idea (best-move stability) lost and was dropped. On 2026-10-23 it beat PyChess's engine
at full strength for the first time. Measured ratings, from 400-game
matches against Stockfish at a set strength, on the scale Stockfish's
levels are tied to (the CCRL list): **about 1900** on 2026-10-24 (64%
against Stockfish 1800), then **about 2030** the same day after
null-move pruning (2027 ± 35, 67.5% against Stockfish 1900), the goal
of about 2000 reached on that scale, and **about 2180** that evening
after check extensions and late move reductions (2184 ± 35, 61.9%
against Stockfish 2100).

## On Lichess
Since 2026-10-26 it plays rated games on Lichess as
[StargazerEngine](https://lichess.org/@/StargazerEngine), 10am to 10pm
US Central, from one small ARM server on AWS (EC2 t4g.small, about $10 a
month) that starts, drains and stops itself on a schedule. A tagged
release builds the exact program it runs; deploys wait for the game in
progress to finish. First rated games: a loss to a 2100 bot, a win
against a 1750.

## Games played in testing
About **33,100** games so far, counted from the recorded test runs (a
few hundred quick checks, such as self-play and container smoke tests,
aren't counted). Updated with each test.

Cost: the AWS bill was **$0.68** (checked by Charlie on 2026-10-24,
before the first Stockfish match). About 15,100 of the games up to then
ran on AWS (the rest were
local checks), so roughly **4.5 cents per 1,000 games**.

| What | Games |
|---|---|
| Milestone 3: proving the rig fair (first AWS runs, local checks, two 2,000-game self-matches) | 6,619 |
| Milestone 4 SPRTs: #16 503, #19 235, #21 588, #22 1,095, #23 1,572, #25 6,282, #28 392, #30 753, #31 264, #34 626 + 2,651, #35 5,827 + 2,695 + 1,815 | 25,298 |
| Stockfish matches: 2026-10-24 against 1800, 1900, then 2100 | 1,200 |
| **Total** | **33,117** |

## What makes it interesting
- **Two targets from one codebase:** a native engine (chess GUIs, Lichess
  bot) and a WebAssembly build that runs on the visitor's own computer, so
  the website demo costs about $0 and has no server to attack.
- **Proven, not guessed:** move generation is proven with perft (exact
  node counts), and every strength change with SPRT matches.
- **Architecture first:** each component is designed and written down
  before code, with the alternatives and reasons.

## How it's built
With Claude Code as a pair programmer: I set the goals, make the design
decisions, own the repo, CI and infrastructure, and review every PR; Claude
writes the Rust and drafts the docs. I don't write Rust, so the design docs
explain every choice in chess terms.
