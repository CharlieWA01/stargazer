# Matches against Stockfish

Two ways to play Stargazer against Stockfish: a **match on AWS** for a
rough rating (hundreds of games, a few minutes), and **games on your own
machine** to watch.

## How the rating works

Stockfish has a strength setting: `UCI_LimitStrength` on, plus
`UCI_Elo` from 1320 to 3190. At a set level it plays weaker moves on
purpose, and its levels are tied to the CCRL rating list. If Stargazer
scores 50% against Stockfish at 1800, it plays at about 1800. Every 10
points of score above 50% is roughly +70 Elo.

Read the number as a rough guide:
- Stockfish's levels were set using longer games (about a minute each),
  so at fast time controls the estimate is looser.
- 400 games gives about ±30 Elo. Use the SPRT, not this, to compare two
  versions of Stargazer.
- The best level to play is one near Stargazer's strength. A score near
  50% gives the tightest estimate; a 90% score says little more than
  "much stronger".

## A match on AWS (the "Run match" button)

1. GitHub → **Actions** → **Run match** → **Run workflow**.
2. Fill in:
   - **test:** the Stargazer version (a branch, tag or commit; default
     `main`).
   - **opponent_elo:** Stockfish's level (default 1800).
   - **games:** an even number (default 400).
   - **tc:** the time control (default 10+0.1). Use 60+0.6 for a number
     closer to how Stockfish's levels were set; it takes about six times
     as long.
   - **vcpus:** default 16. AWS allows 16 at once, so don't start a match
     while an SPRT is running (or give each fewer).
3. The workflow builds Stargazer for ARM and plays the match on Fargate
   Spot with the SPRT container, which also holds Debian's Stockfish. It
   waits up to 30 minutes, then posts the result on its summary page:
   the score, the Elo difference, and the estimated rating (Stockfish's
   level plus the difference). The report names the exact Stockfish
   version.
4. If it took longer: run **SPRT report** with the run id from the
   summary (`match-<number>-<attempt>`).

Cost: the same rates as an SPRT. 400 games at 10+0.1 on 16 vCPUs takes
about 10 minutes, a few cents.

Record each result below.

## Results

| Date | Stargazer | Stockfish level | Games | tc | Score | Estimate | Run |
|---|---|---|---|---|---|---|---|

## On your own machine (to watch games)

On the Chromebook's Linux (Debian):

1. `sudo apt update && sudo apt install stockfish` (it lands in
   `/usr/games/stockfish`).
2. In PyChess: **Edit → Engines → New**, and pick `/usr/games/stockfish`.
3. In its options, turn **UCI_LimitStrength** on and set **UCI_Elo** to
   a level, say 1800. (**Skill Level**, 0 to 20, also weakens it but isn't
   tied to a rating.) At full strength it wins every game.
4. Start a new game with Stargazer as one player and Stockfish as the
   other.

To run the same match as the AWS one on any Linux machine, see the local
mode in `sprt.md`, and add `STARGAZER_MODE=match`,
`STARGAZER_OPPONENT_ELO=<level>` and `STARGAZER_STOCKFISH=<path to
stockfish>`; only `engines/test` is needed.
