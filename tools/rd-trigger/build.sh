#!/usr/bin/env bash
# Build rd-trigger against RenderDoc 1.46's replay API headers, linking the host's librenderdoc.so.
# Run on the host; the compile runs in the craft-build distrobox. Output: tools/rd-trigger/rd-trigger
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
src="${RD_SRC:-${XDG_CACHE_HOME:-$HOME/.cache}/renderdoc-v1.46-src}"
if [ ! -d "$src/renderdoc/api/replay" ]; then
  git clone --depth 1 --branch v1.46 --filter=blob:none --sparse https://github.com/baldurk/renderdoc.git "$src"
  git -C "$src" sparse-checkout set renderdoc/api/replay renderdoc/api/app
fi
distrobox enter craft-build -- bash -lc "cd '$here' && g++ -std=c++17 -O2 -DRENDERDOC_PLATFORM_LINUX \
  -I '$src/renderdoc/api/replay' -I '$src/renderdoc/api/app' rd-trigger.cpp -o rd-trigger \
  -L/run/host/usr/lib -l:librenderdoc.so -Wl,--allow-shlib-undefined -Wl,-rpath,/usr/lib -Wl,--export-dynamic"
echo "built $here/rd-trigger"
