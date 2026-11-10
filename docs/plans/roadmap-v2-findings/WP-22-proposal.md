# WP-22 recheck: proposed edits (plan mode was on, so WP-22.md is unedited)

File: roadmap-v2.md (WP-22 section)

## E1. pin() helper (Interfaces, first bullet)
Today, a missing key makes `grep` exit 1. Under `set -euo pipefail` the script then dies with exit 1 and prints nothing, not exit 2. The `A && B || C` form also risks shellcheck SC2015. Replace with:
```
pin(){ local v; v=$(grep -m1 "^$1: " "$PINS" | cut -d' ' -f2-) || v=; if [ -z "$v" ] || [ "$v" = TBD-at-pin ]; then echo "pin $1 missing or TBD-at-pin: run packaging/pin.sh" >&2; exit 2; fi; printf '%s\n' "$v"; }
```
Also add: `ROOT=$(cd "$(dirname "$0")/.." && pwd)`. Every step goes through `run(){ printf '+'; printf ' %s' "$@"; printf '\n'; [ "${DRY:-0}" = 1 ] || "$@"; }`, which prints raw argv, so `$ORIGIN` appears literally.
The pin keys used are `android-tools.{version,url,sha256}`, `gnirehtet.{version,url,sha256}`, `appimagetool.{url,sha256}` and `appimage-runtime.{url,sha256}`.

## E2. build-tools.sh order
1. Parse args.
2. Read every pin (exit 2).
3. If `--dry-run`, print each step with placeholder paths (`$CACHE/dl.XXXXXX`, no mktemp) and exit 0. This skips the box re-exec and the preflight.
4. Otherwise, box re-exec (skipped when `FLAT3D_IN_BOX=1` or `/run/.containerenv` exists), then preflight (exit 3), then steps 1-6.

The default out is `$ROOT/dist/tools`.

## E3. build-appimage.sh
- The default bin dir is `${CARGO_TARGET_DIR:-$ROOT/target}/release`.
- Recipes are `$ROOT/recipes/*.recipe` under `shopt -s nullglob`. The repo has none today: recipes/ holds only README.md. Always `mkdir -p usr/recipes`.
- The symlink step prints as `+ ln -s bin/flat3d-inject <AppDir>/usr/flat3d-inject` (and the same for flat3d-run).
- `appimagetool` URL: `https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage`. Add it to Inline facts; only the sha is there today.
- The dry-run order matches E2 (pins first, then print and exit 0; no disk checks).

## E4. build-decky.sh
- Defaults: `--root` = `$ROOT`, `--bin-dir` = `${CARGO_TARGET_DIR:-$ROOT/target}/release`, `--tools` = `$ROOT/dist/tools`.
- Recipes come from `<root>/recipes/*.recipe` (nullglob).
- Zip with `(cd "$stage" && python3 -m zipfile -c "$out_abs" flat-to-3d)`, where `out` is made absolute first.
- Note that the zipfile CLI also writes directory entries (`name/`).
- The pnpm line names the local pnpm 9.15.9 (WP-21:72). CI uses 10.34.9, which reads lockfile v9 (bones §7).

## E5. appimage.yml (bones §2 "CI final shape" + T19 win over the current text)
- `runs-on: [self-hosted, linux]`, never `ubuntu-24.04`.
- Steps:
  1. checkout@v4.
  2. apt with `DEBIAN_FRONTEND: noninteractive` and `SUDO=$(command -v sudo || true); $SUDO apt-get update && $SUDO apt-get install -y --no-install-recommends build-essential curl ca-certificates pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev desktop-file-utils`.
  3. `dtolnay/rust-toolchain@1.95.0`.
  4. `packaging/build-appimage.sh --no-tools` with env `APPIMAGE_EXTRACT_AND_RUN: "1"`.
  5. upload-artifact@v4.
- Drop the `id: pin` step and `rust-toolchain.toml`: no WP creates that file (WP-00 pins `@1.95.0` in check.yml).
- The workflow runs only on the owner's dispatch. It is never run in the pass.

## E6. Tests
- Each test writes its own pins fixture into a temp dir from Inline facts and passes `--pins`. It never reads the real packaging/pins, whose keys WP-02 does not guarantee.
- **tools dry-run:** assert the full shas. Run with `--out <tmp>/t` and assert `<tmp>/t` does not exist. Run with the default out and assert `dist/tools/gnirehtet.apk` appears. Put a fake `curl` first on `PATH`, and assert its marker is absent. No `--no-box` is needed, because dry-run skips the box.
- **TBD refusal:** the fixture has `android-tools.sha256: TBD-at-pin` → exit 2. A fixture missing the key also gives exit 2.
- **appimage dry-run:** assert `--runtime-file`, the appimagetool URL, `ln -s bin/flat3d-inject` and `ln -s bin/flat3d-run`. Assert no `libwayland`, `libvulkan` or `libGL`.
- **workflow:** contains `dtolnay/rust-toolchain@1.95.0` and `[self-hosted, linux]`. Contains none of `@stable`, `@master` or `ubuntu-24.04`.
- **decky zip** fake tree:
  - `T/decky/{dist/index.js,package.json,plugin.json,main.py,README.md}`, `T/LICENSE`, `T/recipes/x.recipe`;
  - `B/{flat3d-inject,flat3d-run}` (0755);
  - `D/{adb,gnirehtet,gnirehtet.apk,lib/libx.so}`.
  - Compare the set of names not ending in `/` with the explicit expected set.

## E7. Depends / inline facts
- Depends: WP-01, WP-02 (wave 2 is correct). Drop "WP-00 (`rust-toolchain.toml`)".
- Inline fact: check.yml uses `@stable` at :24,:41 today. WP-00 then pins `@1.95.0`, and WP-01 owns the final file. There is no rust-toolchain.toml.
- `install.rs:34` → `crates/engine/src/install.rs:34`.
- Remove the `ubuntu-24.04` flag.

## Cross
- WP-02: packaging/pins must carry the `fetch: auto` blocks android-tools, gnirehtet, appimagetool and appimage-runtime, with the WP-22 inline-fact url/sha values.
- WP-24: the tools-build step checks the "sha256sum of the zip", but the zip is in a random mktemp dir. Use "build-tools.sh exit 0 (it verifies the pinned sha; exit 4 on mismatch)" instead.
