# 2026-10-20: First game in a GUI (Milestone 2 done)

## Goal
The last item of Milestone 2: Charlie plays Stargazer in a chess GUI.

## What was done
- Tried three GUIs on Charlie's Chromebook (Linux, Debian 12, amd64) and
  settled on PyChess from Debian's own packages. Added the engine with
  Edit → Engines → New, pointing at `target/release/stargazer`.
- Charlie (about 1000 on Lichess) played it, and Stargazer won.
- Marked Milestone 2 done in the roadmap; updated `PORTFOLIO.md` and the
  play-it runbook with the Chromebook setup.

## Problems and fixes
- `apt install cutechess`: no such package in Debian.
- En Croissant 0.15.0's `.deb` installed but wouldn't start
  (`GLIBC_2.39 not found`): it was built for a newer Linux than Debian 12,
  whose core C library is older. Removed it.
- `apt-cache policy pychess xboard polyglot` showed what Debian does have;
  PyChess worked first time.
- `~/Downloads` doesn't exist on the Linux side of a Chromebook, so the
  download landed in the repo folder; deleted it so git wouldn't see it.

## How it was verified
Charlie played a full game in PyChess under a clock: legal moves, a
finished game, no time trouble.

## Impact
Stargazer is a working chess engine. Next is Milestone 3: an SPRT rig, so
each strength change from here on is measured, not guessed.
