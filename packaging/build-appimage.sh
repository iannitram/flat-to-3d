#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh disable=SC1091
. "$(dirname "$0")/lib.sh"

usage() {
  printf '%s\n' 'usage: packaging/build-appimage.sh [--dry-run] [--no-tools] [--tools DIR] [--bin-dir DIR] [--skip-build] [--pins FILE] [--out FILE]' >&2
}
usage_error() { usage; exit 2; }

DRY=0
NO_TOOLS=0
SKIP_BUILD=0
TOOLS="$ROOT/dist/tools"
BIN_DIR="${CARGO_TARGET_DIR:-$ROOT/target}/release"
OUT="$ROOT/dist/Flat_to_3D-x86_64.AppImage"
while (($#)); do
  case $1 in
    --dry-run) DRY=1; shift ;;
    --no-tools) NO_TOOLS=1; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --tools) (($# >= 2)) || usage_error; TOOLS=$2; shift 2 ;;
    --bin-dir) (($# >= 2)) || usage_error; BIN_DIR=$2; shift 2 ;;
    --pins) (($# >= 2)) || usage_error; export PINS=$2; shift 2 ;;
    --out) (($# >= 2)) || usage_error; OUT=$2; shift 2 ;;
    *) usage_error ;;
  esac
done

APPIMAGETOOL_URL=$(pin appimagetool.url)
APPIMAGETOOL_SHA=$(pin appimagetool.sha256)
RUNTIME_URL=$(pin appimage-runtime.url)
RUNTIME_SHA=$(pin appimage-runtime.sha256)

if [ "$DRY" = 1 ]; then
  DRY=1
  if [ "$SKIP_BUILD" = 0 ]; then
    printf '+ cargo build --release --locked -p flat3d-inject -p flat3d-app\n'
    printf '+ cargo build --release --locked -p flat3d-viewer --features xr,capture\n'
  fi
  printf "+ mkdir -p \$tmp/AppDir/usr/bin \$tmp/AppDir/usr/tools \$tmp/AppDir/usr/recipes\n"
  printf "+ cp %s/flat3d-app \$tmp/AppDir/usr/bin/flat3d-app\n" "$BIN_DIR"
  for name in "${PAYLOAD_BIN[@]}"; do
    printf "+ cp %s/%s \$tmp/AppDir/usr/bin/%s\n" "$BIN_DIR" "$name" "$name"
    printf "+ ln -s bin/%s \$tmp/AppDir/usr/%s\n" "$name" "$name"
  done
  if [ "$NO_TOOLS" = 0 ]; then
    for name in "${PAYLOAD_TOOLS[@]}"; do
      printf "+ cp %s/%s \$tmp/AppDir/usr/tools/%s\n" "$TOOLS" "$name" "$name"
    done
    printf "+ cp %s/lib/* \$tmp/AppDir/usr/tools/lib/\n" "$TOOLS"
  fi
  printf "+ shopt -s nullglob; cp %s/recipes/*.recipe \$tmp/AppDir/usr/recipes/\n" "$ROOT"
  printf "+ write \$tmp/AppDir/AppRun flat3d.desktop flat3d.svg\n"
  printf "+ ln -s flat3d.svg \$tmp/AppDir/.DirIcon\n"
  printf "+ curl -fL --proto =https --tlsv1.2 --retry 3 -o \$CACHE/dl.XXXXXX/%s %s\n" "${APPIMAGETOOL_URL##*/}" "$APPIMAGETOOL_URL"
  printf "+ sha256sum -c \$CACHE/dl.XXXXXX/%s (sha256 %s)\n" "${APPIMAGETOOL_URL##*/}" "$APPIMAGETOOL_SHA"
  printf "+ chmod +x \$CACHE/dl.XXXXXX/%s\n" "${APPIMAGETOOL_URL##*/}"
  printf "+ curl -fL --proto =https --tlsv1.2 --retry 3 -o \$CACHE/dl.XXXXXX/%s %s\n" "${RUNTIME_URL##*/}" "$RUNTIME_URL"
  printf "+ sha256sum -c \$CACHE/dl.XXXXXX/%s (sha256 %s)\n" "${RUNTIME_URL##*/}" "$RUNTIME_SHA"
  printf "+ APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 %s --runtime-file \$CACHE/dl.XXXXXX/%s \$tmp/AppDir %s\n" "${APPIMAGETOOL_URL##*/}" "${RUNTIME_URL##*/}" "$OUT"
  printf "+ test no *.so* outside \$tmp/AppDir/usr/tools/lib\n"
  exit 0
fi

if [ "$SKIP_BUILD" = 0 ]; then
  run cargo build --release --locked -p flat3d-inject -p flat3d-app
  run cargo build --release --locked -p flat3d-viewer --features xr,capture
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/flat3d-appimage.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
appdir="$tmp/AppDir"
case $OUT in /*) out_abs=$OUT ;; *) out_abs="$PWD/$OUT" ;; esac
run mkdir -p "$appdir/usr/bin" "$appdir/usr/recipes"
run cp "$BIN_DIR/flat3d-app" "$appdir/usr/bin/flat3d-app"
for name in "${PAYLOAD_BIN[@]}"; do
  run cp "$BIN_DIR/$name" "$appdir/usr/bin/$name"
  run ln -s "bin/$name" "$appdir/usr/$name"
done
if [ "$NO_TOOLS" = 0 ]; then
  run mkdir -p "$appdir/usr/tools/lib"
  for name in "${PAYLOAD_TOOLS[@]}"; do
    run cp "$TOOLS/$name" "$appdir/usr/tools/$name"
  done
  if [ -d "$TOOLS/lib" ]; then
    run cp -a "$TOOLS/lib/." "$appdir/usr/tools/lib/"
  fi
fi
shopt -s nullglob
recipes=("$ROOT"/recipes/*.recipe)
if ((${#recipes[@]})); then run cp "${recipes[@]}" "$appdir/usr/recipes/"; fi
run chmod 755 "$appdir/usr/bin/flat3d-app" "$appdir/usr/bin/"flat3d-*
cat > "$appdir/AppRun" <<'EOF'
#!/bin/sh
HERE=$(dirname "$(readlink -f "$0")"); APPDIR=${APPDIR:-$HERE}
export FLAT3D_SRC_DIR="$APPDIR/usr"; export PATH="$APPDIR/usr/bin:$PATH"
exec "$APPDIR/usr/bin/flat3d-app" "$@"
EOF
cat > "$appdir/flat3d.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Flat to 3D
Comment=Play flat PC games in 3D on a Quest
Exec=flat3d-app
Icon=flat3d
Terminal=false
Categories=Game;Utility;
EOF
cat > "$appdir/flat3d.svg" <<'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
  <rect x="8" y="26" width="112" height="76" rx="20" fill="#263b63"/>
  <path d="M25 49h30v30H25zm48 0h30v30H73z" fill="#91d7ff"/>
  <path d="M57 64h14" stroke="#fff" stroke-width="6"/>
</svg>
EOF
run chmod 755 "$appdir/AppRun"
run ln -s flat3d.svg "$appdir/.DirIcon"

cache_base=${XDG_CACHE_HOME:-${HOME:-$ROOT/.cache}}/flat3d
mkdir -p "$cache_base"
appimage_dl=$(mktemp -d "$cache_base/dl.XXXXXX")
runtime_dl=$(mktemp -d "$cache_base/dl.XXXXXX")
appimagetool="$appimage_dl/${APPIMAGETOOL_URL##*/}"
runtime="$runtime_dl/${RUNTIME_URL##*/}"
run curl -fL --proto '=https' --tlsv1.2 --retry 3 -o "$appimagetool" "$APPIMAGETOOL_URL"
check_sha256 "$APPIMAGETOOL_SHA" "$appimagetool" appimagetool.sha256
run chmod +x "$appimagetool"
run curl -fL --proto '=https' --tlsv1.2 --retry 3 -o "$runtime" "$RUNTIME_URL"
check_sha256 "$RUNTIME_SHA" "$runtime" appimage-runtime.sha256
bundled_lib=$(find "$appdir" -type f -name '*.so*' ! -path "$appdir/usr/tools/lib/*" -print -quit)
final_check test -z "$bundled_lib"
run mkdir -p "$(dirname "$out_abs")"
run env APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$appimagetool" --runtime-file "$runtime" "$appdir" "$out_abs"
