"""Locked test: fps.py --per-eye (alternate-eye rendering, ruling 44: the 30 fps bar applies per eye).
With --per-eye the eyes take turns, so each eye sees half the game's frames: p50 and p1 are halved
before the --min gate, and the result says per_eye. Without the flag nothing changes but the new
per_eye key (false). No device, no network."""
import contextlib
import io
import json
import pathlib
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/stereo"))
import fps  # noqa: E402

# MangoHud preamble, value row, header, then rows. fps 60, 60, 25 -> p50 60; frametime 99th pct 40 -> p1 25.
CSV_60 = "os,cpu\nLinux,0\nfps,frametime\n60,10\n60,20\n25,40\n"
# fps 58, 58, 58 -> p50 58 (29 per eye, just under the 30 bar); frametime 20 -> p1 50 (25 per eye).
CSV_58 = "os,cpu\nLinux,0\nfps,frametime\n58,20\n58,20\n58,20\n"
ROWS_60 = ((60.0, 10.0), (60.0, 20.0), (25.0, 40.0))


class FpsPerEyeTests(unittest.TestCase):
    def run_main(self, text, *args):
        with tempfile.TemporaryDirectory() as td:
            path = pathlib.Path(td, "fps.csv")
            path.write_text(text, encoding="utf-8")
            out = io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
                code = fps.main([str(path), *args])
        return code, out.getvalue()

    def test_json_without_flag_keeps_the_numbers_and_says_per_eye_false(self):
        # Catches: halving applied always, or the per_eye key left out of the JSON.
        code, out = self.run_main(CSV_60, "--min", "30", "--json")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out), {"min": 30.0, "n": 3, "p1": 25.0, "p50": 60.0, "pass": True, "per_eye": False})

    def test_json_with_flag_halves_p50_and_p1_and_says_per_eye_true(self):
        # Catches: only p50 halved (p1 left at 25.0), or per_eye left false.
        code, out = self.run_main(CSV_60, "--min", "30", "--per-eye", "--json")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out), {"min": 30.0, "n": 3, "p1": 12.5, "p50": 30.0, "pass": True, "per_eye": True})

    def test_gate_uses_the_per_eye_rate(self):
        # Catches: the pass/exit gate still reading the whole-game p50 (58 >= 30 would pass).
        code, out = self.run_main(CSV_58, "--min", "30", "--per-eye", "--json")
        self.assertEqual(code, 1)
        self.assertEqual(json.loads(out), {"min": 30.0, "n": 3, "p1": 25.0, "p50": 29.0, "pass": False, "per_eye": True})

    def test_same_file_passes_without_the_flag(self):
        # Catches: --per-eye on by default.
        code, out = self.run_main(CSV_58, "--min", "30", "--json")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(out)["p50"], 58.0)

    def test_text_line_without_flag_is_unchanged(self):
        # Catches: a per_eye word added to the text line when the flag is not given.
        code, out = self.run_main(CSV_60, "--min", "30")
        self.assertEqual(code, 0)
        self.assertEqual(out, "n=3 p50=60.00 p1=25.00 pass=True\n")

    def test_text_line_with_flag_ends_with_per_eye(self):
        # Catches: text output not halved, or the suffix missing or misspelled.
        code, out = self.run_main(CSV_60, "--min", "30", "--per-eye")
        self.assertEqual(code, 0)
        self.assertEqual(out, "n=3 p50=30.00 p1=12.50 pass=True per_eye=True\n")

    def test_bad_input_with_flag_still_exits_2(self):
        # Catches: the flag path skipping the input error handling.
        code, out = self.run_main("no-data\n", "--per-eye", "--json")
        self.assertEqual(code, 2)
        self.assertEqual(out, "")

    def test_summarize_is_unchanged(self):
        # Catches: halving moved into summarize, which the session gates and test_stereo.py also call.
        self.assertEqual(fps.summarize(ROWS_60), {"n": 3, "p50": 60.0, "p1": 25.0})

    def test_sessions_readme_documents_the_flag(self):
        # Catches: the flag shipped without its line in the measurements section.
        text = (ROOT / "sessions/README.md").read_text(encoding="utf-8")
        self.assertIn("`--per-eye`", text)
        self.assertIn("halves p50 and p1", text)


if __name__ == "__main__":
    unittest.main()
