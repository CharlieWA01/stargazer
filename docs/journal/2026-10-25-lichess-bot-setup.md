# 2026-10-25: The Lichess bot's software and releases

## Goal
Build the parts of Milestone 5 that live in this repo: a way to release
the engine, and the bot's config and installer, so the server
(cloud-homelab) has something to run.

## What was done
- **Releases.** `.github/workflows/release.yml` runs on a `v*` tag: it
  checks the tag matches `Cargo.toml`, runs the tests, cross-compiles for
  64-bit ARM Linux and publishes `stargazer-aarch64-linux` with its
  SHA-256 and `bench` count. Charlie chose this over building on the
  server (DECISIONS.md).
- **`bot/`.** lichess-bot's `config.yml` (casual only, bullet and blitz
  plus rapid up to 10+5, increments up to 5 s, one game at a time,
  matchmaking against other bots), the pinned lichess-bot commit, the
  engine release (`ENGINE`, empty until the first tag), the installer
  `setup.sh`, `run.sh`, the systemd service and log rotation.
- **Checks.** `bot/test_bot.py` (run in CI) checks the settings games
  depend on: the bot finishes games before quitting, there's no token in
  the file, one game at a time, casual only, the time limits, the UCI
  options exist in `src/uci.rs`, the service stops with SIGINT, and the
  pins are exact.
- **Runbook:** `docs/runbooks/lichess-bot.md` (token, BOT upgrade, first
  release, deploys, rollback).

## Problems and fixes
- The design said Amazon Linux 2023; Ubuntu 24.04 was picked instead for
  automatic security updates and parity with CI (DECISIONS.md). The
  engine only needs glibc 2.34, so either would run it.
- Installing the AWS CLI as a snap under a locked-down systemd service is
  fragile, so `setup.sh` uses AWS's own installer.
- A test for "casual only" first matched the word "rated" in a comment;
  it now reads values, not comments.

## How it was verified
- lichess-bot's own config loader (at the pinned commit) accepted
  `config.yml`, warning only about Lichess's 100 bot games a day, which
  lichess-bot handles by waiting.
- python-chess drove Stargazer with the config's UCI options and got a
  move.
- `python3 -m unittest discover -s bot -v`: 8 tests pass; each was broken
  on purpose once to see it fail.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- The release workflow and `setup.sh` run for real only after merge (the
  first tag, then the server's first boot); the runbook covers both.

## The first release
Charlie tagged `v0.1.0` after the repo went public. The Release workflow
took about a minute and published `stargazer-aarch64-linux` (SHA-256
`c320870f…01f05e4`, bench 343857 nodes, the same count as `main`'s
SPRT base). The downloaded file matched its checksum and is a 64-bit ARM
Linux program. `bot/ENGINE` now names it.

## First night on Lichess
- First boot failed (the server's firewall blocked Ubuntu's HTTP package
  mirrors; fixed in cloud-homelab), then `setup.sh` ran cleanly: engine
  v0.1.0, bench 343857.
- Its challenges went unanswered for a few minutes: lichess-bot looked
  for opponents rated 2700 to 3300. A new BOT account's rating starts
  high, `opponent_rating_difference` is measured from it, and casual
  games never change it. The first game it got was against a 3000 bot.
  `config.yml` now uses a fixed range, 1600 to 2400, around the ~2180
  estimate from the Stockfish matches (DECISIONS.md).

- Many bots turned down casual challenges ("Please send me a rated
  challenge instead"). Charlie chose rated play now rather than after
  Milestone 4 (DECISIONS.md, 2026-10-26).

- First rated games, 2026-10-26: a loss to a 2100 bot (a blunder, then
  time pressure), a win against a 1750. PORTFOLIO.md and an interview
  story ("Launch night") record the launch.
- Housekeeping, 2026-10-26: DESIGN.md section 6 no longer says Amazon
  Linux 2023 for the host, notes the portfolio tile links to Lichess
  until the browser version exists, and says prod's budget rose (past
  tense); the roadmap's Milestone 5 line says the bot is live.

## Impact
Nothing changes for the engine. The bot is ready to install once the
first release exists and the server is built. The repo gains one
workflow with write access to releases (`contents: write`), triggered
only by tags.
