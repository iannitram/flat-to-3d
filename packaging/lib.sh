# shellcheck shell=bash
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
PINS=${PINS:-$ROOT/packaging/pins}

PAYLOAD_BIN=(flat3d-inject flat3d-run flat3d-viewer)
PAYLOAD_TOOLS=(adb gnirehtet gnirehtet.apk)

pin() {
  local v
  v=$(grep -m1 "^$1: " "$PINS" | cut -d' ' -f2-) || v=
  if [ -z "$v" ] || [ "$v" = TBD-at-pin ]; then
    printf 'pin %s missing or TBD-at-pin: fix: packaging/pin.sh %s\n' "$1" "${1%.*}" >&2
    exit 2
  fi
  printf '%s\n' "$v"
}

run() {
  printf '+'
  printf ' %s' "$@"
  printf '\n'
  if [ "${DRY:-0}" = 1 ]; then
    return 0
  fi
  "$@" || {
    printf 'step failed: %s\n' "$*" >&2
    exit 1
  }
}

check_sha256() {
  local expected=$1 file=$2 key=$3
  printf '+ sha256sum -c %s\n' "$file"
  if [ "${DRY:-0}" = 1 ]; then
    return 0
  fi
  if ! printf '%s  %s\n' "$expected" "$file" | sha256sum -c -; then
    printf 'sha256 mismatch: fix: packaging/pin.sh %s --check\n' "$key" >&2
    exit 4
  fi
}

final_check() {
  printf '+'
  printf ' %s' "$@"
  printf '\n'
  if [ "${DRY:-0}" = 1 ]; then
    return 0
  fi
  "$@" || {
    printf 'step failed: %s\n' "$*" >&2
    exit 1
  }
}
