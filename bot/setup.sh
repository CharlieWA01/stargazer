#!/bin/bash
# Installs or updates Stargazer's Lichess bot on its server (Ubuntu 24.04,
# ARM). Run as root: on first boot by the server's user data (cloud-homelab,
# terraform/prod/stargazer-bot.tf), and for each deploy through Session
# Manager (docs/runbooks/lichess-bot.md).
#
#   setup.sh <stargazer commit> <SSM parameter holding the Lichess token>
#
# Everything comes from the stargazer repo at that one commit: lichess-bot's
# config, the lichess-bot commit to use (LICHESS_BOT_COMMIT) and the engine
# release to run (ENGINE, with its checksum). Running it again with the same
# commit changes nothing; with a new one, it installs the new versions and
# restarts the bot, which first finishes any game it's playing.
set -euo pipefail

COMMIT="$1"
TOKEN_PARAMETER="$2"
REPO="CharlieWA01/stargazer"
HOME_DIR=/opt/stargazer-bot

[[ "$COMMIT" =~ ^[0-9a-f]{40}$ ]] || { echo "need a full commit hash"; exit 1; }

# --- Packages: Python for lichess-bot, the AWS CLI for the token ------------
export DEBIAN_FRONTEND=noninteractive
apt-get update -q
apt-get install -y -q python3-venv git curl unzip
# AWS's own installer (not the snap, which doesn't run well as a hardened
# systemd service), into /usr/local/bin.
if [ ! -x /usr/local/bin/aws ]; then
  curl -fsSL -o /tmp/awscliv2.zip https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip
  unzip -q -o /tmp/awscliv2.zip -d /tmp && /tmp/aws/install && rm -rf /tmp/aws /tmp/awscliv2.zip
fi

# --- A user of its own, with no shell and no sudo ---------------------------
id stargazer-bot > /dev/null 2>&1 \
  || useradd --system --home-dir "$HOME_DIR" --shell /usr/sbin/nologin stargazer-bot
install -d -o stargazer-bot -g stargazer-bot "$HOME_DIR" "$HOME_DIR/engine" \
  "$HOME_DIR/games" "$HOME_DIR/state" /var/log/stargazer-bot

# --- This commit of the stargazer repo's bot/ folder ------------------------
src="$HOME_DIR/src/$COMMIT"
if [ ! -d "$src" ]; then
  rm -rf "$src.tmp"
  git clone --quiet --filter=blob:none --no-checkout "https://github.com/$REPO.git" "$src.tmp"
  git -C "$src.tmp" checkout --quiet "$COMMIT"
  mv "$src.tmp" "$src"
fi
bot="$src/bot"

# --- lichess-bot, at a pinned commit, in its own Python environment ---------
lb_commit=$(tr -d '[:space:]' < "$bot/LICHESS_BOT_COMMIT")
lb="$HOME_DIR/lichess-bot/$lb_commit"
if [ ! -d "$lb" ]; then
  rm -rf "$lb.tmp"
  git clone --quiet https://github.com/lichess-bot-devs/lichess-bot.git "$lb.tmp"
  git -C "$lb.tmp" checkout --quiet "$lb_commit"
  python3 -m venv "$lb.tmp/venv"
  "$lb.tmp/venv/bin/pip" install --quiet -r "$lb.tmp/requirements.txt"
  mv "$lb.tmp" "$lb"
fi

# --- The engine: one GitHub release, checked against its checksum -----------
tag=$(sed -n 's/^tag=//p' "$bot/ENGINE")
sha=$(sed -n 's/^sha256=//p' "$bot/ENGINE")
[ -n "$tag" ] && [ -n "$sha" ] || { echo "bot/ENGINE names no release yet"; exit 1; }
engine="$HOME_DIR/engine/stargazer-$tag"
if [ ! -x "$engine" ]; then
  curl -fsSL -o "$engine.tmp" \
    "https://github.com/$REPO/releases/download/$tag/stargazer-aarch64-linux"
  echo "$sha  $engine.tmp" | sha256sum --check --quiet
  chmod 755 "$engine.tmp"
  mv "$engine.tmp" "$engine"
fi
# The fixed search's node count fingerprints the build (docs/runbooks/sprt.md);
# it goes in the log, to compare with the release notes.
"$engine" bench | tail -n 1

# --- Point the bot at this version ------------------------------------------
ln -sfn "$engine" "$HOME_DIR/engine/stargazer"
ln -sfn "$lb" "$HOME_DIR/lichess-bot/current"
install -m 644 "$bot/config.yml" "$HOME_DIR/config.yml"
install -m 755 "$bot/run.sh" "$HOME_DIR/run.sh"
sed "s|@TOKEN_PARAMETER@|$TOKEN_PARAMETER|" "$bot/stargazer-bot.service" \
  > /etc/systemd/system/stargazer-bot.service
install -m 644 "$bot/logrotate.conf" /etc/logrotate.d/stargazer-bot
echo "$COMMIT" > "$HOME_DIR/deployed-commit"

# --- (Re)start. A stop sends SIGINT: the current game is finished first -----
systemctl daemon-reload
systemctl enable stargazer-bot
systemctl restart stargazer-bot
echo "Stargazer bot running: commit $COMMIT, engine $tag, lichess-bot $lb_commit"
