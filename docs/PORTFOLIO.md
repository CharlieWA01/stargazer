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
Milestone 1 of 6 done (2026-10-20): Stargazer knows every rule of chess,
proven by perft on the six standard test positions, up to 119 million
positions deep from the start, all exact. It doesn't choose moves yet
(Milestone 2: search and evaluation).

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
