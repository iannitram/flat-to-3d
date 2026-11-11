#!/usr/bin/env bash
# Build upstream SpidyVR's Windows DLLs on Linux (run inside the distrobox build container (`$FLAT3D_BUILD_BOX`, default craft-build)).
# 1. Fetches Spidy's pinned dependencies (same repos and commits as its tools/bootstrap.ps1).
# 2. Configures with clang-cl + xwin and builds.
# Usage: tools/build-spidy.sh [build-dir]
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
spidy="$here/third_party/SpidyVR"
build="${1:-$HOME/.cache/flat-to-3d/build-spidy}"
tp="$spidy/third_party"

fetch() { # dest url tag commit
  local dest="$tp/$1" url="$2" tag="$3" commit="$4"
  if [ ! -d "$dest" ]; then
    git clone -q --depth 1 --branch "$tag" "$url" "$dest"
  fi
  local actual
  actual="$(git -C "$dest" rev-parse HEAD)"
  if [ "$actual" != "$commit" ]; then
    echo "$1: revision $actual differs from pinned $commit" >&2
    exit 1
  fi
  echo "$1 $tag verified: $actual"
}

mkdir -p "$tp"
fetch OpenXR-SDK https://github.com/KhronosGroup/OpenXR-SDK.git release-1.1.49 977f6675bc0057d5a54ed290cb5c71c699b1c0ab
fetch imgui https://github.com/ocornut/imgui.git v1.92.9b f1cc2ae15e53a861a874c3034aae6798fde194ab
fetch minhook https://github.com/TsudaKageyu/minhook.git v1.3.4 c3fcafdc10146beb5919319d0683e44e3c30d537

cmake -S "$spidy" -B "$build" -G Ninja \
  -DCMAKE_TOOLCHAIN_FILE="$here/cmake/clang-cl-xwin.cmake" \
  -DCMAKE_BUILD_TYPE=Release \
  -DSPIDY_BUILD_OBSERVER=ON -DSPIDY_BUILD_XR=ON -DSPIDY_BUILD_LAUNCHER=OFF \
  -DBUILD_TESTING=OFF
cmake --build "$build" -j "${JOBS:-6}"
