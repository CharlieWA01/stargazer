# The Lichess bot

Stargazer's Lichess bot: lichess-bot running the Stargazer engine on one
ARM server, 10am to 10pm Central. Design: `docs/DESIGN.md` section 6.
How to set it up, check it and deploy: `docs/runbooks/lichess-bot.md`.

| File | What it is |
|---|---|
| `config.yml` | lichess-bot's settings (no token) |
| `LICHESS_BOT_COMMIT` | the lichess-bot commit to install |
| `ENGINE` | the Stargazer release to run, with its SHA-256 |
| `setup.sh` | installs or updates everything above on the server, then restarts the bot |
| `run.sh` | starts lichess-bot with the token from SSM Parameter Store |
| `stargazer-bot.service` | the systemd service; stopping it finishes the current game first |
| `logrotate.conf` | keeps a week of logs on the server |
| `test_bot.py` | checks the settings a game depends on (run in CI) |
