# Runbook: the Lichess bot

Stargazer plays on Lichess from one small ARM server in workloads-prod,
10am to 10pm Central (DESIGN.md section 6). This repo holds the bot's
software (`bot/`) and its releases; cloud-homelab holds the server
(`terraform/prod/stargazer-bot.tf`, runbook `stargazer-bot.md` there).

Commands below that use AWS run on the Chromebook with the
`cwa-prod-admin` profile, in `us-east-2`.

## How it fits together

- A **release** is a tag `vX.Y.Z` matching `Cargo.toml`. The `Release`
  workflow builds `stargazer-aarch64-linux` and publishes it with its
  SHA-256 and `bench` count on the repo's Releases page.
- `bot/ENGINE` names the release the bot runs (tag and checksum);
  `bot/LICHESS_BOT_COMMIT` pins lichess-bot; `bot/config.yml` is
  lichess-bot's config (no token).
- `bot/setup.sh <commit> <token parameter>` installs exactly what that
  commit of this repo says, then restarts the bot. A restart sends SIGINT:
  lichess-bot finishes the current game, takes no new ones, and exits;
  systemd then starts the new version.
- The Lichess token lives only in SSM Parameter Store
  (`/cwa/stargazer-bot/lichess-token`); `bot/run.sh` reads it at each start.

## One-time setup (Charlie)

1. **Token.** Signed in to Lichess *as the bot account*, create a token
   with only the "Play games with the bot API" (`bot:play`) scope:
   <https://lichess.org/account/oauth/token/create?scopes[]=bot:play>.
2. **Store it** (it's typed, never shown or saved in shell history):
   ```bash
   read -rsp "Lichess token: " TOKEN; echo
   aws ssm put-parameter --profile cwa-prod-admin --region us-east-2 \
     --name /cwa/stargazer-bot/lichess-token --type SecureString --value "$TOKEN"
   unset TOKEN
   ```
3. **Upgrade the account to a BOT.** Only works on an account that has
   never played a game, and **can't be undone**:
   ```bash
   read -rsp "Lichess token: " TOKEN; echo
   curl -d '' https://lichess.org/api/bot/account/upgrade -H "Authorization: Bearer $TOKEN"
   unset TOKEN
   ```
   `{"ok":true}` means done. Add a line to the bot's Lichess profile
   linking to this repo.
4. **First release:** `git tag v0.1.0 origin/main && git push origin v0.1.0`
   (the version in `Cargo.toml`). When the `Release` workflow finishes,
   Claude copies the tag and checksum into `bot/ENGINE` in a PR.
5. **The server:** after that PR merges, cloud-homelab's runbook
   `stargazer-bot.md` (plan, apply, first boot with that commit).

## Is it running?

- Lichess: the bot's profile shows it online (between 10am and 10pm).
- Logs: CloudWatch, log group `/cwa/stargazer-bot`.
- A shell on the server (no SSH; Session Manager):
  ```bash
  aws ssm start-session --profile cwa-prod-admin --region us-east-2 --target <instance id>
  systemctl status stargazer-bot
  cat /opt/stargazer-bot/deployed-commit
  tail -n 50 /var/log/stargazer-bot/lichess-bot.log
  ```
  The instance id is `terraform output stargazer_bot_instance_id` in
  cloud-homelab's `terraform/prod`.

## Deploy a new engine (or config)

Only during playing hours (the server is off otherwise).

1. Tag the release (`vX.Y.Z`, matching `Cargo.toml`; bump it first in a
   PR). Strength changes in it must have passed their SPRT.
2. A PR sets `bot/ENGINE` to the new tag and the checksum from the
   release page. Merge it; note the merge commit.
3. Run the installer at that commit:
   ```bash
   C=<merge commit, 40 characters>
   ID=<instance id>
   aws ssm send-command --profile cwa-prod-admin --region us-east-2 \
     --instance-ids "$ID" --document-name AWS-RunShellScript \
     --comment "Deploy $C" \
     --parameters "commands=[\"curl -fsSL https://raw.githubusercontent.com/CharlieWA01/stargazer/$C/bot/setup.sh -o /root/setup.sh\",\"bash /root/setup.sh $C /cwa/stargazer-bot/lichess-token\"],executionTimeout=[\"4800\"]"
   ```
   It waits for the current game to end, so it can take up to an hour.
   Its last line says which commit, engine and lichess-bot are running.

**Roll back:** the same command with the previous commit (everything it
needs is still on the server).

## Pause the bot

`sudo systemctl stop stargazer-bot` in a Session Manager shell (finishes
the game, then stops). It starts again with the next 10am start, or
`sudo systemctl start stargazer-bot`.
