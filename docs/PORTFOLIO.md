# Stargazer: project summary

*The one page to read before a presentation, resume update or interview.
Details: `docs/journal/` (what happened), `docs/DECISIONS.md` (why),
`docs/DESIGN.md` (the architecture), `docs/interview/` (stories),
`docs/HISTORY.md` (timeline).*

## In one sentence
A chess engine written from scratch in Rust, aiming for about 1800 on
Lichess as a bot, and playable in any visitor's browser on my portfolio site
through WebAssembly.

## Status
Milestone 3 of 6 done (2026-10-22). Milestone 2 (2026-10-20) was the
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
first two (keeping the best move from an unfinished search, and a
transposition table that remembers searched positions) passed at +44 and
+136 Elo.

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
