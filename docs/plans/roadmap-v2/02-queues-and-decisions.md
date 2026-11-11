<!-- split from roadmap-v2.md lines 477-609 at c0e6874 -->
## 3. Executable-code conversions

| ID | Path | In → out | No-device test |
|---|---|---|---|
| T1 | `tools/check.sh` (WP-02) | the only list of CI steps: check.yml runs `tools/check.sh --ci --only rust\|py\|decky`, and `--ci` turns every skip into a failure. Named steps print `FAIL <step>: fix: <hint>`; `--only`, `--list`, `--tests <cargo args>`, a closing pass/skip/fail summary; sets CARGO_TARGET_DIR itself; `--locked`, lock-diff guard, temp XDG_RUNTIME_DIR | yes |
| T2 | `tools/docs/check.py` (WP-25a) | docs + tree + `crates/engine/src/cli.rs` → present/stub paths exist, dead links, mermaid 2-stmt, USAGE⊆table, no `inject/`/`app/src` paths in ROADMAP/code-map outside the mv-map block, private words (allow-list), `crates/*/tests/` exempt | fixtures |
| T3 | `tools/pm/issues_report.py` (WP-02) | `gh issue list --json` (read-only) → ready/blocked/owner-device table from "Blocked by #n"; `--roadmap [--json]` reads this roadmap → ready and blocked WPs, owner-device WPs, ready Q rows (§5), ODs still on their default (§7), and the plan lint, which exits non-zero naming the WP and the fix | fixture JSON + fixture roadmap |
| T4 | `packaging/pin.sh` (WP-02) | `<key>`/`--all-tbd`/`--file` → TOFU sha256 into pins; exit 0/2/3/4/5; never commits | `PIN_ALLOW_FILE=1` + file:// |
| T5 | `tools/re/analyze.sh` + `tools/ghidra/{Flat3dVtOptions,ExportVtMatches}.java` (WP-30) | exe pair → `vt-matches.csv` | `--dry-run` argv + shellcheck when present |
| T6 | `tools/re/fill_map.py` (WP-30) | csv + `games/spiderman2/map.toml` → entries with status pending\|found; `render` → the single unsuffixed generated block in address-map | yes |
| T7 | `tools/re/exe_id.py` (WP-30) | exe → build slug via in-file `KNOWN_BUILDS` / sm2 pin (sha c9b912ef…b738c9, 194262992 B) | yes |
| T8 | `tools/re/analyze.sh all` (WP-30) | analyze→vt→fill→render, overnight | dry-run |
| T9 | `tools/session/session.py run\|render\|check\|pending` (WP-24) | TOML → guided steps, JSON results in `~/.local/state/flat3d/sessions/`, markdown for the issue; `render` → STEP0-RUNSHEET; `pending` → sessions with no PASS result yet, in §5 Q order | `--script` answers |
| T10 | `sessions/*.toml` (WP-24) | data | schema check |
| T11 | `tools/stereo/fps.py` | MangoHud CSV → p50/p1 pass at `--min 30` | yes |
| T12 | `tools/stereo/disparity.py` | PNG/PPM (stdlib zlib decoder) → mean/p90 disparity, pass >1 px | synthetic |
| T13 | `flat3d-inject doctor [--json]` (WP-29, `crates/engine/src/cli/doctor.rs`) | runner → `DoctorReport`; a red report exits 1 and still prints the full report (ruling D4) | FakeRunner |
| T14 | `flat3d-inject setup packages\|install-apps\|launch-env\|fetch` (WP-29/WP-27, `crates/engine/src/cli/{setup,setup_fetch}.rs`) | → `SetupReport` | FakeRunner |
| T15 | link cmds `watch snapshot switch setting install uninstall run start-3d stop-3d` (WP-08, `crates/engine/src/cli/link.rs`) | #120 | yes |
| T16 | view cmds `view rows launch collection toasts logs` (WP-28, `crates/engine/src/cli/view.rs`) + `crates/engine/tests/decky_face.rs` (WP-28) | #121; decky_face.rs runs each `decky/src/engine.ts` wrapper's sample argv, with the trailing `--json`, through `cli::parse_any`, and checks the TS ErrorCode union and the types.ts mirror keys | yes |
| T17 | `list status inject undo` (WP-07, WP-27, `crates/engine/src/cli/{engine,inject,undo}.rs`) | #27a | fake prefix |
| T18 | `packaging/build-tools.sh` (WP-22) | pins → `dist/tools/{adb,lib/}` (no gnirehtet, ruling 42) | dry-run |
| T19 | `packaging/build-appimage.sh` + `.github/workflows/appimage.yml` (WP-22; `[self-hosted, linux]`, `rust-toolchain@1.95.0`) | → `dist/Flat_to_3D-x86_64.AppImage` | dispatch run |
| T20 | `packaging/build-decky.sh` (WP-22) | → `dist/flat-to-3d.zip` | zip listing |
| T21 | `crates/engine/tests/recipes_repo.rs` + `crates/engine/tests/recipe_seeds.rs` (WP-26) | recipes lint + route keys ⇒ pin keys | yes |
| T22 | `tools/build-win.sh` (WP-15) | xwin SDK at `~/.local/share/flat-to-3d/xwin` → `dist/tools/flat3d-uevr-inject.exe` | shellcheck; owner run |

## 5. Owner-device queue (in order; driver = session file or script)

`tools/pm/issues_report.py --roadmap` reads this table, so keep its columns.

| Q | What | Driver | Needs WPs | Closes |
|---|---|---|---|---|
| Q0 | record TBD shas (downloads) | `sessions/q0.toml`: `packaging/pin.sh --all-tbd` (OD8); passes when no `TBD-at-pin` is left in `packaging/pins` | 02 | K22 |
| Q1 | tools build | `sessions/tools-build.toml` → `build-tools.sh` in craft-build | 22 | #127 |
| Q2 | injector exe | `sessions/q2.toml`: `tools/build-win.sh --fetch-sdk` (network; running it is the owner accepting the SDK licence), then `tools/build-win.sh --install`; passes when `flat3d/bin/flat3d-uevr-inject.exe` exists under XDG_DATA_HOME | 15 | — |
| Q3 | install + link smoke, unit, soak, `list --json` | `sessions/link-smoke.toml` | 07,08,09,10,11,12,20 | #26,#39,#125 soak,#128 |
| Q4 | doctor baseline | `flat3d-inject doctor --json` inside link-smoke; passes on exit 0 (a red report exits 1, ruling D4) and when the `quest-app` line shows a version, not `no headset` (that skip still exits 0) | 29 | — |
| Q5 | ZC offline | `sessions/z1-offline.toml` | 24 | #31 |
| Q6 | UEVR live | `sessions/z2-uevr.toml` | 14,15,27,31 | #32,#51 |
| Q7 | ZC screen/mission/recipe | `z3-screen`, `z4-mission`, `z5-recipe` | Q6 | #33,#35,#36 |
| Q8 | AC7 | `a1-install`, `a2-uevr`, `a3-sortie` (OD8 prereqs) | Q6 | #40,#41,#42,#47 |
| Q9 | Geo-11 | `sessions/g1-geo11.toml` (fps.py, disparity.py) | 16,27,31 | #43,#52 |
| Q10 | viewer | `sessions/g2-viewer.toml` (craft-build `--features xr,capture`) | 17,18,19,31 | #44 |
| Q11 | Lana / Control | `g3-lana`, `g4-control` | Q9,Q10 | #45,#48,#46,#49 |
| Q12 | RE overnight | `tools/re/analyze.sh all` (`sessions/q12.toml`: compiles the Java scripts first, then the long analyze steps, then `tools/re/fill_map.py --check`) | 30 | #19 (advances #37) |
| Q13 | SM2 offline, Step 0, Spidy 0.7.0 | `sm2-offline`, `step0`, `spidy-070` | 24 | #7,#5,#155 |
| Q14 | Decky | `sessions/decky.toml` | 21,22,32 | #130-#134 |
| Q15 | app window | `sessions/app-window.toml` | 08,27,29,33 | #78,#122,#123 |
| Q16 | AppImage | `sessions/appimage.toml` | 22,33 | #129 |

- Every session TOML carries `q = "Qn"`, and `session.py check` rejects an unknown Q id.
- Each deferred device claim maps to a session step:
  - link-smoke (Q3): unplug/replug and sleep/wake heal (WP-09); the tun0, gnirehtet package and Wi-Fi-back steps are void (WP-10 dropped, ruling 42); `stop-3d` twice, then status shows stopped, plus a Quest app version check (WP-12); `list --json` (WP-07).
  - decky (Q14): `launch-opts-persist`, `collection-save-persist`, `toast-shows`, `xdg-runtime-set` (WP-32), plus the Game Mode gates (the QAM panel renders, the plugin survives a Game Mode restart).
  - g2-viewer (Q10): ask steps `overlay-in-capture`, `portal-window-kde` and `portal-restore-token` (WP-18), and `xr-runtime-found`, `quad-shows`, `eyes-left-right`, `dmabuf-import`, `recenter-works` and `exits-with-game` (WP-19). WP-24's Session index row has the wording of each.
- **Closing (rulings D1, D5):** the rule lives in `01-new-bones.md` §2 "PR and issue rules" (Refs vs Closes) and §0 "PR body" (`## Verified` / `## Not verified`). In short, a WP named under "Needs WPs" writes `Refs #N`, lists the Q row under `## Not verified`, and the owner or PM closes the issue when its session passes.

## 6. Issue housekeeping (owner/PM; the pass makes no GitHub writes)

- **Close now:** #146 (row exists via #174), #25, #50 (children merged), #38 (folded into #155). Close **#26/#39** after moving their device checks into `sessions/link-smoke.toml` (#35 keeps the in-game check).
- **Rewrite:**
  - #183 (48 files plus the clippy allow, both jobs);
  - #120 (A24 one-actor rule, `run`→`run_with`, `cli::link::run(cmd,ctx)`, `cli::link::request_3d`);
  - #125 (Where → `crates/engine/src/watch/health_loop.rs`; battery/temperature only: the tunnel, heal, throughput and soak path is not planned, ruling 42);
  - #121 (+ `logs`, #75);
  - #122/#123 (`check.yml`, `read_snapshot`, request file; `stop_3d` exists; the app's snapshot tick is 1 s, where #123 and docs/code-map.md:74 say 2 s);
  - #127 (ruling 34, reworded by ruling 42: adb from nmeum source for WiVRn, no gnirehtet, `dist/tools`);
  - #128 (no launch-option edits, ruling 27/40; `unit_text_with`);
  - #130 (`check.yml`; `main.py` shim per OD1);
  - #132 (drop #80); #133 (+#121); #134 (both binaries + tools in `bin/`);
  - #6 (Ghidra never ran); #9 (`crates/viewer`); #44 (Linux PipeWire viewer, crate path, `frame.rs`); #78 (`check.yml`);
  - #76/#116/#117/#118: paths `crates/...`, Done-when `cargo test -p flat3d-app` / `-p flat3d-inject`.
  - #118: the step-1 hint becomes "Run flat3d-inject install." (ruling D3, WP-05);
  - #131: `decky/src/Panel.tsx` → `decky/src/panel.tsx` (WP-32).
- **Split** #27 → 27a (dispatch/list/inject/undo, this pass) and 27b (port route, after #22).
- **New issues:**
  - one per new WP without an issue: 01, 02, 06, 09, 10, 12, 13, 14, 15, 16, 17-19 as #44a/b/c, 24, 25a, 25b, 26, 29, 30, 31;
  - the A5 bug;
  - the `*_with` cleanup follow-up;
  - Grace→Ready (OD4);
  - the `flat_reason` unification follow-up.
- **Labels:** add `needs-owner-device` and apply it to every Q-row issue.
- **Branches/worktrees:** the owner lists the stale ones with `git branch -r --merged` and confirms before any deletion.

## 7. Owner decisions (defaults bold; the pass runs on the defaults)

| OD | Question | Default | Blocks if overruled |
|---|---|---|---|
| OD1 | Ruling 40 wording: is a 30-line `main.py` exec shim (passive plugins can't call) allowed? | **yes, amend ruling text in DECISIONS by owner** | WP-21, WP-32 |
| OD2 | D3: own Windows injector vs UEVRInjector.exe | **own (`flat3d-uevr-inject`)** | WP-15, WP-31 UEVR path |
| OD3 | D5: SPEC.md:77 viewer amendment text (Must section) | **"Linux-native PipeWire→OpenXR quad viewer on WiVRn"** | WP-34 SPEC line only |
| OD4 | Grace→Ready (`state_text`, inject/src/watch.rs:754-755 today → `watch/machine.rs`) | **unchanged; a launch in grace plays flat; follow-up issue** | none |
| OD5 | `UndoRecord.launch_options_before` | **`""`; Decky strips our word** | WP-27, WP-32 |
| OD6 | recipe seeds location | **`crates/engine/tests/fixtures/recipe-seeds/`** (real recipes only at works-now) | WP-26 |
| OD7 | private-name policy O-9 + DECISIONS/SPEC wording O-1..O-8 | **scrub docs; DECISIONS untouched + allow-listed; uncharted3 → docs/archive** | WP-25b, WP-34 |
| OD8 | approve `pin.sh --all-tbd` downloads + AC7 manual prereqs (Nexus 2179/2274, Flat2VR Discord pak) | **approve** | Q0, Q8 |

Owner rulings, 2026-11-09 (section D of the Craft review; full text in `rulings-2026-11-09.md`). This file cites them as "ruling D1" to "ruling D6". The bare D-numbers in the OD table and in the list below are the older plan decisions, not these. This note is the whole fix: the older decisions are not renamed to PD-n, and no WP owns such a rename (WP-34 cannot edit this file).
- **D1 PR rules:** every PR body has `## Verified` (each done-check command and its exit status) and `## Not verified` (each host skip it hit, plus its device Q row) sections, and never claims a run that was not done. A merge takes only the head commit the review passed. The two-commit landings (WP-00) stay as two commits.
- **D2 WP-01 contract, three additions:** `api::error_help(code) -> (retryable, fix)` with a test that every code has an entry; a dispatch hook that lets a command set its own exit code; and a full `cli::COMMANDS` registry in which each command declares its id, args and effects. The usage text, the Decky wrappers, the app's effect args and the code-map Commands table are checked against the registry.
- **D3 WP-05 hint:** fixed in WP-05. The step-1 hint drops "install Quest Tether" (stale under ruling 30) and points at the engine's own setup command. The brief is updated to match. No WP-34 follow-up. **Superseded by ruling 42 (2026-11-10):** the hint may name Quest Tether again.
- **D4 doctor:** a red report exits 1 and still prints the full report.
- **D5 issue closing:** a WP that has a device Q row uses `Refs #N`. The issue is closed when its device session passes (§5).
- **D6 error codes:** closed at 8 for this pass (`ERROR_CODES` in api.rs; the locked `api_response.rs` pins 8). No WP adds a code. A new failure maps to an existing code with a specific message.
- **Ruling 42 (2026-11-10):** Flat to 3D and Quest Tether stay separate apps. The built-in tether (gnirehtet relay, tun0, Wi-Fi move, heal ladder) is removed; only the adb choice WiVRn needs stays (`link/adb.rs`). Quest Tether's state may be shown read-only, never started, stopped or healed. 1.0 promises the 3D picture over WiVRn only. WP-10 is dropped; WP-35..39 are added. Supersedes rulings 30 and 33, and D3 above; full text in `docs/notes/DECISIONS.md`.

Resolved without the owner:
- D4: `*_with` + follow-up.
- D6: adb from source (adb only, for WiVRn; the gnirehtet zip is void, ruling 42).
- `KillMode` default `control-group` (documented in packaging/README).
- pnpm 10.34.9 in CI. Local 9.15.9 reads lockfile v9.
- Rust 1.95.0 pinned in every workflow (matches host rustc 1.95.0).
- No linger.
- Fetch errors map to ApiError only through WP-13's `From<FetchError>` (§2). There are no `manual` or `checksum` codes.

## Not in this pass

IDs are stable: never renumbered or reused. WP-34 copies them into ROADMAP, and a closed item is struck through, not deleted.

- **NP-1. #27b port route** (`crates/engine/src/injectors/port.rs` stays a stub): needs #22.
- **NP-2. #18-#23 port steps:** need RE results (Q12).
- **NP-3. #29, #36, #47-#49 real recipes:** locked tests land only at device works-now (Q7-Q11).
- **NP-4. #34 Z2b:** only if #32 fails.
- **NP-5. #9 and after-1.0 #4, #8, #11-#14.**
- **NP-6. Katanga probe / viewer option (a):** the Linux viewer was chosen.
- **NP-7. Viewer status in `LinkSnapshot`:** it would change locked snapshot JSON (api_snapshot.rs).
- **NP-8. Unifying `flat_reason`:** locked rows.rs and link_view.rs.
- **NP-9. Grace→Ready change:** OD4.
- **NP-10. #37 closure:** needs the analysed exe pair. The column ships as "not yet found".
- **NP-11. Kaon .NET route:** fallback only.
- **NP-12. CI Windows link:** SDK licence, owner-only (Q2).
- **NP-13. Any real download:** Q0.
- **NP-14. #17 tracker and #10 parent:** stay open.

Review note: rejected the `pub use api::NotBuilt;` form because `NotBuilt` is defined at the crate root (inject/src/lib.rs:39-47), not in `api`, and api.rs:103 only implements `From<crate::NotBuilt>`. The bones keep the current lib.rs definition, so `flat3d_inject::NotBuilt` (api_response.rs:5,27) still resolves.

Review note: rejected `python3 -m unittest discover -s decky/tests -t decky` because it raises "Start directory is not importable" unless `decky/tests/__init__.py` exists (Python 3.14 unittest/loader.py:334; reproduced in scratch). Adding `sys.path` lines to a locked decky test is not allowed either. The replacement is `cd decky && python3 -m unittest discover -s tests`, which passes with a bare `import main` (verified).

Review note: rejected `todo!()` stubs for the frozen `cli::<m>::run` because dispatch runs through them in the CLI tests, and a panic would replace today's `not-built` result (cli.rs:147; cli_json.rs:64-66 asserts exit 2 for not-built). The stubs return `Err(NotBuilt(..).into())`.

Review note: rejected `r:&dyn CommandRunner` in `HealthLoop::tick`, `ensure_applied` and `detect_server` because both runner traits take `&mut self` (link.rs:110, tether/adb.rs:64). The frozen forms use `&mut dyn`. Health took `RelayRunner` (void: the relay shape is removed, ruling 42).

Review note: rejected the claim that `@1.95.0` is already pinned. check.yml:26 uses `dtolnay/rust-toolchain@stable` today, so the 1.95.0 pin is new in this pass. The single-runner and single-pin fix is otherwise applied.

Review note (moved from WP-22, where it sat though it is about WP-19 and WP-21): accepted all six corrections, with three choices of route:
- WP-19 moves to wave 3 (depends on WP-17 and WP-18) instead of using WP-01 `todo!()` stubs.
- WP-21 declares the npm registry and moves the network step out of the offline part, because an offline local-pnpm install can't work: the metadata cache is empty and four pinned versions are missing from the store.
- WP-21's `--options` fix goes in the TS `launchSync` helper, not in main.py, because #67's locked order test fixes main.py's argv. The CLI's `--json` stripping (cli.rs:58-62) makes the duplicate flag harmless. (Now: WP-21 runs its own single pnpm registry step; see §0 "Network".)

Review note: the §2 frozen forms were reconciled against the implementing WPs. §2 now freezes the implementer's form for wivrn (WP-12: `link/wivrn.rs`, re-exported as `crate::wivrn`), HealthLoop (WP-11), ensure_applied (WP-27), install (WP-20), wrapper (WP-31) and the viewer frame (WP-17/18/19). It keeps bones' form for `from_env` (BTreeMap/ApiError) and `Ctx` (relay, json; `relay` is removed by WP-36, ruling 42). Rejected:
- closure `from_env`;
- 4-arg `ensure_applied`/`injectors::Applied`;
- `HealthAction`/`WriteSoak`;
- `PixelFormat`/`Cpu` frame;
- `set_wivrn(&WivrnStart)`;
- `LOG_LIMIT`;
- the `wrapper_run.rs:206-207` edit;
- moving WP-33 to wave 5 (its CLI uses of WP-29/WP-27/WP-08 are runtime, not merge deps).

Review note (moved from WP-26, where it sat though it is about WP-24's sessions): WP-24's z2 capture fix does not use the viewer's `--dump-eyes` for UEVR. UEVR submits straight to the OpenXR runtime, so the SBS viewer (geo11-viewer.md:3,65) never sees its frames, and no WiVRn per-eye dump is confirmed in the reports. z2's depth gate is owner-judged and its disparity measure is optional. No viewer WP ships an eye-dump flag, so g2-viewer's disparity measure is optional and never blocks PASS (WP-24).

Review note (moved from WP-26, about WP-24's sessions): q0, q2 and q12 follow §5 (02-queues-and-decisions.md), not v1 roadmap.md. Q0 records the TBD shas with `packaging/pin.sh --all-tbd`. Q2 runs the injector build (`tools/build-win.sh`), not doctor. Q12 is the RE overnight run. WP-24 writes those TOMLs.
