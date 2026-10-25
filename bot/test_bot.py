"""Checks on the Lichess bot's files (bot/), run in CI:
python3 -m unittest discover -s bot -v

They guard the settings a game depends on: a mistake here wouldn't show
until the bot lost games on Lichess.
"""

import re
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


def config_text():
    return (HERE / "config.yml").read_text()


def setting(name):
    """The value of a top-level or nested `name: value` line in config.yml."""
    found = re.search(rf"^\s*{re.escape(name)}:\s*([^#\n]*)", config_text(), re.M)
    return found.group(1).strip().strip('"') if found else None


class ConfigTest(unittest.TestCase):
    def test_stopping_finishes_the_game_first(self):
        # The evening drain and every deploy rely on this.
        self.assertEqual(setting("quit_after_all_games_finish"), "true")

    def test_no_token_in_the_file(self):
        self.assertEqual(setting("token"), "")

    def test_one_game_at_a_time_without_pondering(self):
        self.assertEqual(setting("concurrency"), "1")
        self.assertEqual(setting("ponder"), "false")

    def test_casual_only_for_now(self):
        text = config_text()
        block = re.search(r"^  modes:\n((?:    - .*\n)+)", text, re.M).group(1)
        modes = [line.split("#")[0].strip(" -") for line in block.splitlines()]
        self.assertEqual(modes, ["casual"])
        self.assertEqual(setting("challenge_mode"), "casual")

    def test_rapid_capped_at_10_plus_5(self):
        self.assertEqual(setting("max_base"), "600")
        self.assertEqual(setting("max_increment"), "5")

    def test_uci_options_exist_in_the_engine(self):
        # python-chess refuses an option the engine doesn't list, which
        # would stop every game from starting.
        uci = (HERE.parent / "src" / "uci.rs").read_text()
        offered = set(re.findall(r'"option name (.+?) type', uci))
        block = re.search(r"^  uci_options:.*\n((?:    .*\n)+)", config_text(), re.M).group(1)
        used = {line.split(":")[0].strip() for line in block.splitlines() if ":" in line}
        self.assertTrue(used)
        self.assertLessEqual(used, offered)


class FilesTest(unittest.TestCase):
    def test_service_stops_lichess_bot_gently(self):
        unit = (HERE / "stargazer-bot.service").read_text()
        self.assertIn("KillSignal=SIGINT", unit)   # lichess-bot ignores SIGTERM
        self.assertIn("KillMode=mixed", unit)      # not the engine mid-game

    def test_pins_are_exact(self):
        commit = (HERE / "LICHESS_BOT_COMMIT").read_text().strip()
        self.assertRegex(commit, r"^[0-9a-f]{40}$")
        engine = (HERE / "ENGINE").read_text()
        tag = re.search(r"^tag=(.*)$", engine, re.M).group(1)
        sha = re.search(r"^sha256=(.*)$", engine, re.M).group(1)
        if tag or sha:
            self.assertRegex(tag, r"^v\d+\.\d+\.\d+$")
            self.assertRegex(sha, r"^[0-9a-f]{64}$")


if __name__ == "__main__":
    unittest.main()
