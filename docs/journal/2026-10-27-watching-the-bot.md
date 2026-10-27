# 2026-10-27: Watching the bot (Milestone 5b)

## Goal
Decide how the Lichess bot is watched, and keep the engine side ready
for it. The infrastructure is cloud-homelab's (its journal
2026-10-27-stargazer-bot-monitoring).

## What was done
- Charlie took all five recommendations in the 5b decision doc, after
  pausing to ask for a shared logging and observability foundation (its
  own thread). The outcome: CloudWatch now, with the homelab's Grafana
  reading it in about a month.
- DESIGN.md section 6: the 5b part of the SRE layer marked decided;
  DECISIONS.md: the entry, with what changed from the proposal.
- `bot/log-samples.txt` and `bot/test_bot.py`: one example line per
  metric filter, a test that each pattern matches its own line and no
  other's, and a test (CI only) that lichess-bot at the pinned commit
  still writes each line. CI now fetches lichess-bot at that commit.

## Problems and fixes
- Reading lichess-bot's code (commit f2b38b4) showed it doesn't log
  declined challenges or reconnections. Two proposed SLOs became counts.

## How it was verified
- `python3 -m unittest discover -s bot -v`: passes offline (the source
  check skips), and passes with `LICHESS_BOT_SRC` pointing at a lichess-bot
  checkout of the pinned commit.
- Proved both new tests fail when they should: a wrong pattern failed
  the sample test, and a wrong source fragment failed the source test.
  Both were then restored.
- The sample lines follow lichess-bot's log format and messages; checking
  them against the live log is pending Charlie's grep output.

## Impact
Nothing changes in the engine or the bot's behaviour. A lichess-bot bump
now has to keep these four lines, or update the filters with it.
