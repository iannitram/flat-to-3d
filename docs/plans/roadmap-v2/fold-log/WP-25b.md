# Fold log: WP-25b

Sources: `roadmap-v2-findings/*.json` and `*-proposal.md`, the Craft review, rulings 2026-11-09. One line per item.

## roadmap-v2-findings/WP-25b.json
- own_review (no edits made, all fixes in incoming) -> no action.
- incoming[0] (from WP-02: drop the check.sh append) -> applied. Bones §2 "CI final shape" settles it: check.sh runs check.py behind `[ -f docs/workbench.md ]`, so WP-25b's new file turns the step on.
- incoming[1] (from WP-30: exe dir `<game>-<version>`, marker placement includes the headings) -> applied (matches WP-30's local copy name `sm2-2.810.0.0`).
- incoming[2] (heading has no wave label) -> applied: "(wave 2)", as in 03-wp-index.md.
- incoming[3] (delete the check.sh bullet) -> applied (same as incoming[0]).
- incoming[4] (ROADMAP:23 `inject/` token) -> applied; checked: ROADMAP.md:23 has "the Rust engine in `inject/`". Final no-token step added.
- incoming[5] (code-map path rewrite) -> applied in reduced form. Stale in part: WP-01.md now does the rewrite and the mv-map block itself. WP-25b keeps the leftover fix, the :94 heading, `planned` marking and a check.py run. `--only status,links,mermaid,usage` replaced by a path-scoped run, because WP-25a's rule list may change in its own fold.
- incoming[6] (spell out the three locked cells) -> applied.
- incoming[7] (code-map:194 craft-build) -> applied; checked: code-map.md:194 says "in the `craft-build` distrobox". Merged with :88.
- incoming[8] (plan.md:87 craft-build) -> applied; checked: PORT-SPIDERMAN2.md:87.
- incoming[9] (moved address-map :37 names the second machine) -> applied; checked: address-map.md:37.
- incoming[10] (spell out link fixes) -> applied; checked PORT:7, PORT:53, ROADMAP:3, ROADMAP:157, address-map:3, code-map:5. The "../ links" sentence replaced by "no relative links" (checked: spidy-stereo-core-map.md has none).
- incoming[11] (C26 exact SpidyVR paths) -> applied as given. Not checked: the submodule is not checked out here. Also :147 cites `docs/REFERENCE.md`, so that path is named too.
- incoming[12] (C20 goes to "Where it stands"; keep SPEC :5) -> applied; checked: SPEC.md:13 says "sends the line above back".
- incoming[13] (marker placement exact) -> applied (single unsuffixed pair, :62 to after :160, before `## Next leg` :162; checked against address-map.md).
- incoming[14] (done-check guards) -> applied, with a fix: `a && b || c` would hide a shellcheck failure, and `bash -n a b` checks only the first file. Written as if/else with one `bash -n` per file. Ends in `tools/check.sh` with the `export` form (A2, 00-run-the-pass.md "SCRATCH").
- incoming[15] (stale archive/notes flag) -> applied; checked: no `docs/archive/` in the repo; `docs/notes/**` is in WP-25a's initial allow-list.

## Other findings that name WP-25b
- WP-02.json incoming[11] (from WP-25a: drop WP-02 step 10, WP-25b appends) -> dropped for WP-25b: bones §2 keeps the step in check.sh behind the workbench.md guard, and WP-25b does not touch check.sh.
- WP-25a.json incoming[0] (brace cells, paths rule) -> brace part applied here (three cells spelled out, so no brace cell stays); the rest targets WP-25a.
- WP-30.json incoming[0], [1], [7], [9], [10] -> target WP-30. The marker literal both sides agree on is the unsuffixed pair already in WP-25b; nothing new here. [10]'s headless.sh exit-75 wording matches WP-25b's new :11-12.
- WP-34.json own_review (3): WP-25b creates the `## Commands` table; keep column 1 in WP-25b's form -> applied: column 1 is the registry `id`, columns match WP-34's stated layout.
- WP-34.json incoming[0], [1] -> target WP-34 (allow-list lines). No change here; WP-25b's FLAT3D_BUILD_BOX lines rely on `*\tFLAT3D_BUILD_BOX` staying.
- WP-24-proposal.md:52-53 -> no WP-25b edit (index and WP-34 items).

## Craft review
- A2 (done-check ends in tools/check.sh) -> applied.
- A4 (refusals name the fix) -> applied to the headless.sh lock refusal ("retry when it is gone").
- A4 (one error-code table in code-map, WP-25b L3763) -> applied: 8 codes, exit, `api::error_help` retryable/fix, api.rs named as owner.
- A5 (faces' argv must parse) -> not applicable (no face argv in this WP).
- A12 (generated blocks) -> no change: the `generated` rule is WP-34's; WP-25b only places the markers.
- B WP-25b (workbench table with a check command per tool; 75 and 137 are "retry later") -> applied. Paths in the check commands come from headless.sh (`$F/ghidra`, `$F/jdk/current`).
- Completeness check, idea 6 (AGENTS.md could carry Verified / Not verified) -> applied with rulings D1/D5 (AGENTS.md rule 5 says `Closes #N`, which conflicts with D5).
- C1 -> ignored (this fold). C2 -> not WP-25b.

## Rulings 2026-11-09
- D1 -> applied to AGENTS.md. WP-25b has no device Q row, so no Verified/Not verified block of its own beyond §2.
- D2 -> applied: the Commands table rows come from `cli::COMMANDS`; the error table uses `api::error_help`. Depends now names both.
- D3, D4, D6 -> not WP-25b (D6 is reflected in "the 8 codes").
- D5 -> applied to AGENTS.md. WP-25b itself has no device row (`Owner device: none`).

## Self-checks (no source item)
- The split had turned "Scrub <name> (→ the owner)" into "the owner → the owner" in three places -> rewritten as "the owner's name → ...".
- The workbench Ghidra command `tools/ghidra/analyze.sh sm2 <exe>` is wrong: WP-30 adds `tools/re/analyze.sh` with `analyze <projdir> <exe>`, in wave 3 -> replaced by the existing headless.sh command with `$FLAT3D_RE_HOME` paths.
- Machine name in README delete note -> written as "second-machine notes".
- Flags settled: C1/C3/C4 (OD7) and C9 moved to "Not closed here"; the archive flag became a Files line; the WP-30 marker flag became a Note.

## Check pass (second agent)
- Coverage: every item that targets WP-25b (WP-25b.json incoming[0]-[15], WP-02.json [11], WP-25a.json [0], WP-30.json [0]/[1]/[7]/[9]/[10], WP-34.json own_review and [0]/[1], WP-24-proposal.md:52-53, Craft A2/A4/A7/A12, B WP-25b, idea 6, rulings D1-D6) is logged above. Nothing missed.
- ROADMAP.md:107 links `spiderman2-address-map.md`, which breaks after the git mv and fails check.py's links rule; no source named it -> added ":107 link → `spiderman2/address-map.md`" (completes incoming[10]).
- spidy-stereo-core-map.md:150 also cites `docs/REFERENCE.md:20-24` -> named its full path too (completes incoming[11]).
- The marker sentence cited "docs-tools.md:85, T10"; neither exists in the repo -> now cites bones §2 "One owner per fact" and WP-25a's `generated` rule.
- Open question (where the recipes/README and STEP0 markers come from) is settled by the other WPs as folded: WP-34 step 6 adds the unsuffixed pair to recipes/README.md by hand, and WP-25a treats STEP0-RUNSHEET.md as whole-file generated by WP-24's header. Added one Note saying WP-25b places markers in address-map.md only.
- Workbench check commands used `$FLAT3D_RE_HOME` and `$FLAT3D_BUILD_BOX` bare; unset, they check the wrong path -> the page states the defaults (`$HOME/.local/share/flat-to-3d`, headless.sh:7; `craft-build`) and that cmake reads xwin through `XWIN_DIR` (clang-cl-xwin.cmake:13).
- Depends said WP-02 lands "build-spidy.sh lint edits"; WP-02.md:239 says build-spidy.sh needs no change -> Depends now names only the headless.sh `"$P"` quoting (WP-02.md:19, :240).
- "51 `//!` headers cite them" -> 22 files cite a code-map section today (grep of `//!` lines) -> count corrected.
- Verified against the repo: ROADMAP:3/:23/:94/:157/:204, code-map:5/:21/:88/:94/:194, address-map:3/:37/:41/:62/:160/:162/:165, SPEC:5/:13, PORT:7/:53/:87, the three locked cells (api_response.rs:69, link_stop.rs:112, recipe_load.rs:150 and WP-01.md:602/:634-639), `ERROR_CODES` (api.rs:77-86) and `exit_code` (cli.rs:93-97), headless.sh lines 3-12, private-word hits in every WP-25b file are covered by a scrub, a move or an allow line.
