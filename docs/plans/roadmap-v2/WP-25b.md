<!-- split from roadmap-v2.md lines 3713-3820 at c0e6874 -->
### WP-25b: Docs scrub + contradictions

**Closes:** K13, A11, A16, A18, C2, C5-C8, C10-C19, C30, plus C20, C22 and C25-C29, which sit in the same files. C1/C3/C4 cannot close here (see Flag).

**Files:** as in the index.
- `git mv` moves:
  - `docs/PORT-SPIDERMAN2.md` → `docs/spiderman2/plan.md`
  - `docs/spidy-stereo-core-map.md` → `docs/spiderman2/spidy-core-map.md`
  - `docs/spiderman2-address-map.md` → `docs/spiderman2/address-map.md`
  - `docs/uncharted3-stereo.md` → `docs/archive/uncharted3-stereo.md`
- New: `docs/workbench.md`.
- One line in `tools/check.sh`: append `python3 tools/docs/check.py`. This is the only wave-2 edit to check.sh; the coordinator confirms that no other wave-2 WP touches the file.
- Untouched: `docs/notes/DECISIONS.md` and the SPEC Musts (SPEC.md:15-51).

**Edits:**
- **README.md**, rewritten:
  - Keep:
    - what it is (:3);
    - the ground rules (one line plus a DECISIONS link);
    - build/test (`tools/check.sh`, `cargo test --workspace`);
    - a full top-level layout table (`crates/ decky/ recipes/ packaging/ sessions/ games/ tools/ docs/ cmake/ third_party/ .github/`, C27);
    - the credits (:86-106).
  - Delete :5-57 (status, Hyperion, Legion Go) and :75-84 (moved to workbench.md).
- **SPEC.md** (non-Must lines only):
  - Move the approval history (:5, :7, :9, :11) to CHANGELOG. Keep :3 and :13.
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
- **docs/ROADMAP.md:**
  - :3 → "This is the plan. The SM2 track is `spiderman2/plan.md`" (C10).
  - :94 → `flat3d-inject inject 2075800` (C6).
  - :204 → `docs/spiderman2/address-map.md` (A16).
  - :32 → doctor pointer (C15).
  - :135 → "Step 1 done except the full-hash manifest" (C20).
  - Move :131-143 and :176-184 to workbench.md, and replace the Ghidra command with `tools/ghidra/analyze.sh sm2 <exe>`.
  - Move :145-157 to plan.md, and fix :157's link.
  - Scrub :133-134, :143, :167, :178 (qwen/craft-build). Keep the `iannitram/` repo coordinates.
  - Drop the test counts (A18). This is the only status section (C28).
- **docs/code-map.md:**
  - :3 → one sentence plus a ROADMAP link. :5 → the new spiderman2 paths.
  - :21: split `Watch --> Link[..]    Watch --> Health[..]` onto two lines (A11/C19).
  - :88 → "the distrobox build container (`$FLAT3D_BUILD_BOX`)".
  - :90 → "`ls crates/engine/tests`" (C30; drop the 55-name list and the counts).
  - :152: cite the `decky/main.py` shim (`io`/`command-failed`) instead of #67 (C18).
  - One paths table under "State, IPC and files": `~/.local/share/flat3d`, `~/.local/state/flat3d` (logs, soak, sessions), `~/.config/flat3d/settings`, `$XDG_RUNTIME_DIR`, `$FLAT3D_RE_HOME`.
  - New `## Commands` table, one row per `AnyCommand` word: list status inject undo watch snapshot switch setting install uninstall run start-3d stop-3d view rows launch collection toasts logs setup doctor.
  - Keep the cells `` | `crates/engine/src/{api,link,recipes}.rs` | present | `` (locked api_response.rs:69, link_stop.rs:112, recipe_load.rs:150 after WP-01).
  - Keep the section names Parts, API shape, Interlinks, Packaging, "State, IPC and files" and LinkEvent: 51 `//!` headers cite them.
- **plan.md:**
  - Delete :9-16 (status), :138-142 (moved to workbench) and :145-154 (replaced by links to rulings 20/21/3).
  - Scrub "the owner" (→ "the owner") and Hyperion (→ "a second machine (2026-10-18 scan leg)") at :15, :53, :67, :69, :71, :87, :130, :140-141, :147-154.
  - :121 → "a works-now recipe; `flat3d-run` and the per-game switch apply it" (C5).
  - Merge in ROADMAP :145-157.
- **spidy-core-map.md** :58, :147, :150, :188: prefix `third_party/SpidyVR/` (C26). Fix every `../` link for the new depth.
- **address-map.md:**
  - Delete :5-23 (the Hyperion/MDT narrative; git keeps it). Move :25-37 to plan.md.
  - :41, :165: `B:/private/...` → `$FLAT3D_RE_HOME/exe/<slug>-<version>/`.
  - Put the exact lines `<!-- BEGIN GENERATED -->` and `<!-- END GENERATED -->` around the tables. These are the literal markers from the docs-tools design (docs-tools.md:85, T10). WP-30's renderer must replace exactly what sits between these two literals, with no suffix on either marker.
- **archive/uncharted3-stereo.md:** header "after-1.0 (#13/#4); not on the 1.0 path". the owner → the owner (:5, :18, :20, :108, :153).
- **STEP0-RUNSHEET.md** (hand-scrub; same path, because SPEC M1 cites it):
  - Roles: the owner → **You:**, Claude → **Script:**.
  - :21 path → `$HOME/...`, "written out in full".
  - Delete :7-14 "Already set up".
  - :62 → `docs/spiderman2/plan.md`.
- **recipes/README.md:**
  - :11 → "`name` plus the eight stereo keys (DECISIONS 'Recipe keys')" (C2).
  - :3 → "the five 1.0 games' recipes (SPEC)" (C17).
  - Keep the strings `<appid>-<slug>.recipe` and `key: value` (recipe_load.rs:147).
- **CHANGELOG.md:** append the SPEC approval history and one line for this pass.
- **docs/workbench.md** (new). Sources: README:75-84, ROADMAP:131-143 and :176-184, PORT:87 and :138-142. Contents:
  - the build box (`FLAT3D_BUILD_BOX`, default `craft-build`);
  - xwin at `$FLAT3D_RE_HOME/xwin`;
  - Ghidra 12.1.4 + JDK 21 under `$FLAT3D_RE_HOME`;
  - heavy-job rules: `FLAT3D_HEAVY_LOCK`, the 2 GiB watch, one job at a time.
- **Tools:**
  - `tools/ghidra/headless.sh`:
    - :3 → "refuses while `$FLAT3D_HEAVY_LOCK` exists".
    - :5 → `$FLAT3D_RE_HOME`.
    - :6 → `(see docs/workbench.md#ghidra; results in docs/spiderman2/address-map.md)`.
    - :7 → `F=${FLAT3D_RE_HOME:-${FLAT3D_HOME:-$HOME/.local/share/flat-to-3d}}`.
    - :11-12 → `LOCK=${FLAT3D_HEAVY_LOCK:-}; [ -n "$LOCK" ] && [ -e "$LOCK" ] && { echo "headless: $LOCK exists (another heavy job); not now" >&2; exit 75; }`.
  - `tools/build-spidy.sh:2` → "(run inside the distrobox build container, `$FLAT3D_BUILD_BOX`, default craft-build)".
  - `tools/research/research_game.py:18` → `$FLAT3D_RE_HOME/venv`.
  - `tools/xmatch/targets.py:3` → `docs/spiderman2/spidy-core-map.md`. Keep :1's upstream author credit (GPL attribution).

Locked tests that read docs and must stay green: `api_response.rs`, `link_stop.rs`, `recipe_load.rs` (the code-map cells and the recipes/README strings).

**Depends on:** WP-01 (code-map paths, the `crates/` tree), WP-02 (`check.sh`; the headless.sh and build-spidy.sh lint edits land first), WP-25a (check.py).

**Owner device:** none.

**Done-check:** `python3 tools/docs/check.py && shellcheck tools/ghidra/headless.sh tools/build-spidy.sh && CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test api_response --test link_stop --test recipe_load`

**Flags:**
- C1, C3 and C4 (and C23/C24) are DECISIONS.md edits. Those are owner-blocked (O-1..O-4), and the file stays untouched, so they are dropped from "Closes".
- C9 is the `Cargo.toml` description, which is frozen after wave 1. WP-01 owns it.
- `docs/archive/README.md`, `docs/archive/spiderman2-scan-leg-*.md` and the `docs/notes/research*` moves are not in the file list. Default: delete the narrative (git keeps it) and allow-list `docs/notes/**`.
- WP-30 must use the marker literals given above. If WP-30 already names a suffixed marker, change WP-30, not this WP.

---
