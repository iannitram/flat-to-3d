import contextlib
import hashlib
import io
import json
import pathlib
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import exe_id

HERE = pathlib.Path(__file__).resolve().parent
SCRIPT = HERE / "exe_id.py"


def fake_pe(version):
    """A few bytes that pass for a PE with a version resource: MZ, then the key and VS_FIXEDFILEINFO."""
    major, minor, build, rev = version
    fixed = struct.pack("<4I", 0xFEEF04BD, 0x00010000, (major << 16) | minor, (build << 16) | rev)
    return b"MZ" + b"\0" * 62 + "VS_VERSION_INFO".encode("utf-16-le") + b"\0" * 6 + fixed + b"\0" * 32


class ExeIdTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, name, data):
        p = self.dir / name
        p.write_bytes(data)
        return p, hashlib.sha256(data).hexdigest()

    def test_known_sha_gives_slug_and_no_version(self):
        p, sha = self.write("Spider-Man2.exe", b"not a PE, no version resource\n")
        with patch.dict(exe_id.KNOWN_BUILDS, {sha: ("fixture-1.0", "1.0.0.0")}, clear=True):
            self.assertEqual(exe_id.identify(p), ("fixture-1.0", sha, "-"))

    def test_version_resource_is_read(self):
        p, sha = self.write("game.exe", fake_pe((2, 810, 0, 0)))
        with patch.dict(exe_id.KNOWN_BUILDS, {sha: ("fixture-2.810", "2.810.0.0")}, clear=True):
            self.assertEqual(exe_id.identify(p), ("fixture-2.810", sha, "2.810.0.0"))

    def test_empty_file_has_no_version(self):
        p, sha = self.write("empty.exe", b"")
        with patch.dict(exe_id.KNOWN_BUILDS, {sha: ("empty", "-")}, clear=True):
            self.assertEqual(exe_id.identify(p), ("empty", sha, "-"))

    def test_known_builds_match_inspect_game(self):
        self.assertEqual(exe_id.KNOWN_BUILDS, {
            "e297d4d94f1ffe4febf289745e79e7b6fa233a788e7a00f480fc77c55db81ad1": ("remastered-4.630", "4.630.0.0"),
            "c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9": ("sm2-2.810", "2.810.0.0"),
        })

    def test_unknown_sha_exits_3_with_the_fix(self):
        p, sha = self.write("other.exe", b"some other build\n")
        r = subprocess.run([sys.executable, str(SCRIPT), str(p)], capture_output=True, text=True)
        self.assertEqual(r.returncode, 3)
        self.assertEqual(r.stdout, "")
        self.assertIn(sha, r.stderr)
        self.assertIn("exe_id: unknown " + sha + "; fix: add it to KNOWN_BUILDS in tools/re/exe_id.py", r.stderr)

    def test_main_prints_one_line_and_json(self):
        p, sha = self.write("Spider-Man.exe", fake_pe((4, 630, 0, 0)))
        with patch.dict(exe_id.KNOWN_BUILDS, {sha: ("fixture-4.630", "4.630.0.0")}, clear=True):
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                self.assertEqual(exe_id.main([str(p)]), 0)
            self.assertEqual(out.getvalue(), f"fixture-4.630 {sha} 4.630.0.0\n")
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                self.assertEqual(exe_id.main([str(p), "--json"]), 0)
            self.assertEqual(json.loads(out.getvalue()), {"slug": "fixture-4.630", "sha256": sha, "version": "4.630.0.0"})


if __name__ == "__main__":
    unittest.main()
