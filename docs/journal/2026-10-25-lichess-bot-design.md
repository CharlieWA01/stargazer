# 2026-10-25: Lichess bot design (Milestone 5)

## Goal
Start Milestone 5 in parallel with Milestone 4's time-management SPRTs,
design first: price always-on hosting, check Lichess's bot rules and
lichess-bot's matchmaking against current docs, and sketch the SRE layer.

## What was done
- Checked the rules at the source. Lichess's server code (`BotLimit.scala`)
  caps each bot at 100 games against other bots a day. Bots play challenge
  games only (no pools, no tournaments except bot events) and no
  UltraBullet. The bot upgrade needs an account with no games and can't
  be undone.
- Read lichess-bot's config and code. Matchmaking settings that matter:
  `allow_matchmaking`, `challenge_timeout` (1 minute minimum),
  `challenge_initial_time` / `challenge_increment`,
  `opponent_rating_difference`, `challenge_filter`. It already reads the
  daily-limit error and waits for the reset (`lib/lichess.py`). It stops
  cleanly ("finish current games, then quit", with
  `quit_after_all_games_finish`) only on SIGINT, not SIGTERM.
- Priced the hosts (Fargate pricing page, EC2 instance prices): EC2
  t4g.small $17 to $20 a month always on, Fargate ARM 1 vCPU $32.50,
  0.5 vCPU $18 at half speed. Charlie asked whether EC2 could run on a
  schedule too: yes, a stopped instance pays only for its disk, so 12
  hours a day is about $9 to $10.
- Charlie chose EC2 t4g.small, 10am to 10pm Central (DECISIONS.md;
  DESIGN.md section 6).
- The full proposal (architecture diagram, SLOs, burn-rate alerts,
  drain-aware deploys gated on SPRT, chaos drills, the AI responder,
  security, phases 5a to 5f) is a Claude Doc shared in the project:
  "Stargazer Lichess bot: design proposal".

## Problems and fixes
- ECS gives a stopping container at most 120 seconds, far shorter than a
  game, so "deploys that let games finish" can't rely on the stop signal
  on Fargate. On EC2, systemd can wait as long as a game takes. A plain
  EC2 stop can still cut a game off, so the schedule drains first.
- lichess-bot ignores SIGTERM, which is what systemd sends by default:
  the service needs `KillSignal=SIGINT`.

## How it was verified
Rules and settings read from Lichess's and lichess-bot's current code and
docs (links in the design doc); prices from AWS's pricing pages. Nothing
built or applied yet.

## Impact
No cost yet. Once built: about $9 to $10 a month for the bot, under $1
for the monitoring around it. Still open: budgets, time controls, and
whether the repo goes public (needed for a deploy approval step).
