"""Tests for report.py: turning a test's files into a report.

Run with: python3 -m unittest discover -s rig -v
"""

import unittest

from report import describe

REQUEST = {
    "task_arn": "arn:task",
    "test": "my-branch",
    "test_sha": "abcdef1234567890",
    "base": "main",
    "base_sha": "1234567890abcdef",
    "pr": "12",
    "elo0": "0",
    "elo1": "10",
    "tc": "8+0.08",
    "vcpus": "4",
    "image": "rig-0123",
    "workflow_run": "https://github.com/example/actions/runs/1",
}


def verdict(state, **results):
    return {
        "verdict": state,
        "reason": "because",
        "bench": {"base": 100, "test": 100},
        "results": {"games": 10, "wins": 4, "draws": 3, "losses": 3, "elo": 34.9,
                    "elo_error": 120.0, "llr": 0.2, "llr_bounds": [-2.94, 2.94], **results},
    }


class DescribeTest(unittest.TestCase):
    def test_a_finished_test(self):
        report, state = describe("sprt-1-1", REQUEST, verdict("pass"), "STOPPED", "")
        self.assertEqual(state, "pass")
        self.assertIn("**Pass**", report)
        self.assertIn("| 10 | 4 | 3 | 3 | +34.9 ± 120.0 |", report)
        self.assertIn("(same search)", report)
        self.assertIn("my-branch", report)
        self.assertNotIn("SPRT report", report)

    def test_still_running_points_to_the_report_workflow(self):
        report, state = describe("sprt-1-1", REQUEST, verdict("running"), "RUNNING", "")
        self.assertEqual(state, "running")
        self.assertIn("Still running", report)
        self.assertIn("SPRT report", report)

    def test_a_task_that_died_without_a_verdict_is_an_error(self):
        report, state = describe(
            "sprt-1-1", REQUEST, verdict("running"), "STOPPED", "OutOfMemoryError"
        )
        self.assertEqual(state, "error")
        self.assertIn("OutOfMemoryError", report)

    def test_no_results_yet(self):
        report, state = describe("sprt-1-1", REQUEST, None, "PROVISIONING", "")
        self.assertIsNone(state)
        self.assertIn("Starting", report)

    def test_unknown_elo(self):
        report, _ = describe("sprt-1-1", REQUEST, verdict("running", elo=None), "RUNNING", "")
        self.assertIn("not yet known", report)



MATCH_REQUEST = {**REQUEST, "mode": "match", "opponent_elo": "1800", "base": "", "base_sha": ""}


def match_verdict(state, **results):
    return {
        "verdict": state,
        "reason": "played every game",
        "bench": {"test": 924579},
        "opponent": "Stockfish 17",
        "results": {"games": 400, "wins": 180, "draws": 60, "losses": 160, "elo": 17.4,
                    "elo_error": 31.6, **results},
    }


class MatchTest(unittest.TestCase):
    def test_a_finished_match(self):
        report, state = describe("match-1-1", MATCH_REQUEST, match_verdict("done"), "STOPPED", "")
        self.assertEqual(state, "done")
        self.assertIn("### Match `match-1-1`", report)
        self.assertIn("**Done**", report)
        # 180 wins and 60 draws of 400: 210 points, 52.5%; 1800 + 17 = 1817.
        self.assertIn("| 400 | 180 | 60 | 160 | 52.5% | +17 ± 32 | 1817 ± 32 |", report)
        self.assertIn("Stockfish 17", report)
        self.assertIn("strength 1800", report)
        self.assertNotIn("LLR", report)
        self.assertNotIn("SPRT report", report)

    def test_a_weaker_result_lowers_the_estimate(self):
        verdict = match_verdict("done", elo=-120.0)
        report, _ = describe("match-1-1", MATCH_REQUEST, verdict, "STOPPED", "")
        self.assertIn("| -120 ± 32 | 1680 ± 32 |", report)

    def test_a_running_match_points_to_the_report_workflow(self):
        verdict = match_verdict("running", elo=None)
        report, _ = describe("match-1-1", MATCH_REQUEST, verdict, "RUNNING", "")
        self.assertIn("not yet known", report)
        self.assertIn("SPRT report", report)


if __name__ == "__main__":
    unittest.main()
