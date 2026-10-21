"""Tests for run_test.py: reading fastchess's output and choosing a verdict.

Run with: python3 -m unittest discover -s rig -v
"""

import unittest

from run_test import read_results, verdict_from

# Real fastchess output (v1.8.2-alpha), trimmed: four games, a summary, then
# the SPRT's verdict. "test vs base" means the test engine had White.
LOG = """\
Started game 1 of 800 (test vs base)
Started game 2 of 800 (base vs test)
Finished game 2 (base vs test): 0-1 {Black mates}
Finished game 1 (test vs base): 1/2-1/2 {Draw by 3-fold repetition}
Finished game 3 (test vs base): 0-1 {Black wins by adjudication}
Finished game 4 (base vs test): 0-1 {Black mates}
--------------------------------------------------
Results of test vs base (8+0.08, NULL, NULL, 8moves_v3.pgn):
Elo: 88.74 +/- 118.48, nElo: 160.83 +/- 196.58
LOS: 94.56 %, DrawRatio: 50.00 %, PairsRatio: 2.00
Games: 4, Wins: 2, Losses: 1, Draws: 1, Points: 2.5 (62.50 %)
Ptnml(0-2): [0, 0, 1, 1, 0], WL/DD Ratio: 0.00
LLR: 2.95 (100.4%) (-2.94, 2.94) [-300.00, -100.00]
--------------------------------------------------
SPRT ([-300.00, -100.00]) completed - H1 was accepted
Finished match
"""


class ReadResultsTest(unittest.TestCase):
    def test_counts_games_from_the_test_engines_side(self):
        numbers = read_results(LOG)
        # Game 2: test had Black and won. Game 1: draw. Game 3: test had
        # White and lost. Game 4: test had Black and won.
        self.assertEqual(
            (numbers["games"], numbers["wins"], numbers["draws"], numbers["losses"]),
            (4, 2, 1, 1),
        )

    def test_reads_the_latest_summary(self):
        numbers = read_results(LOG)
        self.assertEqual(numbers["elo"], 88.74)
        self.assertEqual(numbers["elo_error"], 118.48)
        self.assertEqual(numbers["llr"], 2.95)
        self.assertEqual(numbers["llr_bounds"], [-2.94, 2.94])
        self.assertEqual(numbers["pentanomial"], [0, 0, 1, 1, 0])
        self.assertEqual(numbers["summary_after_games"], 4)
        self.assertEqual(numbers["sprt"], "H1")

    def test_nan_becomes_null(self):
        # Early on fastchess can print "+/- nan", which isn't valid JSON.
        numbers = read_results("Elo: -136.97 +/- nan, nElo: -141.84 +/- 240.76\n")
        self.assertEqual(numbers["elo"], -136.97)
        self.assertIsNone(numbers["elo_error"])

    def test_an_interrupted_match_still_counts_its_games(self):
        interrupted = LOG.split("-----")[0] + "Tournament was interrupted.\n"
        numbers = read_results(interrupted)
        self.assertEqual(numbers["games"], 4)
        self.assertNotIn("sprt", numbers)
        self.assertNotIn("elo", numbers)


class VerdictTest(unittest.TestCase):
    def test_sprt_verdicts(self):
        self.assertEqual(verdict_from({"sprt": "H1"}, 0, None)[0], "pass")
        self.assertEqual(verdict_from({"sprt": "H0"}, 0, None)[0], "fail")

    def test_the_sprt_verdict_wins_over_a_late_stop(self):
        self.assertEqual(verdict_from({"sprt": "H0"}, 0, "deadline")[0], "fail")

    def test_stopped_early(self):
        verdict, reason = verdict_from({"games": 20}, 1, "deadline reached")
        self.assertEqual(verdict, "inconclusive")
        self.assertIn("deadline reached", reason)

    def test_ran_out_of_rounds(self):
        self.assertEqual(verdict_from({"games": 800}, 0, None)[0], "inconclusive")

    def test_fastchess_failed(self):
        self.assertEqual(verdict_from({"games": 0}, 1, None)[0], "error")


if __name__ == "__main__":
    unittest.main()
