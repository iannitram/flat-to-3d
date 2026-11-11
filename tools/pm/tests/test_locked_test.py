import hashlib
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / ".." / "locked-test.sh"
SCRIPT = SCRIPT.resolve()
BASH = os.environ.get("FLAT3D_BASH") or (r"C:\Program Files\Git\bin\bash.exe" if os.name == "nt" and Path(r"C:\Program Files\Git\bin\bash.exe").exists() else "bash")


def bash_path(path):
    value = str(path).replace("\\", "/")
    if os.name == "nt" and len(value) > 2 and value[1:3] == ":/":
        value = "/" + value[0].lower() + value[2:]
    return value


class LockedTestTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.scratch = self.root / "scratch"
        (self.scratch / "issues").mkdir(parents=True)
        self.dest = self.root / "tests" / "locked.rs"
        self.body = "// locked body\n#[test]\nfn frozen() {}\n"
        self.snapshot = self.scratch / "issues" / "123.md"
        self.snapshot.write_text("Issue\n\n## Tests (locked)\n```rust\n" + self.body + "```\n", encoding="utf-8", newline="\n")
        self.sha = hashlib.sha256(self.body.encode()).hexdigest()

    def tearDown(self):
        self.tmp.cleanup()

    def run_locked(self, sha=None, *, env=None, dest=None):
        merged = os.environ.copy()
        merged["SCRATCH"] = bash_path(self.scratch)
        if env:
            merged.update(env)
        return subprocess.run([BASH, str(SCRIPT), "123", bash_path(dest or self.dest), sha or self.sha],
                              cwd=SCRIPT.parents[1], text=True, capture_output=True, env=merged)

    def test_ok_writes_exact_locked_body(self):
        result = self.run_locked()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.dest.read_text(), self.body)

    def test_wrong_sha_does_not_write_destination(self):
        result = self.run_locked("0" * 64)
        self.assertEqual(result.returncode, 4)
        self.assertIn("locked-test: sha-mismatch fix:", result.stderr)
        self.assertFalse(self.dest.exists())

    def test_cr_is_rejected_before_hash_comparison(self):
        self.snapshot.write_bytes(self.snapshot.read_bytes().replace(b"body\n", b"body\r\n"))
        result = self.run_locked("0" * 64)
        self.assertEqual(result.returncode, 4)
        self.assertIn("locked-test: cr fix:", result.stderr)
        self.assertFalse(self.dest.exists())

    def test_missing_snapshot_uses_gh_read(self):
        self.snapshot.unlink()
        bin_dir = self.root / "bin"
        bin_dir.mkdir()
        gh = bin_dir / "gh"
        gh.write_text("#!/bin/sh\nprintf '%s\\n' '## Tests (locked)' '```rust' '// locked body' '#[test]' 'fn frozen() {}' '```'\n", encoding="utf-8", newline="\n")
        gh.chmod(0o755)
        env = {"PATH": str(bin_dir) + os.pathsep + os.environ.get("PATH", "")}
        result = self.run_locked(env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.dest.read_text(), self.body)

    def test_failed_gh_is_a_missing_snapshot(self):
        self.snapshot.unlink()
        bin_dir = self.root / "bin"
        bin_dir.mkdir()
        gh = bin_dir / "gh"
        gh.write_text("#!/bin/sh\necho unavailable >&2\nexit 1\n", encoding="utf-8")
        gh.chmod(0o755)
        env = {"PATH": str(bin_dir) + os.pathsep + os.environ.get("PATH", "")}
        result = self.run_locked(env=env)
        self.assertEqual(result.returncode, 3)
        self.assertIn("locked-test: no-snapshot fix:", result.stderr)

    def test_uppercase_sha_matches(self):
        result = self.run_locked(self.sha.upper())
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_destination_gets_normal_mode(self):
        self.assertEqual(self.run_locked().returncode, 0)
        self.assertEqual(self.dest.stat().st_mode & 0o777, 0o644)

    @unittest.skipIf(os.name == "nt" or (hasattr(os, "geteuid") and os.geteuid() == 0), "needs permission checks")
    def test_unwritable_destination_is_not_a_sha_mismatch(self):
        locked = self.root / "ro"
        locked.mkdir()
        locked.chmod(0o555)
        try:
            result = self.run_locked(dest=locked / "sub" / "x.rs")
        finally:
            locked.chmod(0o755)
        self.assertEqual(result.returncode, 4)
        self.assertIn("locked-test: dest-unwritable fix:", result.stderr)

    def test_scratch_unset_is_usage_error(self):
        result = self.run_locked(env={"SCRATCH": ""})
        self.assertEqual(result.returncode, 2)
        self.assertIn("locked-test: usage fix:", result.stderr)


if __name__ == "__main__":
    unittest.main()
