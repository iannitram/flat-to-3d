import os
import pathlib
import re
import shutil
import subprocess
import tempfile
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "tools/re/analyze.sh"
SRC = "/tmp/Spider-Man.exe"
DST = "/tmp/Spider-Man2.exe"
SRC_SLUG = "remastered-4.630"
DST_SLUG = "sm2-2.810"

# Prints `<slug> <sha> <ver>` chosen by the exe's basename; the files need not exist.
EXE_ID_STUB = """#!/bin/bash
case $(basename "$1") in
  Spider-Man.exe) echo "remastered-4.630 e297d4d94f1ffe4febf289745e79e7b6fa233a788e7a00f480fc77c55db81ad1 4.630.0.0" ;;
  Spider-Man2.exe) echo "sm2-2.810 c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9 2.810.0.0" ;;
  *) echo "stub: unknown $1" >&2; exit 3 ;;
esac
"""

# Logs its argv (one call per line) to $STANDIN_LOG; for the vt call it writes the fixture CSV to <out.csv>,
# the argument after the session path that follows ExportVtMatches.java. Exits $STANDIN_RC (default 0).
HEADLESS_STANDIN = """#!/bin/bash
echo "$*" >> "$STANDIN_LOG"
rc=${STANDIN_RC:-0}
[ "$rc" -eq 0 ] || exit "$rc"
prev2= prev1=
for a in "$@"; do
  if [ "$prev2" = ExportVtMatches.java ]; then cp "$STANDIN_CSV" "$a"; fi
  prev2=$prev1 prev1=$a
done
exit 0
"""

VT_CSV = (
    "src_rva,dst_rva,status,similarity,confidence,correlator\n"
    "0x1646e10,0x2c00010,AVAILABLE,0.5,1.0,Exact Function Bytes Match\n"
    "0x164703f,0x2c00230,ACCEPTED,1.0,10.0,Exact Function Instructions Match\n"
    "0x1646e10,0x2c76310,ACCEPTED,0.9,2.0,Combined Function and Data Reference Match\n"
)

LAYOUT = ["tools/re/analyze.sh", "tools/re/exe_id.py", "tools/re/fill_map.py", "tools/xmatch/targets.py",
          "games/spiderman2/map.toml", "docs/spiderman2/address-map.md"]


def executable(path, text):
    path.write_text(text, encoding="utf-8", newline="\n")
    path.chmod(0o755)
    return path


def re_home():
    return pathlib.Path(os.environ.get("FLAT3D_RE_HOME") or pathlib.Path.home() / ".local/share/flat-to-3d")


class AnalyzeSh(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        self.proj = self.dir / "proj"
        self.stub = executable(self.dir / "exe_id_stub.sh", EXE_ID_STUB)
        self.log = self.dir / "headless.log"
        self.csv_fixture = self.dir / "fixture.csv"
        self.csv_fixture.write_text(VT_CSV, encoding="utf-8")
        self.standin = executable(self.dir / "headless_standin.sh", HEADLESS_STANDIN)

    def tearDown(self):
        self.tmp.cleanup()

    def env(self, **kw):
        e = {k: v for k, v in os.environ.items() if k not in ("FLAT3D_HEADLESS", "FLAT3D_EXE_ID")}
        e.update({"FLAT3D_EXE_ID": str(self.stub), "STANDIN_LOG": str(self.log), "STANDIN_CSV": str(self.csv_fixture)})
        e.update(kw)
        return e

    def run_sh(self, script, *args, **env):
        return subprocess.run(["bash", str(script), *args], capture_output=True, text=True, env=self.env(**env),
                              cwd=str(self.dir))

    def analyze_line(self, exe, slug):
        return (f"tools/ghidra/headless.sh {self.proj} spiderman/{slug} -import {exe} -max-cpu 16 "
                f"-log {self.proj}/logs/analyze-{slug}.log")

    def vt_pattern(self, csv):
        return re.compile(
            re.escape(f"tools/ghidra/headless.sh {self.proj} spiderman/{DST_SLUG} -process Spider-Man2.exe "
                      "-preScript Flat3dVtOptions.java -postScript AutoVersionTrackingScript.java / ") + r"(vt-[0-9]+)" +
            re.escape(f" /{SRC_SLUG}/Spider-Man.exe -postScript ExportVtMatches.java /") + r"(vt-[0-9]+)" +
            re.escape(f" {csv} -scriptPath tools/ghidra") + "$")

    def test_analyze_dry_run_prints_the_argv(self):
        r = self.run_sh(SCRIPT, "analyze", str(self.proj), SRC, "--dry-run")
        self.assertEqual((r.returncode, r.stderr), (0, ""))
        self.assertEqual(r.stdout, self.analyze_line(SRC, SRC_SLUG) + "\n")
        self.assertFalse(self.proj.exists())

    def test_vt_dry_run_prints_the_argv(self):
        out = self.dir / "out.csv"
        r = self.run_sh(SCRIPT, "vt", str(self.proj), SRC, DST, str(out), "--dry-run")
        self.assertEqual((r.returncode, r.stderr), (0, ""))
        lines = r.stdout.splitlines()
        self.assertEqual(len(lines), 1)
        m = self.vt_pattern(out).match(lines[0])
        self.assertIsNotNone(m, lines[0])
        self.assertEqual(m.group(1), m.group(2))
        self.assertNotIn("-readOnly", lines[0])
        self.assertNotIn("-folderPath", lines[0])

    def test_all_dry_run_lists_every_step(self):
        out = self.dir / "out.csv"
        r = self.run_sh(SCRIPT, "all", str(self.proj), SRC, DST, str(out), "--dry-run")
        self.assertEqual((r.returncode, r.stderr), (0, ""))
        lines = r.stdout.splitlines()
        self.assertEqual(len(lines), 4)
        self.assertEqual(lines[0], self.analyze_line(SRC, SRC_SLUG))
        self.assertEqual(lines[1], self.analyze_line(DST, DST_SLUG))
        self.assertRegex(lines[2], self.vt_pattern(out))
        self.assertEqual(lines[3], f"python3 tools/re/fill_map.py --map games/spiderman2/map.toml --vt {out} "
                                   "--targets tools/xmatch/targets.py --doc docs/spiderman2/address-map.md")
        self.assertFalse(self.log.exists())

    def test_bad_usage_exits_2(self):
        for args in ([], ["nope"], ["analyze", str(self.proj)], ["vt", str(self.proj), SRC, DST]):
            r = self.run_sh(SCRIPT, *args, "--dry-run")
            self.assertEqual(r.returncode, 2, args)
            self.assertIn("usage: analyze.sh", r.stderr)

    def copy_layout(self):
        repo = self.dir / "repo"
        for rel in LAYOUT:
            (repo / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / rel, repo / rel)
        return repo

    def test_all_runs_the_steps_in_order_then_fill_map(self):
        repo = self.copy_layout()
        out = self.dir / "state/re/vt-matches.csv"
        r = self.run_sh(repo / "tools/re/analyze.sh", "all", str(self.proj), SRC, DST, str(out),
                        FLAT3D_HEADLESS=str(self.standin))
        self.assertEqual(r.returncode, 0, r.stderr)
        calls = self.log.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(calls), 3)
        standin = str(self.standin)
        self.assertEqual(standin + " " + calls[0], self.analyze_line(SRC, SRC_SLUG).replace("tools/ghidra/headless.sh", standin))
        self.assertEqual(standin + " " + calls[1], self.analyze_line(DST, DST_SLUG).replace("tools/ghidra/headless.sh", standin))
        self.assertRegex("tools/ghidra/headless.sh " + calls[2], self.vt_pattern(out))
        self.assertTrue((self.proj / "logs").is_dir())
        self.assertEqual(out.read_text(encoding="utf-8"), VT_CSV)
        d = tomllib.loads((repo / "games/spiderman2/map.toml").read_text(encoding="utf-8"))
        found = {i["rem_rva"]: i for i in d["item"] if i["status"] == "found"}
        self.assertEqual(sorted(found), ["0x1646e10", "0x164703f"])
        self.assertEqual(found["0x1646e10"]["sm2"], "0x2c76310")
        self.assertTrue(found["0x1646e10"]["verified_by"].startswith("vt-matches.csv sha256:"))
        self.assertIn("| camera submit call site | site | `0x164703f` | `e86c2a2500` | `0x2c00230` | - | vt 1.00 |",
                      (repo / "docs/spiderman2/address-map.md").read_text(encoding="utf-8"))
        for rel in ("games/spiderman2/map.toml", "docs/spiderman2/address-map.md"):
            self.assertNotEqual((repo / rel).read_bytes(), (ROOT / rel).read_bytes(), rel)

    def test_busy_and_low_memory_stop_the_chain_with_the_same_code(self):
        for rc in (75, 137):
            with self.subTest(rc=rc):
                repo = self.copy_layout()
                if self.log.exists():
                    self.log.unlink()
                out = self.dir / "out.csv"
                r = self.run_sh(repo / "tools/re/analyze.sh", "all", str(self.proj), SRC, DST, str(out),
                                FLAT3D_HEADLESS=str(self.standin), STANDIN_RC=str(rc))
                self.assertEqual(r.returncode, rc, r.stderr)
                self.assertEqual(len(self.log.read_text(encoding="utf-8").splitlines()), 1)
                self.assertFalse(out.exists())
                for rel in ("games/spiderman2/map.toml", "docs/spiderman2/address-map.md"):
                    self.assertEqual((repo / rel).read_bytes(), (ROOT / rel).read_bytes(), rel)

    def test_unknown_exe_stops_with_the_exe_id_code(self):
        r = self.run_sh(SCRIPT, "analyze", str(self.proj), "/tmp/Other.exe", "--dry-run")
        self.assertEqual(r.returncode, 3)
        self.assertEqual(r.stdout, "")

    def test_committed_executable(self):
        r = subprocess.run(["git", "-C", str(ROOT), "ls-files", "-s", "tools/re/analyze.sh"], capture_output=True, text=True)
        if r.returncode != 0 or not r.stdout:
            self.skipTest("tools/re/analyze.sh is not in the git index here")
        self.assertTrue(r.stdout.startswith("100755 "), r.stdout)

    @unittest.skipUnless(shutil.which("shellcheck"), "shellcheck is not installed")
    def test_shellcheck_clean(self):
        r = subprocess.run(["shellcheck", str(SCRIPT)], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    @unittest.skipUnless((re_home() / "jdk/current/bin/javac").exists() and (re_home() / "ghidra/Ghidra").is_dir(),
                         "no Ghidra and JDK under FLAT3D_RE_HOME (or ~/.local/share/flat-to-3d)")
    def test_ghidra_scripts_compile(self):
        with open(ROOT / "sessions/q12.toml", "rb") as f:
            steps = tomllib.load(f)["step"]
        argv = list(next(s for s in steps if s["id"] == "compile-scripts")["run"])
        argv[-2] = argv[-2].replace("{re_home}", str(re_home()))
        argv[-1] = argv[-1].replace("{repo}", str(ROOT))
        with tempfile.TemporaryDirectory() as t:
            r = subprocess.run(argv, capture_output=True, text=True, env={**os.environ, "TMPDIR": t})
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)


if __name__ == "__main__":
    unittest.main()
