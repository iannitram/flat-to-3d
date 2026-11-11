#!/bin/bash
# headless.sh <analyzeHeadless args...>: run Ghidra headless on the Z13 as a heavy job.
# One heavy job at a time: refuses while qwen.lock exists or another headless Ghidra runs,
# and stops Ghidra if MemAvailable drops under 2 GiB (exit 137).
# Install: Ghidra under ~/.local/share/flat-to-3d/ghidra, Temurin 21 under .../jdk/current
# (see docs/sm2-address-map.md, "Tooling").
F=${FLAT3D_HOME:-$HOME/.local/share/flat-to-3d}
GHIDRA=$F/ghidra
export JAVA_HOME=$F/jdk/current PATH=$F/jdk/current/bin:$PATH
export GHIDRA_HEADLESS_MAXMEM=${GHIDRA_HEADLESS_MAXMEM:-24G}
LOCK=$HOME/.local/share/craft-build/qwen.lock
[ -e "$LOCK" ] && { echo "headless: $LOCK exists, the Qwen homework is running; not now" >&2; exit 75; }
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
