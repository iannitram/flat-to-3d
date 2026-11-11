import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "issues_report.py"
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "issues.json"


class IssuesReportTests(unittest.TestCase):
    def run_report(self, *args, cwd=None, env=None):
        merged = os.environ.copy()
        if env:
            merged.update(env)
        return subprocess.run(["python3", str(SCRIPT), *map(str, args)], cwd=cwd, text=True,
                              capture_output=True, env=merged)

    def test_fixture_statuses_and_exact_json_keys(self):
        result = self.run_report("--json-file", FIXTURE, "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        rows = json.loads(result.stdout)
        by_id = {row["number"]: row for row in rows}
        self.assertEqual(by_id[1]["status"], "blocked")
        self.assertEqual(by_id[1]["blockers"], [113, 130])
        self.assertEqual(by_id[2]["status"], "blocked-external")
        self.assertEqual(by_id[3]["status"], "ready")
        self.assertEqual(by_id[4]["status"], "ready")
        self.assertEqual(by_id[5]["status"], "ready")
        self.assertEqual(by_id[6]["status"], "blocked")
        self.assertTrue(by_id[6]["device"])
        self.assertEqual(set(by_id[1]), {"number", "title", "status", "blockers", "external", "device"})

    def test_none_lines_never_block_even_with_refs_or_bold_markers(self):
        issues = [
            {"number": 1, "title": "a", "body": "Blocked by: none (was #2)", "labels": []},
            {"number": 2, "title": "b", "body": "**Blocked by**: none", "labels": []},
            {"number": 3, "title": "c", "body": "**Blocked by**: #2", "labels": []},
            {"number": 4, "title": "d", "body": "**Blocked by**: Step 0", "labels": []},
        ]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "issues.json"
            path.write_text(json.dumps(issues), encoding="utf-8")
            result = self.run_report("--json-file", path, "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        by_id = {row["number"]: row for row in json.loads(result.stdout)}
        self.assertEqual((by_id[1]["status"], by_id[1]["blockers"]), ("ready", []))
        self.assertEqual((by_id[2]["status"], by_id[2]["external"]), ("ready", []))
        self.assertEqual((by_id[3]["status"], by_id[3]["blockers"]), ("blocked", [2]))
        self.assertEqual((by_id[4]["status"], by_id[4]["external"]), ("blocked-external", ["Step 0"]))

    def make_plan(self, root, *, wp02_files="tools/check.sh, tools/pm/issues_report.py", wp02_dep="WP-00",
                  wp02_body_dep=None, wp02_wave=1, extra_rows="", frozen="- `tools/ghidra/headless.sh` (WP-02, WP-25b)"):
        plan = root / "docs" / "plans" / "roadmap-v2"
        plan.mkdir(parents=True, exist_ok=True)
        rows = ["- WP-00 (wave 0) Format gate | closes K | files: .gitignore | depends: - | device: none",
                f"- WP-02 (wave {wp02_wave}) Repo tooling | closes K9 | files: {wp02_files} | depends: {wp02_dep} | device: none"]
        if extra_rows:
            rows.extend(extra_rows.splitlines())
        (plan / "03-wp-index.md").write_text("Waves: w0 = 00; w1 = 02\n\n" + "\n".join(rows) + "\n", encoding="utf-8")
        (plan / "WP-00.md").write_text("**Files:**\n- `.gitignore`\n\n**Depends on:** none\n", encoding="utf-8")
        (plan / "WP-02.md").write_text(
            "**Files (exclusive):**\n- `tools/check.sh`\n- `tools/pm/issues_report.py`\n\n**Depends on:** " + wp02_dep + ".\n",
            encoding="utf-8")
        if wp02_body_dep is not None:
            body = (plan / "WP-02.md").read_text(encoding="utf-8").replace("**Depends on:** " + wp02_dep + ".", "**Depends on:** " + wp02_body_dep + ".")
            (plan / "WP-02.md").write_text(body, encoding="utf-8")
        for extra in extra_rows.splitlines():
            match = __import__("re").match(r"- (WP-\d{2}[a-d]?) \(wave \d+\).*? \| files: (.*?) \| depends: (.*?) \|", extra)
            if match:
                wp, files, deps = match.groups()
                file_lines = "\n".join(f"- `{part.strip()}`" for part in files.split(","))
                (plan / f"{wp}.md").write_text(f"**Files:**\n{file_lines}\n\n**Depends on:** {deps}.\n", encoding="utf-8")
        (plan / "00-run-the-pass.md").write_text("Frozen after wave 1\n" + frozen + "\nEnforcement\n", encoding="utf-8")
        (plan / "02-queues-and-decisions.md").write_text(
            "## 5. Owner-device queue\n| Q | What | Driver | Needs WPs | Closes |\n|---|---|---|---|---|\n"
            "| Q0 | pins | `sessions/q0.toml` | 02 | K22 |\n| Q1 | other | x | Q0 | — |\n"
            "## 7. Owner decisions\n| OD | Question | Default | Blocks if overruled |\n|---|---|---|---|\n"
            "| OD8 | pins | **approve** | Q0 |\n",
            encoding="utf-8")
        return plan

    def init_repo(self, root):
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
        subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.test"], check=True)

    def commit(self, root, subject):
        subprocess.run(["git", "-C", str(root), "add", "-A"], check=True)
        subprocess.run(["git", "-C", str(root), "commit", "-q", "-m", subject], check=True)

    def test_roadmap_uses_environment_base_for_both_modes_and_explicit_wins(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            root.mkdir()
            plan = self.make_plan(root)
            (root / ".gitignore").write_text("target/\n")
            self.init_repo(root)
            self.commit(root, "base")
            base = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
            tree = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD^{tree}"], text=True).strip()
            side = subprocess.check_output(["git", "-C", str(root), "commit-tree", tree, "-p", base, "-m", "side"], text=True).strip()
            wp00 = subprocess.check_output(["git", "-C", str(root), "commit-tree", tree, "-p", base, "-p", side,
                                              "-m", "Merge branch 'v2/wp-00-format' into test"], text=True).strip()
            merged = subprocess.check_output(["git", "-C", str(root), "commit-tree", tree, "-p", wp00, "-p", side,
                                              "-m", "Merge branch 'v2/wp-02-tooling' into test"], text=True).strip()
            subprocess.run(["git", "-C", str(root), "update-ref", "refs/heads/env-base", merged], check=True)
            subprocess.run(["git", "-C", str(root), "update-ref", "refs/heads/arg-base", wp00], check=True)
            env = {"FLAT3D_BASE": "env-base"}
            noarg = self.run_report("--roadmap", "--json", "--plan-dir", plan, cwd=root, env=env)
            self.assertEqual(noarg.returncode, 0, noarg.stderr)
            self.assertNotIn("WP-02", json.loads(noarg.stdout)["ready"])
            explicit = self.run_report("--roadmap", "--json", "--plan-dir", plan, "--base", "arg-base", cwd=root, env=env)
            self.assertEqual(explicit.returncode, 0, explicit.stderr)
            self.assertIn("WP-02", json.loads(explicit.stdout)["ready"])

    def test_good_plan_has_no_lint(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            root.mkdir()
            plan = self.make_plan(root)
            (root / ".gitignore").write_text("target/\n")
            self.init_repo(root)
            self.commit(root, "base")
            result = self.run_report("--roadmap", "--json", "--plan-dir", plan, "--base", "HEAD", cwd=root)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            self.assertEqual(json.loads(result.stdout)["lint"], [])

    def test_each_plan_lint_rule_fires_once(self):
        cases = {
            "files-drift": dict(wp02_files="tools/check.sh"),
            "depends-drift": dict(wp02_body_dep="-"),
            "depends-wave": dict(wp02_dep="WP-03", extra_rows="- WP-03 (wave 2) Later | closes K | files: tools/late.sh | depends: - | device: none"),
            "wave-file-clash": dict(extra_rows="- WP-03 (wave 1) Clash | closes K | files: tools/check.sh | depends: - | device: none"),
            "frozen-path": dict(extra_rows="- WP-03 (wave 2) Frozen | closes K | files: tools/ghidra/headless.sh | depends: WP-02 | device: none",
                                wp02_wave=1),
        }
        for expected, options in cases.items():
            with self.subTest(rule=expected), tempfile.TemporaryDirectory() as temp:
                root = Path(temp) / "repo"
                root.mkdir()
                plan = self.make_plan(root, **options)
                self.init_repo(root)
                self.commit(root, "base")
                result = self.run_report("--roadmap", "--json", "--plan-dir", plan, "--base", "HEAD", cwd=root)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertTrue(result.stdout, result.stderr)
                rules = [x["rule"] for x in json.loads(result.stdout)["lint"]]
                self.assertEqual(rules.count(expected), 1, rules)

    def test_scope_uses_base_and_reports_each_out_of_scope_file(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            root.mkdir()
            plan = self.make_plan(root)
            (root / ".gitignore").write_text("target/\n")
            (root / "tools").mkdir()
            (root / "tools" / "check.sh").write_text("old\n")
            self.init_repo(root)
            self.commit(root, "base")
            base = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
            (root / "tools" / "check.sh").write_text("new\n")
            (root / "README.md").write_text("unexpected\n")
            env = {"FLAT3D_BASE": base}
            bad = self.run_report("--scope", "WP-02", "--plan-dir", plan, cwd=root, env=env)
            self.assertEqual(bad.returncode, 1)
            self.assertIn("README.md is not in WP-02's files", bad.stderr)
            (root / "README.md").unlink()
            good = self.run_report("--scope", "WP-02", "--plan-dir", plan, cwd=root, env=env)
            self.assertEqual(good.returncode, 0, good.stderr)


if __name__ == "__main__":
    unittest.main()
