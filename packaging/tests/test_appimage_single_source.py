"""Locked test for the AppImage single-source issue: the committed AppRun, desktop file and
icon are what the build copies, the desktop file is validator-clean, and CI checks the payload."""

from __future__ import annotations

import configparser
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[2]
PACKAGING = ROOT / "packaging"
SCRIPT = PACKAGING / "build-appimage.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "appimage.yml"
MAIN_CATEGORIES = {
    "AudioVideo",
    "Audio",
    "Video",
    "Development",
    "Education",
    "Game",
    "Graphics",
    "Network",
    "Office",
    "Science",
    "Settings",
    "System",
    "Utility",
}
PINS = (
    "appimagetool.url: https://example.invalid/appimagetool-x86_64.AppImage\n"
    "appimagetool.sha256: " + "0" * 64 + "\n"
    "appimage-runtime.url: https://example.invalid/runtime-x86_64\n"
    "appimage-runtime.sha256: " + "1" * 64 + "\n"
)


def desktop_entry() -> configparser.SectionProxy:
    parser = configparser.ConfigParser(interpolation=None)
    parser.optionxform = str
    assert parser.read(PACKAGING / "flat3d.desktop", encoding="utf-8")
    return parser["Desktop Entry"]


class AppImageSingleSourceTests(unittest.TestCase):
    def test_script_has_no_heredoc_and_copies_the_committed_files(self) -> None:
        text = SCRIPT.read_text(encoding="utf-8")
        self.assertNotIn("<<", text)
        for name, dest in (
            ("AppRun", "AppRun"),
            ("flat3d.desktop", "flat3d.desktop"),
            ("flat3d.svg", "flat3d.svg"),
        ):
            self.assertIn(f'run cp "$ROOT/packaging/{name}" "$appdir/{dest}"', text)

    def test_dry_run_prints_the_three_copies_and_no_write_line(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            pins = Path(temporary) / "pins"
            pins.write_text(PINS, encoding="utf-8")
            result = subprocess.run(
                ["bash", str(SCRIPT), "--dry-run", "--pins", str(pins)],
                cwd=ROOT,
                env=os.environ.copy(),
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        root = ROOT.as_posix()
        for name in ("AppRun", "flat3d.desktop", "flat3d.svg"):
            self.assertIn(
                f"+ cp {root}/packaging/{name} $tmp/AppDir/{name}\n", result.stdout
            )
        self.assertNotIn("+ write ", result.stdout)
        self.assertIn("+ ln -s flat3d.svg $tmp/AppDir/.DirIcon\n", result.stdout)

    def test_committed_icon_is_the_svg_named_by_the_desktop_file(self) -> None:
        self.assertEqual(desktop_entry()["Icon"], "flat3d")
        icon = PACKAGING / "flat3d.svg"
        self.assertTrue(icon.is_file())
        root = ET.parse(icon).getroot()
        self.assertEqual(root.tag, "{http://www.w3.org/2000/svg}svg")
        self.assertEqual(root.get("viewBox"), "0 0 128 128")

    def test_desktop_file_has_exactly_one_main_category(self) -> None:
        categories = [c for c in desktop_entry()["Categories"].split(";") if c]
        self.assertEqual(categories, ["Utility"])
        self.assertEqual(len([c for c in categories if c in MAIN_CATEGORIES]), 1)
        self.assertTrue(desktop_entry()["Categories"].endswith(";"))

    @unittest.skipUnless(
        shutil.which("desktop-file-validate"), "desktop-file-validate is not installed"
    )
    def test_desktop_file_validator_prints_nothing(self) -> None:
        result = subprocess.run(
            ["desktop-file-validate", str(PACKAGING / "flat3d.desktop")],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stdout + result.stderr, "")

    def test_workflow_validates_the_desktop_file_and_checks_the_extracted_payload(
        self,
    ) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("desktop-file-validate packaging/flat3d.desktop", text)
        self.assertIn("--appimage-extract", text)
        for member in (
            "usr/bin/flat3d-app",
            "usr/bin/flat3d-inject",
            "usr/bin/flat3d-run",
            "usr/bin/flat3d-viewer",
            "usr/systems/steamos-strix-halo.system",
            "AppRun",
            "flat3d.desktop",
            "flat3d.svg",
        ):
            self.assertIn(member, text.split("--appimage-extract", 1)[1])


if __name__ == "__main__":
    unittest.main()
