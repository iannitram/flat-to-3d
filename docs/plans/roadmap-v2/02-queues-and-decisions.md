<!-- split from roadmap-v2.md lines 477-609 at c0e6874 -->
## 3. Executable-code conversions

| ID | Path | In → out | No-device test |
|---|---|---|---|
| T1 | `tools/check.sh` (WP-02) | runs the CI steps locally, `--locked`, lock-diff guard, temp XDG_RUNTIME_DIR | yes |
| T2 | `tools/docs/check.py` (WP-25a) | docs + tree + `crates/engine/src/cli.rs` → present/stub paths exist, dead links, mermaid 2-stmt, USAGE⊆table, no `inject/`/`app/src` paths in ROADMAP/code-map outside the mv-map block, private words (allow-list), `crates/*/tests/` exempt | fixtures |
| T3 | `tools/pm/issues_report.py` (WP-02) | `gh issue list --json` (read-only) → ready/blocked/owner-device table from "Blocked by #n" | fixture JSON |
| T4 | `packaging/pin.sh` (WP-02) | `<key>`/`--all-tbd`/`--file` → TOFU sha256 into pins; exit 0/2/3/4/5; never commits | `PIN_ALLOW_FILE=1` + file:// |
| T5 | `tools/re/analyze.sh` + `tools/ghidra/{Flat3dVtOptions,ExportVtMatches}.java` (WP-30) | exe pair → `vt-matches.csv` | `--dry-run` argv + shellcheck when present |
| T6 | `tools/re/fill_map.py` (WP-30) | csv + `games/spiderman2/map.toml` → entries with status pending\|found; `render` → the single unsuffixed generated block in address-map | yes |
| T7 | `tools/re/exe_id.py` (WP-30) | exe → build slug via in-file `KNOWN_BUILDS` / sm2 pin (sha c9b912ef…b738c9, 194262992 B) | yes |
| T8 | `tools/re/analyze.sh all` (WP-30) | analyze→vt→fill→render, overnight | dry-run |
| T9 | `tools/session/session.py run\|render\|check` (WP-24) | TOML → guided steps, JSON results in `~/.local/state/flat3d/sessions/`, markdown for the issue; `render` → STEP0-RUNSHEET | `--script` answers |
| T10 | `sessions/*.toml` (WP-24) | data | schema check |
| T11 | `tools/stereo/fps.py` | MangoHud CSV → p50/p1 pass at `--min 30` | yes |
| T12 | `tools/stereo/disparity.py` | PNG/PPM (stdlib zlib decoder) → mean/p90 disparity, pass >1 px | synthetic |
| T13 | `flat3d-inject doctor [--json]` (WP-29, `crates/engine/src/cli/doctor.rs`) | runner → `DoctorReport` | FakeRunner |
| T14 | `flat3d-inject setup packages\|install-apps\|launch-env\|fetch` (WP-29/WP-27, `crates/engine/src/cli/{setup,setup_fetch}.rs`) | → `SetupReport` | FakeRunner |
| T15 | link cmds `watch snapshot switch setting install uninstall run start-3d stop-3d` (WP-08, `crates/engine/src/cli/link.rs`) | #120 | yes |
| T16 | view cmds `view rows launch collection toasts logs` (WP-28, `crates/engine/src/cli/view.rs`) | #121 | yes |
| T17 | `list status inject undo` (WP-07, WP-27, `crates/engine/src/cli/{engine,inject,undo}.rs`) | #27a | fake prefix |
| T18 | `packaging/build-tools.sh` (WP-22) | pins → `dist/tools/{adb,lib/,gnirehtet,gnirehtet.apk}` | dry-run |
| T19 | `packaging/build-appimage.sh` + `.github/workflows/appimage.yml` (WP-22; `[self-hosted, linux]`, `rust-toolchain@1.95.0`) | → `dist/Flat_to_3D-x86_64.AppImage` | dispatch run |
| T20 | `packaging/build-decky.sh` (WP-22) | → `dist/flat-to-3d.zip` | zip listing |
| T21 | `crates/engine/tests/recipes_repo.rs` + `crates/engine/tests/recipe_seeds.rs` (WP-26) | recipes lint + route keys ⇒ pin keys | yes |
| T22 | `tools/build-win.sh` (WP-15) | xwin SDK at `~/.local/share/flat-to-3d/xwin` → `dist/tools/flat3d-uevr-inject.exe` | shellcheck; owner run |

## 5. Owner-device queue (in order; driver = session file or script)

| Q | What | Driver | Needs WPs | Closes |
|---|---|---|---|---|
| Q0 | record TBD shas (downloads) | `packaging/pin.sh --all-tbd` (OD8) | 02 | K22 |
| Q1 | tools build | `sessions/tools-build.toml` → `build-tools.sh` in craft-build | 22 | #127 |
| Q2 | injector exe | `tools/build-win.sh --install` (xwin fetch, licence accept) | 15 | — |
| Q3 | install + link smoke, unit, soak | `sessions/link-smoke.toml` | 08,09,10,11,12,20 | #26,#39,#125 soak,#128 |
| Q4 | doctor baseline | `flat3d-inject doctor --json` inside link-smoke | 29 | — |
| Q5 | ZC offline | `sessions/z1-offline.toml` | 24 | #31 |
| Q6 | UEVR live | `sessions/z2-uevr.toml` | 14,15,27,31 | #32,#51 |
| Q7 | ZC screen/mission/recipe | `z3-screen`, `z4-mission`, `z5-recipe` | Q6 | #33,#35,#36 |
| Q8 | AC7 | `a1-install`, `a2-uevr`, `a3-sortie` (OD8 prereqs) | Q6 | #40,#41,#42,#47 |
| Q9 | Geo-11 | `sessions/g1-geo11.toml` (fps.py, disparity.py) | 16,27,31 | #43,#52 |
| Q10 | viewer | `sessions/g2-viewer.toml` (craft-build `--features xr,capture`) | 17,18,19,31 | #44 |
| Q11 | Lana / Control | `g3-lana`, `g4-control` | Q9,Q10 | #45,#48,#46,#49 |
| Q12 | RE overnight | `tools/re/analyze.sh all` (`sessions/q12.toml`) | 30 | #19 (advances #37) |
| Q13 | SM2 offline, Step 0, Spidy 0.7.0 | `sm2-offline`, `step0`, `spidy-070` | 24 | #7,#5,#155 |
| Q14 | Decky | `sessions/decky.toml` | 21,22,32 | #130-#134 |
| Q15 | app window | `sessions/app-window.toml` | 08,27,29,33 | #78,#122,#123 |
| Q16 | AppImage | `sessions/appimage.toml` | 22,33 | #129 |

## 6. Issue housekeeping (owner/PM; the pass makes no GitHub writes)

- **Close now:** #146 (row exists via #174), #25, #50 (children merged), #38 (folded into #155). Close **#26/#39** after moving their device checks into `sessions/link-smoke.toml` (#35 keeps the in-game check).
- **Rewrite:**
  - #183 (48 files plus the clippy allow, both jobs);
  - #120 (A24 one-actor rule, `run`→`run_with`, `cli::link::run(cmd,ctx)`, `cli::link::request_3d`);
  - #125 (Where → `crates/engine/tests/tether_health.rs`, `crates/engine/src/watch/health_loop.rs`; remote tun0);
  - #121 (+ `logs`, #75);
  - #122/#123 (`check.yml`, `read_snapshot`, request file; `stop_3d` exists);
  - #127 (ruling 34: adb from nmeum source, gnirehtet zip, `dist/tools`);
  - #128 (no launch-option edits, ruling 27/40; `unit_text_with`);
  - #130 (`check.yml`; `main.py` shim per OD1);
  - #132 (drop #80); #133 (+#121); #134 (both binaries + tools in `bin/`);
  - #6 (Ghidra never ran); #9 (`crates/viewer`); #44 (Linux PipeWire viewer, crate path, `frame.rs`); #78 (`check.yml`);
  - #76/#116/#117/#118: paths `crates/...`, Done-when `cargo test -p flat3d-app` / `-p flat3d-inject`.
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
| OD4 | Grace→Ready (watch.rs:90,93 → `watch/machine.rs`) | **unchanged; a launch in grace plays flat; follow-up issue** | none |
| OD5 | `UndoRecord.launch_options_before` | **`""`; Decky strips our word** | WP-27, WP-32 |
| OD6 | recipe seeds location | **`crates/engine/tests/fixtures/recipe-seeds/`** (real recipes only at works-now) | WP-26 |
| OD7 | private-name policy O-9 + DECISIONS/SPEC wording O-1..O-8 | **scrub docs; DECISIONS untouched + allow-listed; uncharted3 → docs/archive** | WP-25b, WP-34 |
| OD8 | approve `pin.sh --all-tbd` downloads + AC7 manual prereqs (Nexus 2179/2274, Flat2VR Discord pak) | **approve** | Q0, Q8 |

Resolved without the owner:
- D4: `*_with` + follow-up.
- D6: adb from source + gnirehtet zip.
- `KillMode` default `control-group` (documented in packaging/README).
- pnpm 10.34.9 in CI. Local 9.15.9 reads lockfile v9.
- Rust 1.95.0 pinned in every workflow (matches host rustc 1.95.0).
- No linger.
- Fetch errors map to ApiError only through WP-13's `From<FetchError>` (§2). There are no `manual` or `checksum` codes.

## Not in this pass

- **#27b port route** (`crates/engine/src/injectors/port.rs` stays a stub): needs #22.
- **#18-#23 port steps:** need RE results (Q12).
- **#29, #36, #47-#49 real recipes:** locked tests land only at device works-now (Q7-Q11).
- **#34 Z2b:** only if #32 fails.
- **#9 and after-1.0 #4, #8, #11-#14.**
- **Katanga probe / viewer option (a):** the Linux viewer was chosen.
- **Viewer status in `LinkSnapshot`:** it would change locked snapshot JSON (api_snapshot.rs).
- **Unifying `flat_reason`:** locked rows.rs and link_view.rs.
- **Grace→Ready change:** OD4.
- **#37 closure:** needs the analysed exe pair. The column ships as "not yet found".
- **Kaon .NET route:** fallback only.
- **CI Windows link:** SDK licence, owner-only (Q2).
- **Any real download:** Q0.
- **#17 tracker and #10 parent:** stay open.

Review note: rejected the `pub use api::NotBuilt;` form because `NotBuilt` is defined at the crate root (inject/src/lib.rs:39-47), not in `api`, and api.rs:103 only implements `From<crate::NotBuilt>`. The bones keep the current lib.rs definition, so `flat3d_inject::NotBuilt` (api_response.rs:5,27) still resolves.

Review note: rejected `python3 -m unittest discover -s decky/tests -t decky` because it raises "Start directory is not importable" unless `decky/tests/__init__.py` exists (Python 3.14 unittest/loader.py:334; reproduced in scratch). Adding `sys.path` lines to a locked decky test is not allowed either. The replacement is `cd decky && python3 -m unittest discover -s tests`, which passes with a bare `import main` (verified).

Review note: rejected `todo!()` stubs for the frozen `cli::<m>::run` because dispatch runs through them in the CLI tests, and a panic would replace today's `not-built` result (cli.rs:147; cli_json.rs:64-66 asserts exit 2 for not-built). The stubs return `Err(NotBuilt(..).into())`.

Review note: rejected `r:&dyn CommandRunner` in `HealthLoop::tick`, `ensure_applied` and `detect_server` because both runner traits take `&mut self` (link.rs:110, tether/adb.rs:64). The frozen forms use `&mut dyn`. Health takes `RelayRunner`, following the exec rule that tether/health use the relay shape.

Review note: rejected the claim that `@1.95.0` is already pinned. check.yml:26 uses `dtolnay/rust-toolchain@stable` today, so the 1.95.0 pin is new in this pass. The single-runner and single-pin fix is otherwise applied.

Review note: the §2 frozen forms were reconciled against the implementing WPs. §2 now freezes the implementer's form for wivrn (WP-12: `link/wivrn.rs`, re-exported as `crate::wivrn`), HealthLoop (WP-11), ensure_applied (WP-27), install (WP-20), wrapper (WP-31) and the viewer frame (WP-17/18/19). It keeps bones' form for `from_env` (BTreeMap/ApiError) and `Ctx` (relay, json). Rejected:
- closure `from_env`;
- 4-arg `ensure_applied`/`injectors::Applied`;
- `HealthAction`/`WriteSoak`;
- `PixelFormat`/`Cpu` frame;
- `set_wivrn(&WivrnStart)`;
- `LOG_LIMIT`;
- the `wrapper_run.rs:206-207` edit;
- moving WP-33 to wave 5 (its CLI uses of WP-29/WP-27/WP-08 are runtime, not merge deps).
