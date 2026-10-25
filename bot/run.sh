#!/bin/bash
# Starts lichess-bot (systemd runs this as the stargazer-bot user). The
# Lichess token is read from SSM Parameter Store with the server's own role,
# so it is never written to disk.
set -euo pipefail
LICHESS_BOT_TOKEN=$(/usr/local/bin/aws ssm get-parameter --region us-east-2 --name "$TOKEN_PARAMETER" \
  --with-decryption --query Parameter.Value --output text)
export LICHESS_BOT_TOKEN
cd /opt/stargazer-bot/lichess-bot/current
exec venv/bin/python lichess-bot.py --config /opt/stargazer-bot/config.yml \
  --logfile /var/log/stargazer-bot/lichess-bot.log --disable_auto_logging
