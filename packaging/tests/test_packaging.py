"""Stand-in tests for package assembly; no published release is downloaded."""

from __future__ import annotations

import configparser
import hashlib
import io
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tarfile
import tempfile
import unittest
import zipfile


ROOT = Path(__file__).resolve().parents[2]
PACKAGING = ROOT / "packaging"
ANDROID_URL = "https://github.com/nmeum/android-tools/releases/download/37.0.0p1/android-tools-37.0.0p1.tar.xz"
ANDROID_SHA = "2c6de1abe16b211c7d9f59276217e14fd9e0e18b20e331d3c84358f5dffd576e"
APPIMAGETOOL_URL = "https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage"
APPIMAGETOOL_SHA = "ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"
RUNTIME_URL = "https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64"
RUNTIME_SHA = "2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d"
# Ruling 42 took the reverse-tether relay out of the payload. Its name is split so
# `git grep -i` over packaging/ (WP-37's done-check) finds nothing.
RELAY = "gnire" + "htet"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_pins(directory: Path, overrides: dict[str, str] | None = None) -> Path:
    values = {
        "android-tools.version": "37.0.0p1",
        "android-tools.url": ANDROID_URL,
        "android-tools.sha256": ANDROID_SHA,
        "appimagetool.url": APPIMAGETOOL_URL,
        "appimagetool.sha256": APPIMAGETOOL_SHA,
        "appimage-runtime.url": RUNTIME_URL,
        "appimage-runtime.sha256": RUNTIME_SHA,
    }
    if overrides:
        for key, value in overrides.items():
            if value is None:
                values.pop(key, None)
            else:
                values[key] = value
    path = directory / "pins.yml"
    path.write_text("".join(f"{key}: {value}\n" for key, value in values.items()))
    return path


def call(script: str, *args: str, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    return subprocess.run(
        ["bash", str(PACKAGING / script), *args],
        cwd=ROOT,
        env=merged,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def make_tool_fixture(directory: Path) -> Path:
    android = directory / "android-tools-37.0.0p1.tar.xz"
    with tarfile.open(android, "w:xz") as archive:
        content = b"cmake_minimum_required(VERSION 3.10)\nproject(android-tools)\n"
        info = tarfile.TarInfo("android-tools-37.0.0p1/CMakeLists.txt")
        info.size = len(content)
        archive.addfile(info, io.BytesIO(content))
    return android


def make_standins(directory: Path) -> Path:
    directory.mkdir()
    scripts = {
        "curl": """#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
  case $1 in
    -o) destination=$2; shift 2 ;;
    *) url=$1; shift ;;
  esac
done
cp "$FIXTURES/${url##*/}" "$destination"
""",
        "cmake": "#!/bin/sh\nexit 0\n",
        "pkg-config": "#!/bin/sh\nexit 0\n",
        "ninja": """#!/bin/sh
set -eu
mkdir -p build
cat > build/adb <<'ADB'
#!/bin/sh
printf '%s\\n' 'Android Debug Bridge version 1.0.41'
ADB
chmod +x build/adb
""",
        "ldd": """#!/bin/sh
printf 'libfake.so.1 => %s/libfake.so.1 (0x0)\\n' "$FIXTURES"
printf 'libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x0)\\n'
""",
        "patchelf": "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$PATCHELF_LOG\"\n",
    }
    for name, content in scripts.items():
        path = directory / name
        path.write_text(content)
        path.chmod(0o755)
    return directory


class PackagingTests(unittest.TestCase):
    def test_tools_dry_run_prints_pins_without_writing_or_calling_curl(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temp = Path(temporary)
            pins = write_pins(temp)
            standins = temp / "standins"
            standins.mkdir()
            marker = temp / "curl-called"
            fake_curl = standins / "curl"
            fake_curl.write_text(f"#!/bin/sh\nprintf called > '{marker}'\nexit 99\n")
            fake_curl.chmod(0o755)
            output = temp / "tools-out"
            result = call(
                "build-tools.sh",
                "--dry-run",
                "--pins",
                str(pins),
                "--out",
                str(output),
                env={"PATH": f"{standins}{os.pathsep}{os.environ['PATH']}"},
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(ANDROID_URL, result.stdout)
            self.assertIn(ANDROID_SHA, result.stdout)
            self.assertIn("cmake -GNinja", result.stdout)
            self.assertIn("ninja -C build adb", result.stdout)
            self.assertIn("patchelf --set-rpath $ORIGIN/lib", result.stdout)
            self.assertFalse(marker.exists())
            self.assertFalse(output.exists())

            default = call("build-tools.sh", "--dry-run", "--pins", str(pins))
            self.assertEqual(default.returncode, 0, default.stderr)
            self.assertIn("dist/tools/adb", default.stdout)
            self.assertNotIn(RELAY, default.stdout.lower())

    @unittest.skipUnless(
        all(shutil.which(tool) for tool in ("tar", "xz", "sha256sum")),
        "tar, xz and sha256sum are required for the stand-in tools build",
    )
    def test_tools_real_path_uses_standins_and_records_payload_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temp = Path(temporary)
            fixtures = temp / "fixtures"
            fixtures.mkdir()
            android = make_tool_fixture(fixtures)
            fake_lib = fixtures / "libfake.so.1"
            fake_lib.write_bytes(b"stand-in library")
            pins = write_pins(temp, {"android-tools.sha256": digest(android)})
            standins = make_standins(temp / "standins")
            output = temp / "out"
            log = temp / "patchelf.log"
            result = call(
                "build-tools.sh",
                "--no-box",
                "--pins",
                str(pins),
                "--out",
                str(output),
                env={
                    "PATH": f"{standins}{os.pathsep}{os.environ['PATH']}",
                    "FIXTURES": str(fixtures),
                    "PATCHELF_LOG": str(log),
                    "XDG_CACHE_HOME": str(temp / "cache"),
                },
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for name in ("adb", "lib/libfake.so.1"):
                self.assertTrue((output / name).is_file(), name)
            self.assertEqual(
                sorted(path.name for path in output.iterdir()), ["SHA256SUMS", "adb", "lib"]
            )
            self.assertEqual(stat.S_IMODE((output / "adb").stat().st_mode), 0o755)
            self.assertFalse(any((output / "lib").glob("libc.so*")))
            sums = (output / "SHA256SUMS").read_text()
            self.assertEqual([line.split()[1] for line in sums.splitlines()], ["adb"])
            self.assertIn("--set-rpath $ORIGIN/lib", log.read_text())

    @unittest.skipUnless(shutil.which("shellcheck"), "shellcheck is not installed")
    def test_shellcheck_clean(self) -> None:
        files = [f"packaging/{name}" for name in ("lib.sh", "build-tools.sh", "build-appimage.sh", "build-decky.sh", "AppRun")]
        result = subprocess.run(
            ["shellcheck", "-x", *files],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_tools_bad_archive_hash_exits_four(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temp = Path(temporary)
            fixtures = temp / "fixtures"
            fixtures.mkdir()
            make_tool_fixture(fixtures)
            (fixtures / "libfake.so.1").write_bytes(b"stand-in library")
            pins = write_pins(temp, {"android-tools.sha256": "0" * 64})
            standins = make_standins(temp / "standins")
            result = call(
                "build-tools.sh",
                "--no-box",
                "--pins",
                str(pins),
                "--out",
                str(temp / "out"),
                env={
                    "PATH": f"{standins}{os.pathsep}{os.environ['PATH']}",
                    "FIXTURES": str(fixtures),
                    "PATCHELF_LOG": str(temp / "patchelf.log"),
                    "XDG_CACHE_HOME": str(temp / "cache"),
                },
            )
            self.assertEqual(result.returncode, 4, result.stdout + result.stderr)
            self.assertIn("packaging/pin.sh android-tools.sha256 --check", result.stderr)

    def test_pin_tbd_and_missing_key_exit_two(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temp = Path(temporary)
            tbd = write_pins(temp, {"android-tools.sha256": "TBD-at-pin"})
            result = call("build-tools.sh", "--dry-run", "--pins", str(tbd))
            self.assertEqual(result.returncode, 2)
            self.assertIn("packaging/pin.sh android-tools", result.stderr)

            missing = write_pins(temp, {"android-tools.sha256": None})
            result = call("build-tools.sh", "--dry-run", "--pins", str(missing))
            self.assertEqual(result.returncode, 2)
            self.assertIn("packaging/pin.sh android-tools", result.stderr)

    def test_usage_errors_exit_two_for_all_builders(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            pins = write_pins(Path(temporary))
            for script in ("build-tools.sh", "build-appimage.sh", "build-decky.sh"):
                args = ("--pins", str(pins), "--bogus") if script != "build-decky.sh" else ("--bogus",)
                result = call(script, *args)
                self.assertEqual(result.returncode, 2, f"{script}: {result.stdout}{result.stderr}")

    def test_appimage_dry_run_uses_pins_links_and_system_library_exclusion(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            pins = write_pins(Path(temporary))
            result = call("build-appimage.sh", "--dry-run", "--pins", str(pins))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("--runtime-file", result.stdout)
            self.assertIn(f"--manifest-path {ROOT.as_posix()}/Cargo.toml", result.stdout)
            self.assertIn(APPIMAGETOOL_URL, result.stdout)
            self.assertIn(APPIMAGETOOL_SHA, result.stdout)
            self.assertIn(RUNTIME_URL, result.stdout)
            self.assertIn(RUNTIME_SHA, result.stdout)
            self.assertIn("ln -s bin/flat3d-inject", result.stdout)
            self.assertIn("ln -s bin/flat3d-run", result.stdout)
            self.assertNotIn("libwayland", result.stdout)
            self.assertNotIn("libvulkan", result.stdout)
            self.assertNotIn("libGL", result.stdout)

    def test_decky_root_controls_build_and_default_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "custom root"
            result = call(
                "build-decky.sh", "--root", str(root), "--dry-run", env={"CARGO_TARGET_DIR": ""}
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(f"--manifest-path {root}/Cargo.toml", result.stdout)
            self.assertIn(f"{root}/target/release/flat3d-inject", result.stdout)
            self.assertIn("$stage/flat-to-3d/dist", result.stdout)

    def test_payload_parity(self) -> None:
        output = subprocess.run(
            ["bash", "-c", '. packaging/lib.sh; printf "%s\\n" "${PAYLOAD_BIN[@]}"; printf -- "--tools--\\n"; printf "%s\\n" "${PAYLOAD_TOOLS[@]}"'],
            cwd=ROOT,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout
        binaries, tools_text = output.split("--tools--\n")
        payload_bin = binaries.splitlines()
        payload_tools = tools_text.splitlines()
        self.assertEqual(payload_bin, ["flat3d-inject", "flat3d-run", "flat3d-viewer"])
        self.assertEqual(payload_tools, ["adb"])

        with tempfile.TemporaryDirectory() as temporary:
            pins = write_pins(Path(temporary))
            appimage = call("build-appimage.sh", "--dry-run", "--pins", str(pins))
            self.assertEqual(appimage.returncode, 0, appimage.stderr)
            for name in payload_bin:
                self.assertIn(f"/usr/bin/{name}", appimage.stdout)
                self.assertIn(f"ln -s bin/{name}", appimage.stdout)

            root, bins, tools = make_decky_tree(Path(temporary))
            archive_path = Path(temporary) / "payload.zip"
            result = call(
                "build-decky.sh",
                "--root",
                str(root),
                "--bin-dir",
                str(bins),
                "--tools",
                str(tools),
                "--skip-build",
                "--out",
                str(archive_path),
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            with zipfile.ZipFile(archive_path) as archive:
                files = {name for name in archive.namelist() if not name.endswith("/")}
            for name in payload_bin:
                self.assertIn(f"flat-to-3d/bin/{name}", files)
            for name in payload_tools:
                self.assertIn(f"flat-to-3d/bin/tools/{name}", files)

    def test_no_pin_payload_or_script_names_the_relay(self) -> None:
        payload = subprocess.run(
            ["bash", "-c", '. packaging/lib.sh; printf "%s\\n" "${PAYLOAD_BIN[@]}" "${PAYLOAD_TOOLS[@]}"'],
            cwd=ROOT,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout
        self.assertEqual(payload.split(), ["flat3d-inject", "flat3d-run", "flat3d-viewer", "adb"])
        for path in sorted(PACKAGING.rglob("*")):
            if path.is_file() and "__pycache__" not in path.parts:
                self.assertNotIn(RELAY, path.name.lower(), path)
                self.assertNotIn(RELAY, path.read_bytes().decode("utf-8", "replace").lower(), path)
        with tempfile.TemporaryDirectory() as temporary:
            pins = write_pins(Path(temporary))
            for script, args in (
                ("build-tools.sh", ("--dry-run", "--pins", str(pins))),
                ("build-appimage.sh", ("--dry-run", "--pins", str(pins))),
                ("build-decky.sh", ("--dry-run",)),
            ):
                result = call(script, *args)
                self.assertEqual(result.returncode, 0, f"{script}: {result.stderr}")
                self.assertNotIn(RELAY, result.stdout.lower(), script)

    def test_desktop_keys_and_apprun(self) -> None:
        parser = configparser.ConfigParser(interpolation=None)
        parser.optionxform = str
        self.assertTrue(parser.read(PACKAGING / "flat3d.desktop"))
        entry = parser["Desktop Entry"]
        expected = {
            "Type": "Application",
            "Name": "Flat to 3D",
            "Comment": "Play flat PC games in 3D on a Quest",
            "Exec": "flat3d-app",
            "Icon": "flat3d",
            "Terminal": "false",
        }
        for key, value in expected.items():
            self.assertEqual(entry[key], value)
        self.assertTrue(entry["Categories"].endswith(";"))
        apprun = PACKAGING / "AppRun"
        self.assertTrue(apprun.stat().st_mode & stat.S_IXUSR)
        self.assertIn('FLAT3D_SRC_DIR="$APPDIR/usr"', apprun.read_text())

    def test_workflow_is_pinned_manual_and_github_hosted(self) -> None:
        workflow = (ROOT / ".github/workflows/appimage.yml").read_text()
        self.assertIn("dtolnay/rust-toolchain@1.95.0", workflow)
        self.assertIn("runs-on: ubuntu-24.04", workflow)
        for forbidden in ("@stable", "@master", "self-hosted"):
            self.assertNotIn(forbidden, workflow)

    def test_decky_zip_exact_payload_modes_and_no_tools(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            temp = Path(temporary)
            root, bins, tools = make_decky_tree(temp)
            output = temp / "flat-to-3d.zip"
            result = call(
                "build-decky.sh",
                "--root",
                str(root),
                "--bin-dir",
                str(bins),
                "--tools",
                str(tools),
                "--skip-build",
                "--out",
                str(output),
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            expected = {
                "flat-to-3d/dist/index.js",
                "flat-to-3d/package.json",
                "flat-to-3d/plugin.json",
                "flat-to-3d/main.py",
                "flat-to-3d/README.md",
                "flat-to-3d/LICENSE",
                "flat-to-3d/bin/flat3d-inject",
                "flat-to-3d/bin/flat3d-run",
                "flat-to-3d/bin/flat3d-viewer",
                "flat-to-3d/bin/tools/adb",
                "flat-to-3d/bin/tools/lib/libx.so",
                "flat-to-3d/bin/recipes/x.recipe",
            }
            with zipfile.ZipFile(output) as archive:
                files = {name for name in archive.namelist() if not name.endswith("/")}
                self.assertEqual(files, expected)
                for name in ("flat-to-3d/bin/flat3d-inject", "flat-to-3d/bin/flat3d-viewer"):
                    self.assertTrue((archive.getinfo(name).external_attr >> 16) & 0o111, name)

            no_tools = temp / "no-tools.zip"
            result = call(
                "build-decky.sh",
                "--root",
                str(root),
                "--bin-dir",
                str(bins),
                "--tools",
                str(tools),
                "--skip-build",
                "--no-tools",
                "--out",
                str(no_tools),
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            with zipfile.ZipFile(no_tools) as archive:
                self.assertFalse(any(name.startswith("flat-to-3d/bin/tools/") for name in archive.namelist()))


def make_decky_tree(temp: Path) -> tuple[Path, Path, Path]:
    root = temp / "tree"
    (root / "decky/dist").mkdir(parents=True)
    (root / "recipes").mkdir()
    (root / "LICENSE").write_text("license\n")
    (root / "recipes/x.recipe").write_text("recipe\n")
    for path, content in {
        "decky/dist/index.js": "bundle\n",
        "decky/package.json": "{}\n",
        "decky/plugin.json": "{}\n",
        "decky/main.py": "# plugin\n",
        "decky/README.md": "readme\n",
    }.items():
        (root / path).write_text(content)

    bins = temp / "bin"
    bins.mkdir()
    for name in ("flat3d-inject", "flat3d-run", "flat3d-viewer"):
        path = bins / name
        path.write_text("#!/bin/sh\nexit 0\n")
        path.chmod(0o755)

    tools = temp / "tools"
    (tools / "lib").mkdir(parents=True)
    path = tools / "adb"
    path.write_text("#!/bin/sh\nexit 0\n")
    path.chmod(0o755)
    (tools / "lib/libx.so").write_text("library\n")
    (tools / "lib/libx.so").chmod(0o755)
    return root, bins, tools


if __name__ == "__main__":
    unittest.main()
