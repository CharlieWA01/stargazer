"""Checks on the Lichess bot's files (bot/), run in CI:
python3 -m unittest discover -s bot -v

They guard the settings a game depends on: a mistake here wouldn't show
until the bot lost games on Lichess.
"""

import os
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

    def test_rated_play(self):
        # Rated from 2026-10-26: accepts both, challenges rated.
        text = config_text()
        block = re.search(r"^  modes:\n((?:    - .*\n)+)", text, re.M).group(1)
        modes = [line.split("#")[0].strip(" -") for line in block.splitlines()]
        self.assertEqual(modes, ["casual", "rated"])
        self.assertEqual(setting("challenge_mode"), "rated")

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


# The lines cloud-homelab's metric filters count in lichess-bot's log
# (terraform/prod/stargazer-observer.tf, DECISIONS.md 2026-10-27 there).
# Keep these patterns the same as that file's `bot_log_metrics`, in
# CloudWatch's syntax: a "quoted" term must be in the line, a -word must
# not be.
LOG_METRICS = {
    "ChallengesAccepted": '"INFO Accept "',
    "ConnectionDrops": '"Control stream error, reconnecting"',
    "Restarts": '"Will restart lichess-bot"',
    "ChallengesRefused": '"ERROR {\'error\':"',
    "Errors": '" ERROR " -matchmaking',
}

# Where lichess-bot's code writes each line (the level for Errors comes
# from its log format, not one message).
LOG_SOURCES = {
    "ChallengesAccepted": 'logger.info(f"Accept {',
    "ConnectionDrops": 'logger.warning(f"Control stream error, reconnecting',
    "Restarts": 'logger.info("Will restart lichess-bot")',
    "ChallengesRefused": "logger.error(response)",
    "Errors": '%(levelname)s %(message)s',
}


def matches(pattern, line):
    """Does a CloudWatch text pattern (quoted terms, -word exclusions) match the line?"""
    terms = re.findall(r'(-?)(?:"([^"]*)"|(\S+))', pattern)
    for minus, quoted, word in terms:
        found = (quoted or word) in line
        if found == bool(minus):
            return False
    return True


def log_samples():
    samples = {}
    for line in (HERE / "log-samples.txt").read_text().splitlines():
        if line and not line.startswith("#"):
            name, sample = line.split("\t", 1)
            samples[name] = sample
    return samples


class LogMetricsTest(unittest.TestCase):
    def test_every_pattern_matches_its_sample_line(self):
        samples = log_samples()
        self.assertEqual(set(samples), set(LOG_METRICS))
        for name, pattern in LOG_METRICS.items():
            with self.subTest(name):
                self.assertTrue(matches(pattern, samples[name]), samples[name])

    def test_patterns_dont_match_each_others_lines(self):
        samples = log_samples()
        for name, pattern in LOG_METRICS.items():
            for other, sample in samples.items():
                if other != name:
                    with self.subTest(pattern=name, line=other):
                        self.assertFalse(matches(pattern, sample), sample)

    @unittest.skipUnless(os.environ.get("LICHESS_BOT_SRC"),
                         "set LICHESS_BOT_SRC to a lichess-bot checkout (CI does)")
    def test_pinned_lichess_bot_still_writes_these_lines(self):
        """A lichess-bot bump that rewords a message would silently zero a metric."""
        src = Path(os.environ["LICHESS_BOT_SRC"])
        code = "\n".join(p.read_text() for p in (src / "lib").glob("*.py"))
        for name, fragment in LOG_SOURCES.items():
            with self.subTest(name):
                self.assertIn(fragment, code)


if __name__ == "__main__":
    unittest.main()
