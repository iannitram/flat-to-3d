#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=lib.sh disable=SC1091
. "$(dirname "$0")/lib.sh"

usage() {
  printf '%s\n' 'usage: packaging/build-tools.sh [--dry-run] [--no-box] [--pins FILE] [--out DIR]' >&2
}
usage_error() { usage; exit 2; }

ORIGINAL_ARGS=("$@")
DRY=0
NO_BOX=0
OUT="$ROOT/dist/tools"
OUT_DISPLAY=dist/tools
while (($#)); do
  case $1 in
    --dry-run) DRY=1; shift ;;
    --no-box) NO_BOX=1; shift ;;
    --pins) (($# >= 2)) || usage_error; export PINS=$2; shift 2 ;;
    --out) (($# >= 2)) || usage_error; OUT=$2; OUT_DISPLAY=$2; shift 2 ;;
    *) usage_error ;;
  esac
done

ANDROID_VERSION=$(pin android-tools.version)
ANDROID_URL=$(pin android-tools.url)
ANDROID_SHA=$(pin android-tools.sha256)
GNIREHTET_VERSION=$(pin gnirehtet.version)
GNIREHTET_ARCHIVE="gnirehtet-rust-linux64-v${GNIREHTET_VERSION}.zip"
GNIREHTET_URL=$(pin gnirehtet.url)
GNIREHTET_SHA=$(pin gnirehtet.sha256)

if [ "$DRY" = 1 ]; then
  DRY=1
  printf '+ mkdir -p %s\n' "$OUT_DISPLAY"
  printf "+ curl -fL --proto =https --tlsv1.2 --retry 3 -o \$CACHE/dl.XXXXXX/%s %s\n" "${ANDROID_URL##*/}" "$ANDROID_URL"
  printf "+ sha256sum -c \$CACHE/dl.XXXXXX/%s (sha256 %s)\n" "${ANDROID_URL##*/}" "$ANDROID_SHA"
  printf "+ curl -fL --proto =https --tlsv1.2 --retry 3 -o \$CACHE/dl.XXXXXX/%s %s\n" "${GNIREHTET_URL##*/}" "$GNIREHTET_URL"
  printf "+ sha256sum -c \$CACHE/dl.XXXXXX/%s (sha256 %s)\n" "${GNIREHTET_URL##*/}" "$GNIREHTET_SHA"
  printf "+ tar -xJf \$CACHE/dl.XXXXXX/%s\n" "${ANDROID_URL##*/}"
  printf '+ cmake -GNinja -S android-tools-%s -B build -DCMAKE_BUILD_TYPE=Release\n' "$ANDROID_VERSION"
  printf '+ ninja -C build adb\n'
  printf '+ find build -type f -name adb -perm -u+x\n'
  printf '+ cp adb %s/adb\n' "$OUT_DISPLAY"
  printf '+ ldd adb (copy non-system libraries to %s/lib)\n' "$OUT_DISPLAY"
  printf "+ patchelf --set-rpath \$ORIGIN/lib %s/adb\n" "$OUT_DISPLAY"
  printf "+ unzip -j -o \$CACHE/dl.XXXXXX/%s */gnirehtet */gnirehtet.apk -d %s\n" "${GNIREHTET_URL##*/}" "$OUT_DISPLAY"
  printf '+ chmod 755 %s/adb %s/gnirehtet %s/lib; chmod 644 %s/gnirehtet.apk\n' "$OUT_DISPLAY" "$OUT_DISPLAY" "$OUT_DISPLAY" "$OUT_DISPLAY"
  printf '+ sha256sum adb gnirehtet gnirehtet.apk > %s/SHA256SUMS\n' "$OUT_DISPLAY"
  printf '+ %s/adb version | grep -q "Android Debug Bridge"\n' "$OUT_DISPLAY"
  printf '+ ldd %s/adb (no not found)\n' "$OUT_DISPLAY"
  exit 0
fi

if [ "$NO_BOX" = 0 ] && [ "${FLAT3D_IN_BOX:-0}" != 1 ] && [ ! -e /run/.containerenv ]; then
  exec distrobox enter "${FLAT3D_BUILD_BOX:-craft-build}" -- env FLAT3D_IN_BOX=1 bash "$0" "${ORIGINAL_ARGS[@]}"
fi

missing=()
for tool in cmake ninja pkg-config patchelf unzip sha256sum curl ldd; do
  command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if ((${#missing[@]})); then
  printf 'missing tools: %s\nfix: run inside the craft-build box (drop --no-box), or install: %s\n' \
    "${missing[*]}" "${missing[*]}" >&2
  exit 3
fi

case $OUT in /*) ;; *) OUT="$PWD/$OUT" ;; esac
cache_base=${XDG_CACHE_HOME:-${HOME:-$ROOT/.cache}}/flat3d
mkdir -p "$cache_base"
android_dl=$(mktemp -d "$cache_base/dl.XXXXXX")
gnirehtet_dl=$(mktemp -d "$cache_base/dl.XXXXXX")
android_archive="$android_dl/${ANDROID_URL##*/}"
gnirehtet_archive="$gnirehtet_dl/$GNIREHTET_ARCHIVE"
run curl -fL --proto '=https' --tlsv1.2 --retry 3 -o "$android_archive" "$ANDROID_URL"
check_sha256 "$ANDROID_SHA" "$android_archive" android-tools.sha256
run curl -fL --proto '=https' --tlsv1.2 --retry 3 -o "$gnirehtet_archive" "$GNIREHTET_URL"
check_sha256 "$GNIREHTET_SHA" "$gnirehtet_archive" gnirehtet.sha256

run mkdir -p "$OUT/lib"
(
  cd "$android_dl"
  run tar -xJf "$android_archive"
  run cmake -GNinja -S "android-tools-$ANDROID_VERSION" -B build -DCMAKE_BUILD_TYPE=Release
  run ninja -C build adb
)
adb_bin=$(find "$android_dl/build" -type f -name adb -perm -u+x -print -quit)
if [ -z "$adb_bin" ]; then
  printf 'step failed: find build -type f -name adb -perm -u+x\n' >&2
  exit 1
fi
run cp "$adb_bin" "$OUT/adb"
printf '+ ldd %s\n' "$adb_bin"
ldd_text=$(ldd "$adb_bin") || { printf 'step failed: ldd %s\n' "$adb_bin" >&2; exit 1; }
while IFS= read -r line; do
  soname='' source=''
  read -r first second third _ <<<"$line"
  if [ "$second" = '=>' ] && [[ ${third:-} = /* ]]; then
    soname=$first
    source=$third
  elif [[ ${first:-} = /* ]]; then
    source=$first
    soname=${first##*/}
  fi
  [ -n "$source" ] || continue
  case $soname in
    linux-vdso*|ld-linux*|libc.so*|libm.so*|libpthread.so*|libdl.so*|librt.so*|libudev.so*) continue ;;
  esac
  run cp -L "$source" "$OUT/lib/$soname"
  run patchelf --set-rpath "\$ORIGIN" "$OUT/lib/$soname"
done <<<"$ldd_text"
run patchelf --set-rpath "\$ORIGIN/lib" "$OUT/adb"
run unzip -j -o "$gnirehtet_archive" '*/gnirehtet' '*/gnirehtet.apk' -d "$OUT"
run chmod 755 "$OUT/adb" "$OUT/gnirehtet"
run find "$OUT/lib" -type f -exec chmod 755 {} +
run chmod 644 "$OUT/gnirehtet.apk"
run bash -c "cd \"\$1\" && sha256sum adb gnirehtet gnirehtet.apk > \"\$2\"" _ "$OUT" "$OUT/SHA256SUMS"
final_check bash -c "\"\$1\" version | grep -q \"Android Debug Bridge\"" _ "$OUT/adb"
final_check bash -c "if ldd \"\$1\" | grep -q \"not found\"; then exit 1; fi" _ "$OUT/adb"
