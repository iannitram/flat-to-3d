#!/usr/bin/env bash
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"

debug=false
install_exe=false
dry_run=false
fetch_sdk=false
for arg in "$@"; do
  case "$arg" in
    --debug) debug=true ;;
    --install) install_exe=true ;;
    --dry-run) dry_run=true ;;
    --fetch-sdk) fetch_sdk=true ;;
    *)
      printf 'unknown option: %s\n' "$arg" >&2
      exit 2
      ;;
  esac
done

XWIN=${FLAT3D_XWIN_DIR:-$HOME/.local/share/flat-to-3d/xwin}
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target/win}
RUSTFLAGS="-Clinker=rust-lld -Ctarget-feature=+crt-static -Lnative=$XWIN/crt/lib/x86_64 -Lnative=$XWIN/sdk/lib/um/x86_64 -Lnative=$XWIN/sdk/lib/ucrt/x86_64"
target=x86_64-pc-windows-msvc
fix='run tools/build-win.sh --fetch-sdk, or $HOME/.local/share/flat-to-3d/tools/xwin --accept-license splat --output "$XWIN"'

if [[ "$fetch_sdk" == true ]]; then
  "$HOME/.local/share/flat-to-3d/tools/xwin" --accept-license splat --output "$XWIN"
fi

if [[ ! -d "$XWIN/crt/lib/x86_64" || ! -d "$XWIN/sdk/lib/um/x86_64" || ! -d "$XWIN/sdk/lib/ucrt/x86_64" || \
      ! -f "$XWIN/crt/lib/x86_64/libcmt.lib" || ! -f "$XWIN/crt/lib/x86_64/libvcruntime.lib" || \
      ! -f "$XWIN/sdk/lib/um/x86_64/kernel32.lib" || ! -f "$XWIN/sdk/lib/ucrt/x86_64/libucrt.lib" ]]; then
  printf 'Windows MSVC SDK libraries are missing under %s; fix: %s\n' "$XWIN" "$fix" >&2
  exit 3
fi

if ! rustup target list --installed | grep -Fxq "$target"; then
  printf 'Rust target %s is missing; fix: rustup target add %s\n' "$target" "$target" >&2
  exit 3
fi

build=(cargo build --locked)
[[ "$debug" == false ]] && build+=(--release)
build+=(-p flat3d-uevr-inject --target "$target")
source_exe="$CARGO_TARGET_DIR/$target/$([[ "$debug" == true ]] && printf debug || printf release)/flat3d-uevr-inject.exe"
dist_exe="$repo/dist/tools/flat3d-uevr-inject.exe"
install_exe_path="${XDG_DATA_HOME:-$HOME/.local/share}/flat3d/bin/flat3d-uevr-inject.exe"

if [[ "$dry_run" == true ]]; then
  printf 'XWIN=%s\n' "$XWIN"
  printf 'RUSTFLAGS=%s\n' "$RUSTFLAGS"
  printf 'CARGO_TARGET_DIR=%s\n' "$CARGO_TARGET_DIR"
  printf 'RUSTFLAGS=%q CARGO_TARGET_DIR=%q' "$RUSTFLAGS" "$CARGO_TARGET_DIR"
  printf ' %q' "${build[@]}"
  printf '\n'
  printf 'mkdir -p %q && cp %q %q\n' "$repo/dist/tools" "$source_exe" "$dist_exe"
  if [[ "$install_exe" == true ]]; then
    printf 'install -Dm755 %q %q\n' "$dist_exe" "$install_exe_path"
  fi
  exit 0
fi

RUSTFLAGS="$RUSTFLAGS" CARGO_TARGET_DIR="$CARGO_TARGET_DIR" "${build[@]}"
mkdir -p "$repo/dist/tools"
cp "$source_exe" "$dist_exe"
magic=$(od -An -N2 -tc "$dist_exe" | tr -d ' \n')
if [[ "$magic" != MZ ]]; then
  printf 'built file does not have an MZ header: %s\n' "$dist_exe" >&2
  exit 1
fi
sha256sum "$dist_exe"

if [[ "$install_exe" == true ]]; then
  install -Dm755 "$dist_exe" "$install_exe_path"
fi
