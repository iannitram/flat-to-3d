import hashlib
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "packaging" / "pin.sh"
BASH = os.environ.get("FLAT3D_BASH") or (r"C:\Program Files\Git\bin\bash.exe" if os.name == "nt" and Path(r"C:\Program Files\Git\bin\bash.exe").exists() else "bash")


def parse(path):
    result = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        left, value = line.split(": ", 1)
        key, field = left.split(".", 1)
        result.setdefault(key, {})[field] = value
    return result


def bash_path(path):
    value = str(path).replace("\\", "/")
    if os.name == "nt" and len(value) > 2 and value[1:3] == ":/":
        value = "/" + value[0].lower() + value[2:]
    return value


class PinTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.blob = self.root / "blob"
        self.blob.write_bytes(b"trusted fixture\n")
        self.data = self.blob.read_bytes()
        self.sha = hashlib.sha256(self.data).hexdigest()
        self.md5 = hashlib.md5(self.data).hexdigest()
        self.pins = self.root / "pins"
        self.write_pins()
        self.cache = self.root / "cache"

    def tearDown(self):
        self.tmp.cleanup()

    def write_pins(self, *, url=None, size=None, sha="TBD-at-pin", md5=None, pinned=None):
        lines = ["# fixture", "t.fetch: auto", "t.version: 1", f"t.url: {url or self.blob.as_uri()}",
                 f"t.size: {len(self.data) if size is None else size}", f"t.sha256: {sha}", "t.license: test"]
        if url == "TBD-at-pin":
            lines.append("t.page: https://example.test/page")
        if md5 is not None:
            lines.append(f"t.md5: {md5}")
        if pinned is not None:
            lines.append(f"t.pinned: {pinned}")
        lines.extend(["manual.fetch: manual", "manual.page: https://example.test/page", "manual.license: test", "crate.ashpd: 0.13.13"])
        self.pins.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")

    def run_pin(self, *args, env=None):
        merged = os.environ.copy()
        merged.update({"XDG_CACHE_HOME": bash_path(self.cache), "PIN_ALLOW_FILE": "1"})
        if env:
            merged.update(env)
        converted = [bash_path(arg) if isinstance(arg, Path) else str(arg) for arg in args]
        return subprocess.run([BASH, str(SCRIPT), *converted, "--pins", bash_path(self.pins)],
                              cwd=ROOT, text=True, capture_output=True, env=merged)

    def assert_token(self, result, code, token):
        self.assertEqual(result.returncode, code, result.stdout + result.stderr)
        self.assertIn(f"pin: {token} ", result.stderr)

    def test_real_pin_schema_and_required_keys(self):
        table = parse(ROOT / "packaging" / "pins")
        required = {"uevr": "auto", "android-tools": "auto", "gnirehtet": "auto", "appimagetool": "auto",
                    "appimage-runtime": "auto", "geo11": "auto", "lana-fix": "auto", "control-fix": "auto",
                    "ac7-profile": "auto", "wivrn-apk": "auto", "ac7-prereqs": "manual"}
        for key, fetch in required.items():
            self.assertEqual(table[key]["fetch"], fetch)
        for key, row in table.items():
            if key in {"crate", "npm"}:
                continue
            self.assertNotIn("#", " ".join(row.values()))
            if row.get("fetch") == "auto":
                for field in ("version", "url", "size", "sha256", "license"):
                    self.assertIn(field, row)
                self.assertRegex(row["sha256"], r"^(?:[a-fA-F0-9]{64}|TBD-at-pin)$")
                if row["url"] == "TBD-at-pin":
                    self.assertEqual(row["sha256"], "TBD-at-pin")
                    self.assertIn("page", row)
                if row["size"] == "TBD-at-pin":
                    self.assertEqual(row["sha256"], "TBD-at-pin")
            elif row.get("fetch") == "manual":
                self.assertIn("page", row)
                self.assertIn("license", row)

    def test_tbd_sha_filled_without_other_line_changes(self):
        before = self.pins.read_text()
        result = self.run_pin("t")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("pin: ok", result.stdout)
        self.assertIn(f"t.sha256: {self.sha}", self.pins.read_text())
        old = [line for line in before.splitlines() if not line.startswith("t.sha256:") and not line.startswith("t.pinned:")]
        new = [line for line in self.pins.read_text().splitlines() if not line.startswith("t.sha256:") and not line.startswith("t.pinned:")]
        self.assertEqual(new, old)
        self.assertRegex(self.pins.read_text(), r"t\.pinned: \d{4}-\d{2}-\d{2} tofu size$")

    def test_tbd_size_filled_next_to_sha(self):
        self.write_pins(size="TBD-at-pin")
        result = self.run_pin("t")
        self.assertEqual(result.returncode, 0, result.stderr)
        lines = self.pins.read_text().splitlines()
        sha_line = lines.index(f"t.sha256: {self.sha}")
        size_line = lines.index(f"t.size: {len(self.data)}")
        self.assertEqual(abs(sha_line - size_line), 1)

    def test_tbd_url_fails_with_named_page(self):
        self.write_pins(url="TBD-at-pin")
        result = self.run_pin("t")
        self.assert_token(result, 3, "url-tbd")
        self.assertIn("t.url from t.page", result.stderr)

    def test_size_mismatch_leaves_pins_unchanged(self):
        self.write_pins(size=len(self.data) + 1)
        before = self.pins.read_bytes()
        self.assert_token(self.run_pin("t"), 4, "size-mismatch")
        self.assertEqual(self.pins.read_bytes(), before)

    def test_md5_mismatch(self):
        self.write_pins(md5="0" * 32)
        self.assert_token(self.run_pin("t"), 4, "md5-mismatch")

    def test_already_pinned_requires_check(self):
        self.write_pins(sha=self.sha)
        self.assert_token(self.run_pin("t"), 5, "already-pinned")

    def test_check_good_and_wrong_sha(self):
        self.write_pins(sha=self.sha)
        self.assertEqual(self.run_pin("t", "--check").returncode, 0)
        self.write_pins(sha="0" * 64)
        self.assert_token(self.run_pin("t", "--check"), 4, "sha-mismatch")

    def test_manual_key(self):
        self.assert_token(self.run_pin("manual"), 3, "manual")
        self.assertIn("https://example.test/page", self.run_pin("manual").stderr)

    def test_file_scheme_is_refused_without_fixture_override(self):
        result = self.run_pin("t", env={"PIN_ALLOW_FILE": "0"})
        self.assert_token(result, 3, "scheme")

    def test_unknown_and_crate_keys(self):
        self.assert_token(self.run_pin("missing"), 2, "unknown-key")
        self.assert_token(self.run_pin("crate"), 2, "not-a-fetch-key")

    def test_size_must_be_digits_or_tbd(self):
        for bad in ("$(touch pwned)", "1+1", "12abc"):
            self.write_pins(size=bad)
            before = self.pins.read_bytes()
            self.assert_token(self.run_pin("t"), 2, "bad-pins")
            self.assertEqual(self.pins.read_bytes(), before)

    def test_file_with_all_tbd_is_usage_error(self):
        self.assert_token(self.run_pin("--all-tbd", "--file", self.blob), 2, "usage")

    def test_dry_run_does_not_write(self):
        before = self.pins.read_bytes()
        result = self.run_pin("t", "--dry-run")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"t.sha256: {self.sha}", result.stdout)
        self.assertEqual(self.pins.read_bytes(), before)

    def gh_fixture(self, digest, exit_code=0):
        bin_dir = self.root / "bin"
        bin_dir.mkdir(exist_ok=True)
        gh = bin_dir / "gh"
        gh.write_text(f"#!/bin/sh\n[ {exit_code} -eq 0 ] || exit {exit_code}\nprintf '%s\\n' '{digest}'\n", encoding="utf-8")
        gh.chmod(0o755)
        return str(bin_dir) + os.pathsep + os.environ.get("PATH", "")

    def gh_case(self, digest, code, token=None):
        self.write_pins(url="https://github.com/o/r/releases/download/v1/blob")
        result = self.run_pin("t", "--file", self.blob, env={"PATH": self.gh_fixture(digest)})
        if token:
            self.assert_token(result, code, token)
        else:
            self.assertEqual(result.returncode, code, result.stdout + result.stderr)
        return result

    def test_github_digest_match_is_recorded(self):
        result = self.gh_case("sha256:" + self.sha, 0)
        self.assertIn("tofu size+gh-digest", self.pins.read_text())

    def test_github_digest_mismatch(self):
        self.gh_case("sha256:" + "0" * 64, 4, "digest-mismatch")

    def test_github_empty_digest_is_skipped(self):
        self.gh_case("null", 0)
        self.assertNotIn("gh-digest", self.pins.read_text())

    def test_github_api_failure_is_retryable(self):
        self.write_pins(url="https://github.com/o/r/releases/download/v1/blob")
        result = self.run_pin("t", "--file", self.blob, env={"PATH": self.gh_fixture("", 1)})
        self.assert_token(result, 3, "gh-failed")


if __name__ == "__main__":
    unittest.main()
