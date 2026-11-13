#!/bin/bash
# analyze.sh: the Ghidra analyse and Version Tracking (VT) driver for the Spider-Man 2 address map.
#   analyze.sh analyze <projdir> <exe> [--dry-run]
#   analyze.sh vt <projdir> <src-exe> <dst-exe> <out.csv> [--dry-run]
#   analyze.sh all <projdir> <src-exe> <dst-exe> <out.csv> [--dry-run]
# analyze imports and analyses one exe into project `spiderman` at /<slug>/<basename>;
# vt runs Auto Version Tracking from src (Remastered) to dst (Spider-Man 2) in a fresh session and
# writes every match to <out.csv>; all runs analyze src, analyze dst, vt, then tools/re/fill_map.py.
# Slugs come from tools/re/exe_id.py (or $FLAT3D_EXE_ID). Ghidra runs through
# ${FLAT3D_HEADLESS:-tools/ghidra/headless.sh}, which gets this environment unchanged and reads
# FLAT3D_RE_HOME and FLAT3D_HEAVY_LOCK itself. Its exit 75 (busy) and 137 (low memory) stop the
# chain and are this script's exit code. --dry-run prints each argv, one per line, and runs nothing.
set -u

usage() {
  echo "usage: analyze.sh analyze <projdir> <exe> [--dry-run]" >&2
  echo "       analyze.sh vt <projdir> <src-exe> <dst-exe> <out.csv> [--dry-run]" >&2
  echo "       analyze.sh all <projdir> <src-exe> <dst-exe> <out.csv> [--dry-run]" >&2
  exit 2
}

# Make a path absolute against the caller's cwd, before the cd to the repo root.
abspath() { case $1 in /*) printf '%s' "$1" ;; *) printf '%s/%s' "$PWD" "$1" ;; esac; }

DRY=0
ARGS=()
for a in "$@"; do
  if [ "$a" = "--dry-run" ]; then DRY=1; else ARGS+=("$a"); fi
done
[ "${#ARGS[@]}" -ge 1 ] || usage
MODE=${ARGS[0]}
case $MODE in
  analyze) [ "${#ARGS[@]}" -eq 3 ] || usage ;;
  vt | all) [ "${#ARGS[@]}" -eq 5 ] || usage ;;
  *) usage ;;
esac
PATHS=()
for a in "${ARGS[@]:1}"; do PATHS+=("$(abspath "$a")"); done
if [ -n "${FLAT3D_EXE_ID:-}" ]; then EXE_ID=$(abspath "$FLAT3D_EXE_ID"); else EXE_ID=; fi
# The test seam: a set FLAT3D_HEADLESS is taken against the caller's cwd; the default is repo-relative.
if [ -n "${FLAT3D_HEADLESS:-}" ]; then HEADLESS=$(abspath "$FLAT3D_HEADLESS"); else HEADLESS=tools/ghidra/headless.sh; fi

cd "$(dirname "$0")/../.." || exit 1

# run <argv...>: print it (dry run) or run it; a failure ends the script with its exit code.
run() {
  if [ "$DRY" = 1 ]; then
    echo "$*"
    return 0
  fi
  "$@"
  local rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "analyze: $1 exited $rc; stopping" >&2
    exit "$rc"
  fi
}

# slug <exe>: the first field of exe_id's `<slug> <sha> <version>`.
slug() {
  local out rc
  if [ -n "$EXE_ID" ]; then out=$("$EXE_ID" "$1"); else out=$(python3 tools/re/exe_id.py "$1"); fi
  rc=$?
  [ "$rc" -eq 0 ] || exit "$rc"
  out=${out%% *}
  [ -n "$out" ] || { echo "analyze: no slug for $1; fix: add the build to KNOWN_BUILDS in tools/re/exe_id.py" >&2; exit 3; }
  printf '%s' "$out"
}

do_analyze() { # <projdir> <exe>
  local proj=$1 exe=$2 s
  s=$(slug "$exe") || exit $?
  [ "$DRY" = 1 ] || mkdir -p "$proj/logs" || exit 1
  run "$HEADLESS" "$proj" "spiderman/$s" -import "$exe" -max-cpu 16 -log "$proj/logs/analyze-$s.log"
}

do_vt() { # <projdir> <src-exe> <dst-exe> <out.csv>
  local proj=$1 src=$2 dst=$3 csv=$4 ss ds session
  ss=$(slug "$src") || exit $?
  ds=$(slug "$dst") || exit $?
  # AutoVersionTrackingScript refuses an existing session, so each run gets a fresh name.
  session=vt-$(date +%s)
  [ "$DRY" = 1 ] || mkdir -p "$(dirname "$csv")" || exit 1
  run "$HEADLESS" "$proj" "spiderman/$ds" -process "$(basename "$dst")" \
    -preScript Flat3dVtOptions.java \
    -postScript AutoVersionTrackingScript.java / "$session" "/$ss/$(basename "$src")" \
    -postScript ExportVtMatches.java "/$session" "$csv" \
    -scriptPath tools/ghidra
}

case $MODE in
  analyze) do_analyze "${PATHS[0]}" "${PATHS[1]}" ;;
  vt) do_vt "${PATHS[@]}" ;;
  all)
    do_analyze "${PATHS[0]}" "${PATHS[1]}"
    do_analyze "${PATHS[0]}" "${PATHS[2]}"
    do_vt "${PATHS[@]}"
    run python3 tools/re/fill_map.py --map games/spiderman2/map.toml --vt "${PATHS[3]}" \
      --targets tools/xmatch/targets.py --doc docs/spiderman2/address-map.md
    ;;
esac
exit 0
