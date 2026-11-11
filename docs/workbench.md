# Workbench

Unset `FLAT3D_RE_HOME` means `$HOME/.local/share/flat-to-3d`; unset `FLAT3D_BUILD_BOX` means `craft-build`. These are the defaults used by the checks below.

| Tool | Where | Check command |
|---|---|---|
| Build box | `$FLAT3D_BUILD_BOX` | `distrobox enter "$FLAT3D_BUILD_BOX" -- clang-cl --version` |
| xwin | `$FLAT3D_RE_HOME/xwin`; `cmake/clang-cl-xwin.cmake` reads it through `XWIN_DIR` (default `$HOME/.local/share/flat-to-3d/xwin`) | `ls "$FLAT3D_RE_HOME/xwin"` |
| Ghidra 12.1.4 | `$FLAT3D_RE_HOME/ghidra` | `test -x "$FLAT3D_RE_HOME/ghidra/support/analyzeHeadless"` |
| JDK 21 | `$FLAT3D_RE_HOME/jdk/current` | `"$FLAT3D_RE_HOME/jdk/current/bin/java" -version` |

Build the Windows DLLs inside the distrobox build container: `distrobox enter "$FLAT3D_BUILD_BOX" -- tools/build-spidy.sh` (output: `~/.cache/flat-to-3d/build-spidy`).

Ghidra analysis uses `tools/ghidra/headless.sh "$FLAT3D_RE_HOME/ghidra-projects" spiderman/sm2 -import "$FLAT3D_RE_HOME/exe/sm2-2.810.0.0/Spider-Man2.exe" -max-cpu 16 -log "$FLAT3D_RE_HOME/logs/sm2-analyze.log"`. Results go in `docs/spiderman2/address-map.md`.

Heavy jobs use `FLAT3D_HEAVY_LOCK`, a 2 GiB free-memory watch, and one job at a time. Exit 75 (another heavy job or the lock) and exit 137 (stopped by the memory watch) both mean retry later.
