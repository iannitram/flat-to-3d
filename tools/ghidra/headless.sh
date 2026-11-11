#!/bin/bash
# headless.sh <analyzeHeadless args...>: run Ghidra headless on the Z13 as a heavy job.
# One heavy job at a time: refuses while `$FLAT3D_HEAVY_LOCK` exists or another headless Ghidra runs,
# and stops Ghidra if MemAvailable drops under 2 GiB (exit 137).
# Install: `$FLAT3D_RE_HOME/ghidra`, Temurin 21 under `$FLAT3D_RE_HOME/jdk/current`
# (see docs/workbench.md#ghidra; results in docs/spiderman2/address-map.md).
F=${FLAT3D_RE_HOME:-${FLAT3D_HOME:-$HOME/.local/share/flat-to-3d}}
GHIDRA=$F/ghidra
export JAVA_HOME=$F/jdk/current PATH=$F/jdk/current/bin:$PATH
export GHIDRA_HEADLESS_MAXMEM=${GHIDRA_HEADLESS_MAXMEM:-24G}
LOCK=${FLAT3D_HEAVY_LOCK:-}; [ -n "$LOCK" ] && [ -e "$LOCK" ] && { echo "headless: $LOCK exists (another heavy job); retry when it is gone" >&2; exit 75; }
pgrep -f ghidra.app.util.headless.AnalyzeHeadless >/dev/null && { echo "headless: another headless Ghidra is running" >&2; exit 75; }
"$GHIDRA/support/analyzeHeadless" "$@" & P=$!
while kill -0 "$P" 2>/dev/null; do
  a=$(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo)
  if [ "$a" -lt 2048 ]; then
    echo "headless: MemAvailable ${a} MiB < 2048, stopping Ghidra $(date +%T)" >&2
    pkill -TERM -f ghidra.app.util.headless.AnalyzeHeadless; sleep 5
    pkill -KILL -f ghidra.app.util.headless.AnalyzeHeadless; exit 137
  fi
  sleep 2
done
wait "$P"
