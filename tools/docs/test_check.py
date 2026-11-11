"""Fixture tests for the documentation checker."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

CHECK = Path(__file__).with_name("check.py")


class CheckerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def put(self, name, body):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body, encoding="utf-8")
        return path

    def call(self, *args):
        return subprocess.run([sys.executable, str(CHECK), "--root", str(self.root), *args], capture_output=True, text=True)

    def findings(self, rule, *paths):
        result = self.call("--only", rule, *paths)
        return result.returncode, result.stdout

    def test_status_missing_and_planned_and_bare_brace_glob(self):
        self.put("docs/code-map.md", "## Parts\n| Path | Status |\n|---|---|\n| `src/a.rs` | present |\n| `src/{b,c}.rs` | stub |\n| `nope/` | skeleton |\n| `planned.rs` | planned |\n")
        self.put("src/a.rs", "")
        self.put("src/b.rs", "")
        code, out = self.findings("status", "docs/code-map.md")
        self.assertEqual(code, 1)
        lines = out.splitlines()
        self.assertEqual(len(lines), 2, out)
        self.assertTrue(lines[0].startswith("docs/code-map.md:5: status: missing path: src/c.rs"), out)
        self.assertTrue(lines[1].startswith("docs/code-map.md:6: status: missing path: nope"), out)

    def test_status_root_paths_and_previous_dir(self):
        self.put("docs/code-map.md", "## Parts\n| Path | Status |\n|---|---|\n| `recipes/` | present |\n| `decky/` | skeleton |\n| `cmake/clang.cmake` | present |\n| `.github/workflows/check.yml` | present |\n| `crates/engine/src/a.rs`, `crates/engine/src/b.rs` | present |\n| `c.rs` | present |\n")
        for name in ("cmake/clang.cmake", ".github/workflows/check.yml", "crates/engine/src/b.rs", "crates/engine/src/c.rs", "crates/engine/src/a.rs"):
            self.put(name, "")
        (self.root / "recipes").mkdir()
        (self.root / "decky").mkdir()
        self.assertEqual(self.findings("status", "docs/code-map.md"), (0, ""))

    def test_status_relative_to_previous_path_dir(self):
        self.put("docs/code-map.md", "## Parts\n| Path | Status |\n|---|---|\n| `crates/engine/src/cli.rs`, `cli/link.rs` | present |\n| `games/zero-company/`, `ace-combat-7/` | present |\n| `crates/engine/src/cli.rs`, `cli/nowhere.rs` | present |\n")
        for name in ("crates/engine/src/cli.rs", "crates/engine/src/cli/link.rs"):
            self.put(name, "")
        (self.root / "games/zero-company").mkdir(parents=True)
        (self.root / "games/ace-combat-7").mkdir()
        code, out = self.findings("status", "docs/code-map.md")
        self.assertEqual(code, 1)
        lines = out.splitlines()
        self.assertEqual(len(lines), 1, out)
        self.assertTrue(lines[0].startswith("docs/code-map.md:6: status: missing path: cli/nowhere.rs"), out)

    def test_mermaid_header_lines_are_not_findings(self):
        self.put("docs/x.md", "```mermaid\nflowchart TD\nsubgraph X\nclassDef a fill:#fff\nA --> B\nend\n```\n")
        self.assertEqual(self.findings("mermaid", "docs/x.md"), (0, ""))

    def test_links_anchor_http_and_fence(self):
        self.put("docs/page.md", "[dead](missing.md) [anchor](#top) [web](https://example.com)\n```\n[ignored](also-missing.md)\n```\n")
        code, out = self.findings("links", "docs/page.md")
        self.assertEqual(code, 1)
        self.assertIn("missing.md", out)
        self.assertNotIn("also-missing", out)

    def test_links_root_and_title(self):
        self.put("docs/real.md", "")
        self.put("top.md", "")
        self.put("docs/page.md", '[a](/top.md) [b](real.md "title") [c](/nope.md) [d](gone.md "t")\n')
        code, out = self.findings("links", "docs/page.md")
        self.assertEqual(code, 1)
        self.assertEqual(sorted(l.split(": dead link: ")[1].split(" (")[0] for l in out.splitlines()), ["/nope.md", "gone.md"], out)

    def test_mermaid_shape_and_chain(self):
        self.put("docs/x.md", '```mermaid\nWatch --> Link["a: b"]    Watch --> Health["c"]\nA --> B --> C\nA -.-> B |"ok"|\n```\n')
        code, out = self.findings("mermaid", "docs/x.md")
        self.assertEqual(code, 1)
        self.assertEqual(len(out.splitlines()), 1, out)
        self.assertIn("docs/x.md:2: mermaid: 2 statements", out)

    def test_mermaid_ignores_other_fences_and_quoted_arrow(self):
        self.put("docs/x.md", '```bash\na b --> c d\n```\n```\nplain text here --> and more\n```\n```mermaid\nA["x --> y"] --> B\n```\n')
        self.assertEqual(self.findings("mermaid", "docs/x.md"), (0, ""))

    def test_usage_table_and_braced_format(self):
        cli = self.put("crates/engine/src/cli.rs", '"usage: flat3d-inject list | inject <appid>"\nformat!("usage: flat3d-inject {form}")\n')
        self.put("docs/code-map.md", "## Commands\n| `list` | List |\n")
        code, out = self.findings("usage", "docs/code-map.md")
        self.assertEqual(code, 1)
        self.assertIn("inject", out)
        cli.write_text('format!("usage: flat3d-inject {form}")\n', encoding="utf-8")
        code, out = self.findings("usage", "docs/code-map.md")
        self.assertEqual((code, out), (0, ""))

    def test_paths_exemption(self):
        self.put("docs/code-map.md", "inject/old\n<!-- mv-map -->\ninject/allowed\n<!-- /mv-map -->\ncrates/engine/src/a.rs\n")
        code, out = self.findings("paths", "docs/code-map.md")
        self.assertEqual(code, 1)
        self.assertIn(":1:", out)
        self.assertNotIn(":3:", out)

    def test_ids_spec_ruling_and_date(self):
        self.put("SPEC.md", "| M1 | supported |\n")
        self.put("docs/notes/DECISIONS.md", "## Rulings\n| 26 | okay |\n| 27 | okay |\nRuling 14 retired\n")
        self.put("docs/x.md", "M99, M1; rulings 26, 27; ruling 2026-10-31; ruling 14\n")
        code, out = self.findings("ids", "docs/x.md")
        self.assertEqual(code, 1)
        self.assertIn("M99", out)
        self.assertEqual([l.split(": ids: ")[1].split(" (")[0] for l in out.splitlines()], ["unknown spec item M99"], out)
        self.assertNotIn("unknown ruling", out)

    def test_ids_lists_ranges_and_and(self):
        self.put("SPEC.md", "")
        rows = "".join(f"| {n} | ok |\n" for n in (24, 25, 26, 50))
        self.put("docs/notes/DECISIONS.md", "## Rulings\n" + rows)
        self.put("docs/x.md", "rulings 24, 25, and 35\nrulings 24-26 and 50\nrulings 24-27 and 50\n")
        code, out = self.findings("ids", "docs/x.md")
        self.assertEqual(code, 1)
        self.assertEqual(sorted(l.split(": ids: ")[1].split(" (")[0] + "@" + l.split(":")[1] for l in out.splitlines()), ["unknown ruling 27@3", "unknown ruling 35@1"], out)

    def test_generated_markers_route_keys_and_runsheet(self):
        self.put("docs/x.md", "<!-- BEGIN GENERATED -->\n")
        self.put("crates/engine/src/recipes.rs", 'const UEVR_KEYS: &[&str] = &["alpha", "beta"];\nconst GEO11_KEYS: &[&str] = &["gamma"];\n')
        self.put("recipes/README.md", "<!-- BEGIN GENERATED -->\n`alpha` `gamma`\n<!-- END GENERATED -->\n")
        code, out = self.findings("generated", "docs/x.md", "recipes/README.md")
        self.assertEqual(code, 1)
        self.assertIn("beta", out)
        self.assertIn("markers", out)

    def test_private_allow_and_test_exemption(self):
        self.put("tools/docs/private-allow.txt", "docs/ok.md\tHyp" + "erion\n")
        self.put("docs/ok.md", "Hyp" + "erion\n")
        self.put("docs/bad.md", "Hyp" + "erion\n")
        self.put("docs/tests/sample.md", "Hyp" + "erion\n")
        code, out = self.findings("private", "docs/ok.md", "docs/bad.md", "docs/tests/sample.md")
        self.assertEqual(code, 1)
        self.assertIn("docs/bad.md", out)
        self.assertNotIn("docs/ok.md", out)
        self.assertNotIn("sample.md", out)

    def test_plans_and_repo_slug_and_allow_devnull(self):
        self.put("docs/plans/p.md", "Hyp" + "erion M99 [x](gone.md) inject/old\n")
        self.put("docs/ok.md", "see iannitram/flat-to-3d\n")
        self.put("docs/allowed.md", "Hyp" + "erion\n")
        self.put("tools/docs/private-allow.txt", "docs/allowed.md\n")
        self.assertEqual(self.call("docs/plans/p.md").returncode, 0)
        self.assertEqual(self.findings("private", "docs/ok.md"), (0, ""))
        self.assertEqual(self.findings("private", "docs/allowed.md"), (0, ""))
        result = self.call("--only", "private", "--allow", "/dev/null", "docs/allowed.md")
        self.assertEqual(result.returncode, 1)
        self.assertIn("docs/allowed.md:1: private:", result.stdout)

    def test_paths_changed_and_json(self):
        (self.root / ".git").mkdir()
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.put("docs/code-map.md", "clean\n")
        self.put("docs/b.md", "inject/legacy\n")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        email = "test" + "@example.com"
        subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=" + email, "commit", "-qm", "base"], check=True)
        self.put("docs/code-map.md", "changed\n")
        self.put("docs/b.md", "inject/legacy\n")
        result = self.call("--only", "paths", "--changed", "HEAD")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.put("docs/code-map.md", "inject/now\n")
        result = self.call("--only", "paths", "--changed", "HEAD", "--json")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads(result.stdout)[0]["rule"], "paths")

    def test_only_unknown_and_missing_paths(self):
        result = self.call("--only", "unknown")
        self.assertEqual(result.returncode, 2)
        self.assertIn("status", result.stderr)
        result = self.call("missing.md")
        self.assertEqual(result.returncode, 2)


if __name__ == "__main__":
    unittest.main()
