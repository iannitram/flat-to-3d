<!-- split from roadmap-v2.md lines 3713-3820 at c0e6874 -->
### WP-25b: Docs scrub + contradictions (wave 2)

**Closes:** K13, A11, A16, A18, C2, C5-C8, C10-C19, C30, plus C20, C22 and C25-C29, which sit in the same files.

**Not closed here:**
- C1, C3, C4, C23 and C24 are DECISIONS.md edits. OD7 keeps that file untouched, and the wording is owner-blocked (O-1..O-4).
- C9 is the `Cargo.toml` description, frozen after wave 1. WP-01 owns it.

**Files** (the whole list; the index row says the same):
- Edited in place: `README.md`, `SPEC.md`, `PIPELINE.md`, `AGENTS.md`, `CHANGELOG.md`, `docs/ROADMAP.md`, `docs/code-map.md`, `docs/STEP0-RUNSHEET.md`, `recipes/README.md`, `tools/ghidra/headless.sh` (after WP-02), `tools/build-spidy.sh`, `tools/research/research_game.py`, `tools/xmatch/targets.py`.
- `git mv` moves:
  - `docs/PORT-SPIDERMAN2.md` → `docs/spiderman2/plan.md`
  - `docs/spidy-stereo-core-map.md` → `docs/spiderman2/spidy-core-map.md`
  - `docs/spiderman2-address-map.md` → `docs/spiderman2/address-map.md`
  - `docs/uncharted3-stereo.md` → `docs/archive/uncharted3-stereo.md`
- New: `docs/workbench.md`.
- `tools/check.sh` is not touched. WP-02 owns it, and its docs step already runs `python3 tools/docs/check.py` behind the guard `[ -f docs/workbench.md ]` (bones §2 "CI final shape"; check.yml uses the same guard). Creating `docs/workbench.md` turns that step on, so the full-repo `check.py` must pass on this WP's branch.
- Untouched: `docs/notes/DECISIONS.md` and the SPEC Musts (SPEC.md:15-51).
- No `docs/archive/README.md` or scan-leg files exist today. `docs/notes/**` stays where it is (WP-25a allow-lists it).

**Line numbers:** every `:N` below is a line of the file as at the base commit `2812f64`. `docs/code-map.md` was edited by WP-01 in wave 1 (path tokens, two new rows, the mv-map block), so its lines may have moved: find each code-map line by the quoted text, not by its number. Make the edits inside one file from the bottom up, or find by content, so an earlier edit does not shift a later line number.

**Must not touch:** `docs/notes/**`, `docs/plans/**`, `tools/check.sh`, `tools/docs/**` (the allow-list included), `Cargo.*`, `crates/**`, `.github/**`, `decky/**`. No GitHub writes. If the full-repo `check.py` reports a finding in a file outside the Files list, do not edit that file: stop and report the finding.

**Edits:**
- **README.md**, rewritten:
  - Keep:
    - what it is (:3);
    - the ground rules (one line plus a DECISIONS link);
    - build/test (`tools/check.sh`, `cargo test --workspace`);
    - a full top-level layout table (`crates/ decky/ recipes/ packaging/ sessions/ games/ tools/ docs/ cmake/ third_party/ .github/`, C27);
    - the credits (:86-106).
  - Delete :5-57 (status, second-machine and Legion Go notes) and :75-84 (moved to workbench.md).
- **SPEC.md** (non-Must lines only):
  - Move the older approval history (:7, :9, :11) to CHANGELOG. Keep :3, :5 (the current approval) and :13, because :13's "the line above" means :5.
  - :78: replace with "see `flat3d-inject doctor`" (C15).
  - :79: "…the headset chain and the wrapper route on a real game" (C25).
  - :80: link → `docs/spiderman2/plan.md`.
  - :83: "one Cargo workspace; engine `crates/engine` (package `flat3d-inject`), serde + serde_json only" (C7).
  - :88: "inline fixtures in `crates/engine/tests/*.rs`; `cargo test --workspace`; see `tools/check.sh`", plus the plan.md link (C8).
  - Leave :77 (OD3, WP-34) and :40 M20 (O-7).
- **PIPELINE.md:4** → `Spec: SPEC.md  Roadmap: docs/ROADMAP.md  Code map: docs/code-map.md` (C13). The notes at :7 become one line pointing to ROADMAP "Where it stands" (C14).
- **AGENTS.md:**
  - :5 → `docs/ROADMAP.md` (C12). Add the rule "run `tools/check.sh` (exports a temp `XDG_RUNTIME_DIR`); the docs check forbids private details".
  - :7 → "This project is Linux-only (SPEC, R04); say whether you built on the host or in the build container" (C22).
  - :9 (rule 5, `Closes #N`) → "write `Closes #N`, or `Refs #N (code half)` when the WP has a device Q row; the issue closes when that device session passes" (ruling D5). Add the PR-body rule (ruling D1): `## Verified` lists each done-check command and its exit status; `## Not verified` lists each host skip and its device Q row; never claim a run that was not done.
- **docs/ROADMAP.md:**
  - :3 → "This is the plan. The SM2 track is `spiderman2/plan.md`" (C10).
  - :23: "the Rust engine in `inject/`" → "the Rust engine in `crates/engine/`".
  - :94 → `flat3d-inject inject 2075800` (C6).
  - :204 → `docs/spiderman2/address-map.md` (A16).
  - :107: the link `spiderman2-address-map.md` → `spiderman2/address-map.md`.
  - :32 → doctor pointer (C15).
  - Move :131-143 and :176-184 to workbench.md. Scrub :133-134, :143, :167, :178 (qwen/craft-build). Keep the `iannitram/` repo coordinates.
  - Write "Step 1 done except the full-hash manifest" (C20) into ROADMAP "Where it stands", not into workbench.md (its old line :135 moves out).
  - Move :145-157 to plan.md. The merged :157 link becomes `#what-the-file-lists-already-show` (that section moves into plan.md too).
  - Drop the test counts (A18). "Where it stands" is the only status section (C28).
  - Last step: no `inject/` or `app/src` token is left (check.py).
- **docs/code-map.md** (WP-01 already rewrote the `inject/` → `crates/engine/` and `app/src` → `crates/app/src` paths and added the `<!-- mv-map -->` block):
  - Fix any `inject/` or `app/src` token WP-01 left outside the mv-map block, using the bones §2 Tree paths. The :94 heading is "API shape (crates/engine/src/api.rs)", keeping the "API shape" name. A row whose file does not exist is marked `planned`.
  - :3 → one sentence plus a ROADMAP link. :5 → `spiderman2/spidy-core-map.md` and `spiderman2/address-map.md`.
  - :21: split `Watch --> Link[..]    Watch --> Health[..]` onto two lines (A11/C19).
  - :88 and :194: "in the `craft-build` distrobox" → "in the distrobox build container (`$FLAT3D_BUILD_BOX`)".
  - :90 → "`ls crates/engine/tests`" (C30; drop the 55-name list and the counts).
  - :152: cite the `decky/main.py` shim (`io`/`command-failed`) instead of #67 (C18).
  - One error-code table under "API shape": the 8 `ERROR_CODES` (`usage not-built not-found bad-recipe refused link-not-ready command-failed io`), each with its exit (`usage` and `not-built` 2, the rest 1) and the `api::error_help` retryable flag and fix text. Say that `api.rs` owns them and the table copies it.
  - One paths table under "State, IPC and files": `~/.local/share/flat3d`, `~/.local/state/flat3d` (logs, soak, sessions), `~/.config/flat3d/settings`, `$XDG_RUNTIME_DIR`, `$FLAT3D_RE_HOME`.
  - New `## Commands` table under Parts, one row per `cli::COMMANDS` entry (WP-01c), in registry order. Column 1 is the entry's `id` exactly as typed (`list`, `setup fetch`), because check.py's usage rule reads column 1. Columns: `| Command | Group | Module | JSON (api.rs type) | WP |`, the layout WP-34 updates in place.
    - Read the rows from `pub static COMMANDS` in `crates/engine/src/cli.rs`. Command = `id` in backticks, Group = `group`, JSON = `output` (`—` when empty).
    - Module = the `crates/engine/src/cli/*.rs` file that handles the command: `cli/engine.rs` for group Engine, except `inject` → `cli/inject.rs` and `undo` → `cli/undo.rs`; `cli/link.rs` for Link; `cli/view.rs` for View; `cli/setup.rs` for Setup, except `setup fetch` → `cli/setup_fetch.rs`; `cli/doctor.rs` for Doctor.
    - WP = the `WP-NN` named in that module's `NotBuilt("… (WP-NN)")` stub message; `—` when the module has no such stub (it already works). WP-34 corrects this column later, so do not research it further.
  - Keep these three Parts rows, each its own row and starting with exactly this text (the locked asserts at api_response.rs:69, link_stop.rs:112 and recipe_load.rs:150 each look for their own cell):
    - ``| `crates/engine/src/api.rs` | present |``
    - ``| `crates/engine/src/link.rs` | present |``
    - ``| `crates/engine/src/recipes.rs` | present |``
  - Keep the section names Parts, API shape, Interlinks, Packaging, "State, IPC and files" and LinkEvent: the `//!` module headers cite them (22 files today).
  - Run `python3 tools/docs/check.py docs/code-map.md docs/ROADMAP.md` until it is clean.
- **plan.md:**
  - Delete :9-16 (status), :138-142 (moved to workbench) and :145-154 (replaced by links to rulings 20/21/3).
  - Scrub the owner's name (→ "the owner") and the second machine's name (→ "a second machine (2026-10-18 scan leg)") at :15, :53, :67, :69, :71, :87, :130, :140-141, :147-154.
  - :87: "`craft-build` distrobox" → "the distrobox build container (`$FLAT3D_BUILD_BOX`, see ../workbench.md)".
  - :121 → "a works-now recipe; `flat3d-run` and the per-game switch apply it" (C5).
  - Links: :7 → `spidy-core-map.md`; :53 → `#what-the-file-lists-already-show`.
  - Merge in ROADMAP :145-157 and address-map :25-37.
- **spidy-core-map.md** (C26): give each cited SpidyVR file its full path under the submodule: :58 `third_party/SpidyVR/include/spidy/presentation_gate.hpp`, :147 `third_party/SpidyVR/docs/VALIDATION.md` (and `third_party/SpidyVR/docs/REFERENCE.md`), :150 `third_party/SpidyVR/tools/inspect_game.py` (and its `docs/REFERENCE.md:20-24` → `third_party/SpidyVR/docs/REFERENCE.md:20-24`), :188 `third_party/SpidyVR/docs/REFERENCE.md`. The file has no relative links, so the move needs no link fixes in it.
- **address-map.md:**
  - Delete :5-23 (the second-machine narrative; git keeps it). Move :25-37 to plan.md, and scrub the second machine's name in the moved :37 → "a second machine (2026-10-18 scan leg)".
  - :3 link → `spidy-core-map.md`.
  - :41, :165: `B:/private/...` → `$FLAT3D_RE_HOME/exe/<game>-<version>/` (e.g. `sm2-2.810.0.0`).
  - Generated block: one line `<!-- BEGIN GENERATED -->` just before `### Priority 1` (:62), and one line `<!-- END GENERATED -->` after the last Priority 3 table row (:160), before `## Next leg` (:162). The block holds the three Priority headings and their tables. These are the literal markers bones §2 ("One owner per fact") says WP-25b defines, and WP-25a's `generated` rule reads. WP-30's renderer replaces exactly what sits between these two literals, with no suffix on either marker.
- **archive/uncharted3-stereo.md:** header "after-1.0 (#13/#4); not on the 1.0 path". The owner's name → "the owner" (:5, :18, :20, :108, :153).
- **STEP0-RUNSHEET.md** (hand-scrub; same path, because SPEC M1 cites it):
  - Roles: the owner's name → **You:**, Claude → **Script:**.
  - :21 path → `$HOME/...`, "written out in full".
  - Delete :7-14 "Already set up".
  - :62 → `docs/spiderman2/plan.md`.
- **recipes/README.md:**
  - :11 → "`name` plus the eight stereo keys (DECISIONS 'Recipe keys')" (C2).
  - :3 → "the five 1.0 games' recipes (SPEC)" (C17).
  - Keep the strings `<appid>-<slug>.recipe` and `key: value` (recipe_load.rs:147).
- **CHANGELOG.md:** append the SPEC approval history (:7, :9, :11) and one line for this pass.
- **docs/workbench.md** (new). Sources: README:75-84, ROADMAP:131-143 and :176-184, PORT:87 and :138-142.
  - One table, one row per tool, with columns `| Tool | Where | Check command |`:
    - the build box: `$FLAT3D_BUILD_BOX`, default `craft-build`; check `distrobox enter "$FLAT3D_BUILD_BOX" -- clang-cl --version`;
    - xwin: `$FLAT3D_RE_HOME/xwin`; check `ls "$FLAT3D_RE_HOME/xwin"`;
    - Ghidra 12.1.4: `$FLAT3D_RE_HOME/ghidra`; check `test -x "$FLAT3D_RE_HOME/ghidra/support/analyzeHeadless"`;
    - JDK 21: `$FLAT3D_RE_HOME/jdk/current`; check `"$FLAT3D_RE_HOME/jdk/current/bin/java" -version`.
  - Above the table, one line gives the defaults the check commands assume: `FLAT3D_RE_HOME` unset means `$HOME/.local/share/flat-to-3d` (headless.sh :7, after this WP's rename under Tools below), and `FLAT3D_BUILD_BOX` unset means `craft-build`. The xwin row says that `cmake/clang-cl-xwin.cmake` reads it through `XWIN_DIR` (default `$HOME/.local/share/flat-to-3d/xwin`, :13).
  - Heavy-job rules: `FLAT3D_HEAVY_LOCK`, the 2 GiB watch, one job at a time.
  - Exit codes: `headless.sh` exit 75 (another heavy job, or the lock) and exit 137 (stopped by the 2 GiB watch) both mean "retry later".
  - The Ghidra command from ROADMAP :176-184 is written with `tools/ghidra/headless.sh` and `$FLAT3D_RE_HOME` paths, with no home path. `tools/re/analyze.sh` comes later (WP-30, wave 3), so the page does not name it yet.
- **Tools:**
  - `tools/ghidra/headless.sh`:
    - :3 → "refuses while `$FLAT3D_HEAVY_LOCK` exists".
    - :5 → `$FLAT3D_RE_HOME`.
    - :6 → `(see docs/workbench.md#ghidra; results in docs/spiderman2/address-map.md)`.
    - :7 → `F=${FLAT3D_RE_HOME:-${FLAT3D_HOME:-$HOME/.local/share/flat-to-3d}}`. This is a rename: the line today reads `F=${FLAT3D_HOME:-$HOME/.local/share/flat-to-3d}`, and `FLAT3D_HOME` appears nowhere else in the repo. `FLAT3D_RE_HOME` is the name every plan file and doc uses from this WP on; the old name stays only as this fallback, so an existing setup keeps working. Until this WP lands, the script reads `FLAT3D_HOME` only.
    - :11-12 → `LOCK=${FLAT3D_HEAVY_LOCK:-}; [ -n "$LOCK" ] && [ -e "$LOCK" ] && { echo "headless: $LOCK exists (another heavy job); retry when it is gone" >&2; exit 75; }`.
  - `tools/build-spidy.sh:2` → "(run inside the distrobox build container, `$FLAT3D_BUILD_BOX`, default craft-build)".
  - `tools/research/research_game.py:18` → `$FLAT3D_RE_HOME/venv`.
  - `tools/xmatch/targets.py:3` → `docs/spiderman2/spidy-core-map.md`. Keep :1's upstream author credit (GPL attribution).

Locked tests that read docs and must stay green: `api_response.rs`, `link_stop.rs`, `recipe_load.rs` (the code-map cells and the recipes/README strings).

**Depends on:** WP-01 (code-map paths, the `crates/` tree, `cli::COMMANDS`, `api::error_help`), WP-02 (`check.sh`; its headless.sh `"$P"` quoting lands first, and it leaves build-spidy.sh unchanged), WP-25a (check.py).

**Owner device:** none.

**Done-check:** `export CARGO_TARGET_DIR=$SCRATCH/target && python3 tools/docs/check.py && if command -v shellcheck >/dev/null; then shellcheck tools/ghidra/headless.sh tools/build-spidy.sh; else bash -n tools/ghidra/headless.sh && bash -n tools/build-spidy.sh; fi && python3 -W error -m py_compile tools/research/research_game.py tools/xmatch/targets.py && cargo test --locked -p flat3d-inject --test api_response --test link_stop --test recipe_load && tools/check.sh`

**Notes:**
- WP-30 uses the marker literals given above. If WP-30 names a suffixed marker, WP-30 changes, not this WP.
- WP-25b places the marker pair only in address-map.md. WP-34 adds the same unsuffixed pair to recipes/README.md by hand (its step 6). This overrides the recommended answer to `questions.md` question 46 (a suffixed pair placed by WP-25b): do not place any marker in recipes/README.md. STEP0-RUNSHEET.md has no markers: WP-24's header line marks the whole file as generated.
- `tools/docs/check.py` always skips `docs/plans/` (WP-25a, "Always skipped"). So the plan files, which quote the rule words, demo ids and marker strings, cannot fail this WP's full-repo `check.py` run, and this WP does not edit them.

---
