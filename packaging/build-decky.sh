#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh disable=SC1091
. "$(dirname "$0")/lib.sh"

usage() {
  printf '%s\n' 'usage: packaging/build-decky.sh [--dry-run] [--no-tools] [--tools DIR] [--bin-dir DIR] [--root DIR] [--skip-build] [--out FILE]' >&2
}
usage_error() { usage; exit 2; }

DRY=0
NO_TOOLS=0
SKIP_BUILD=0
TOOLS="$ROOT/dist/tools"
BIN_DIR="${CARGO_TARGET_DIR:-$ROOT/target}/release"
ROOT_DIR=$ROOT
OUT="$ROOT/dist/flat-to-3d.zip"
while (($#)); do
  case $1 in
    --dry-run) DRY=1; shift ;;
    --no-tools) NO_TOOLS=1; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --tools) (($# >= 2)) || usage_error; TOOLS=$2; shift 2 ;;
    --bin-dir) (($# >= 2)) || usage_error; BIN_DIR=$2; shift 2 ;;
    --root) (($# >= 2)) || usage_error; ROOT_DIR=$2; shift 2 ;;
    --out) (($# >= 2)) || usage_error; OUT=$2; shift 2 ;;
    *) usage_error ;;
  esac
done

if [ "$DRY" = 1 ]; then
  DRY=1
  if [ "$SKIP_BUILD" = 0 ]; then
    printf '+ pnpm -C %s/decky i --frozen-lockfile\n' "$ROOT_DIR"
    printf '+ pnpm -C %s/decky run build\n' "$ROOT_DIR"
    printf '+ cargo build --release --locked -p flat3d-inject\n'
    printf '+ cargo build --release --locked -p flat3d-viewer --features xr,capture\n'
  fi
  printf "+ mkdir -p \$stage/flat-to-3d/bin/recipes\n"
  for file in dist/index.js package.json plugin.json main.py README.md; do
    printf "+ cp %s/decky/%s \$stage/flat-to-3d/%s\n" "$ROOT_DIR" "$file" "$file"
  done
  printf "+ cp %s/LICENSE \$stage/flat-to-3d/LICENSE\n" "$ROOT_DIR"
  for name in "${PAYLOAD_BIN[@]}"; do
    printf "+ cp %s/%s \$stage/flat-to-3d/bin/%s\n" "$BIN_DIR" "$name" "$name"
  done
  if [ "$NO_TOOLS" = 0 ]; then
    for name in "${PAYLOAD_TOOLS[@]}"; do
      printf "+ cp %s/%s \$stage/flat-to-3d/bin/tools/%s\n" "$TOOLS" "$name" "$name"
    done
    printf "+ cp %s/lib/* \$stage/flat-to-3d/bin/tools/lib/\n" "$TOOLS"
  fi
  printf "+ shopt -s nullglob; cp %s/recipes/*.recipe \$stage/flat-to-3d/bin/recipes/\n" "$ROOT_DIR"
  printf "+ chmod 755 \$stage/flat-to-3d/bin/*\n"
  printf "+ (cd \$stage && python3 -m zipfile -c %s flat-to-3d)\n" "$OUT"
  exit 0
fi

if [ "$SKIP_BUILD" = 0 ]; then
  run pnpm -C "$ROOT_DIR/decky" i --frozen-lockfile
  run pnpm -C "$ROOT_DIR/decky" run build
  run cargo build --release --locked -p flat3d-inject
  run cargo build --release --locked -p flat3d-viewer --features xr,capture
fi

stage=$(mktemp -d "${TMPDIR:-/tmp}/flat3d-decky.XXXXXX")
trap 'rm -rf "$stage"' EXIT
package="$stage/flat-to-3d"
case $OUT in /*) out_abs=$OUT ;; *) out_abs="$PWD/$OUT" ;; esac
run mkdir -p "$package/dist" "$package/bin/recipes"
for file in index.js package.json plugin.json main.py README.md; do
  source=$ROOT_DIR/decky/$file
  destination="$package/$file"
  if [ "$file" = index.js ]; then
    source=$ROOT_DIR/decky/dist/index.js
    destination="$package/dist/index.js"
  fi
  run cp "$source" "$destination"
done
run cp "$ROOT_DIR/LICENSE" "$package/LICENSE"
for name in "${PAYLOAD_BIN[@]}"; do
  run cp "$BIN_DIR/$name" "$package/bin/$name"
done
run chmod 755 "$package/bin/"flat3d-*
if [ "$NO_TOOLS" = 0 ]; then
  run mkdir -p "$package/bin/tools/lib"
  for name in "${PAYLOAD_TOOLS[@]}"; do
    run cp "$TOOLS/$name" "$package/bin/tools/$name"
  done
  if [ -d "$TOOLS/lib" ]; then run cp -a "$TOOLS/lib/." "$package/bin/tools/lib/"; fi
  run chmod 755 "$package/bin/tools/adb" "$package/bin/tools/gnirehtet"
  run chmod 644 "$package/bin/tools/gnirehtet.apk"
  if [ -d "$package/bin/tools/lib" ]; then run find "$package/bin/tools/lib" -type f -exec chmod 755 {} +; fi
fi
shopt -s nullglob
recipes=("$ROOT_DIR"/recipes/*.recipe)
if ((${#recipes[@]})); then run cp "${recipes[@]}" "$package/bin/recipes/"; fi
run mkdir -p "$(dirname "$out_abs")"
run bash -c 'cd "$1" && python3 -m zipfile -c "$2" flat-to-3d' _ "$stage" "$out_abs"
