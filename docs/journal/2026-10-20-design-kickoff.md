# 2026-10-20: Design kickoff: goals, browser target, the board

## Goal
Restart Stargazer from its skeleton: decide what "done" means, how the
engine will be shown off (portfolio site, Lichess), and the first design
question (how the engine stores a position). Then bring the repo up to the
shared documentation standard.

## What was done
1. **Read the repo against the plan.** The earlier idea was "compile it to
   WebAssembly for a portfolio tile". The repo showed Stargazer at Milestone 0:
   it answers the UCI handshake but can't play, and DESIGN.md forbids code
   before a section is Decided. So the browser tile became a design
   *constraint* to settle now and a Milestone 2 deliverable, not the next task.
2. **Lichess bot explained.** A fresh account (zero games) upgraded to a BOT
   account; lichess-bot (open-source Python) connects it to the engine over
   UCI; the connection is outbound only. Hosting options compared: own
   machine, homelab mini PC (chosen, later), always-on cloud server (about
   $5 to $15 a month; Lambda can't hold the connection). The username
   "Stargazer" was taken; Charlie registered another and left it unplayed.
3. **Goals settled.** About 1800 on Lichess with code Charlie understands;
   2000 maybe later. Rust and UCI kept. Repo private for now.
4. **NNUE and ML.** Charlie wants an ML project eventually. Discussed whether
   Stargazer's NNUE should be it; decided no (niche, not "from scratch", not
   needed for 1800). The evaluation will be swappable so NNUE stays possible.
5. **"Runs in a browser" constraint** recorded: no I/O or clock in the core,
   single-threaded in the browser, bridge crates outside the core.
6. **Board representation.** A write-up compared mailbox, bitboards and a
   hybrid in chess terms; Charlie chose the **hybrid**. Merged in PR #2 with
   the goals, constraint and tile plan (play in browser plus a Lichess link).
7. **Docs standard** adopted in PR #3: this journal, `DECISIONS.md` (the
   decision log moved out of DESIGN.md), roadmap, portfolio, interview
   stories, history, PR template and the two docs workflows.

## Problems and fixes
- **Jargon.** "Board representation" read as "a board you can see", which
  Lichess already has. Reframed: UCI is the API; the internal board is the
  database schema behind it. Worth remembering for later write-ups.
- **The Lichess username was taken.** No impact: the engine's UCI name and
  the account name don't need to match.

## How it was verified
- PR #2: `cargo fmt --check` and `cargo test` locally; CI green.
- PR #3: CI and the new `docs-check` run on the PR itself.

## Impact
No engine code yet. Milestone 1 (board and legal moves, proven by perft) is
current, with its first design question settled. No cost added.
