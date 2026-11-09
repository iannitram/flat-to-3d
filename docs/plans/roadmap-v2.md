# flat-to-3d roadmap v2 (2026-11-09)

Sections 0-7 and WP-01 are the corrected contract. The other WPs predate it: before coding a WP, read sections 0 and 2 (they win), then the WP, then `roadmap-v2-findings/WP-NN.json` and `WP-NN-proposal.md` if present (reviewer fixes checked against the code, not yet applied).

## 0. How to run the pass

- **Base:** flat-to-3d `main@2812f64`. One branch per WP, `v2/wp-NN-<slug>`. Merge order: wave 0 → wave 1 → … → wave 5. Inside a wave, WPs own disjoint files and merge in any order. A WP starts only after every WP in its `depends` is merged. A WP never depends on a WP in its own wave.
- **Wave 0 (WP-00, #183) lands alone.** It is format-only plus a clippy allow. Every later diff then stays readable.
- **Wave 1:** WP-01 (Rust reshape + bones, one agent), WP-02 (repo tooling, no Rust) and WP-25a (`tools/docs/*` only, no Rust) run in parallel. WP-01 is the only WP that may:
  - move files;
  - edit root `Cargo.toml`/`Cargo.lock`, any crate `Cargo.toml`, `crates/*/src/lib.rs`, `crates/engine/src/{api,cli,face,watch,exec,paths,testing}.rs`, the `crates/engine/src/cli/*.rs` entry signatures, the facade lines of `crates/engine/src/link.rs`, `crates/viewer/src/{frame,source,spec,gpu,xr}.rs`, `crates/app/src/lib.rs` (until WP-33) or `.github/workflows/check.yml`;
  - add a `mod` line.

  After wave 1, a WP that needs a new item adds it **inside its own file** behind a signature WP-01 already froze. The pub items other WPs call are listed in §2 "Frozen signatures" and "Wave-owned pub APIs".
- **SCRATCH:** `$SCRATCH` is the agent's own scratch dir. `<agent scratch>` means `$SCRATCH`. Every done-check starts with `export CARGO_TARGET_DIR=$SCRATCH/target`, a single `export`, so every cargo call in the chain uses it. A done-check written as `CARGO_TARGET_DIR=… cmd1 && cmd2` is read as that export.
- **Issue snapshot step:** this is a read-only GitHub read, not a write. The WP that needs it runs it; WP-00 does not.
  - Command: `mkdir -p $SCRATCH/issues; for n in 67 76 116 117 118; do gh issue view $n -R iannitram/flat-to-3d --json body -q .body > $SCRATCH/issues/$n.md; done`.
  - Where a WP says "WP-00 pre-step" for `$SCRATCH/issues/N.md`, it means this step, run by that WP itself. The WP keeps its sha256 check and its `gh` fallback.
- **Network:** none unless a WP declares it. No WP runs a network pre-step for another WP. Declared uses:
  - WP-01: one `cargo fetch` (crates.io source);
  - WP-21: its own single pnpm registry step (lockfile + store), then everything offline;
  - WP-02: release metadata reads for the pins table. It downloads no archives; unknown shas stay `TBD-at-pin`;
  - any WP: the issue snapshot step and other read-only `gh` reads.
- **Paths after WP-01:**
  - `inject/` and `app/` stop existing. Every path in WP-02..WP-34 and §3 is `crates/engine/{src,tests}/…`, `crates/app/{src,tests}/…`, `crates/viewer/…` or `crates/uevr-inject/…`.
  - "Existing test helpers" means `crates/engine/src/testing.rs` (`flat3d_inject::testing::FakeRunner`, §2), not `inject/tests`.
  - `tools/docs/check.py` (WP-25a) fails when an `inject/` or `app/src` path token appears in `docs/ROADMAP.md` or `docs/code-map.md`. The one exemption is the WP-01 git-mv block between `<!-- mv-map -->` markers.
- **Frozen signatures win:** a WP row whose call differs from a §2 "Frozen signatures" or "Wave-owned pub APIs" entry is wrong. The worker uses the §2 form and does not add a second one.
- **Cargo.lock is frozen after WP-01.**
  - WP-01 declares every dependency the pass needs, including the viewer's optional deps and uevr-inject's cfg(windows) deps.
  - It runs one online `cargo fetch`, because openxr and ashpd 0.13.13 are not cached. That is a crates.io source fetch, not a release binary.
  - CI and `tools/check.sh` build with `--locked`, and check.sh fails on `git diff --exit-code Cargo.lock`.
- **Toolchain:** every workflow pins `dtolnay/rust-toolchain@1.95.0` (host rustc/clippy 1.95.0). Nothing adds a `rust-toolchain.toml`. The host has only `stable` (= 1.95.0), so a toml naming 1.95.0 would make rustup download a toolchain.
- **Locked tests (R28):**
  - The locked set is the 49 `inject/tests/*.rs` files whose line 1 starts `//! Locked`, plus the test bodies in issues #67, #76, #116, #117 and #118.
  - The other 6 moved files carry no header and are ordinary tests: `event_ring_wrap`, `launch_options_quotes`, `runs_extra_lines`, `tether_runtime`, `tether_tunnel_review` and `wrapper_run`.
  - Locked bodies are committed verbatim, test first, then the code. The only sanctioned edits to locked tests:
    - WP-00 rustfmt (owner answer 1);
    - WP-01's 7 path edits (line numbers as at 2812f64; if WP-00's rustfmt shifted a line, find it by content). Together they are 14 `-`/`+` diff lines and leave all three files rustfmt-clean:

      | File:line | Edit |
      |---|---|
      | `api_response.rs:67`, `link_stop.rs:110` | `../docs/code-map.md` → `../../docs/code-map.md` |
      | `recipe_load.rs:138`, `recipe_load.rs:146` | `..` → `../..` |
      | `api_response.rs:69` | `inject/src/api.rs` → `crates/engine/src/api.rs` |
      | `link_stop.rs:112` | `inject/src/link.rs` → `crates/engine/src/link.rs` |
      | `recipe_load.rs:150` | `inject/src/recipes.rs` → `crates/engine/src/recipes.rs` |

      `wrapper_run.rs:206-207` is not edited: it is unheadered, and its `target/wrapper-tests` already sits under `.gitignore` `target/`.
    - One format-only `rustfmt --edition 2021 <file>` commit right after a WP's verbatim issue-body commit (owner answer 1). It touches only that WP's own locked file: WP-03 #76, WP-04 #116, WP-05 #118, WP-23 #117.

  Locked files never get `sys.path` or `use` edits beyond these.
- **Build and test:**
  - Use `CARGO_TARGET_DIR=$SCRATCH/target` and `tools/check.sh`, which exists from wave 1.
  - The viewer's `capture` feature builds only where libpipewire/libspa/libclang dev headers exist (CI apt, or craft-build). On the host, check only `-p flat3d-viewer` default + `--features xr`.
  - uevr-inject on the host: `cargo check -p flat3d-uevr-inject --target x86_64-pc-windows-msvc`, with the target installed via rustup. Linking happens only in `tools/build-win.sh`.
  - shellcheck is not on the host. Scripts run it when it is present (CI, or craft-build) and fall back to `bash -n` otherwise. CI's shellcheck step is the gate.
- **Never in the pass:**
  - downloading release archives (fetch and pin tests use `file://` fixtures);
  - `pin.sh` real runs;
  - GitHub writes (section 6 is for the owner/PM);
  - editing `docs/notes/DECISIONS.md`.
- **Device work** is written as `sessions/*.toml` and run by the owner with `tools/session/session.py run` (section 5).
- **Frozen after wave 1 (only the named WP may edit):**
  - nobody:
    - `crates/engine/src/{api,cli,lib,face,watch,paths,testing}.rs` (including the `watch.rs`/`face.rs` facades);
    - the `cli/*.rs` `run` signatures and the extra cli pub fns in §2 (the named WPs fill in the bodies only);
    - `crates/viewer/src/{frame,source,spec,gpu,xr}.rs`;
    - `Cargo.*` and `check.yml`.
  - `crates/engine/src/exec.rs`: WP-10 may change only the body of `ProcessRunner::run` and add private helpers. The traits and pub names stay as WP-01 wrote them.
  - `crates/engine/src/link.rs`: WP-12 owns the bodies. The WP-01 facade lines (`pub mod wivrn;` and the re-exports) stay.
  - `crates/app/src/lib.rs`: WP-33.
  - `docs/code-map.md`: WP-25b (w2), then WP-34 (w5).
  - `docs/ROADMAP.md`: WP-25b, WP-34.
  - `docs/spiderman2/address-map.md`: WP-25b (w2), then WP-30 (w3, the generated block only).
  - `tools/ghidra/headless.sh`: WP-02 (w1, quoting only), then WP-25b (w2).
  - `tools/check.sh`: WP-02 only.
  - `packaging/pins`: WP-02 writes it in wave 1. After that nobody in the pass edits it; the owner fills the TBDs via `pin.sh` (Q0).

## 1. Audit verdict (v1 roadmap A1-A24 + critique, with fixes now owned)

| # | Finding | Verdict | Owner |
|---|---|---|---|
| A1 | `cli_link::parse_link` (cli_link.rs:50) is never called | true | WP-01 pre-wires `parse_any`/dispatch + frozen `cli::<m>::run`; WP-08 bodies |
| A2/A3/A4 | stale doc comments main.rs:1, lib.rs:5-6, injectors.rs:1 | true | WP-01 |
| A5 | wrapper.rs:131 reads `{id}.recipe`, but recipes.rs:218-221 requires `<appid>-<slug>.recipe` (bug: every real recipe plays flat) | **bug** | WP-01 `catalog::find_recipe` + `run_with` |
| A6 | snapshot `tunnel = tether!=Off`, `health/stats: None` (watch.rs:761-781) | true | WP-01 pre-wires `HealthLoop::tick`; WP-11 body |
| A7 | `uevr::apply` env is empty (uevr.rs:74-102); `prepare` passes `&[]` | true | WP-14, WP-27, WP-31 |
| A8 | WiVRn start lacks `--early-active-runtime --no-publish-service`; Flatpak dir hard-coded (wrapper.rs:227-229) | true | WP-12 (`link/wivrn.rs`), WP-31 |
| A9/A10 | code-map lists `cli_view.rs` and `rows.rs`, but neither exists | true | WP-01 rows → `planned`; WP-03, WP-28 |
| A11 | code-map:21 mermaid has 2 statements per line | true | WP-25b + check.py (WP-25a) |
| A12 | `watch::read_settings` (watch.rs:826) is the only reader; there is no writer | true | WP-01 `switches::{read_settings,write_settings}` |
| A13 | locked `decky/tests/test_run_engine.py` (#67) is missing | true | WP-21 commits it verbatim |
| A14 | viewer "Windows binary in Proton" (code-map:85, SPEC:77) | superseded: Linux PipeWire→OpenXR quads | WP-17/18/19; SPEC via OD3 |
| A15 | bare `flat3d-run` fails (Game Mode PATH) | true | WP-20 `add_wrapper_at`; legacy stays (D4) |
| A16 | ROADMAP:204 and `tools/ghidra/headless.sh:6` point at `docs/sm2-address-map.md` | true | WP-25b |
| A17 | VT has no command written down | true | WP-30 |
| A18 | test counts in docs | drop | WP-25b |
| A19 | no fmt/clippy in CI; 48 files fail fmt; clippy fails at `switches_settings.rs:19` | true | WP-00 |
| A20 | stop_3d errors on the second Stop; no `allow-debugging`; undo never saved | true | WP-12, WP-09, WP-27 |
| A21 | #127 body drifted | true | section 6 |
| A22 | untracked `app/Cargo.lock`, `inject/Cargo.lock`; iced without `smol` | true | WP-01 |
| A23 | `dispatch` wires only Undo (cli.rs:141-149); unit test cli.rs:198 asserts `list` unbuilt (not locked) | true | WP-01 replaces the unit test |
| A24 | locked `cli_json.rs` runs `start-3d/stop-3d --json` for real | rule: no fresh snapshot ⇒ `link-not-ready`, spawn nothing; check.sh exports a temp `XDG_RUNTIME_DIR` | WP-08, WP-02 |
| K1 | v1 root lists a missing `viewer` member | fixed | WP-01 creates the `crates/viewer` stub |
| K2 | v1 merged two locked runner traits | fixed | §2 exec split (two distinct traits) |
| K3 | `module_inception` (`service/service.rs`); mod.rs/x.rs mix (E0761) | fixed | facade-file form everywhere (`x.rs` + `x/`), no `mod.rs`, no `x/x.rs` |
| K4 | v1 `wrapper::run(paths)` is wrong | fixed | real locked sig wrapper.rs:119 kept; `run_with` added |
| K5 | `run_with` loses NotFound→flat (wrapper.rs:129-131) | fixed | `switches::read_settings` NotFound→Default |
| K6 | route env `Vec<(String,String)>` can't carry unset/sidecar | fixed | `RouteEnv{env,unset,sidecar}`; `ExecSpec{argv,env,unset,sidecar}` |
| K7 | self dev-dep `features=["testing"]` duplicates types | fixed | `#[doc(hidden)] pub mod testing` always compiled |
| K8 | heal reconnect has no call site | fixed | WP-01 pre-wires `HealthTick.heal`; WP-09 executes the heal |
| K9 | `unittest discover -s tools` doesn't recurse without `__init__.py` | fixed | per-folder loop (§2 CI); decky run from `decky/` |
| K10 | shellcheck missing on host; old scripts never linted | fixed | CI apt; WP-02 lints the 2 scripts; check.sh skips with notice |
| K11 | check.py "commands == USAGE" contradicts locked USAGE | fixed | one-way USAGE ⊆ table |
| K12 | no node/pnpm in CI | fixed | setup-node 22.23.4 + pnpm 10.34.9, per-step `hashFiles` guards |
| K13 | doc contradictions C1-C19, C30 and private names (AGENTS.md:12) unowned | fixed | WP-25b (C1/C3/C4 stay open: DECISIONS untouched) |
| K14 | tether/mod.rs and the production callers of new link/tether fns unowned | fixed | WP-10 owns `tether.rs`; WP-09 switches the callers |
| K15 | watch.rs:915,922 tests fake "Tether command failed"/"Could not run tether command" | keep the prefixes | WP-10 appends `: exit N; <last 4 stderr lines>` |
| K16 | tun0 read from host-side `/sys` is wrong (tun0 is on the headset) | fixed | `adb shell ip addr show tun0` + `cat /proc/net/dev` |
| K17 | `flatpak info --show-location` prints the deploy dir | fixed | strip `/<arch>/<branch>/<commit>`; `--user` then `--system` |
| K18 | locked link_stop.rs: "Access denied" stays an error | honour | WP-12 |
| K19 | pins written by two WPs | fixed | `packaging/pins` owned by WP-02 only |
| K20 | windows-gnu can't link here | fixed | msvc + xwin + rust-lld in `tools/build-win.sh` |
| K21 | no WP built/installed the injector exe | fixed | WP-15 `--install` |
| K22 | geo11/wivrn-apk sha unknown | fixed | TOFU `pin.sh` (OD8); `fetch.rs` refuses `TBD-at-pin`; wivrn-apk sha recorded by the pins research |
| K23 | viewer xr/capture can't compile in CI without headers | fixed | apt in CI; split WP-17 (pure) / WP-18 (capture) / WP-19 (xr); shared `frame.rs` frozen by WP-01 |
| K24 | locked install_plan.rs:14-16,48 fixes `unit_text` | honour | `unit_text_with` new |
| K25 | SM2 appid is 2651280; "check with list" is circular | fixed | WP-26 seeds use 2651280 |
| K26 | `InjectResult.launched` undefined | fixed | `false` from CLI `inject` (it prepares, never launches) |
| K27 | #117 needs only `rows::GameRow` | fixed | games page in wave 2 (WP-23) |
| K28 | rows.rs fixes the `flat_reason` texts | honour | not unified (Not-in-pass) |
| K29 | `cli_undo.rs` gap; #37 HEROVR column | fixed | WP-27; WP-30 |
| K30 | Grace→Ready, `launch_options_before` | owner | OD4, OD5 |
| K31 | wave-parallel WPs disagreed on wivrn/HealthLoop/ensure_applied/EnginePaths/cli/viewer signatures | fixed | §2 "Frozen signatures" (WP-01) |
| K32 | bones, WP-01 and the implementing WPs disagreed on the wivrn path and types, HealthLoop, ensure_applied, viewer frame, from_env, Ctx, install and FakeRunner | fixed | §2 reconciled: the implementing WP's form for wivrn (WP-12), HealthLoop (WP-11), ensure_applied (WP-27), install (WP-20), wrapper (WP-31) and frame (WP-17/18/19); bones' form for from_env and Ctx |

## 2. New bones

**Tree** (`*` new; `→` git mv). Facade-file form only: a module with children is `x.rs` + `x/`. No `mod.rs` anywhere, so clippy `mod_module_files`/`module_inception` stay clean and there is no E0761.
```
Cargo.toml* Cargo.lock*  .gitignore  tools/{check.sh*,build-win.sh*,docs/*,pm/*,session/*,stereo/*,re/*,ghidra/*,xmatch/,research/}
crates/engine/ (pkg flat3d-inject) ← inject/
  src/lib.rs main.rs bin/flat3d-run.rs api.rs cli.rs cli/{link←cli_link,engine*,inject*,undo*,view*,setup*,setup_fetch*,doctor*}.rs
      exec.rs* testing.rs* paths.rs* fsx.rs* sys.rs* catalog.rs*
      face.rs* face/{collection,headset_view,link_view,toasts,view}.rs ← src/*.rs, face/{rows*,link_model*,logs*}.rs
      injectors.rs injectors/{uevr,geo11*,port*,fetch*}.rs
      watch.rs(facade*) watch/{service←watch.rs,frames*,machine*,files*,run*,health_loop*}.rs
      tether.rs←tether/mod.rs tether/{adb,heal,plan,relay,stepaside,wifi}.rs
      link.rs link/wivrn.rs*  wrapper.rs install.rs launch_options.rs undo.rs setup.rs state.rs switches.rs recipes.rs steam.rs
      health.rs stats.rs status.rs runs.rs
  tests/*.rs (55 moved: 49 locked by header + 6 unheadered: event_ring_wrap, launch_options_quotes, runs_extra_lines, tether_runtime, tether_tunnel_review, wrapper_run)
  tests/{paths,fsx,catalog_find,switches_io,state_io,route_settings,cli_any}.rs* tests/fixtures/*
crates/app/ (pkg flat3d-app) ← app/   src/{lib,main,engine*,headset_page*,games_page*,setup_page*,logs_page*,view_{headset,games,setup,logs}*}.rs tests/*
crates/viewer/* (pkg flat3d-viewer) src/{lib,frame,spec,layout,status,source,main}.rs source/{test_pattern,pipewire,gamescope,portal}.rs gpu.rs gpu/import.rs xr.rs xr/{session,swapchain,frame}.rs tests/spec.rs
crates/uevr-inject/* (pkg flat3d-uevr-inject, lib flat3d_uevr_inject) src/{lib,main,args,pick,win}.rs
decky/{main.py,plugin.json,tests/*} packaging/{pins*,pin.sh*,build-*.sh*,AppRun*,flat3d.desktop*,flat3d-link.service*,tests/*} sessions/*.toml* games/spiderman2/map.toml*
```
**Root Cargo.toml:** `[workspace] resolver="2" members=["crates/*"] default-members=["crates/engine","crates/viewer","crates/uevr-inject"]`. Then:
- `[workspace.package] edition="2021" license="GPL-3.0-only"`;
- `[workspace.dependencies]` serde(derive), serde_json, `iced={version="0.14.0",features=["smol"]}`, `flat3d-inject={path="crates/engine"}`, `flat3d-viewer={path="crates/viewer",default-features=false}`, `windows-sys="=0.61.2"`;
- `[workspace.lints.clippy] field_reassign_with_default="allow"`; each crate sets `[lints] workspace=true`.

Viewer:
- `[features] default=[]`, `xr=["dep:openxr","dep:ash"]`, `capture=["dep:pipewire","dep:ashpd","dep:tokio"]`.
- `openxr={version="=0.22.0",features=["loaded"]}`, `ash="=0.38.0"`, `pipewire="=0.10.1"`, `ashpd={version="=0.13.13",default-features=false,features=["screencast","tokio"]}`, tokio (features `rt`, `time`; optional), libc (not optional).
- `[[bin]] name="flat3d-viewer" required-features=["xr","capture"]`.

uevr-inject:
- windows-sys goes under `[target.'cfg(windows)'.dependencies]` with the features `Win32_System_Diagnostics_ToolHelp`, `Win32_System_Memory`, `Win32_System_Threading`, `Win32_System_LibraryLoader`, `Win32_Foundation`, `Win32_System_Diagnostics_Debug` and `Win32_Security`. `Win32_Security` is there because `CreateRemoteThread` is gated on it and takes `*const SECURITY_ATTRIBUTES`.
- `src/lib.rs` (WP-01) is `pub mod args; pub mod pick; #[cfg(windows)] pub mod win;`.
- WP-01 stubs are an empty `args.rs`/`pick.rs`, an empty `win.rs`, and a `main.rs` that exits 3 on non-Windows. WP-15 overwrites all four bodies.
- The final exit codes are WP-15's: 2 bad args, 3 not Windows, 4 no game found, 5 inject failure.

**git-mv map** (`<!-- mv-map -->` block, the one check.py exemption):
- `inject/**`→`crates/engine/**` and `app/**`→`crates/app/**`;
- `cli_link.rs`→`cli/link.rs`;
- `{collection,headset_view,link_view,toasts,view}.rs`→`face/`;
- `tether/mod.rs`→`tether.rs`;
- `watch.rs`→`watch/service.rs`, then frames/machine/files/run are split out; the new `watch.rs` is facade-only.

`cli/undo.rs` is new. Today's working undo dispatch body moves into it, so it is not a git mv.

Untracked `inject/Cargo.lock`, `app/Cargo.lock` and per-crate `target/` are deleted.

**Facade re-exports (explicit, never glob):**
- `lib.rs`:
  - `pub mod` for every top-level module, including `pub mod face;`, `pub mod paths;`, `pub mod exec;`, `pub mod catalog;`, `pub mod fsx;`, `pub mod sys;` and `#[doc(hidden)] pub mod testing;`.
  - `pub use cli::link as cli_link; pub use face::{collection, headset_view, link_view, rows, toasts, view}; pub use link::wivrn;`. The last one makes `crate::wivrn::…` and `crate::link::wivrn::…` the same module.
  - `NotBuilt` keeps its current crate-root definition (lib.rs:39-47, `From<NotBuilt> for ApiError` api.rs:103). Locked `api_response.rs:5,27` imports `flat3d_inject::NotBuilt`.
- `face.rs`: `pub mod collection; pub mod headset_view; pub mod link_view; pub mod toasts; pub mod view; pub mod rows; pub mod link_model; pub mod logs;`. The app uses `flat3d_inject::face::logs::is_error_line`.
- `watch.rs` re-exports exactly the list below. Locked tests need 13 of these names: `step, Effect, Event, Inputs, Note, Phase, Watch, GRACE_SECS, parse_track_frames, Device, link_commands, note_kind, note_text` (watch_manual.rs:4-6, watch_frames.rs:3, watch_grace.rs:3).
  - `pub use frames::{parse_track_frames, Device};`
  - `pub use machine::{step, link_commands, note_text, note_kind, note_event, Effect, Event, Inputs, Note, Phase, Watch, GRACE_SECS};`. `note_event` stays a `pub fn` in machine.rs (today it is pub, watch.rs:452). `Service` calls it. WP-09 may not make it private or shadow it with a method of the same name.
  - `pub use files::{write_snapshot, Paths};`
  - `pub use service::{Feed, Service, TrackDevices};`
  - `pub use run::run; pub use health_loop::{HealthLoop, HealthTick};`
  - `files` is not named `paths`, so it doesn't clash with crate-level `paths::EnginePaths`.
- `link.rs`: `pub mod wivrn; pub use crate::exec::{CommandOutput, CommandRunner, SystemRunner}; pub use wivrn::{FlatpakScope, QuestApp, WivrnDirs, WivrnStart, QUEST_APPS, SERVER_FLAGS, detect_quest_app, detect_server};`. `WIVRN_APP` (link.rs:10) stays where it is.
- `tether/adb.rs`: `pub use crate::exec::{ProcessRunner, RelayRunner, RelayRunner as CommandRunner};`
- Viewer `lib.rs`: `pub mod frame; pub mod spec; pub mod layout; pub mod status; pub mod source; #[cfg(feature="xr")] pub mod xr; #[cfg(feature="xr")] pub mod gpu;`. `gpu.rs` is `pub mod import;`. `xr.rs` is `pub mod session; pub mod swapchain; pub mod frame;`.
- App `crates/app/src/lib.rs`: WP-01 adds these lines above the existing `Tab` code: `pub mod engine; pub mod headset_page; pub mod games_page; pub mod setup_page; pub mod logs_page; pub mod view_headset; pub mod view_games; pub mod view_setup; pub mod view_logs;`.
  - Locked imports: `headset_page::{update, Action, HeadsetState, Msg}`, `flat3d_inject::rows::GameRow`, `api::{AdbSource,LinkSnapshot,LinkState}`, `setup::Packages`.
  - Page stubs carry the types exactly as in #116/#117/#118:
    - `headset_page::update` returns `None` and is not `#[must_use]`;
    - `setup_page::Step` derives Debug, Clone, PartialEq and Eq; `steps`→`Vec::new()` and `all_done`→`false`;
    - `GamesState::default()` has filter `"all"`;
    - `logs_page` has no limit constant of its own (see `face` below).

**Glob-import guard:** locked tests glob `install::*` (install_plan.rs:3), `api::*` (api_snapshot.rs:3) and `tether::relay::*` (tether_tunnel.rs:3, tether_relay_cmds.rs:3, tether_tunnel_review.rs:3).
- No new pub item in `api`, `install` or `tether::relay` may be named `json`, `c`, `p`, `H` or `full`. These are the local or explicit names those tests use: install_plan.rs:5-11, api_snapshot.rs:4,6.
- No new glob re-export goes into those three modules. The existing items stay as they are.

**Exec trait split (`exec.rs`).** Two distinct traits stay re-exported under their locked names:
- Link shape: `CommandRunner { fn run(&mut self, argv:&[String]) -> CommandOutput }`, plus `CommandOutput` and `SystemRunner`. Re-exported as `link::{CommandOutput, CommandRunner}` (tether_adb_runner.rs:3).
- Relay shape: `RelayRunner { fn run(&mut self,&[String])->Result<String,String>; fn start_relay(&mut self,&[String],&str)->Result<(),String>; fn stop_relay(&mut self)->Result<(),String> }` (verbatim from adb.rs:64-68), plus `ProcessRunner`. Re-exported as `tether::adb::CommandRunner`. `tether_runtime.rs:2` imports it under that name; the file is unheadered, and the import is kept for compile, not because the file is locked.
- `AdbRunner<R: link::CommandRunner>` keeps its bound and constructor (adb.rs:171-191) and still implements `link::CommandRunner`. The alias in `tether::adb` does not touch `AdbRunner`. `TetherAdb<R: RelayRunner>` (adb.rs:134) is unchanged too.
- No blanket impl. `testing::FakeRunner` implements both traits; its API is frozen below.
- Every new fn names its trait: `&mut dyn CommandRunner` (link/wivrn/doctor/fetch/setup/injectors) or `&mut dyn RelayRunner` (tether/health).
- WP-01's compile check includes `cargo test -p flat3d-inject --test tether_adb_runner --test tether_runtime`.

**Legacy `*_with` rule (D4):** locked fns keep their bodies; production calls the new ones. WP-01 writes the stubs; the full forms are under "Frozen signatures" (wrapper, link, install) and "Wave-owned pub APIs" (uevr, launch_options).
- `wrapper::run_with` keeps the locked `run` shape (wrapper.rs:119). It uses:
  - `catalog::find_recipe` (`{id}.recipe` or `{id}-*.recipe`; the first dir wins);
  - `switches::read_settings` (NotFound→Default);
  - `injectors::ensure_applied` (stub Err → flat). Before that call it looks up `steam::Game` and `recipes::route_settings(&text)` (WP-31 body); a game that is not installed plays flat.
- Also new:
  - `prepare_with_route` and `exec_spec_with`;
  - `link::start_3d_commands_with(s:&WivrnStart)->Vec<Vec<String>>`;
  - `launch_options::add_wrapper_at(options:&str, wrapper_abs:&str)->String`, whose stub delegates to `add_wrapper` (launch_options.rs:77);
  - `uevr::profile_text_full` (WP-14 adds it in uevr.rs);
  - `install::unit_text_with(&UnitSpec)->String`.
- `RouteEnv{env:Vec<(String,String)>,unset:Vec<String>,sidecar:Option<Vec<String>>}` lives in `injectors`. `ExecSpec` (wrapper.rs:213) gains `unset` and `sidecar`.
- Follow-up issue: delete the legacy fns once the locked tests are re-cut.

**Bones WP-01 freezes.** Stub bodies return `ApiError{code:"not-built"}`/`Err`/neutral values. Parameters take a `_` prefix. No `todo!()` or panics.
- `cli::parse_any`→`AnyCli{command:AnyCommand{Engine,Link,View,Setup,Doctor},json}` + dispatch.
- `api` DTOs (fields under Frozen signatures), `TOAST_TITLE="Flat to 3D"` and the `GameRow` re-export; Serialize on `HealthView` (face/view.rs) and `ToastBatch` (face/toasts.rs).
- `state::{read_snapshot,write_request}` and `switches`, full. `catalog::find_recipe`, full: a dir that does not exist is skipped, and the result is `Ok(None)` when no dir has a match.
- `recipes::{UEVR_KEYS,GEO11_KEYS,PORT_KEYS,RouteSettings,route_settings,check_route_settings}`, full (rules below).
- `paths.rs`, full.
- `link::wivrn::WivrnDirs::user_default`, full.
- `testing::FakeRunner`, full.
- `viewer::spec` (ViewerSpec, `to_args`/`from_args`), full.
- `viewer::frame`, full; `viewer::source::FrameSource`, full (re-exports `frame::Frame`).

**Frozen signatures.** WP-01 writes them. Later WPs fill in bodies only and call these forms verbatim; no WP adds a second form.
- **`paths.rs`** is written in full by WP-01 and has no owner afterwards.
  - Struct: `pub struct EnginePaths{pub home:PathBuf, pub uid:u32, pub state_dir:PathBuf, pub config:PathBuf, pub data:PathBuf, pub cache_dir:PathBuf, pub log_dir:PathBuf, pub bin:PathBuf, pub recipe_dirs:Vec<PathBuf>}`.
  - Constructor: `EnginePaths::from_env(env:&BTreeMap<String,String>, uid:u32)->Result<EnginePaths,ApiError>`. A missing `HOME` gives `not-found`. Nobody passes a closure. Production callers (WP-31 `flat3d-run.rs`, WP-33 `engine.rs`, `main.rs`) use `from_env(&std::env::vars().collect(), sys::current_uid())`, or their own env map.
  - Accessors, all `->PathBuf` unless noted:

    | Accessor | Returns |
    |---|---|
    | `settings()` | `home/.config/flat3d/settings` |
    | `link_log()` | `log_dir/link.log` |
    | `run_log()` | `log_dir/run.log` |
    | `cache()` | `$XDG_CACHE_HOME/flat3d`, else `home/.cache/flat3d` (equals `cache_dir`) |
    | `uevr_cache()` | `cache()/uevr` (WP-13's `cache_dir` rule for the key `uevr`) |
    | `env_file(appid:u32)` | `data/env/{appid}.env` |
    | `wrapper_bin()` | `bin/flat3d-run` (absolute) |
    | `viewer_bin()` | `bin/flat3d-viewer` (absolute) |
    | `pins()` | `data/pins` |
    | `run_paths()` | `->RunPaths` |
    | `watch_paths()` | `->watch::Paths` |

    `log_dir` is `home/.local/state/flat3d` (watch.rs:323, flat3d-run.rs:34).
  - Locked `state::*_path(&str)` callers pass `p.state_dir.to_string_lossy()`.
  - Each accessor has a case in `crates/engine/tests/paths.rs`.
- **`fsx.rs` / `sys.rs`** (full): `fsx::{write_atomic(&Path,&[u8])->io::Result<()>, append_line, read_opt(&Path)->io::Result<Option<String>>}`, `sys::{current_uid()->u32, unix_now()->u64}`.
- **`link/wivrn.rs`** (WP-12 body; `crate::wivrn` is the same module):
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FlatpakScope{User,System}` + `pub fn flag(self)->&'static str` (`--user`/`--system`);
  - `pub const SERVER_FLAGS:[&str;2]=["--early-active-runtime","--no-publish-service"]` and `pub const QUEST_APPS:[&str;5]`;
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct WivrnDirs{pub scope:FlatpakScope, pub app_dir:PathBuf, pub deploy:PathBuf, pub version:Option<String>}` + `pub fn user_default(home:&Path)->WivrnDirs`. It is full in WP-01: scope User, app_dir `home/.local/share/flatpak/app/io.github.wivrn.wivrn`, deploy `app_dir/current/active`, version None;
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct QuestApp{pub package:String, pub version:Option<String>}`. It has no tuple form and no `label`;
  - `pub struct WivrnStart{pub scope:Option<FlatpakScope>, pub app:String, pub flags:Vec<String>}` with `impl Default`: scope None, app `link::WIVRN_APP`, flags `SERVER_FLAGS`;
  - `app_dir_from_location(out:&str)->Option<PathBuf>`, `version_from_info(out:&str)->Option<String>`, `version_name(dumpsys:&str)->Option<String>`, `versions_match(server:&str, app:&str)->bool`;
  - `detect_server(r:&mut dyn CommandRunner)->Result<WivrnDirs,String>` and `detect_quest_app(r:&mut dyn CommandRunner, serial:&str)->Option<QuestApp>`.
  - Callers:
    - WP-09: `detect_quest_app(r,&serial)`, then `.package`;
    - WP-10: `Tether::set_wivrn(app,flags)`, see Wave-owned pub APIs;
    - WP-29: `detect_server`, `.version`;
    - WP-31: `detect_server(r).map_err(|e| format!("wivrn: {e}"))`.
  - The retired names `WIVRN_SERVER_FLAGS`, `QUEST_APP_CANDIDATES`, `WivrnDirs{scope,root}` and `detect_dirs` are not used.
- **`link.rs`**: `start_3d_commands_with(s:&WivrnStart)->Vec<Vec<String>>`, which has the same shape as the locked `start_3d_commands`. The stub delegates to `start_3d_commands()`; WP-12 writes the body.
- **`watch/health_loop.rs`** (WP-11 body; WP-09 calls it):
  - `HealthLoop::new(start_unix:u64, mem_start_kb:u64, soak_file:Option<PathBuf>)->Self`;
  - `tick(&mut self, now:u64, mode:TetherMode, playing:bool, r:&mut dyn RelayRunner)->HealthTick`;
  - `pub struct HealthTick{pub health:Option<api::Health>, pub stats:Option<api::Stats>, pub tunnel:bool, pub heal:Option<tether::heal::Heal>, pub toast:Option<(api::EventKind,String)>}`.
  - The loop appends its own soak lines to `soak_file`. Nobody else writes that file.
  - WP-11 may add `reconnects(&self)->u32` and `finish(self, now:u64, mem_end_kb:u64, orphans:u32)->SoakSummary` in its own file.
  - There is no `take_toast`, no `pub playing`, no `HealthAction` and no `WriteSoak`.
  - WP-01's `publish()` (watch.rs:784) calls `tick(sys::unix_now(), …)`, pushes `toast` to the ring, and keeps `heal` for WP-09, which executes it. Service builds `HealthLoop::new(sys::unix_now(), 0, None)`.
- **`injectors.rs`** (WP-27 body):
  - `ensure_applied(game:&steam::Game, recipe:&Recipe, settings:&RouteSettings, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<RouteEnv,String>` (stub Err);
  - `#[derive(Debug, Clone, PartialEq, Eq, Default)] pub struct RouteEnv{pub env:Vec<(String,String)>, pub unset:Vec<String>, pub sidecar:Option<Vec<String>>}`.
  - WP-27 and WP-31 call this form. There is no `injectors::Applied`. `uevr::Applied` (uevr.rs:60) is unrelated and stays.
- **`wrapper.rs`** (WP-31 body):
  - `pub struct ExecSpec{pub argv:Vec<String>, pub env:Vec<(String,String)>, pub unset:Vec<String>, pub sidecar:Option<Vec<String>>}`. Legacy `exec_spec` sets `vec![]`/`None`;
  - `exec_spec_with(plan:&Plan, argv:&[String], route:&RouteEnv, wivrn:&WivrnDirs, existing_fs_rw:Option<&str>)->ExecSpec`;
  - `prepare_with_route(argv:&[String], env:&BTreeMap<String,String>, snapshot:Result<Option<&LinkSnapshot>,String>, settings:Result<&Settings,String>, recipe:Result<Option<&Recipe>,String>, route:&mut dyn FnMut(u32,&Recipe)->Result<(RouteEnv,WivrnDirs),String>, pid:u32, now:u64)->Launch`;
  - `run_with<T>(argv:&[String], env:&BTreeMap<String,String>, paths:&EnginePaths, pid:u32, now:u64, runner:&mut dyn CommandRunner, launcher:impl FnOnce(&Launch)->io::Result<T>)->(Launch, io::Result<T>)`.
  - WP-01's route closure stub returns `Err("not-built: route (WP-31)")`. A5 is still fixed through `find_recipe`.
- **`install.rs`** (WP-20 body):
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct UnitSpec{pub description:String, pub exec_start:String, pub restart_sec:u32, pub start_limit_interval_sec:u32, pub start_limit_burst:u32, pub wanted_by:String}` with `impl Default` giving `"Flat to 3D headset link"`, `"%h/.local/bin/flat3d-inject watch"`, 3, 60, 10, `"default.target"`;
  - `unit_text_with(spec:&UnitSpec)->String`. The stub delegates to `unit_text(&spec.exec_start)` (install.rs:5);
  - `execute(plan:&InstallPlan, r:&mut dyn CommandRunner)->Result<api::InstallReport,ApiError>` and `execute_uninstall(plan:&UninstallPlan, r:&mut dyn CommandRunner)->Result<api::InstallReport,ApiError>`.
  - There is no second report type: `api::InstallReport{written,ran,summary}` (api.rs:255) is the only one. WP-20 may add fields to `InstallPlan`/`UninstallPlan` in its own file, because the locked tests only read fields.
- **`recipes.rs`** (full in WP-01):
  - `pub type RouteSettings = BTreeMap<String,String>;`
  - `UEVR_KEYS:[&str;6]=["uevr-exe","uevr-rendering","uevr-2d-screen","uevr-native-stereo-fix","uevr-delay","uevr-profile"]`; `GEO11_KEYS:[&str;7]=["geo11-fix","geo11-exe","geo11-layout","geo11-separation","geo11-convergence","geo11-d3dx","geo11-remove-first"]`; `PORT_KEYS:[&str;0]`.
  - `route_settings(text:&str)->RouteSettings` reads the recipe text:
    - each line splits at the first `:`, and both sides are trimmed;
    - one pair of surrounding quotes is stripped;
    - `#` lines and blank lines are skipped;
    - only `uevr-*`/`geo11-*`/`port-*` keys are kept; the 9 recipe KEYS are never returned.
  - `check_route_settings(route:&str, m:&RouteSettings)->Result<(),String>`:
    - `route` is the raw `stereo-route` text (`Recipe.route`), mapped with `injectors::route_of`;
    - Port and no-route accept only an empty map, and a key outside the route's list is refused;
    - value rules:

      | Key | Rule |
      |---|---|
      | `uevr-exe` | non-empty (required) |
      | `uevr-rendering` | `native` or `synced` |
      | `uevr-2d-screen`, `uevr-native-stereo-fix` | `yes` or `no` |
      | `uevr-delay` | u32 in `0..=120` |
      | `uevr-profile` | a pin key `[a-z0-9-]+` |
      | `geo11-fix` | a pin key `[a-z0-9-]+` (required) |
      | `geo11-exe` | non-empty (required) |
      | `geo11-layout` | `sbs`, `tab`, `sbs_reversed` or `tab_reversed` |
      | `geo11-separation` | `0..=100` |
      | `geo11-convergence` | f32 > 0 |
      | `geo11-d3dx` | `k=v;k=v` |
      | `geo11-remove-first` | `;`-separated relative paths |
- **`cli.rs`**: `pub struct Ctx<'a>{pub paths:&'a EnginePaths, pub runner:&'a mut dyn CommandRunner, pub relay:&'a mut dyn RelayRunner, pub json:bool}`. `now` is `sys::unix_now()` at the call site and is never a Ctx field. `main` builds a `SystemRunner` and a `ProcessRunner`.
  - `pub struct AnyCli{pub command:AnyCommand, pub json:bool}`; `pub enum AnyCommand{Engine(Command), Link(LinkCommand), View(ViewCommand), Setup(SetupCommand), Doctor(DoctorCommand)}`.
  - `pub fn parse_any(args:&[String])->Result<AnyCli,ApiError>`:
    - it splits at the first argv element exactly equal to `--` and removes `--json` from the head only;
    - it then tries `cli::link::parse_link` → `cli::view::parse` → setup → doctor → the locked `parse`;
    - usage errors have code `usage` and keep the locked text.
  - The parsed enums:
    - `ViewCommand{View, Rows{filter:String}, Launch{add:bool, options:String}, Collection{current:Option<Vec<u32>>}, Toasts{last_seq:u64}, Logs{limit:usize}}`. `Logs.limit` defaults to `headset_view::DEFAULT_LOG_LINES` when `--limit` is absent;
    - `SetupCommand{Packages, InstallApps, LaunchEnv{appid:Option<u32>}, Fetch{key:String}}`. `launch-env abc` and a `fetch` without a key give a usage error;
    - `DoctorCommand{Run}`.
    - There is no `InjectCommand`.
  - Each module has exactly one `run` (cli.rs:13 for `cli::Command`):

    | Module | Signature |
    |---|---|
    | `cli/engine.rs`, `cli/inject.rs`, `cli/undo.rs` | `pub fn run(cmd:&cli::Command, ctx:&mut Ctx<'_>)->Result<serde_json::Value,ApiError>` |
    | `cli/link.rs` | `run(cmd:&LinkCommand, ctx)`, same return |
    | `cli/view.rs` | `run(cmd:&ViewCommand, ctx)`, same return |
    | `cli/setup.rs` | `run(cmd:&SetupCommand, ctx)`, same return |
    | `cli/doctor.rs` | `run(cmd:&DoctorCommand, ctx)`, same return |

    The stub body is `Err(NotBuilt("cli::<m>::run").into())`, the same as today's dispatch fallback (cli.rs:147). `cli/undo.rs` gets today's working undo body.
  - The only other pub items under `cli/`:
    - `cli::link::request_3d(r:state::Request, p:&EnginePaths, now:u64)->Result<api::LinkReport,ApiError>`. Dispatch sends `Command::Start3d/Stop3d` here with `sys::unix_now()` and serialises the result with `serde_json::to_value`;
    - `cli::view::parse(args:&[String])->Option<Result<ViewCommand,String>>`. None means "not a view verb". The stub accepts at least `view`, and WP-28 fills the grammar;
    - `cli::setup_fetch::setup_fetch(key:&str, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::SetupReport,ApiError>` (WP-27 body; WP-29's `setup fetch` calls it);
    - `parse_link`, `LinkCommand` and `SettingKey` stay pub, because they are locked.
  - WP-07/08/27/28/29 implement only these bodies. Helpers stay private inside their own file.
- **`api.rs`** DTOs:
  - Shapes:
    - `LinkView{state:LinkState, label:String, hint:String, start:bool, stop:bool, health:HealthView, rx:String, tx:String, uptime:String, tether:TetherMode, tunnel:bool, auto_3d:bool, adb_source:AdbSource}`;
    - `LaunchText{options:String}`;
    - `CollectionPlan{name:String, wanted:Vec<u32>, add:Vec<u32>, remove:Vec<u32>}`;
    - `LogLines{lines:Vec<String>, text:String}`;
    - `DoctorCheck{name:String, ok:bool, detail:String, fix:String}`, `DoctorReport{ok:bool, checks:Vec<DoctorCheck>}`;
    - `SetupReport{steps:Vec<String>, summary:String}`;
    - `Toast{title:String, body:String}`.
  - Every field is `pub`. Every DTO derives `Debug, Clone, PartialEq, Serialize, Deserialize`, except `LinkView`, which derives `Debug, Clone, PartialEq, Serialize` only.
  - Existing `InstallReport` (api.rs:255), `LinkReport` (api.rs:56), `Health`, `Stats` and `EventKind` are unchanged.
- **`face`**:
  - `rows::{GameRow, game_rows(games:&[api::Game], filter:&str, settings:&switches::Settings, link:LinkState)->Vec<GameRow>}`. Both are exactly as in issue #76 (WP-03 body);
  - `link_model::link_view(snap:Option<LinkSnapshot>, now:u64)->api::LinkView`. The stub returns a Stopped view; WP-28 writes the body;
  - `logs::{log_lines(text:&str, max:usize)->Vec<String>, read_logs(p:&EnginePaths, max:usize)->Vec<String>, is_error_line(line:&str)->bool}`. The stubs return `Vec::new()`, `Vec::new()` and `false`. WP-06 writes the bodies and may add private `ERROR_WORDS`;
  - The single log limit is `headset_view::DEFAULT_LOG_LINES` (500, headset_view.rs:21). WP-28 and WP-33 pass it, and there is no `LOG_LIMIT`.
- **`testing.rs`** (`#[doc(hidden)] pub mod testing`, path `flat3d_inject::testing::FakeRunner`, full):
  - `pub struct FakeRunner{pub ran:Vec<Vec<String>>, ..}`. `FakeRunner::new()` treats unscripted argv as success with empty stdout/stderr.
  - Builders: `reply(self, prefix:&[&str], out:CommandOutput)->Self`, `ok(self, prefix:&[&str], stdout:&str)->Self` and `fail(self, prefix:&[&str], stderr:&str)->Self`.
  - Replies form a queue: the first unconsumed entry whose prefix matches the argv is consumed.
  - It implements link `CommandRunner`, which returns that `CommandOutput`.
  - It implements `RelayRunner`: `Ok(stdout)` on success, else `Err(stderr)`. `start_relay` records `["relay", argv.., name]` and `stop_relay` records `["relay","stop"]`; both return Ok.
  - Every call pushes its argv to `ran`.
  - A WP may still use a local fake.
- **`crates/viewer/src/frame.rs`** (full; WP-17/18/19 use it unchanged):
  - `pub struct Plane{pub fd:OwnedFd, pub offset:u32, pub stride:u32}`;
  - `pub enum FrameData{Mem{stride:u32, bytes:Vec<u8>}, DmaBuf{planes:Vec<Plane>, modifier:u64}}`;
  - `pub struct Frame{pub width:u32, pub height:u32, pub fourcc:u32, pub seq:u64, pub data:FrameData}`, with Debug and no Clone.
  - There is no `PixelFormat` and no `Cpu` variant. `fourcc` is a DRM fourcc, and NV12 uses two planes.
- **`crates/viewer/src/source.rs`** (full): `pub use crate::frame::Frame; pub trait FrameSource { fn next(&mut self, timeout: std::time::Duration) -> Result<Option<Frame>, String>; }`, then `pub mod test_pattern; #[cfg(feature="capture")] pub mod pipewire; #[cfg(feature="capture")] pub mod gamescope; #[cfg(feature="capture")] pub mod portal;`.
- **`crates/viewer/src/spec.rs`** (full):
  - `#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] pub struct ViewerSpec{pub layout:Layout, pub source:Source, pub swap_eyes:bool, pub crop_aspect:Option<f32>, pub screen_width_m:f32, pub distance_m:f32}`;
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(rename_all="snake_case")] pub enum Layout{#[default] Sbs, Tab, #[serde(rename="sbs_reversed")] SbsRev, #[serde(rename="tab_reversed")] TabRev}`;
  - `pub enum Source{#[default] Auto, Gamescope, Portal, TestPattern}`, with the same derives and snake_case (`test_pattern`);
  - `pub fn from_args(args:&[String])->Result<ViewerSpec,String>` and `pub fn to_args(&self)->Vec<String>`:
    - `to_args` writes `--layout <l> --source <s>`, then `--swap-eyes` when true and `--crop-aspect <f>` when Some;
    - `--screen-width-m <f>` and `--distance-m <f>` are written only when > 0.0;
    - `from_args` returns Err on an unknown flag.
  - 0.0 means the viewer default, which WP-19 resolves to 3.0 m / 2.5 m.
  - `tests/spec.rs` round-trips every Layout and Source, including `TabRev`↔`"tab_reversed"` and `TestPattern`↔`"test_pattern"`.

**Wave-owned pub APIs.** The owning WP adds these inside its own file after wave 1. Later WPs call exactly these forms.
- WP-10 (`tether.rs`, `tether/plan.rs`, `tether/wifi.rs`):
  - `run_plan(steps:&[TetherStep], r:&mut dyn RelayRunner, sleep:&mut dyn FnMut(u64))->Result<(),String>`;
  - `run_plan_in(steps, ctx:&PlanCtx, r, sleep, out:&mut PlanRun)->Result<(),String>`, with `PlanCtx{pub relay:Vec<String>}` and `PlanRun{pub ran:Vec<Vec<String>>, pub tun0:bool}`;
  - `Tether::set_sleep(&mut self, Box<dyn FnMut(u64)>)`;
  - `Tether::set_wivrn(&mut self, app:String, flags:Vec<String>)`;
  - `Tether::sync_wifi(&mut self)->wifi::SyncDecision` and `Tether::last_wifi(&self)->Option<&wifi::SyncDecision>`;
  - `wifi::connect_argv(ssid, sec, psk)->Vec<String>`.
  - WP-09 calls these.
- WP-12 (`link.rs`): `start_3d_with(runner:&mut dyn CommandRunner, s:&WivrnStart)->Result<LinkState,LinkError>`.
- WP-13 (`injectors/fetch.rs`):
  - `pin(key:&str)->Result<Pin,FetchError>`;
  - `cache_dir(pin:&Pin, p:&EnginePaths)->PathBuf`, which is `p.cache()/<key>/<version>`;
  - `fetch(pin:&Pin, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<PathBuf,FetchError>`;
  - `impl From<FetchError> for ApiError`. This is the only fetch→ApiError mapping:

    | FetchError | ApiError code |
    |---|---|
    | UnknownKey | `not-found` |
    | TbdAtPin, Manual, BadPin, Scheme | `refused` |
    | Command, SizeMismatch, ShaMismatch | `command-failed` |
    | Io | `io` |

    WP-27 uses this mapping; inside `ensure_applied` (String errors) it uses `e.to_string()`.
- WP-14 (`injectors/uevr.rs`): `apply_with(game:&steam::Game, settings:&RouteSettings, p:&EnginePaths)->Result<(uevr::Applied,RouteEnv),String>`, `profile_text_full(&UevrSettings,&UevrExtras)->String` and `uevr_dir(p)->PathBuf` (`p.uevr_cache()/UEVR_VERSION`).
- WP-16 (`injectors/geo11.rs`):
  - `apply(cache_dir:&Path, game_dir:&Path, appid:u32, m:&RouteSettings, viewer_exe:&Path)->Result<(geo11::Applied,RouteEnv),String>`. Its `RouteEnv.sidecar` comes from `ViewerSpec::to_args`;
  - `viewer_check(viewer_exe:&Path)->Result<(),ApiError>`, which returns code `not-found` unless the path is an executable file;
  - callers pass `p.viewer_bin()`.
- WP-20 (`launch_options.rs`): `add_wrapper_at(options:&str, wrapper_abs:&str)->String`, called with `p.wrapper_bin()`.

`ERROR_CODES` stays the 8 locked codes (api_response.rs:10: `usage not-built not-found bad-recipe refused link-not-ready command-failed io`). No WP adds a code.

**CI final shape (`check.yml`, WP-01).** Every job uses `runs-on: [self-hosted, linux]` (check.yml:18,36) and `dtolnay/rust-toolchain@1.95.0` (host rustc 1.95.0). WP-22's `appimage.yml` uses the same runner and pin. Neither file uses `ubuntu-24.04`, `@stable` or `@master`.
- Job `rust`:
  - apt via the runner's `SUDO=$(command -v sudo || true)` pattern (check.yml:22): `build-essential curl ca-certificates pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev libpipewire-0.3-dev libspa-0.2-dev libclang-dev clang shellcheck libarchive-tools`. `libarchive-tools` provides bsdtar for WP-13's zip unpack;
  - `cargo fmt --all --check`;
  - `cargo clippy --locked --workspace --all-targets -- -D warnings`;
  - `cargo clippy --locked -p flat3d-viewer --all-targets --features xr,capture -- -D warnings`;
  - `cargo test --locked --workspace` with `XDG_RUNTIME_DIR=$(mktemp -d)`;
  - `rustup target add x86_64-pc-windows-msvc && cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings` (no SDK, no link).
- Job `py`:
  - `python3 -m py_compile $(git ls-files '*.py')`;
  - the per-folder loop `for d in $(git ls-files '*test_*.py'|xargs -n1 dirname|sort -u|grep -vx 'decky/tests'); do python3 -m unittest discover -s "$d" -p 'test_*.py'||exit 1; done`;
  - step `if: hashFiles('decky/tests/test_*.py')!=''` → `cd decky && python3 -m unittest discover -s tests -p 'test_*.py'`. The cwd puts `decky/` on sys.path for the locked `import main`, with no `__init__.py` and no edit to locked files;
  - step `if: hashFiles('tools/check.sh')!=''` → `shellcheck $(git ls-files '*.sh')`;
  - step `if: hashFiles('docs/workbench.md')!=''` → `python3 tools/docs/check.py`. The guard is WP-25b's new file, not check.py: check.py lands in wave 1 (WP-25a), but the repo passes a full run only after WP-25b. `tools/check.sh` uses the same guard (`[ -f docs/workbench.md ]`).
- Job `decky`: setup-node 22.23.4 and pnpm 10.34.9. Every step is guarded with `if: hashFiles('decky/package.json')!=''`, never at job level. Steps: `pnpm i --frozen-lockfile && pnpm run typecheck && pnpm run build`.

**One owner per fact:**

| Fact | Owner |
|---|---|
| part status (`present`/`stub`/`planned`) | `docs/code-map.md` (check.py: `present`/`stub` ⇒ path exists; no `inject/` path tokens) |
| JSON shapes, error codes | `api.rs` |
| command names | `cli::parse_any` + code-map table (USAGE ⊆ table) |
| cross-WP signatures | §2 "Frozen signatures" (WP-01) and "Wave-owned pub APIs" |
| issue state, waves | ROADMAP.md + `tools/pm/issues_report.py` |
| rulings | `docs/notes/DECISIONS.md` (untouched) |
| URLs, versions, hashes | `packaging/pins` (`<key>.<field>: <value>`) |
| SM2 addresses | `games/spiderman2/map.toml` (WP-30; doc block generated) |
| route keys | `recipes.rs` key lists (recipes/README generated text checked) |
| device procedures | `sessions/*.toml` (STEP0-RUNSHEET generated) |
| test counts | CI only |

**Final doc set:**
- README.md, SPEC.md, AGENTS.md, PIPELINE.md, CHANGELOG.md;
- docs/{ROADMAP,code-map,STEP0-RUNSHEET(generated),workbench}.md, docs/notes/DECISIONS.md;
- docs/spiderman2/{plan,spidy-core-map,address-map}.md (← PORT-SPIDERMAN2, spidy-stereo-core-map, spiderman2-address-map);
- docs/archive/uncharted3-stereo.md;
- recipes/README.md, packaging/README.md, sessions/README.md.

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

## 4. Work packages

### Index
Waves: w0 = 00; w1 = 01, 02, 25a; w2 = 03-07, 11-18, 20-24, 25b, 26; w3 = 08, 10, 19, 27, 28, 30; w4 = 09, 29, 31, 32, 33; w5 = 34.

- WP-00 (wave 0) cargo fmt + clippy gate (#183) | closes #183 | files: inject/src/**, inject/tests/** (rustfmt only), inject/Cargo.toml, app/src/**, app/Cargo.toml, .github/workflows/check.yml | depends: - | device: none
- WP-01 (wave 1) Workspace reshape + frozen bones | closes A1-A5, A9, A10, A12, A22, A23, C9; K1-K8; bug A5 (new issue) | files: Cargo.toml, Cargo.lock, inject/**, app/**, crates/engine/**, crates/app/**, crates/viewer/**, crates/uevr-inject/**, .github/workflows/check.yml, docs/code-map.md | depends: WP-00 | device: none
- WP-02 (wave 1) Repo tooling: check.sh, pins, pin.sh, issues report | closes K9-K11 (partly), K19, K22 (mechanism), T1, T3, T4 | files: tools/check.sh, packaging/pins, packaging/pin.sh, packaging/README.md, packaging/tests/test_pins.py, tools/pm/issues_report.py, tools/pm/tests/test_issues_report.py, tools/pm/tests/fixtures/issues.json, tools/ghidra/headless.sh, .gitignore | depends: WP-00 | device: none (pin.sh real run = Q0)
- WP-25a (wave 1) Docs check | closes T2 | files: tools/docs/check.py, tools/docs/test_check.py, tools/docs/private-allow.txt | depends: WP-00 | device: none
- WP-03 (wave 2) GameRow rows view (#76) | closes #76 | files: crates/engine/src/face/rows.rs, crates/engine/tests/rows.rs | depends: WP-01 | device: none
- WP-04 (wave 2) App Headset page (#116) | closes #116 | files: crates/app/src/headset_page.rs, crates/app/tests/headset_page.rs | depends: WP-01 | device: none
- WP-05 (wave 2) App Setup page (#118) | closes #118 | files: crates/app/src/setup_page.rs, crates/app/tests/setup_page.rs | depends: WP-01 | device: none
- WP-06 (wave 2) Logs view model + app Logs page | closes gap: logs (part of #121/#122) | files: crates/engine/src/face/logs.rs, crates/engine/tests/face_logs.rs, crates/app/src/logs_page.rs, crates/app/tests/logs_page.rs | depends: WP-01 | device: none
- WP-07 (wave 2) list + status commands, catalog join (#27a part) | closes #27 (list/status half) | files: crates/engine/src/catalog.rs, crates/engine/src/cli/engine.rs, crates/engine/src/steam.rs, crates/engine/tests/cli_list.rs | depends: WP-01 | device: none
- WP-11 (wave 2) Health loop (#125) | closes #125 (code), A6 | files: crates/engine/src/watch/health_loop.rs, crates/engine/src/tether/heal.rs, crates/engine/tests/tether_health.rs | depends: WP-01 | device: Q3 soak
- WP-12 (wave 2) Link stop idempotence + WiVRn detection | closes A8, A20 (stop), K17, K18 | files: crates/engine/src/link.rs (bodies; keep the WP-01 facade lines), crates/engine/src/link/wivrn.rs, crates/engine/tests/link_stop_twice.rs, crates/engine/tests/wivrn_detect.rs | depends: WP-01 | device: Q3
- WP-13 (wave 2) Pinned fetch (fetch.rs) | closes gap: fetch for #51/#52 | files: crates/engine/src/injectors/fetch.rs, crates/engine/tests/fetch.rs, crates/engine/tests/fixtures/pins-sample | depends: WP-01, WP-02 | device: Q0
- WP-14 (wave 2) UEVR route file half | closes #51 (file half), A7 (uevr) | files: crates/engine/src/injectors/uevr.rs, crates/engine/tests/uevr_apply.rs | depends: WP-01, WP-02 | device: Q6
- WP-15 (wave 2) flat3d-uevr-inject.exe + build-win.sh | closes K20, K21 | files: crates/uevr-inject/src/main.rs, crates/uevr-inject/src/args.rs, crates/uevr-inject/src/pick.rs, crates/uevr-inject/src/win.rs, crates/uevr-inject/tests/args.rs, tools/build-win.sh | depends: WP-01 | device: Q2
- WP-16 (wave 2) Geo-11 route pure half | closes #52 (code half) | files: crates/engine/src/injectors/geo11.rs, crates/engine/tests/geo11.rs | depends: WP-01 | device: Q9
- WP-17 (wave 2) Viewer pure modules (#44a) | closes #44 (part a) | files: crates/viewer/src/layout.rs, crates/viewer/src/status.rs, crates/viewer/src/source/test_pattern.rs, crates/viewer/tests/layout.rs, crates/viewer/tests/status.rs | depends: WP-01 | device: none
- WP-18 (wave 2) Viewer capture (#44b) | closes #44 (part b) | files: crates/viewer/src/source/pipewire.rs, crates/viewer/src/source/gamescope.rs, crates/viewer/src/source/portal.rs | depends: WP-01 | device: Q10
- WP-20 (wave 2) Install execute + launch options (#128) | closes #128 (code), A15, K24 | files: crates/engine/src/install.rs, crates/engine/src/launch_options.rs, packaging/flat3d-link.service, crates/engine/tests/install_execute.rs, crates/engine/tests/launch_options_at.rs | depends: WP-01 | device: Q3
- WP-21 (wave 2) Decky scaffold (#130) | closes #130, A13 | files: decky/main.py, decky/plugin.json, decky/package.json, decky/pnpm-lock.yaml, decky/rollup.config.js, decky/tsconfig.json, decky/src/index.tsx, decky/src/engine.ts, decky/src/types.ts, decky/tests/test_run_engine.py, decky/README.md | depends: WP-01 (network: its own pnpm step, §0) | device: Q14
- WP-22 (wave 2) Packaging scripts (#127/#129/#134) | closes #127 (script), #129 (script), #134 (script) | files: packaging/build-tools.sh, packaging/build-appimage.sh, packaging/AppRun, packaging/flat3d.desktop, packaging/build-decky.sh, packaging/tests/test_packaging.py, .github/workflows/appimage.yml | depends: WP-01, WP-02 | device: Q1, Q16
- WP-23 (wave 2) App Games page (#117) | closes #117 | files: crates/app/src/games_page.rs, crates/app/tests/games_page.rs | depends: WP-01 | device: none
- WP-24 (wave 2) Session runner, stereo gates, device sessions | closes T9-T12; drivers for every owner-device issue | files: tools/session/session.py, tools/session/test_session.py, tools/stereo/fps.py, tools/stereo/disparity.py, tools/stereo/test_stereo.py, sessions/README.md, sessions/q0.toml, sessions/q2.toml, sessions/q12.toml, sessions/step0.toml, sessions/sm2-offline.toml, sessions/link-smoke.toml, sessions/tools-build.toml, sessions/z1-offline.toml, sessions/z2-uevr.toml, sessions/z3-screen.toml, sessions/z4-mission.toml, sessions/z5-recipe.toml, sessions/a1-install.toml, sessions/a2-uevr.toml, sessions/a3-sortie.toml, sessions/g1-geo11.toml, sessions/g2-viewer.toml, sessions/g3-lana.toml, sessions/g4-control.toml, sessions/app-window.toml, sessions/appimage.toml, sessions/decky.toml, sessions/spidy-070.toml | depends: WP-01, WP-25a | device: all Q rows run these
- WP-25b (wave 2) Docs scrub + contradictions | closes K13, A11, A16, A18, C2, C5-C8, C10-C20, C22, C25-C30 | files: README.md, SPEC.md, PIPELINE.md, AGENTS.md, CHANGELOG.md, docs/ROADMAP.md, docs/code-map.md, docs/PORT-SPIDERMAN2.md, docs/spidy-stereo-core-map.md, docs/spiderman2-address-map.md, docs/spiderman2/plan.md, docs/spiderman2/spidy-core-map.md, docs/spiderman2/address-map.md, docs/uncharted3-stereo.md, docs/archive/uncharted3-stereo.md, docs/workbench.md, docs/STEP0-RUNSHEET.md, recipes/README.md, tools/ghidra/headless.sh (after WP-02), tools/build-spidy.sh, tools/research/research_game.py, tools/xmatch/targets.py | depends: WP-01, WP-02, WP-25a | device: none
- WP-26 (wave 2) Recipe seeds + repo recipe lint | closes T21, K25 | files: crates/engine/tests/fixtures/recipe-seeds/2075800-star-wars-zero-company.recipe, crates/engine/tests/fixtures/recipe-seeds/502500-ace-combat-7.recipe, crates/engine/tests/fixtures/recipe-seeds/1608230-planet-of-lana.recipe, crates/engine/tests/fixtures/recipe-seeds/870780-control.recipe, crates/engine/tests/fixtures/recipe-seeds/2651280-spider-man-2.recipe, crates/engine/tests/recipes_repo.rs, crates/engine/tests/recipe_seeds.rs | depends: WP-01, WP-02 | device: none
- WP-08 (wave 3) Link commands wiring (#120) | closes #120, A24 | files: crates/engine/src/cli/link.rs, crates/engine/tests/cli_link_run.rs | depends: WP-01, WP-07, WP-20 | device: Q3 (link-smoke)
- WP-10 (wave 3) Tether plans, runner errors, Wi-Fi sync | closes gap M25, K15 | files: crates/engine/src/tether.rs, crates/engine/src/tether/plan.rs, crates/engine/src/tether/wifi.rs, crates/engine/src/exec.rs (ProcessRunner::run body + private helpers only), crates/engine/tests/tether_plan_run.rs, crates/engine/tests/tether_runtime.rs | depends: WP-01, WP-12 | device: Q3
- WP-19 (wave 3) Viewer OpenXR + main (#44c) | closes #44 (part c) | files: crates/viewer/src/xr/session.rs, crates/viewer/src/xr/swapchain.rs, crates/viewer/src/xr/frame.rs, crates/viewer/src/gpu/import.rs, crates/viewer/src/main.rs | depends: WP-01, WP-17, WP-18 | device: Q10
- WP-27 (wave 3) inject/undo commands + ensure_applied (#27a) | closes #27 (27a), A20 (save_record), gaps D1, D9 | files: crates/engine/src/injectors.rs, crates/engine/src/undo.rs, crates/engine/src/cli/inject.rs, crates/engine/src/cli/undo.rs, crates/engine/src/cli/setup_fetch.rs, crates/engine/tests/inject_fake_prefix.rs, crates/engine/tests/ensure_applied.rs, crates/engine/tests/cli_undo.rs, crates/engine/tests/cli_setup_fetch.rs | depends: WP-01, WP-02, WP-07, WP-13, WP-14, WP-16, WP-20 | device: Q6, Q9
- WP-28 (wave 3) View commands (#121) | closes #121, #75 | files: crates/engine/src/cli/view.rs, crates/engine/src/face/link_model.rs, crates/engine/tests/cli_view.rs | depends: WP-01, WP-03, WP-06, WP-07, WP-20 | device: none
- WP-30 (wave 3) RE tooling + address map (#19 tools, #37 column) | closes A17, T5-T8, #37 (column only) | files: tools/re/analyze.sh, tools/re/exe_id.py, tools/re/fill_map.py, tools/re/test_fill_map.py, tools/re/test_analyze_sh.py, tools/re/test_exe_id.py, tools/ghidra/Flat3dVtOptions.java, tools/ghidra/ExportVtMatches.java, games/spiderman2/map.toml, docs/spiderman2/address-map.md (generated block only) | depends: WP-02, WP-25b | device: Q12
- WP-09 (wave 4) Watch service correctness | closes A20 (allow-debugging), K8, K14 (callers), M25 wiring | files: crates/engine/src/watch/machine.rs, crates/engine/src/watch/service.rs, crates/engine/tests/watch_service.rs | depends: WP-01, WP-10, WP-11, WP-12 | device: Q3
- WP-29 (wave 4) doctor + setup commands | closes T13, T14, gap D8 | files: crates/engine/src/cli/doctor.rs, crates/engine/src/cli/setup.rs, crates/engine/tests/cli_doctor.rs, crates/engine/tests/cli_setup.rs | depends: WP-01, WP-12, WP-13, WP-15, WP-20, WP-22, WP-27 | device: Q4
- WP-31 (wave 4) Wrapper route end-to-end | closes A5 (complete), A7, A8 (wrapper), gap D1 | files: crates/engine/src/wrapper.rs, crates/engine/src/bin/flat3d-run.rs, crates/engine/tests/wrapper_route.rs | depends: WP-01, WP-12, WP-16, WP-27 | device: Q6, Q9, Q10
- WP-32 (wave 4) Decky panel, launch sync, collection, toasts (#131-#133) | closes #131, #132, #133 (code) | files: decky/src/index.tsx, decky/src/panel.tsx, decky/src/launch.ts, decky/src/collection.ts, decky/src/toasts.ts | depends: WP-21, WP-28 | device: Q14
- WP-33 (wave 4) App views + main wiring (#122/#123) | closes #122, #123 (code) | files: crates/app/src/lib.rs, crates/app/src/main.rs, crates/app/src/engine.rs, crates/app/src/view_headset.rs, crates/app/src/view_games.rs, crates/app/src/view_setup.rs, crates/app/src/view_logs.rs, crates/app/tests/app_wiring.rs | depends: WP-04, WP-05, WP-06, WP-23, WP-28 (its doctor/setup/inject/link calls are runtime CLI calls, not merge deps) | device: Q15
- WP-34 (wave 5) Docs sync after the pass | closes A2-A4 doc remainder, OD3 SPEC line, runsheet generation | files: docs/code-map.md, docs/ROADMAP.md, CHANGELOG.md, SPEC.md, recipes/README.md, docs/STEP0-RUNSHEET.md, tools/docs/private-allow.txt, README.md | depends: WP-00, WP-01, WP-02, WP-03, WP-04, WP-05, WP-06, WP-07, WP-08, WP-09, WP-10, WP-11, WP-12, WP-13, WP-14, WP-15, WP-16, WP-17, WP-18, WP-19, WP-20, WP-21, WP-22, WP-23, WP-24, WP-25a, WP-25b, WP-26, WP-27, WP-28, WP-29, WP-30, WP-31, WP-32, WP-33 | device: none

### WP-00: cargo fmt + clippy gate (#183)

**Closes:** #183 (also A19). Lands alone in wave 0, on branch `v2/wp-00-fmt-clippy` from `main@2812f64`.

**Files (exclusive):**
- `inject/src/**` and `inject/tests/**`: format only.
- `inject/Cargo.toml`: adds the lints table.
- `.github/workflows/check.yml`.
- `app/src/**` and `app/Cargo.toml`: owned by this WP, but nothing changes in them (see Behaviour 3).

Rehearsed on a scratch copy (`git archive 2812f64`, at `$SCRATCH/wp00`): **48 files change, +1157/−256.**
- **10 src files:**
  - `inject/src/cli.rs`
  - `inject/src/cli_link.rs`
  - `inject/src/injectors.rs`
  - `inject/src/injectors/uevr.rs` (8 hunks)
  - `inject/src/lib.rs` (2)
  - `inject/src/link.rs`
  - `inject/src/link_view.rs`
  - `inject/src/tether/plan.rs`
  - `inject/src/undo.rs` (4)
  - `inject/src/watch.rs`
- **38 test files, all locked.** The ones with the most hunks are `watch_plug` 10, `status` 8, `watch_manual` 8, `install_plan` 7 and `wrapper_plan` 7. `app/`: 0 diffs.

**Signatures:** none. No Rust item changes. The only additions are TOML and YAML:
```toml
# inject/Cargo.toml (append after [dependencies])
# #183: locked tests/switches_settings.rs:19,31,55 build Settings by field reassignment (R28, not edited).
[lints.clippy]
field_reassign_with_default = "allow"
```
WP-01 later moves this into `[workspace.lints.clippy]` and gives each crate `[lints] workspace=true`.

**Behaviour:**
1. **Commit 1, "cargo fmt (format only, #183)":**
   - Run `cargo fmt --manifest-path inject/Cargo.toml` and `cargo fmt --manifest-path app/Cargo.toml`.
   - There is no `rustfmt.toml`, so the default style for edition 2021 applies.
   - Nothing else goes in this commit.
   - rustfmt's defaults reorder `use` lists and `mod` lines, for example in `lib.rs` (`headset_view` ↔ `health`, `stats`, `switches`) and `cli.rs:` `{…, ProcessRunner, TetherAdb}`.
   - It also wraps two match/closure bodies in `{}` (`cli_link.rs:119` `parse_run` arm; `uevr.rs` test closure).
   - All of this is token-neutral apart from ordering, `,` and `{}`. That is sanctioned for locked tests by owner answer 1. rustfmt does not touch comments (`wrap_comments=false`), so the `//! Locked…` headers stay byte-identical.
2. **Commit 2, "clippy allow + CI gate":** add the `inject/Cargo.toml` lints table, then change `check.yml` as below.
3. **No app lints to fix.** `cargo clippy --manifest-path app/Cargo.toml --all-targets -- -D warnings` already exits 0 at 2812f64 with clippy 0.1.95. The "fix remaining app lints by hand" step therefore does nothing. If a newer CI clippy reports a lint, it gets a behaviour-neutral hand fix in commit 2.
4. **check.yml, both jobs:**
   - Pin the toolchain and add `components: clippy, rustfmt`.
   - Insert fmt and clippy steps **before** build and test.
   - Use no `--locked`, because no `Cargo.lock` is tracked until WP-01 (`app/Cargo.lock` and `inject/Cargo.lock` are untracked).
```yaml
# engine job (replaces check.yml:24)
      - uses: dtolnay/rust-toolchain@1.95.0
        with: { components: "clippy, rustfmt" }
      - run: cargo fmt --manifest-path inject/Cargo.toml --check
      - run: cargo clippy --manifest-path inject/Cargo.toml --all-targets -- -D warnings
#   (then the existing build/test/python steps, check.yml:25-30)
# app job (replaces check.yml:41)
      - uses: dtolnay/rust-toolchain@1.95.0
        with: { components: "clippy, rustfmt" }
      - run: cargo fmt --manifest-path app/Cargo.toml --check
      - run: cargo clippy --manifest-path app/Cargo.toml --all-targets -- -D warnings
#   (then the existing test/check steps, check.yml:42-43)
```
- Flag: the bones and research use `dtolnay/rust-toolchain@stable`. The host toolchain is rustc/clippy 1.95.0 (2026-04-14). By 2026-11 a stable runner is several releases newer, and a new default clippy lint would turn the `-D warnings` gate red with no code change.
  - Pin `@1.95.0`, the version this gate was verified on.
  - WP-01 should keep the pin (or add `rust-toolchain.toml`) and raise it deliberately.
  - If the PM wants `@stable`, the done-check below must be re-run on the CI clippy version.
- Flag: the bones' "48 files fail fmt" is confirmed (48). The issue body says only `cli_link.rs`, which is section 6's rewrite of #183.
- Flag: rustfmt also changes tokens in non-locked src (`{}` around a match arm in `cli_link.rs` `parse_run`). It is behaviour-neutral, and the done-check below allows for it.

**Tests:**
- Locked: all 55 `inject/tests/*.rs`. Only rustfmt edits them, and their test names and assertions are unchanged.
  - Rehearsal: **inject 459 passed / 0 failed** (lib + 2 bins + 55 test files + doctests).
  - App: **4 passed / 0 failed** (`app/src/lib.rs` unit tests).
- New: none.

**Inline facts:**
- clippy failure on main: `tests/switches_settings.rs:20:5` and `:32:5`, "field assignment outside of initializer for an instance created with Default::default()" (`field_reassign_with_default`). `let mut s = Settings::default();` is at :19, :31 and :55; the lint fires twice.
- Every other inject target and every app target is already clippy-clean.
- `[lints]` in Cargo.toml needs cargo ≥ 1.74; the host has 1.95.0.
- Runner `[self-hosted, linux]`, Ubuntu 24.04 container. The apt step needs no change for WP-00.

**Depends on:** nothing. Every other WP depends on this one.

**Owner device:** none.

**Done-check** (on the branch, `CARGO_TARGET_DIR` set to the agent scratch):
```sh
set -e
for m in inject app; do
  cargo fmt --manifest-path $m/Cargo.toml --check
  cargo clippy --manifest-path $m/Cargo.toml --all-targets -- -D warnings
  cargo test  --manifest-path $m/Cargo.toml
done
# format-only proof for commit 1: per file, the multiset of non-whitespace chars minus , { } is unchanged
git diff --name-only HEAD~2 HEAD~1 | python3 -c '
import sys,re,subprocess as s
k=lambda t: sorted(re.sub(r"[\s,{}]","",t))
g=lambda r,f: s.run(["git","show",f"{r}:{f}"],capture_output=True,text=True).stdout
bad=[f for f in sys.stdin.read().split() if k(g("HEAD~2",f))!=k(g("HEAD~1",f))]
sys.exit(f"not format-only: {bad}" if bad else 0)'
git diff --name-only HEAD~1 HEAD | grep -vxE 'inject/Cargo.toml|\.github/workflows/check.yml|app/src/.*' && exit 1 || true
```
Expected: fmt 0/0, clippy 0/0, `test result` totals 459 and 4 with 0 failed, and an empty "not format-only" list. The rehearsal on 2812f64 passed every step.

### WP-01: Workspace reshape + frozen bones

**Closes:** A1-A5, A9, A10, A12, A22 and A23, C9, K1-K8, and bug A5 (new issue). It also does the workspace part of the reshape. Most bodies stay stubs, and later WPs fill them in. **WP-01 is the only WP that moves files, adds `mod` lines, dependencies or features, or edits `Cargo.*`/`check.yml`. `Cargo.lock` is frozen after it.** Every frozen form below is copied from bones §2 ("Frozen signatures" and "Bones WP-01 freezes"). If this file and bones §2 disagree, bones wins.

**Depends on:** WP-00 (rustfmt baseline). **Wave:** 1, in parallel with WP-02 and WP-25a. **Owner device:** none (host). **Network:** one online `cargo fetch` (crates.io source; openxr and ashpd 0.13.13 are not cached). Nothing else.

**Files (exclusive):**
- **Moves** (`git mv`, so history follows):
  - `inject/**` → `crates/engine/**` and `app/**` → `crates/app/**`.
  - Inside the engine:
    - `src/cli_link.rs` → `src/cli/link.rs`;
    - `src/{collection,headset_view,link_view,toasts,view}.rs` → `src/face/`;
    - `src/tether/mod.rs` → `src/tether.rs`;
    - `src/watch.rs` → `src/watch/service.rs`. Then `frames`, `machine`, `files` and `run` are split out of `service.rs` into their own files, and a new `src/watch.rs` holds only the facade.
  - Packages, libs and bins keep their names: `flat3d-inject` / `flat3d_inject` / `[[bin]] flat3d-inject` and `flat3d-app` / `flat3d_app`.
  - Delete the untracked `inject/Cargo.lock`, `app/Cargo.lock` and the per-crate `target/` dirs, but only after step 1 of "Fetching dependencies" has seeded the root lock.
  - **There is no `mod.rs` anywhere and no `x/x.rs`.** Every module with children uses the facade-file form (`x.rs` + `x/`).
- **New:**
  - Root: `Cargo.toml`, `Cargo.lock`.
  - Engine `crates/engine/src/`:
    - `exec.rs`, `testing.rs`, `paths.rs`, `fsx.rs`, `sys.rs`, `catalog.rs`, `face.rs`;
    - `face/{rows,link_model,logs}.rs`;
    - `link/wivrn.rs`;
    - `cli/{engine,inject,undo,view,setup,setup_fetch,doctor}.rs`. `cli/undo.rs` is new because today's undo dispatch body moves into it; it is not a git mv;
    - `watch/{frames,machine,files,run,health_loop}.rs`, plus the facade `watch.rs`;
    - `injectors/{geo11,port,fetch}.rs`.
  - Engine `crates/engine/tests/`: `paths.rs`, `fsx.rs`, `catalog_find.rs`, `switches_io.rs`, `state_io.rs`, `route_settings.rs`, `cli_any.rs`.
  - App `crates/app/src/`: `engine.rs`, `headset_page.rs`, `games_page.rs`, `setup_page.rs`, `logs_page.rs`, `view_headset.rs`, `view_games.rs`, `view_setup.rs`, `view_logs.rs`.
  - Viewer `crates/viewer/`:
    - `Cargo.toml`;
    - `src/{lib,main,frame,spec,source,layout,status}.rs`;
    - `src/source/{test_pattern,pipewire,gamescope,portal}.rs`;
    - `src/gpu.rs`, `src/gpu/import.rs`;
    - `src/xr.rs`, `src/xr/{session,swapchain,frame}.rs`;
    - `tests/spec.rs`.
  - uevr-inject: `crates/uevr-inject/{Cargo.toml, src/{lib,main,args,pick,win}.rs}`.
- **Edited:**
  - Engine `src/`:
    - `lib.rs`, `main.rs` (stale `//!` on line 1, A2), `cli.rs`, `cli/link.rs` (the moved parse half), `link.rs` (facade lines plus the `start_3d_commands_with` stub), `tether/adb.rs`, `injectors.rs` (A4 doc on line 1, mod lines, `RouteEnv`, `ensure_applied`);
    - `wrapper.rs`, `bin/flat3d-run.rs`, `state.rs`, `switches.rs`, `recipes.rs`, `api.rs`, `install.rs`, `launch_options.rs`;
    - `face/view.rs` and `face/toasts.rs` (Serialize derives).
  - `crates/engine/Cargo.toml`, `crates/app/Cargo.toml`, `crates/app/src/lib.rs` (the 9 mod lines).
  - The 3 locked-path test files `api_response.rs`, `link_stop.rs` and `recipe_load.rs` get the 7 sanctioned edits (see Tests). `wrapper_run.rs` is **not** edited.
  - `docs/code-map.md`, `.github/workflows/check.yml`.
- **Ownership after this WP:**
  - `exec.rs`: WP-01 owns it because it holds the trait split. WP-10 may change only the body of `ProcessRunner::run` and add private helpers.
  - `link.rs`: WP-12 owns the bodies, and the WP-01 facade lines stay.
  - `crates/app/src/lib.rs`: WP-33 owns it from wave 4.
  - Everything else in bones §0 "Frozen after wave 1" is frozen.

---

#### Cargo

Root `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/*"]
default-members = ["crates/engine", "crates/viewer", "crates/uevr-inject"]

[workspace.package]
edition = "2021"
license = "GPL-3.0-only"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
iced = { version = "0.14.0", features = ["smol"] }
flat3d-inject = { path = "crates/engine" }
flat3d-viewer = { path = "crates/viewer", default-features = false }
windows-sys = "=0.61.2"

[workspace.lints.clippy]
field_reassign_with_default = "allow"
```

Every crate:
- keeps its own `version = "0.0.0"` and `publish = false`;
- sets `edition.workspace = true`, `license.workspace = true` and `[lints] workspace = true`.

`crates/engine/Cargo.toml`:
- Keeps `[package] name = "flat3d-inject"`, `[lib] name = "flat3d_inject"` and `[[bin]] name = "flat3d-inject" path = "src/main.rs"`. Locked `cli_json.rs` needs `env!("CARGO_BIN_EXE_flat3d-inject")`.
- `[[bin]] flat3d-run` stays as it is today.
- `description = "Flat to 3D engine: installed games, recipes, stereo routes, the link service and the launch wrapper"` (C9).
- Deps: `serde`, `serde_json` and `flat3d-viewer`, each `{ workspace = true }`.
  - Because the viewer has no default features, the engine never pulls in openxr or pipewire.
  - This edge is one-way: the viewer never depends on the engine.
- `sys::current_uid` keeps today's `extern "C" { fn getuid() -> u32; }` (flat3d-run.rs:8-13), so the engine needs no `libc`.

`crates/app/Cargo.toml`: `flat3d-inject`, `iced` (adds `smol`) and `serde_json`, each `{ workspace = true }`. Today it has `path = "../inject"`.

`crates/viewer/Cargo.toml` (package `flat3d-viewer`, lib `flat3d_viewer`):
```toml
[features]
default = []
xr = ["dep:openxr", "dep:ash"]
capture = ["dep:pipewire", "dep:ashpd", "dep:tokio"]

[dependencies]
serde = { workspace = true }
libc = "0.2"
openxr = { version = "=0.22.0", features = ["loaded"], optional = true }
ash = { version = "=0.38.0", optional = true }
pipewire = { version = "=0.10.1", optional = true }
ashpd = { version = "=0.13.13", default-features = false, features = ["screencast", "tokio"], optional = true }
tokio = { version = "1", default-features = false, features = ["rt", "time"], optional = true }

[[bin]]
name = "flat3d-viewer"
path = "src/main.rs"
required-features = ["xr", "capture"]
```
`libc` is not optional because WP-19 uses it for `sigaction`/`pidfd`.

`crates/uevr-inject/Cargo.toml` (package `flat3d-uevr-inject`, lib `flat3d_uevr_inject`, bin `flat3d-uevr-inject`):
```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { workspace = true, features = [
  "Win32_System_Diagnostics_ToolHelp", "Win32_System_Memory", "Win32_System_Threading",
  "Win32_System_LibraryLoader", "Win32_Foundation", "Win32_System_Diagnostics_Debug",
  "Win32_Security",
] }
```
`Win32_Security` is there because `CreateRemoteThread` is gated on it and takes `*const SECURITY_ATTRIBUTES`.

**Fetching dependencies** (the executor's job):
1. `cp app/Cargo.lock Cargo.lock` as the seed. Do this before the git mv deletes the untracked file, and keep a copy in `$SCRATCH`.
2. Run `cargo fetch` once, online. It is the single declared network use, and it adds the missing crates while keeping the seeded pins.
   - Do **not** run `cargo generate-lockfile` or `cargo update`, because they re-resolve everything.
3. Check: `cargo metadata --locked --offline --format-version 1 >/dev/null`.
4. Commit `Cargo.lock`. After WP-01 it never changes, because every gate uses `--locked`.
5. Cargo.lock lists optional dependencies even when their features are off, so this one lock covers `xr`, `capture` and the `cfg(windows)` deps.

---

#### Module facades (explicit re-exports, never `*`)

Engine `lib.rs`:
- Fix the stale doc (lines 1-6, A3).
- Add a `pub mod` line for every top-level module, including:
  ```rust
  pub mod face; pub mod paths; pub mod exec; pub mod catalog; pub mod fsx; pub mod sys;
  #[doc(hidden)] pub mod testing;
  pub use cli::link as cli_link;
  pub use face::{collection, headset_view, link_view, rows, toasts, view};
  pub use link::wivrn;
  ```
  - The old top-level `pub mod collection; …; pub mod cli_link;` lines are removed. The re-exports keep every locked `flat3d_inject::<name>` path resolving.
  - `pub use link::wivrn;` makes `crate::wivrn` and `crate::link::wivrn` the same module.
- `NotBuilt` keeps its crate-root definition (lib.rs:39-47). Locked `api_response.rs:5,27` imports `flat3d_inject::NotBuilt`, and `From<NotBuilt> for ApiError` stays at api.rs:103.

`face.rs`: `pub mod collection; pub mod headset_view; pub mod link_view; pub mod toasts; pub mod view; pub mod rows; pub mod link_model; pub mod logs;`

`watch.rs` (facade only):
```rust
mod frames; mod machine; mod files; mod service; mod run; mod health_loop;
pub use frames::{parse_track_frames, Device};
pub use machine::{step, link_commands, note_text, note_kind, note_event, Effect, Event, Inputs, Note, Phase, Watch, GRACE_SECS};
pub use files::{write_snapshot, Paths};
pub use service::{Feed, Service, TrackDevices};
pub use run::run;
pub use health_loop::{HealthLoop, HealthTick};
```
- The submodules may be `pub mod` instead, if that keeps the moved code's `super::` paths simpler. The re-export list is exactly this one either way.
- `note_event` stays a `pub fn` in `machine.rs` (watch.rs:452 today).
- The submodule is named `files`, not `paths`, so it never clashes with `crate::paths`.
- Locked tests need 13 of these names (watch_manual.rs:4-6, watch_frames.rs:3, watch_grace.rs:3).

`link.rs`, at the top:
```rust
pub mod wivrn;
pub use crate::exec::{CommandOutput, CommandRunner, SystemRunner};
pub use wivrn::{FlatpakScope, QuestApp, WivrnDirs, WivrnStart, QUEST_APPS, SERVER_FLAGS, detect_quest_app, detect_server};
```
- `WIVRN_APP` (link.rs:10) stays where it is.
- Also add `pub fn start_3d_commands_with(_s: &WivrnStart) -> Vec<Vec<String>>`. Its stub body is `start_3d_commands()`, and WP-12 writes the real one.

`tether/adb.rs`: `pub use crate::exec::{ProcessRunner, RelayRunner, RelayRunner as CommandRunner};`
- `AdbRunner<R: link::CommandRunner>` keeps its bound, constructor and `link::CommandRunner` impl (adb.rs:171-191).
- `TetherAdb<R: RelayRunner>` (adb.rs:134) is unchanged.

`injectors.rs`: `pub mod uevr; pub mod geo11; pub mod port; pub mod fetch;`. The 3 new files are empty stubs.

`cli.rs`:
- Adds `pub mod engine; pub mod inject; pub mod undo; pub mod link; pub mod view; pub mod setup; pub mod setup_fetch; pub mod doctor;`.
- **Removes `use crate::undo;` (cli.rs:6)**, which would clash with `pub mod undo` (E0255). `cli/undo.rs` calls `crate::undo::…` by its full path.

App `crates/app/src/lib.rs`: add these lines above the existing `Tab` code:
```rust
pub mod engine; pub mod headset_page; pub mod games_page; pub mod setup_page; pub mod logs_page;
pub mod view_headset; pub mod view_games; pub mod view_setup; pub mod view_logs;
```

Viewer `lib.rs`:
```rust
pub mod frame; pub mod spec; pub mod layout; pub mod status; pub mod source;
#[cfg(feature = "xr")] pub mod xr;
#[cfg(feature = "xr")] pub mod gpu;
```
- `gpu.rs` holds `pub mod import;`.
- `xr.rs` holds `pub mod session; pub mod swapchain; pub mod frame;`.

uevr-inject `src/lib.rs`: `pub mod args; pub mod pick; #[cfg(windows)] pub mod win;`

**Glob-import guard:** no new pub item in `api`, `install` or `tether::relay` may be named `json`, `c`, `p`, `H` or `full`, and none of those three modules gets a glob re-export. Locked tests glob-import them (install_plan.rs:3, api_snapshot.rs:3, tether_tunnel.rs:3, tether_relay_cmds.rs:3, tether_tunnel_review.rs:3).

---

#### Exec split (`exec.rs`, full)

Move these verbatim:
- link shape: `CommandRunner { fn run(&mut self, argv: &[String]) -> CommandOutput }`, plus `CommandOutput` and `SystemRunner` (link.rs:100-140);
- relay shape: `RelayRunner` (renamed from `tether::adb::CommandRunner`), with `fn run(&mut self, &[String]) -> Result<String, String>`, `fn start_relay(&mut self, &[String], &str) -> Result<(), String>` and `fn stop_relay(&mut self) -> Result<(), String>`, verbatim from adb.rs:64-68, plus `ProcessRunner` (adb.rs:63-72).

There is no blanket impl.

#### Frozen signatures (stubs compile; WP-01 writes the "full" ones in full)

Stub rules:
- A stub that returns `Result` gives `Err(NotBuilt("<path>").into())` / `ApiError{code:"not-built"}` or `Err("not-built: <fn> (WP-xx)".into())`.
- Any other stub returns a neutral value, or delegates to the legacy fn as stated.
- Parameters take a `_` prefix. There is no `todo!()`, `unimplemented!()` or panic.

**`paths.rs`** (full; no owner afterwards):
```rust
pub struct EnginePaths { pub home: PathBuf, pub uid: u32, pub state_dir: PathBuf, pub config: PathBuf,
    pub data: PathBuf, pub cache_dir: PathBuf, pub log_dir: PathBuf, pub bin: PathBuf, pub recipe_dirs: Vec<PathBuf> }
impl EnginePaths { pub fn from_env(env: &BTreeMap<String, String>, uid: u32) -> Result<EnginePaths, ApiError> }
```
- A missing or empty `HOME` gives `ApiError` code `not-found`.
- Fields:

  | Field | Value |
  |---|---|
  | `state_dir` | `state::state_dir(env XDG_RUNTIME_DIR, uid)` |
  | `config` | `home/.config/flat3d` |
  | `data` | `undo::data_dir(env XDG_DATA_HOME, home)`, i.e. `$XDG_DATA_HOME/flat3d` else `home/.local/share/flat3d`. This is today's `engine_data_dir` |
  | `cache_dir` | `$XDG_CACHE_HOME/flat3d` (non-empty) else `home/.cache/flat3d` |
  | `log_dir` | `home/.local/state/flat3d` (watch.rs:323, flat3d-run.rs:34) |
  | `bin` | `home/.local/bin` |
  | `recipe_dirs` | `[$FLAT3D_SRC_DIR/recipes]` when set and non-empty, then `data/recipes` |

- Accessors:

  | Accessor | Returns |
  |---|---|
  | `settings()` | `home/.config/flat3d/settings` |
  | `link_log()` | `log_dir/link.log` |
  | `run_log()` | `log_dir/run.log` |
  | `cache()` | `cache_dir` |
  | `uevr_cache()` | `cache()/uevr` |
  | `env_file(appid: u32)` | `data/env/{appid}.env` |
  | `wrapper_bin()` | `bin/flat3d-run` (absolute) |
  | `viewer_bin()` | `bin/flat3d-viewer` (absolute) |
  | `pins()` | `data/pins` |
  | `run_paths() -> RunPaths` | `{state: state_dir lossy, settings: settings(), recipes: data/recipes, log: run_log()}` |
  | `watch_paths() -> watch::Paths` | `{home, state_dir}` as lossy strings |

  Every accessor returns `PathBuf` except the last two.
- Production callers use `from_env(&std::env::vars().collect(), sys::current_uid())`. Locked `state::*_path(&str)` callers pass `p.state_dir.to_string_lossy()`.

**`fsx.rs` / `sys.rs`** (full):
- `fsx::write_atomic(&Path, &[u8]) -> io::Result<()>` writes a tmp file in the same dir, then renames it.
- `fsx::append_line(&Path, &str) -> io::Result<()>` creates the parent dirs.
- `fsx::read_opt(&Path) -> io::Result<Option<String>>` gives `None` when the file is NotFound.
- `sys::current_uid() -> u32` and `sys::unix_now() -> u64`.

**`state.rs` / `switches.rs` / `catalog.rs`** (full):
- `state::read_snapshot(p: &EnginePaths) -> Result<Option<LinkSnapshot>, String>`. A missing file gives `Ok(None)`.
- `state::write_request(p: &EnginePaths, r: Request) -> io::Result<()>`. `Request` is `Copy` (state.rs:117-118).
- `switches::read_settings(path: &Path) -> Result<Settings, String>`. NotFound gives `Ok(Settings::default())`.
- `switches::write_settings(path: &Path, s: &Settings) -> io::Result<()>`. It goes through `fsx::write_atomic`.
- `catalog::find_recipe(dirs: &[PathBuf], appid: u32) -> Result<Option<(PathBuf, String)>, String>`:
  - it tries `{id}.recipe`, then the sorted `{id}-*.recipe`;
  - the first dir with a match wins;
  - **a dir that does not exist is skipped**;
  - the result is `Ok(None)` when no dir has a match.

**`recipes.rs`** (full):
- `pub type RouteSettings = BTreeMap<String, String>;`
- `UEVR_KEYS: [&str; 6] = ["uevr-exe","uevr-rendering","uevr-2d-screen","uevr-native-stereo-fix","uevr-delay","uevr-profile"]`
- `GEO11_KEYS: [&str; 7] = ["geo11-fix","geo11-exe","geo11-layout","geo11-separation","geo11-convergence","geo11-d3dx","geo11-remove-first"]`
- `PORT_KEYS: [&str; 0]`
- `route_settings(text: &str) -> RouteSettings`:
  - each line splits at the first `:`, and both sides are trimmed;
  - one pair of surrounding quotes is stripped;
  - `#` lines and blank lines are skipped;
  - only `uevr-*`/`geo11-*`/`port-*` keys are kept, so it never returns the 9 recipe KEYS.
- `check_route_settings(route: &str, m: &RouteSettings) -> Result<(), String>`:
  - `route` is the raw `stereo-route` text (`Recipe.route`), mapped with `injectors::route_of`;
  - Port and no-route accept only an empty map;
  - a key outside the route's key list is refused;
  - value rules:

    | Key | Rule |
    |---|---|
    | `uevr-exe` | non-empty (required) |
    | `uevr-rendering` | `native` or `synced` |
    | `uevr-2d-screen`, `uevr-native-stereo-fix` | `yes` or `no` |
    | `uevr-delay` | u32 in `0..=120` |
    | `uevr-profile` | a pin key `[a-z0-9-]+` |
    | `geo11-fix` | a pin key `[a-z0-9-]+` (required) |
    | `geo11-exe` | non-empty (required) |
    | `geo11-layout` | `sbs`, `tab`, `sbs_reversed` or `tab_reversed` |
    | `geo11-separation` | `0..=100` |
    | `geo11-convergence` | f32 > 0 |
    | `geo11-d3dx` | `k=v;k=v` |
    | `geo11-remove-first` | `;`-separated relative paths |

**`link/wivrn.rs`** (WP-12 writes the bodies; only `user_default` is full now):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FlatpakScope { User, System }
impl FlatpakScope { pub fn flag(self) -> &'static str }            // full: "--user" / "--system"
pub const SERVER_FLAGS: [&str; 2] = ["--early-active-runtime", "--no-publish-service"];
pub const QUEST_APPS: [&str; 5] = [/* WP-12's five package names */];
#[derive(Debug, Clone, PartialEq, Eq)] pub struct WivrnDirs { pub scope: FlatpakScope, pub app_dir: PathBuf, pub deploy: PathBuf, pub version: Option<String> }
impl WivrnDirs { pub fn user_default(home: &Path) -> WivrnDirs }   // full
#[derive(Debug, Clone, PartialEq, Eq)] pub struct QuestApp { pub package: String, pub version: Option<String> }
pub struct WivrnStart { pub scope: Option<FlatpakScope>, pub app: String, pub flags: Vec<String> }
impl Default for WivrnStart   // full: scope None, app link::WIVRN_APP, flags SERVER_FLAGS
pub fn app_dir_from_location(out: &str) -> Option<PathBuf>        // stub None
pub fn version_from_info(out: &str) -> Option<String>             // stub None
pub fn version_name(dumpsys: &str) -> Option<String>              // stub None
pub fn versions_match(server: &str, app: &str) -> bool            // stub false
pub fn detect_server(r: &mut dyn CommandRunner) -> Result<WivrnDirs, String>          // stub Err
pub fn detect_quest_app(r: &mut dyn CommandRunner, serial: &str) -> Option<QuestApp>  // stub None
```
- `user_default`: scope User, `app_dir = home/.local/share/flatpak/app/io.github.wivrn.wivrn`, `deploy = app_dir/current/active`, version None.
- `QUEST_APPS`: copy WP-12's five entries. If WP-12 has not written them yet, use WP-12.md's list verbatim.
- Do not use these retired names: `WIVRN_SERVER_FLAGS`, `QUEST_APP_CANDIDATES`, `detect_dirs` and `WivrnDirs{scope,root}`.

**`watch/health_loop.rs`** (WP-11 writes the bodies):
```rust
pub struct HealthTick { pub health: Option<api::Health>, pub stats: Option<api::Stats>, pub tunnel: bool,
    pub heal: Option<tether::heal::Heal>, pub toast: Option<(api::EventKind, String)> }
pub struct HealthLoop { /* private */ }
impl HealthLoop {
    pub fn new(start_unix: u64, mem_start_kb: u64, soak_file: Option<PathBuf>) -> Self
    pub fn tick(&mut self, now: u64, mode: TetherMode, playing: bool, r: &mut dyn RelayRunner) -> HealthTick
}
```
- The stub `tick` returns `HealthTick{health: None, stats: None, tunnel: mode != TetherMode::Off, heal: None, toast: None}`. That is today's snapshot behaviour (watch.rs:761-781), so no locked watch test changes.
- There is no `take_toast`, no `pub playing`, no `HealthAction` and no `WriteSoak`. WP-11 may later add `reconnects()`/`finish()` in its own file.

**`injectors.rs`** (WP-27 writes the body):
```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteEnv { pub env: Vec<(String, String)>, pub unset: Vec<String>, pub sidecar: Option<Vec<String>> }
pub fn ensure_applied(game: &steam::Game, recipe: &Recipe, settings: &RouteSettings, p: &EnginePaths, r: &mut dyn CommandRunner) -> Result<RouteEnv, String>   // stub Err
```
There is no `injectors::Applied`. `uevr::Applied` (uevr.rs:60) is unrelated and stays.

**`wrapper.rs`** (WP-31 writes the bodies; the locked `prepare`/`run`/`exec_spec` at wrapper.rs:29,119,219 keep their signatures and bodies):
```rust
pub struct ExecSpec { pub argv: Vec<String>, pub env: Vec<(String, String)>, pub unset: Vec<String>, pub sidecar: Option<Vec<String>> }
pub fn exec_spec_with(plan: &Plan, argv: &[String], route: &RouteEnv, wivrn: &WivrnDirs, existing_fs_rw: Option<&str>) -> ExecSpec
pub fn prepare_with_route(argv: &[String], env: &BTreeMap<String, String>, snapshot: Result<Option<&LinkSnapshot>, String>,
    settings: Result<&Settings, String>, recipe: Result<Option<&Recipe>, String>,
    route: &mut dyn FnMut(u32, &Recipe) -> Result<(RouteEnv, WivrnDirs), String>, pid: u32, now: u64) -> Launch
pub fn run_with<T>(argv: &[String], env: &BTreeMap<String, String>, paths: &EnginePaths, pid: u32, now: u64,
    runner: &mut dyn CommandRunner, launcher: impl FnOnce(&Launch) -> io::Result<T>) -> (Launch, io::Result<T>)
```
- Legacy `exec_spec` sets `unset: vec![]` and `sidecar: None`.
- The WP-01 bodies of `exec_spec_with` and `prepare_with_route` delegate to the legacy forms. A route `Err` plays flat.
- **`run_with` (A5 fix, real now)** uses:
  - `catalog::find_recipe(&paths.recipe_dirs, appid)` instead of `{id}.recipe` (wrapper.rs:131);
  - `switches::read_settings(&paths.settings())` (NotFound → Default, K5);
  - a route closure that returns `Err("not-built: route (WP-31)")`, so every route plays flat until WP-31.

**`install.rs`** (WP-20 writes the bodies):
- `#[derive(Debug, Clone, PartialEq, Eq)] pub struct UnitSpec { pub description: String, pub exec_start: String, pub restart_sec: u32, pub start_limit_interval_sec: u32, pub start_limit_burst: u32, pub wanted_by: String }`, with `impl Default` giving `"Flat to 3D headset link"`, `"%h/.local/bin/flat3d-inject watch"`, `3`, `60`, `10` and `"default.target"`.
- `pub fn unit_text_with(spec: &UnitSpec) -> String`. The stub is `unit_text(&spec.exec_start)` (install.rs:5).
- `pub fn execute(plan: &InstallPlan, r: &mut dyn CommandRunner) -> Result<api::InstallReport, ApiError>` and `pub fn execute_uninstall(plan: &UninstallPlan, r: &mut dyn CommandRunner) -> Result<api::InstallReport, ApiError>`. Both stubs return not-built.
- There is no second report type: `api::InstallReport{written,ran,summary}` (api.rs:255) is the only one.

**`launch_options.rs`**: `pub fn add_wrapper_at(options: &str, wrapper_abs: &str) -> String`. The stub is `add_wrapper(options)` (launch_options.rs:77).

**`cli.rs`** (frozen after this WP):
```rust
pub struct Ctx<'a> { pub paths: &'a EnginePaths, pub runner: &'a mut dyn CommandRunner, pub relay: &'a mut dyn RelayRunner, pub json: bool }
pub struct AnyCli { pub command: AnyCommand, pub json: bool }
pub enum AnyCommand { Engine(Command), Link(LinkCommand), View(ViewCommand), Setup(SetupCommand), Doctor(DoctorCommand) }
pub enum ViewCommand { View, Rows { filter: String }, Launch { add: bool, options: String },
    Collection { current: Option<Vec<u32>> }, Toasts { last_seq: u64 }, Logs { limit: usize } }
pub enum SetupCommand { Packages, InstallApps, LaunchEnv { appid: Option<u32> }, Fetch { key: String } }
pub enum DoctorCommand { Run }
pub fn parse_any(args: &[String]) -> Result<AnyCli, ApiError>
pub fn execute_with(args: &[String], ctx: &mut Ctx<'_>) -> Output
```
- All the enums derive `Debug, Clone, PartialEq, Eq`. There is no `InjectCommand`, and `now` is never a `Ctx` field.
- `parse_any`:
  1. Split at the first argv element exactly equal to `--`.
  2. Remove `--json` from the head only.
  3. Try, in order: `cli::link::parse_link` → `cli::view::parse` → setup (private) → doctor (private) → the locked `parse`. Each parser gets the head plus `--` plus the tail, unchanged.
  - The setup grammar: `setup packages`, `setup install-apps`, `setup launch-env [<appid>]` and `setup fetch <key>`. `launch-env abc` and `fetch` with no key give a usage error.
  - The doctor grammar is `doctor`.
  - A usage error has code `usage` and keeps the locked text.
  - `Logs.limit` defaults to `headset_view::DEFAULT_LOG_LINES` (500).
- `execute(args) -> Output` keeps its signature, and `main` still calls `cli::run`. It:
  1. parses first, so a usage error never needs HOME;
  2. builds `EnginePaths::from_env(&std::env::vars().collect(), sys::current_uid())`, plus a `SystemRunner` and a `ProcessRunner`;
  3. calls the shared dispatch.
- The render name is `Command::name()` for an engine command, and the first head token otherwise.
- **Dispatch:**

  | Command | Goes to |
  |---|---|
  | `List`, `Status` | `cli::engine::run` |
  | `Inject` | `cli::inject::run` |
  | `Undo` | `cli::undo::run` |
  | `Start3d`/`Stop3d` | `cli::link::request_3d(Request::Start3d\|Stop3d, ctx.paths, sys::unix_now())`, serialised with `serde_json::to_value` |
  | Link, View, Setup, Doctor | their module's `run` |

- Per-module entry points (exactly one `run` each):

  | Module | Signature | WP-01 body |
  |---|---|---|
  | `cli/engine.rs`, `cli/inject.rs` | `pub fn run(cmd: &cli::Command, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError>` | `Err(NotBuilt("cli::<m>::run").into())` |
  | `cli/undo.rs` | same | **working**: today's dispatch body, `crate::undo::undo_game(&ctx.paths.data, appid)` |
  | `cli/link.rs` | `run(cmd: &LinkCommand, ctx)`, same return | not-built |
  | `cli/view.rs` | `run(cmd: &ViewCommand, ctx)` | not-built |
  | `cli/setup.rs` | `run(cmd: &SetupCommand, ctx)` | not-built |
  | `cli/doctor.rs` | `run(cmd: &DoctorCommand, ctx)` | not-built |

- The only other pub items under `cli/`:
  - `cli::link::request_3d(r: state::Request, p: &EnginePaths, now: u64) -> Result<api::LinkReport, ApiError>`. The stub returns not-built and spawns nothing (A24).
  - `cli::view::parse(args: &[String]) -> Option<Result<ViewCommand, String>>`. `None` means "not a view verb". The stub accepts at least `view`, and WP-28 fills the grammar.
  - `cli::setup_fetch::setup_fetch(key: &str, p: &EnginePaths, r: &mut dyn CommandRunner) -> Result<api::SetupReport, ApiError>`. The stub returns not-built.
  - `parse_link`, `LinkCommand` and `SettingKey` stay pub because they are locked.
- **Unit test (A23):** replace `execute_human_not_built_goes_to_stderr` (cli.rs:198), which asserts that `list` is unbuilt and is not locked, with `dispatch_routes_each_family`. It builds a `Ctx` from a temp HOME and `FakeRunner`s, then checks:
  - `list` gives `not built yet: cli::engine::run`;
  - `snapshot` reaches `cli::link::run`;
  - `view` reaches `cli::view::run`;
  - `undo 5` returns the undo result or error, not not-built.

**`api.rs`** DTOs:
- Every field is `pub`. Each type derives `Debug, Clone, PartialEq, Serialize, Deserialize`, except `LinkView`, which derives `Debug, Clone, PartialEq, Serialize` only.
  - `LinkView{state:LinkState, label:String, hint:String, start:bool, stop:bool, health:HealthView, rx:String, tx:String, uptime:String, tether:TetherMode, tunnel:bool, auto_3d:bool, adb_source:AdbSource}`
  - `LaunchText{options:String}`
  - `CollectionPlan{name:String, wanted:Vec<u32>, add:Vec<u32>, remove:Vec<u32>}`
  - `LogLines{lines:Vec<String>, text:String}`
  - `DoctorCheck{name:String, ok:bool, detail:String, fix:String}`, `DoctorReport{ok:bool, checks:Vec<DoctorCheck>}`
  - `SetupReport{steps:Vec<String>, summary:String}`
  - `Toast{title:String, body:String}`
- `pub const TOAST_TITLE: &str = "Flat to 3D";` and `pub use crate::face::rows::GameRow;`. The re-export is a named one, which the glob guard allows.
- Add `Serialize` to `HealthView` (face/view.rs:15) and `ToastBatch` (face/toasts.rs:6).
- `InstallReport`, `LinkReport`, `Health`, `Stats`, `EventKind` and `ERROR_CODES` (the 8 locked codes) are unchanged.

**`face`:**
- `rows::GameRow` has the fields and derives exactly as in issue #76. `rows::game_rows(games: &[api::Game], filter: &str, settings: &switches::Settings, link: LinkState) -> Vec<GameRow>` is a stub returning `Vec::new()`; WP-03 writes the body.
- `link_model::link_view(snap: Option<LinkSnapshot>, now: u64) -> api::LinkView` is a stub that returns a Stopped view built from today's `link_view`/`headset_view` helpers; WP-28 writes the body.
- `logs::log_lines(text: &str, max: usize) -> Vec<String>`, `logs::read_logs(p: &EnginePaths, max: usize) -> Vec<String>` and `logs::is_error_line(line: &str) -> bool` are stubs returning `Vec::new()`, `Vec::new()` and `false`. WP-06 writes the bodies.
- The single log limit is `headset_view::DEFAULT_LOG_LINES` (500). There is no `LOG_LIMIT`.

**`testing.rs`** (`#[doc(hidden)] pub mod testing`, path `flat3d_inject::testing::FakeRunner`, full):
- `pub struct FakeRunner { pub ran: Vec<Vec<String>>, /* private reply queue */ }`. `FakeRunner::new()` treats unscripted argv as success with empty stdout and stderr.
- Builders: `reply(self, prefix: &[&str], out: CommandOutput) -> Self`, `ok(self, prefix: &[&str], stdout: &str) -> Self` and `fail(self, prefix: &[&str], stderr: &str) -> Self`.
- The replies form a queue: the first unconsumed entry whose prefix matches the argv is consumed.
- `impl link::CommandRunner` returns that `CommandOutput`.
- `impl RelayRunner`:
  - `run` gives `Ok(stdout)` on success, else `Err(stderr)`;
  - `start_relay` records `["relay", argv.., name]`, and `stop_relay` records `["relay", "stop"]`. Both return `Ok`.
- Every call pushes its argv to `ran`.

**Viewer** (full unless marked stub):
- `frame.rs` (Debug, no Clone, no `PixelFormat`, no `Cpu`; `fourcc` is a DRM fourcc, and NV12 uses two planes):
  ```rust
  pub struct Plane { pub fd: OwnedFd, pub offset: u32, pub stride: u32 }
  pub enum FrameData { Mem { stride: u32, bytes: Vec<u8> }, DmaBuf { planes: Vec<Plane>, modifier: u64 } }
  pub struct Frame { pub width: u32, pub height: u32, pub fourcc: u32, pub seq: u64, pub data: FrameData }
  ```
- `source.rs`:
  ```rust
  pub use crate::frame::Frame;
  pub trait FrameSource { fn next(&mut self, timeout: std::time::Duration) -> Result<Option<Frame>, String>; }
  pub mod test_pattern;
  #[cfg(feature = "capture")] pub mod pipewire;
  #[cfg(feature = "capture")] pub mod gamescope;
  #[cfg(feature = "capture")] pub mod portal;
  ```
- `spec.rs`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
  pub struct ViewerSpec { pub layout: Layout, pub source: Source, pub swap_eyes: bool, pub crop_aspect: Option<f32>, pub screen_width_m: f32, pub distance_m: f32 }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(rename_all = "snake_case")]
  pub enum Layout { #[default] Sbs, Tab, #[serde(rename = "sbs_reversed")] SbsRev, #[serde(rename = "tab_reversed")] TabRev }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(rename_all = "snake_case")]
  pub enum Source { #[default] Auto, Gamescope, Portal, TestPattern }
  impl ViewerSpec { pub fn from_args(args: &[String]) -> Result<ViewerSpec, String>; pub fn to_args(&self) -> Vec<String> }
  ```
  - `to_args` writes `--layout <l> --source <s>`, then `--swap-eyes` when true and `--crop-aspect <f>` when Some.
  - `--screen-width-m <f>` and `--distance-m <f>` are written only when > 0.0. 0.0 means the viewer default, which WP-19 resolves to 3.0 m / 2.5 m.
  - The strings are the serde names. `from_args` returns Err on an unknown flag.
- Stubs that compile empty: `layout.rs`, `status.rs`, `source/*.rs`, `gpu/import.rs` and `xr/*.rs`.
- `main.rs` is `fn main() {}` plus a one-line `eprintln!("flat3d-viewer: not built (WP-19)")` and `std::process::exit(2)`. It builds only with `--features xr,capture`.

**uevr-inject stubs:**
- `args.rs` and `pick.rs` are empty, and `win.rs` is empty.
- `main.rs`:
  - on non-Windows it prints `flat3d-uevr-inject: windows only` and exits 3;
  - on Windows it prints `flat3d-uevr-inject: not built (WP-15)` and exits 5.
- WP-15 overwrites all four bodies. The final exit codes are WP-15's: 2 bad args, 3 not Windows, 4 no game found, 5 inject failure.

**App page stubs** (types exactly as in #116, #117 and #118, so their test-first commits compile):
- `headset_page::{update, Action, HeadsetState, Msg}`. `update` returns `None` and is not `#[must_use]`.
- `setup_page::Step` derives `Debug, Clone, PartialEq, Eq`. `steps` returns `Vec::new()` and `all_done` returns `false`.
- `games_page`: `GamesState::default()` has filter `"all"`.
- `logs_page{LogsState, Msg, Action}`, with no limit constant of its own.
- Msg and Action derive `Debug, Clone, PartialEq`.
- `engine.rs` and `view_*.rs` are empty, and WP-33 fills them.

---

#### Behaviour wired now

- **watch:**
  - The private `read_settings` (watch.rs:826) is removed. Callers use `switches::read_settings(Path::new(&p)).unwrap_or_default()` (A12).
  - `Service` builds `HealthLoop::new(sys::unix_now(), 0, None)`.
  - `publish()` (watch.rs:784) calls `tick(sys::unix_now(), mode, playing, &mut Shared(Rc::clone(&self.runner)))`, pushes `toast` to the event ring, and keeps `heal` in a private field for WP-09 to execute (K8). `current()` takes `&HealthTick`.
- **flat3d-run:**
  - builds `EnginePaths::from_env(&env, sys::current_uid())`;
  - calls `run_with` with a `SystemRunner`;
  - in the launcher, applies `env_remove` for each `unset` and spawns the `sidecar` (when Some) with `--until-pid <pid>`.
- **Doc fixes:** `lib.rs:1-6` (A3), `main.rs:1` (A2) and `injectors.rs:1` (A4).
- **`docs/code-map.md`:**
  - Rewrite every `inject/` → `crates/engine/` and `app/src` → `crates/app/src` path token, using the new module paths (`cli/link.rs`, `face/*.rs`, `tether.rs`, `watch.rs` + `watch/`).
  - Add the `<!-- mv-map -->` … `<!-- /mv-map -->` block holding the bones §2 git-mv map. It is the one exemption in `check.py`.
  - The `cli_view.rs` and `rows.rs` parts become their own rows (`crates/engine/src/cli/view.rs`, `crates/engine/src/face/rows.rs`) marked `planned` (A9/A10).
  - Rows 56, 59 and 70 (as at 2812f64) are written exactly as ``| `crates/engine/src/recipes.rs` | present |``, ``| `crates/engine/src/link.rs` | present |`` and ``| `crates/engine/src/api.rs` | present |``. The locked tests read these cells.
  - Rows for the new stubs say `stub`.
- **`.github/workflows/check.yml`:** exactly bones §2 "CI final shape".
  - Every job uses `runs-on: [self-hosted, linux]` and `dtolnay/rust-toolchain@1.95.0`.
  - Job `rust`:
    - apt via the existing `SUDO=$(command -v sudo || true)` pattern: `build-essential curl ca-certificates pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev libpipewire-0.3-dev libspa-0.2-dev libclang-dev clang shellcheck libarchive-tools`;
    - `cargo fmt --all --check`;
    - `cargo clippy --locked --workspace --all-targets -- -D warnings`;
    - `cargo clippy --locked -p flat3d-viewer --all-targets --features xr,capture -- -D warnings`;
    - `XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked --workspace`;
    - `rustup target add x86_64-pc-windows-msvc && cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings`.
  - Job `py`:
    - `python3 -m py_compile $(git ls-files '*.py')`;
    - `for d in $(git ls-files '*test_*.py'|xargs -n1 dirname|sort -u|grep -vx 'decky/tests'); do python3 -m unittest discover -s "$d" -p 'test_*.py'||exit 1; done`;
    - step `if: hashFiles('decky/tests/test_*.py')!=''` → `cd decky && python3 -m unittest discover -s tests -p 'test_*.py'`;
    - step `if: hashFiles('tools/check.sh')!=''` → `shellcheck $(git ls-files '*.sh')`;
    - step `if: hashFiles('docs/workbench.md')!=''` → `python3 tools/docs/check.py`.
  - Job `decky`: setup-node 22.23.4 and pnpm 10.34.9. Every step carries `if: hashFiles('decky/package.json')!=''`, never at job level. Steps: `pnpm i --frozen-lockfile && pnpm run typecheck && pnpm run build`.

---

#### Tests

**Locked tests:**
- All 55 moved files stay verbatim (`git mv`) except **7 sanctioned path edits in 3 files**. Together they are 14 `-`/`+` diff lines, and all three files stay rustfmt-clean. Line numbers are as at 2812f64; if WP-00's rustfmt shifted a line, find it by content.

  | File:line | Edit |
  |---|---|
  | `api_response.rs:67` | `.join("../docs/code-map.md")` → `.join("../../docs/code-map.md")` |
  | `api_response.rs:69` | `` `inject/src/api.rs` `` → `` `crates/engine/src/api.rs` `` |
  | `link_stop.rs:110` | `.join("../docs/code-map.md")` → `.join("../../docs/code-map.md")` |
  | `link_stop.rs:112` | `` `inject/src/link.rs` `` → `` `crates/engine/src/link.rs` `` |
  | `recipe_load.rs:138` | `.join("..")` → `.join("../..")` |
  | `recipe_load.rs:146` | `.join("..")` → `.join("../..")` |
  | `recipe_load.rs:150` | `` `inject/src/recipes.rs` `` → `` `crates/engine/src/recipes.rs` `` |

- `wrapper_run.rs:206-207` is **not** edited. It is unheadered, and its `CARGO_MANIFEST_DIR/target/wrapper-tests` now resolves under `crates/engine/target/`, which `.gitignore`'s `target/` already covers.
- The 6 unheadered files (`event_ring_wrap`, `launch_options_quotes`, `runs_extra_lines`, `tether_runtime`, `tether_tunnel_review`, `wrapper_run`) are ordinary tests, but WP-01 still leaves them byte-identical.

**New engine tests** (`crates/engine/tests/`):
- `paths.rs`:
  - `from_env(&map, 1000)` with XDG set and with XDG unset;
  - a missing HOME gives code `not-found`;
  - `FLAT3D_SRC_DIR` puts `$FLAT3D_SRC_DIR/recipes` first;
  - one case per accessor: `settings`, `link_log`, `run_log`, `cache` (XDG set and unset), `uevr_cache`, `env_file(5)`, `wrapper_bin`, `viewer_bin`, `pins`, `run_paths` and `watch_paths`.
- `fsx.rs`: the atomic write leaves no tmp file; append creates the parent dirs; `read_opt` on a missing file gives `None`.
- `catalog_find.rs`: exact name; dashed name; the first dir wins; **a missing dir is skipped**; no match gives `Ok(None)`.
- `switches_io.rs`: write then read round-trips; a missing file gives Default.
- `state_io.rs`: a missing snapshot gives `Ok(None)`; `write_request` then `parse_request` round-trips.
- `route_settings.rs`:
  - each value rule, once ok and once bad;
  - an unknown `uevr-x` key is refused;
  - Port with a non-empty map is refused;
  - `route_settings` drops recipe KEYS and `#` lines, and strips one pair of quotes.
- `cli_any.rs`:
  - `run 5 -- a --json` keeps `--json` in the tail and gives `json=false`;
  - `view --json` gives `View` with `json=true`;
  - `logs` gives `Logs{limit: DEFAULT_LOG_LINES}`;
  - `setup launch-env abc` gives code `usage`;
  - `fly` gives code `usage` with the locked USAGE text.

**New viewer test:** `crates/viewer/tests/spec.rs`.
- `to_args`/`from_args` round-trip every `Layout` and `Source`, with `TabRev` ↔ `"tab_reversed"`, `SbsRev` ↔ `"sbs_reversed"` and `TestPattern` ↔ `"test_pattern"`.
- `ViewerSpec::default()` gives 0.0 for both f32 fields, and `to_args` then omits them.
- An unknown flag gives Err.

**Inline facts:**
- iced 0.14.0 needs the `smol` feature.
- openxr 0.22.0 with `loaded` dlopens the loader, so it needs no link library.
- ashpd 0.13.13 has MSRV 1.87.
- A `dep:` feature needs `optional = true` on its dependency.
- In windows-sys 0.61.2, `CreateRemoteThread` needs `Win32_Security` + `Win32_System_Threading`.
- The `x86_64-pc-windows-msvc` target is already installed on the host, so `clippy --target` needs no download. Linking happens only in `tools/build-win.sh` (WP-15).
- The `capture` feature is not built on the host, which lacks the pipewire, spa and clang headers. CI checks it.
- `decky/main.py:3` still names `inject/src/cli.rs`. That is left for WP-21, which owns the file.

---

**Done-check** (run after committing on `v2/wp-01-workspace`):
```sh
export CARGO_TARGET_DIR=$SCRATCH/target
B=$(git merge-base main HEAD)
test ! -e inject && test ! -e app \
&& test -z "$(git ls-files '*mod.rs')" \
&& cargo metadata --locked --offline --format-version 1 >/dev/null \
&& cargo fmt --all --check \
&& cargo clippy --locked --workspace --all-targets -- -D warnings \
&& cargo clippy --locked -p flat3d-viewer --all-targets --features xr -- -D warnings \
&& XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked --workspace \
&& XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test tether_adb_runner --test tether_runtime --test cli_json \
&& cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings \
&& git diff --exit-code Cargo.lock \
&& test "$(git ls-tree --name-only "$B" inject/tests/ | grep -c '\.rs$')" = 55 \
&& for f in $(git ls-tree --name-only "$B" inject/tests/); do test -f "crates/engine/tests/${f#inject/tests/}" || exit 1; done \
&& for t in paths fsx catalog_find switches_io state_io route_settings cli_any; do test -f "crates/engine/tests/$t.rs" || exit 1; done \
&& test -f crates/viewer/tests/spec.rs \
&& test "$(for f in $(git ls-tree --name-only "$B" inject/tests/); do git show "$B:$f" | diff -u - "crates/engine/tests/${f#inject/tests/}"; done | grep -c '^[-+][^-+]')" = 14
```
- `$B` is the merge-base, so the check stays valid after `main` moves.
- The last line proves that exactly the 7 sanctioned edits (14 diff lines) touched the 55 moved test files.
- Python, shellcheck and decky are not run here. Their CI steps are guarded until WP-02, WP-21 and WP-25b land.

### WP-02: Repo tooling: check.sh, pins, pin.sh, issues report

**Closes:** one local gate, TOFU pinning of downloads, and the issue readiness report.

**Files (exclusive):**
- New:
  - `tools/check.sh`
  - `packaging/{pins,pin.sh,README.md}`
  - `packaging/tests/test_pins.py`
  - `tools/pm/issues_report.py`
  - `tools/pm/tests/test_issues_report.py`
  - `tools/pm/tests/fixtures/issues.json`
  - The WP index must use these paths too: not `tools/pm/test_issues_report.py`, not `tools/pm/fixtures/`.
- Edited:
  - `tools/ghidra/headless.sh`: quote `"$P"` in `kill -0 $P` and `wait $P`, for SC2086.
  - `.gitignore`.

**Signatures:**
```
tools/check.sh                      # no args; run from repo root
packaging/pin.sh <key> [--file <path>] [--check] [--dry-run] [--pins <path>]
packaging/pin.sh --all-tbd [--dry-run] [--pins <path>]
tools/pm/issues_report.py [--json-file <path>] [--json] [--device-label <name>]
```

**Behaviour:**

`check.sh`:
- Uses `set -euo pipefail` and runs from `git rev-parse --show-toplevel`.
- Every step is guarded by file existence, so it works before and after WP-01. With no root `Cargo.toml`, it prints `skip: rust (no workspace yet)`.
- Steps, in order:
  1. `cargo fmt --all --check`
  2. `cargo clippy --locked --workspace --all-targets -- -D warnings`
  3. `cargo clippy --locked -p flat3d-viewer --features xr -- -D warnings`
  4. `--features xr,capture`, only if `pkg-config --exists libpipewire-0.3`; otherwise it prints `notice: capture needs libpipewire-0.3, run in the distrobox build container`.
  5. msvc clippy of uevr-inject, if `rustup target list --installed | grep -qx x86_64-pc-windows-msvc`.
  6. `XDG_RUNTIME_DIR=$(mktemp -d)` with `trap 'rm -rf "$rt"' EXIT`, then `cargo test --locked --workspace`.
  7. `python3 -W error -m py_compile $(git ls-files '*.py')`.
  8. CI's loop, the same as in check.yml:
     ```sh
     for d in $(git ls-files '*test_*.py' | xargs -n1 dirname | sort -u); do python3 -m unittest discover -s "$d"; done
     ```
     This covers `packaging/tests`, `tools/pm/tests`, `tools/session`, `tools/stereo`, `tools/docs`, `tools/re` and any later folder.
  9. `shellcheck $(git ls-files '*.sh')`, or the notice `shellcheck missing`.
  10. `tools/docs/check.py`, if present.
  11. `pnpm -C decky install --frozen-lockfile && pnpm -C decky build`, if `decky/package.json` exists and `pnpm` is installed.
  12. `git diff --exit-code -- Cargo.lock`.
- Respects any `CARGO_TARGET_DIR` already set.

`packaging/pins`:
- Holds the pre-pass pin table (see Flags), one `key: value` per line; a value is everything after the first `: `.
- `fetch: auto` keys need `version`, `url`, `size`, `sha256` (64 hex or `TBD-at-pin`) and `license`.
- `fetch: manual` keys need `page` and `license`.
- `crate.*` and `npm.*` keys need only `version`.
- A comment is a whole line starting with `#`. There are no inline comments.

`pin.sh`:
1. Parse the pins file. Exit 2 for a bad pins file, a bad key or bad usage.
2. A manual key exits 3 with the page URL.
3. A hex sha without `--check` exits 5 (already pinned).
4. Download:
   - Command: `curl -fL --proto '=https' --proto-redir '=https' --tlsv1.2 --retry 3 --max-filesize $((size+1)) -o "$tmp/f"`.
   - `tmp=$(mktemp -d "${XDG_CACHE_HOME:-$HOME/.cache}/flat3d/pin.XXXXXX")`, removed by a trap.
   - `--file <path>` skips the download.
   - `PIN_ALLOW_FILE=1` changes the protocol flag to `--proto '=https,file'`. Without it, a `file://` URL exits 3.
   - A network failure exits 3.
5. Verify:
   - size with `stat -c %s`;
   - md5 with `md5sum` (when the pin has `md5`);
   - for a `https://github.com/<o>/<r>/releases/download/<tag>/<name>` URL with `gh` installed:
     ```sh
     gh api repos/<o>/<r>/releases/tags/<tag> --jq '.assets[]|select(.name=="<name>")|.digest'
     ```
     must equal `sha256:<hex>`.
   - Any mismatch exits 4 and leaves the file byte-identical.
6. Write:
   - `--check` exits 0 without writing.
   - `--dry-run` prints the new lines.
   - Otherwise rewrite only `<key>.sha256` and `<key>.pinned: <YYYY-MM-DD> tofu size+md5[+gh-digest]` (append or replace), via awk into a temp file in the same dir, then `mv`.
7. Never extract, run or `git add` anything. Print `review: git diff --stat -- packaging/pins`.
- `--all-tbd` runs every auto key whose sha is `TBD-at-pin` and exits with the highest code.

`issues_report.py`:
- Input: `gh issue list --state open --limit 500 --json number,title,body,labels`, or `--json-file`.
- Find blockers with the regex `[Bb]locked by:?\**\s*(.*)` (rest of the line), then take the `#(\d+)` numbers from it.
- Status per issue:
  - `ready`: no blocker line, an empty body, or `none…`.
  - `blocked`: an open `#n`.
  - A `#n` that is not in the open set is treated as closed.
  - `blocked-external`: text with no `#`, e.g. `Step 0 (SPEC M1) passing on 0.3.8`.
- `owner-device`: the label (default `needs-owner-device`).
- Output: a markdown table `# | title | status | blockers | device`, or `--json`.

**Tests (all new, stdlib `unittest` style so the discover loop finds them):**
- `packaging/tests/test_pins.py` uses a temp copy of the pins file, `--pins`, and `PIN_ALLOW_FILE=1` with `file://` fixtures. Cases:
  - the schema of the real pins file;
  - TBD filled: sha and the pinned line are written, and no other line changes;
  - size mismatch → 4, file unchanged;
  - md5 mismatch → 4;
  - already pinned → 5;
  - `--check` on a good file → 0;
  - manual key → 3;
  - `file://` without `PIN_ALLOW_FILE` → 3;
  - unknown key → 2;
  - `--dry-run` → no change.
- `tools/pm/tests/test_issues_report.py`, which reads `tools/pm/tests/fixtures/issues.json`:
  - The fixture holds all 3 blocker formats (`Blocked by: #130, #113`, `**Blocked by:** Step 0…`, `Blocked by: none (…)`), an empty body, a closed blocker and the device label.
  - Each issue gets its expected status; the JSON shape is checked.

**Inline facts:**
- `.gitignore` already has `build/ dist/ node_modules/ target/`.
- `.gitignore` additions: `decky/node_modules/`, `decky/dist/`, `crates/*/Cargo.lock`.
- shellcheck is not installed on the host.
- The host has Python 3.14.6, and all 14 tracked `.py` files pass `-W error`.
- No tracked `test_*.py` exists yet.
- Python 3.12+ `unittest discover` exits 5 when a folder has no tests, so pytest-only test files fail the loop loudly.
- `tools/build-spidy.sh` already uses `set -euo pipefail` and needs no change.
- The label `needs-owner-device` does not exist yet; 65 issues are open.
- `packaging/README.md` covers:
  - the pins format;
  - pin.sh usage and exit codes;
  - the rule not to redistribute binaries (users fetch them);
  - the KillMode note.

**Flags:**
- `Flag:` **needs network research (pre-pass, metadata only).** No scratch report has the pin table for adb (platform-tools), gnirehtet, UEVR, .NET and Geo-11.
  - Collect `version`, `url`, `size` and `license` from GitHub release-API JSON (`repos/<o>/<r>/releases/tags/<tag>`: `.assets[].{name,size,browser_download_url,digest}`) or vendor pages.
  - Set `sha256` from `.digest` where GitHub gives one, else `TBD-at-pin`.
  - No binary downloads in the pre-pass.
  - Attach the table to the roadmap before WP-02 starts.
- `Flag:` exit codes follow the WP index (3 = fetch/manual, 5 = already pinned). The research used 3 = manual and 5 = network.
- `Flag:` the research pins put an inline `# …` comment after `crate.ashpd`. Move it to its own line, or the parser reads it as part of the value.
- `Flag:` `.gitignore` already covers `dist/` and `node_modules/`. The explicit lines are only documentation.
- `Flag:` `headless.sh` is edited again by WP-25 and WP-30 (`FLAT3D_HOME`, `qwen.lock`, `docs/sm2-address-map.md` renames). WP-02 touches only the `"$P"` quoting.
- `Flag:` the full `tools/check.sh` run needs WP-01 merged. Before that, its Rust steps skip.
- `Flag:` the WPs that add tests under `tools/session`, `tools/stereo`, `tools/docs` and `tools/re` must write them as `unittest.TestCase`. Plain pytest functions are not collected.

**Depends on:** none to land. Its full gate needs WP-01. Its pins content needs the network pre-pass.
**Owner device:** none (host).

**Done-check (before WP-01):**
```sh
bash -n tools/check.sh packaging/pin.sh tools/ghidra/headless.sh \
&& python3 -W error -m unittest discover -s packaging/tests \
&& python3 -W error -m unittest discover -s tools/pm/tests \
&& tools/check.sh
```
After WP-01, the gate is `tools/check.sh` exiting 0 with no `skip: rust` line.

Review note: rejected the [build] fix "`pub use crate::route::RouteSettings` in recipes" because no `route` module exists in the repo (inject/src has no route.rs) or in WP-01's file list. Used `pub type RouteSettings = BTreeMap<String,String>` in `recipes.rs` instead, which matches what `route_settings()` already returns.
Review note: rejected "`#[serde(rename_all="snake_case")]` on Layout" as the whole fix because snake_case turns `SbsRev` into `sbs_rev`, not `sbs_reversed`. Kept `rename_all` and added `#[serde(rename="sbs_reversed")]` on `SbsRev`.
Review note: rejected the [locked] "§0/WP-01 = 9 path lines, done-check 18" count because the accepted correction that drops `wrapper_run.rs:206-207` (CARGO_MANIFEST_DIR/target is gitignored by `.gitignore` `target/`) leaves 7 lines (api_response.rs:67,69; link_stop.rs:110,112; recipe_load.rs:138,146,150). The done-check expects 14. Note: recipe_load's first `..` is on line 138, not 137.
Review note: rejected [parallel] "`for d in tools/*/; pytest $d`" because pytest is not stdlib and that glob misses `packaging/tests`. Used the [build] fix's `git ls-files '*test_*.py'` + `unittest discover` loop in both check.sh and check.yml.
Review note: rejected [parallel] "drop uevr-inject lib.rs or make WP-15 own it" because mod lines are WP-01-only and WP-15's `tests/args.rs` needs a lib target. lib.rs stays in WP-01, and WP-15's "no lib.rs" text is what must change.
Review note: rejected [parallel] "`cargo generate-lockfile --offline`" because generate-lockfile re-resolves every package and discards the pins seeded from `app/Cargo.lock`. Used seed + `cargo update --workspace --offline` (index-metadata fallback) instead.
Review note: rejected [parallel] "similarity >=90" as the locked-test check because a similarity threshold cannot prove that only the 7 sanctioned lines changed. Kept the rename count (55 `R` lines), the new-file count (7 `A` lines) and the exact per-file content diff count (14).

### WP-03: GameRow rows view (#76)
**Closes:** #76 (E10c). SPEC M11, M24.
**Files (exclusive):** `crates/engine/src/face/rows.rs` (WP-01 made the stub), `crates/engine/tests/rows.rs` (locked).
**Flag:** the scope line in the computed task (`rows(games,&[Recipe],settings,&LinkSnapshot)`) does not match the locked test. The locked test calls `flat3d_inject::rows::game_rows(&[Game], &str, &Settings, LinkState)`, and the locked test wins (R28). `has_recipe` comes from `api::Game.has_recipe` (api.rs:11), so no `recipes` parameter is needed.
**Flag:** the issue body is not rustfmt-clean: rustfmt changes 68 diff lines (checked with `rustfmt --emit stdout` on stdin). Commit 1 is the body verbatim. Commit 2 is format-only, `cargo fmt -p flat3d-inject`, which owner answer 1 covers. Without commit 2, CI `cargo fmt --all --check` fails.
**Input:** the issue snapshot `$SCRATCH/issues/76.md`, which the WP-00 pre-step writes (see the extract line under Tests). The WP itself makes no network call. The snapshot did not exist at review time (`ls $SCRATCH/issues` returned ENOENT). If the pre-step has not run, the WP needs `gh` read access (see the fallback).

**Signatures** (checked at main@2812f64; paths before the move):
```rust
//! The game rows the faces draw. docs/code-map.md, API shape, GameRow.
use serde::Serialize;
use crate::api::{Game, LinkState};
use crate::switches::Settings;
use crate::status::{filter_by_status, sort_games, status_colour, status_label}; // status.rs stays at src root (bones tree)
use crate::link_view::flat_reason;                                              // via lib.rs re-export of face::link_view
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameRow { pub appid: u32, pub name: String, pub status: String, pub status_label: String,
                     pub status_colour: String, pub switch_on: bool, pub plays_3d: bool, pub flat_reason: Option<String> }
pub fn game_rows(games: &[Game], filter: &str, settings: &Settings, link: LinkState) -> Vec<GameRow>;
```
Existing helpers:
- `status_label(&str)->&'static str` (status.rs:14)
- `status_colour(&str)->&'static str` (status.rs:26)
- `sort_games(&[Game])->Vec<Game>` (status.rs:67)
- `filter_by_status(&[Game],&str)->Vec<Game>` (status.rs:73; `"all"` keeps every game)
- `flat_reason(LinkState, switch_on, has_recipe, &str)->Option<String>` (link_view.rs:29)
- `Settings::game_on(&self,u32)->bool` (switches.rs:27)
- `Settings{auto_3d,tunnel,off:BTreeSet<u32>}` (switches.rs:9)
- `Game{appid,name,install_dir,status,has_recipe}` (api.rs:6)

Keep the `GameRow` struct exactly as WP-01 froze it. `api.rs` already has `pub use crate::face::rows::GameRow`.

**Behaviour:**
- `sort_games(&filter_by_status(games, filter))`, then map each game:
  - `switch_on = settings.game_on(g.appid)`;
  - `status_label` and `status_colour` from the helpers, `.to_string()`;
  - `flat_reason = flat_reason(link, switch_on, g.has_recipe, &g.status)`;
  - `plays_3d = flat_reason.is_none()`.
- Pure: no IO, no clock. About 30 lines.
- Do not change any `flat_reason` text (K28; unifying them is a follow-up issue).

**Tests (locked, verbatim):** `crates/engine/tests/rows.rs`, from the `## Tests (locked)` block of #76.
- 6 tests: `rows_are_sorted_with_label_and_colour`, `the_filter_keeps_one_status`, `only_a_works_now_game_with_a_recipe_a_switch_and_a_ready_link_plays_in_3d`, `the_switch_comes_from_the_settings`, `a_link_that_is_not_ready_is_the_reason`, `the_json_has_exactly_these_keys`.
- Extract it from the snapshot with:
  ~~~
  awk '/^## Tests \(locked\)/{f=1} f&&/^```rust/{g=1;next} g&&/^```/{exit} g' "$SCRATCH/issues/76.md" > crates/engine/tests/rows.rs
  ~~~
  Before fmt, sha256 must be `f7f72d10415a1d7fd27bcfe26515b7e57375694ef8105ba4975f2aab78c4c1b4`, with no CR characters. If they differ, the snapshot is stale or damaged: stop and redo the WP-00 pre-step. Do not edit the file by hand.
- Fallback, only if the snapshot is missing (requires `gh` read access): pipe `gh issue view 76 --json body -q .body` into the same `awk` program in place of the file.
- Imports: `flat3d_inject::api::{Game, LinkState}`, `flat3d_inject::rows::game_rows`, `flat3d_inject::switches::Settings`, and `serde_json`, which is a normal engine dependency (inject/Cargo.toml).
- No new tests.

**Inline facts:**
- Colours: works-now `#2e9e5b`, likely `#8bc34a`, untested `#b0b0b0` (status.rs:28-34).
- Labels: "Works now", "Likely", "Untested".
- Reason texts, checked in this order (link_view.rs:31-47): `"No recipe yet"` → `"Status is {label}"` → `"3D switch is off"` → `"3D is not ready: {link_label.to_lowercase()}"` (for example "no headset", link_view.rs:8).
- JSON keys, sorted: `appid flat_reason name plays_3d status status_colour status_label switch_on`. `flat_reason` is `null` when it is `None`; there is no `skip_serializing_if`.

**Depends on:** WP-01; WP-00 pre-step (issue snapshot). **Owner device:** none.
**Done-check:**
```
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test rows && cargo fmt --all --check && cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings && tools/check.sh
```

---

### WP-04: App Headset page (#116)
**Closes:** #116 (A1). SPEC M24.
**Files (exclusive):** `crates/app/src/headset_page.rs` (WP-01 made the stub, and `lib.rs` already has `pub mod headset_page;`), `crates/app/tests/headset_page.rs` (locked).
**Flag:** rustfmt changes 32 diff lines of the issue body. Commit it verbatim, then a format-only commit (`cargo fmt -p flat3d-app`).
**Flag:** the host has no `pkg-config` (buildenv §1). If the iced build fails on the host, run the done-check inside `distrobox enter craft-build -- …`. CI has the apt libraries.
**Input:** the issue snapshot `$SCRATCH/issues/116.md`, which the WP-00 pre-step writes. The WP makes no network call.

**Signatures:**
```rust
//! The Headset page: state, messages, update. docs/code-map.md, Parts, crates/app/src.
use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::headset_view::headset_buttons;            // headset_view.rs:13 -> Buttons{start,stop} (:8)
#[derive(Debug, Clone, Default, PartialEq)]                  // LinkSnapshot is PartialEq, not Eq (api.rs:199)
pub struct HeadsetState { pub snapshot: Option<LinkSnapshot>, pub busy: bool, pub error: Option<String> }
#[derive(Debug, Clone, PartialEq)]
pub enum Msg { Snapshot(Result<LinkSnapshot, String>), StartPressed, StopPressed, Done(Result<(), String>), SetAuto3d(bool), SetTunnel(bool) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action { Start3d, Stop3d, SetSetting { key: &'static str, on: bool }, Refresh }
pub fn update(state: &mut HeadsetState, msg: Msg) -> Option<Action>;
```

**Behaviour** (from the #116 brief). `link = state.snapshot.as_ref().map(|s| s.state).unwrap_or(LinkState::Stopped)`.
- `Snapshot(Ok(s))`: store it, set `error=None`, return `None`.
- `Snapshot(Err(e))`: set `error=Some(e)`, keep the old snapshot, return `None`.
- `StartPressed`: if `busy || !headset_buttons(link).start`, return `None`. Otherwise set `busy=true` and return `Some(Start3d)`.
- `StopPressed`: the same with `.stop` and `Stop3d`.
  - Buttons rule: up = Starting|Ready, so start=!up and stop=up (headset_view.rs:14-17).
- `Done(Ok(()))`: `busy=false`, `error=None`, return `Some(Refresh)`.
- `Done(Err(e))`: `busy=false`, `error=Some(e)`, return `None`.
- `SetAuto3d(on)`: if a snapshot exists, set its `auto_3d=on`. Return `Some(SetSetting{key:"auto-3d",on})`.
- `SetTunnel(on)`: change nothing (`snapshot.tunnel` means traffic is flowing, not the switch). Return `Some(SetSetting{key:"tunnel",on})`.
- Actions are requests only. WP-33's `engine.rs` writes the request file (`state::write_request`) or the settings file (`switches::write_settings`). The page never runs adb and does no IO.

**Tests (locked, verbatim):** `crates/app/tests/headset_page.rs`, from #116 (the issue prose path `app/tests/` becomes `crates/app/tests/`; the body is unchanged).
- 9 tests: `a_new_page_has_nothing`, `a_snapshot_is_stored_and_clears_the_error`, `a_failed_read_keeps_the_old_snapshot`, `start_runs_once_when_the_link_is_down`, `start_with_no_snapshot_counts_as_stopped`, `start_is_ignored_while_the_link_is_up_and_stop_while_it_is_down`, `done_frees_the_buttons_and_refreshes`, `a_failed_command_shows_the_error`, `the_switches_ask_for_a_setting`.
- Extract it with the WP-03 `awk` command on `"$SCRATCH/issues/116.md"`, writing to `crates/app/tests/headset_page.rs`. sha256 must be `9858cfe6e7d7b473ace5b092f0c3d8999394db51950799969dadac009c19b3d9`. If the hash differs, redo the pre-step. If the snapshot is missing, use the WP-03 `gh` fallback with `116` (requires `gh` read access).
- Imports: `flat3d_app::headset_page::{update, Action, HeadsetState, Msg}`, `flat3d_inject::api::{LinkSnapshot, LinkState}`. It uses `LinkSnapshot::stopped(5)` (api.rs:216).
- No new tests.

**Inline facts:** the setting keys are exactly `"auto-3d"` and `"tunnel"` (the code-map `SettingState` row).
**Depends on:** WP-01; WP-00 pre-step (issue snapshot). **Owner device:** none.
**Done-check:**
```
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-app --test headset_page && cargo clippy --locked -p flat3d-app --all-targets -- -D warnings
```

---

### WP-05: App Setup page (#118)
**Closes:** #118 (A3). SPEC M24.
**Files (exclusive):** `crates/app/src/setup_page.rs`, `crates/app/tests/setup_page.rs` (locked).
**Flag:** rustfmt changes 45 diff lines of the issue body. Commit it verbatim, then a format-only commit.
**Flag:** the locked surface is only `Step`, `steps` and `all_done`. The scope adds `SetupState`, `Msg`, `Action::RunSetup(kind)` and `update`. These are additions the lock does not test, so their tests go in an inline `#[cfg(test)] mod tests` in `setup_page.rs`, because the WP owns only the locked test file.
**Flag:** the step-1 hint in the brief says "install Quest Tether", which is stale under ruling 30 (tether is ported in). It is kept verbatim (the brief is the spec, and the test checks only that `v[5].hint` is non-empty). Follow-up for WP-34.
**Input:** the issue snapshot `$SCRATCH/issues/118.md`, which the WP-00 pre-step writes. The WP makes no network call.

**Signatures:**
```rust
//! The Setup page checklist. docs/code-map.md, Parts, crates/app/src.
use flat3d_inject::api::{AdbSource, LinkSnapshot, LinkState};   // AdbSource api.rs:149 (Copy, Eq, kebab-case)
use flat3d_inject::setup::Packages;                             // setup.rs:17-21 {wivrn,gnirehtet}: Debug,Clone,Copy,PartialEq,Eq
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step { pub title: &'static str, pub done: bool, pub manual: bool, pub hint: &'static str }
pub fn steps(packages: Option<&Packages>, snapshot: Option<&LinkSnapshot>) -> Vec<Step>;
pub fn all_done(steps: &[Step]) -> bool;                        // steps.iter().filter(|s| !s.manual).all(|s| s.done)
// additive (scope):
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum SetupKind { Packages, InstallApps }
impl SetupKind { pub fn args(self) -> &'static [&'static str] } // ["setup","packages","--json"] | ["setup","install-apps","--json"]
pub fn adb_source_text(snapshot: Option<&LinkSnapshot>) -> &'static str;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetupState { pub packages: Option<Packages>, pub snapshot: Option<LinkSnapshot>, pub busy: bool, pub error: Option<String> }
#[derive(Debug, Clone, PartialEq)]
pub enum Msg { Packages(Result<Packages, String>), Snapshot(Result<LinkSnapshot, String>), RunPressed(SetupKind), Done(Result<(), String>) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Action { RunSetup(SetupKind), Refresh }
pub fn update(state: &mut SetupState, msg: Msg) -> Option<Action>;
```

**Behaviour.** `steps` returns exactly 6 steps in this order. `link` is the snapshot's state, or none when there is no snapshot.
1. `"adb found"`: done when `snap.adb_source != AdbSource::Missing`; not done with no snapshot. Hint: `"Run flat3d-inject install, or install Quest Tether."`
2. `"Headset plugged in"`: done when link is Unauthorised, Starting or Ready. Hint: `"Plug in the Quest with a USB cable."`
3. `"USB debugging allowed"`: done when link is Starting or Ready. Hint: `"Put the headset on and allow USB debugging."`
4. `"WiVRn on the headset"`: done when `packages.wivrn`. Hint: `"Install WiVRn on the Quest."`
5. `"Tunnel app on the headset"`: done when `packages.gnirehtet`. Hint: `"Install the gnirehtet app on the Quest."`
6. `"Seated boundary"`: `manual:true`, `done:false` always. Hint: `"Set a seated boundary in the headset; this cannot be checked from the PC."`

Steps 1-5 have `manual:false`.

`adb_source_text` returns:
- Running: `"Using the adb that is already running"`
- Bundled: `"Using the bundled adb"`
- Missing: `"No adb found"`
- None: `"Unknown until the link service runs"`

`update`:
- `Packages(Ok)` and `Snapshot(Ok)`: store, clear `error`, return `None`.
- `Packages(Err)` and `Snapshot(Err)`: set `error`, keep the old value, return `None`.
- `RunPressed(k)`: if busy, return `None`. Otherwise set busy and return `Some(RunSetup(k))`.
- `Done(Ok)`: `busy=false`, return `Some(Refresh)`.
- `Done(Err(e))`: `busy=false`, `error=Some(e)`, return `None`.

WP-33 maps `RunSetup(k)` to `flat3d-inject <k.args()>` (T14; WP-29 implements `setup packages|install-apps`). That makes it the producer of `Packages`, which issues.md:14 found missing. The page does no IO.

**Tests:**
- Locked, verbatim: `crates/app/tests/setup_page.rs`, from #118. Extract it with the WP-03 `awk` command on `"$SCRATCH/issues/118.md"`. sha256 must be `894496afb6de33af1f6297c06ddf256cf2588a8d65947626373ef6d4c9c1b824`. If the hash differs, redo the pre-step. If the snapshot is missing, use the WP-03 `gh` fallback with `118` (requires `gh` read access).
  - 9 tests: `six_steps_in_order`, `with_nothing_known_nothing_is_done`, `a_ready_headset_with_both_apps_is_all_done_but_the_boundary`, `an_unauthorised_headset_is_plugged_in_but_not_allowed`, `starting_counts_as_allowed`, `no_headset_and_a_stopped_service_are_not_plugged_in`, `a_missing_adb_is_not_found`, `each_app_is_checked_alone`, `only_the_boundary_is_manual`.
  - Imports: `flat3d_app::setup_page::{all_done, steps}` and `::Step`, `api::{AdbSource,LinkSnapshot,LinkState}`, `setup::Packages`. It uses `const BOTH: Packages = Packages{..}`, which needs pub fields (present).
- New, inline `mod tests`:
  - `run_pressed_once_while_busy`;
  - `done_err_sets_error_and_frees`;
  - `failed_packages_read_keeps_old`;
  - `kind_args_exact` (both argv slices);
  - `adb_source_text_all_four`.

**Depends on:** WP-01; WP-00 pre-step (issue snapshot). **Owner device:** none.
**Done-check:**
```
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-app --test setup_page --lib && cargo clippy --locked -p flat3d-app --all-targets -- -D warnings
```

---

### WP-06: Logs view model + app Logs page
**Closes:** the logs gap (part of #121/#122; the `LogLines` DTO from code-map.md:149, which engine-b.md:70 found missing).
**Files (exclusive):** `crates/engine/src/face/logs.rs` (WP-01 made the stub; `face.rs` has `pub mod logs;`), `crates/engine/tests/face_logs.rs`, `crates/app/src/logs_page.rs` (WP-01 made the stub), `crates/app/tests/logs_page.rs`.
**Frozen by WP-01 (do not change these signatures):**
- `read_logs(p:&EnginePaths, max:usize)->Vec<String>` does the I/O.
- `log_lines(text:&str, limit:usize)->Vec<String>` is pure.
- `is_error_line(&str)->bool`.

WP-01 exports `face::logs::{log_lines, read_logs, is_error_line}`. WP-28's `logs` command calls `read_logs(&p, DEFAULT_LOG_LINES)` and wraps the result in `api::LogLines{lines, text: log_copy_text(&lines)}` for JSON. WP-28 must not keep its own `log_lines(text,limit)` that reads only link.log; it reuses this one.
**Flag (paths):** `read_logs` takes every path from `EnginePaths` (roadmap.md:200-203):
- `p.link_log()` and `p.run_log()` sit under `p.log_dir` = `$HOME/.local/state/flat3d` (writers: watch.rs:323 and flat3d-run.rs:34).
- `p.runs()` is `<p.state_dir>/runs`. `p.state_dir` is the **runtime** dir `$XDG_RUNTIME_DIR/flat3d` or `/tmp/flat3d-<uid>` (state.rs:86, state.rs:107).

Do not mix up `state_dir` (runtime) with `log_dir` (XDG state). The doc comment must say this.

**Signatures (engine):**
```rust
//! The log lines both faces show. docs/code-map.md, API shape, LogLines.
use crate::paths::EnginePaths;
pub const TAIL_BYTES: u64 = 64 * 1024;
pub const ERROR_WORDS: [&str; 6] = ["error", "failed", "could not", "refused", "not found", "exec:"];
/// Pure: newest first, '\r' stripped, blank lines dropped, truncated to `limit`.
/// Delegates to headset_view::log_lines_newest_first (headset_view.rs:23-33; its locked tests inject/tests/headset_view.rs:26-40 stay).
pub fn log_lines(text: &str, limit: usize) -> Vec<String>;
/// I/O: link.log + run.log (log_dir) + run records (state_dir/runs), merged newest first, at most `max`.
pub fn read_logs(p: &EnginePaths, max: usize) -> Vec<String>;
pub fn is_error_line(line: &str) -> bool;   // lowercase contains any ERROR_WORDS
```

**Behaviour (`read_logs`):**
1. **Read each log file.** For each `(path, tag)` in `[(p.link_log(), "link"), (p.run_log(), "run")]`:
   - Open the file. If that fails (missing, is a dir, EACCES), skip it.
   - If len > `TAIL_BYTES`, seek to `len-TAIL_BYTES` and drop the first partial line.
   - Decode with `String::from_utf8_lossy`. Split it with the same rule as `log_lines` (split on `\n`, strip `\r`, drop blank lines), but keep file order for the ts step.
   - Each line's ts is the first space-separated token as `u64`. The writers use the formats `"{now} {line}"` (watch.rs:814) and `"{now} appid=.. pid=.. msg"` (wrapper.rs:170).
   - If the ts doesn't parse, inherit the previous line's ts in that file (0 at the start) and keep the whole line as the rest.
   - Output `"{ts} {tag}: {rest}"`.
2. **Read the run records.** `read_dir(p.runs())` (skip on error), entries named `run-*.txt`:
   - Read the file. Parse it with `runs::parse_run_record` (runs.rs:38; header `flat3d-run 1`, tab-separated `appid/pid/mode`) and skip `None`.
   - ts = `metadata.modified()` as UNIX seconds (skip on error).
   - Output `"{ts} record: appid={a} pid={p} mode={3d|flat}"`.
3. **Merge.** Key = (ts desc, seq desc), where seq is the global read order (link, then run, then records). Truncate to `max` and return the `Vec<String>`. `read_logs` builds no `text`; callers that need it use `headset_view::log_copy_text` (headset_view.rs:35-37), which is `join("\n")`.

No journalctl, no clock, no writes.

**Signatures (app):**
```rust
//! The Logs page: state, messages, update. docs/code-map.md, Parts, crates/app/src.
use flat3d_inject::face::logs::is_error_line;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub enum Filter { #[default] All, Errors }
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogsState { pub lines: Vec<String>, pub filter: Filter, pub loading: bool, pub error: Option<String> }
#[derive(Debug, Clone, PartialEq)] pub enum Msg { Refresh, Lines(Vec<String>), Failed(String), SetFilter(Filter), CopyPressed }
#[derive(Debug, Clone, PartialEq, Eq)] pub enum Action { Load, Copy(String) }
pub fn update(state: &mut LogsState, msg: Msg) -> Option<Action>;
pub fn visible(state: &LogsState) -> Vec<&str>;   // Errors -> is_error_line
```
`Msg::Lines(Vec<String>)` is the name WP-33 matches on (`logs_page::Msg::Lines`). WP-33's `engine.rs` sends `Lines(read_logs(&p, DEFAULT_LOG_LINES))`. When it cannot build `EnginePaths`, or a CLI call fails, it sends `Failed(e)`. `Failed` keeps the old error path that `Loaded(Err)` covered.

`update`:
- `Refresh`: if loading, return `None`. Otherwise set `loading=true` and return `Some(Load)`.
- `Lines(v)`: set `lines=v`, `error=None`, `loading=false`, return `None`.
- `Failed(e)`: keep the lines, set `error=Some(e)`, `loading=false`, return `None`.
- `SetFilter(f)`: store it, return `None`.
- `CopyPressed`: if `visible` is empty, return `None`. Otherwise return `Some(Copy(visible.join("\n")))`. WP-33 turns that into `iced::clipboard::write` (iced-packaging.md:20).

**Tests (new):**
- `crates/engine/tests/face_logs.rs`. Build `EnginePaths` as a struct literal (pub fields), with `log_dir` and `state_dir` under `env!("CARGO_TARGET_TMPDIR")/face_logs/<test>/{state,rt}`. Set record mtimes with `File::set_modified(UNIX_EPOCH+Duration::from_secs(t))`. Cases:
  - `merges_three_sources_newest_first`: link `100 a` / `300 c`, run `200 appid=1 pid=2 b`, record mtime 250 → order `300 link: c`, `250 record: appid=1 pid=2 mode=3d`, `200 run: …`, `100 link: a`;
  - `max_truncates_and_blanks_drop`;
  - `untimed_line_inherits_previous_ts`;
  - `missing_dirs_give_empty`;
  - `link_log_as_dir_and_garbage_record_are_skipped`;
  - `tail_reads_last_64k_without_partial_line`;
  - `log_lines_matches_newest_first`: `log_lines("a\r\n\n b\n",500)==["b"... ]` style, equal to `headset_view::log_lines_newest_first` on the same input;
  - `is_error_line_words`: true for `"Tether command failed: exit 1"`, `"Could not run tether command"`, `"exec: No such file"`; false for `"100 link: ready"`.
- `crates/app/tests/logs_page.rs`:
  - `new_page_is_empty`;
  - `refresh_loads_once_while_loading`;
  - `lines_stores_and_clears_error`;
  - `failed_keeps_old_lines`;
  - `errors_filter_shows_only_error_lines`;
  - `copy_copies_visible_lines`;
  - `copy_with_nothing_is_none`.

**Inline facts:**
- Run record format: `"flat3d-run 1\nappid\t{a}\npid\t{p}\nmode\t{3d|flat}\n"` (runs.rs:25-33).
- Record path: `<rt>/runs/run-<pid>.txt` (state.rs:111-113).
- `DEFAULT_LOG_LINES=500` (headset_view.rs:21) is the default `max` that WP-28 and WP-33 pass.
- Error prefixes to keep: "Tether command failed" and "Could not run tether command" (K15).

**Depends on:** WP-01 (frozen `read_logs`/`log_lines`/`is_error_line`, `EnginePaths`). **Owner device:** none.
**Done-check:**
```
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test face_logs --test headset_view && cargo test --locked -p flat3d-app --test logs_page && cargo clippy --locked --workspace --all-targets -- -D warnings
```
Run it in craft-build if the iced build fails on the host.

---

**WP-00 pre-step (needed by WP-03/04/05 and WP-23):** before any WP starts, with `gh` read access, run once:
```
mkdir -p "$SCRATCH/issues" && for n in 76 116 118 <WP-23's issue numbers>; do gh issue view "$n" --json body -q .body > "$SCRATCH/issues/$n.md"; done
```
Then run each WP's `awk` extract into a temp file and check the sha256 values above. A mismatch blocks the wave.

Review note: I applied both corrections. Neither was rejected.
- **Correction 1:** `EnginePaths` with `log_dir`, `state_dir`, `link_log()`, `run_log()` and `runs()` is already planned in WP-01 (roadmap.md:200-203). `log_lines(text,limit)` maps directly onto the existing `log_lines_newest_first` (inject/src/headset_view.rs:23-33). I added `Msg::Failed(String)` so the error path that `Loaded(Err)` covered is not lost.
- **Correction 2:** `$SCRATCH/issues/` does not exist yet (ls ENOENT), so the pre-step is required.
- **Not re-checked:** I did not re-verify the sha256 values against live issue bodies, because this pass made no network calls.

### WP-07: list and status commands, catalog join (#27a part)
- **Closes:** #27a (the `list` and `status` bodies). code-map.md:109 says `list` returns `[Game]`. code-map.md:111 says `status` returns `LinkReport`.
- **Files (exclusive):**
  - `crates/engine/src/catalog.rs`
  - `crates/engine/src/cli/engine.rs`
  - `crates/engine/src/steam.rs`
  - `crates/engine/tests/cli_list.rs`
- **Signatures:**
  - `pub struct Catalog { pub installed: Vec<steam::Game>, pub recipes: BTreeMap<u32, recipes::Recipe>, pub games: Vec<api::Game> }`
  - `pub fn catalog(p:&EnginePaths) -> Result<Catalog, ApiError>`
  - `pub fn join(games:&[steam::Game], recipes:&BTreeMap<u32,recipes::Recipe>) -> Vec<api::Game>`
  - Keep `find_recipe` from WP-01.
  - `steam::library_of(g:&steam::Game) -> Option<PathBuf>` (a pure path strip)
  - `steam::compatdata(steam_root:&Path, g:&steam::Game) -> Option<PathBuf>`
  - `cli::engine::list(p) -> Result<Vec<api::Game>,ApiError>`
  - `cli::engine::status(p) -> Result<api::LinkReport,ApiError>`
  - `cli::engine::status_at(p, now:u64) -> Result<api::LinkReport,ApiError>`
- **Behaviour:**
  - **Steam root.** It is `p.home.join(steam::DEFAULT_STEAM_ROOT)` (steam.rs:10), so EnginePaths gets no new field.
  - **`steam::Game` stays at 3 fields** (`appid`, `name`, `install_dir`; steam.rs:17-21). Locked steam_installed_games.rs:52-53 and :94 build it as a literal.
  - **`installed_games`** (steam.rs:29) keeps its current behaviour:
    - a missing vdf gives `Ok([])`;
    - the root library is always read;
    - absolute library paths only;
    - dedup prefers a copy whose dir exists;
    - sorted by lowercase name, then appid.
  - **`library_of`** strips the trailing `steamapps/common/<dir>` from `install_dir` (built at steam.rs:107). It returns None if the path does not have that shape.
  - **`compatdata`** reuses the private `parse` (:162) and `library_paths` (:60). It returns the first existing `<lib>/steamapps/compatdata/<appid>`, checking in this order:
    1. `library_of(g)`;
    2. the root;
    3. the remaining libraries in vdf order.
    It returns None when none exists.
  - **`catalog`** makes one `find_recipe(&p.recipe_dirs, appid)` call per installed appid, then `recipes::parse_recipe` (:113) and `check_recipe` (:164).
    - It does NOT use `load_recipes` (:199), because that call fails on a missing folder and on legacy `{id}.recipe` names (`BadFileName`). That is the same file the wrapper reads (wrapper.rs:131).
    - A bad recipe gives `ApiError("bad-recipe", "<path>: <why>")`.
    - A vdf I/O error gives `"io"`.
  - **`join`**:
    - `status` is the recipe's status, or `"untested"` when there is no recipe (code-map.md:128);
    - `has_recipe` is set;
    - `install_dir` is `display().to_string()`;
    - it keeps the order of `installed_games`;
    - `api::Game` keeps its 5 fields.
  - **`status_at`**:
    - `state::read_snapshot(p)` is frozen by WP-01 (roadmap.md:210-212); it is not in state.rs today. `Some(s)` gives `s.effective(now)` and `None` gives `LinkSnapshot::stopped(now)` (api.rs:216, :233). A stale snapshot is one more than 10 s old (`is_stale`, state.rs:143).
    - The result is `LinkReport{state, message}`.
    - A read error gives `"io"`.
  - WP-01's dispatch serialises the result through `render` (cli.rs:102). `--json` prints the api DTOs. Human mode prints pretty JSON.
  - Only codes already in `ERROR_CODES` (api.rs:77-86) are used: `io` and `bad-recipe`.
  - Start-3d and stop-3d are NOT part of this WP (they are WP-08).
- **Bones rule (applies to all WPs, stated here because WP-07 is the first to touch one of these structs):** these 8 structs get **no new pub fields**, because locked or unheadered tests build them as literals:
  - `steam::Game` (steam_installed_games.rs:52-53, :94)
  - `api::Game` (api_types.rs:24, collection.rs:7, status.rs:8)
  - `LinkSnapshot` (api_snapshot.rs:7; :21-23 asserts the exact key set)
  - `HealthView` (view_health.rs:9-10)
  - `wrapper::RunPaths` (wrapper.rs:12-17, wrapper_run.rs:214)
  - `Packages` (setup_packages.rs:14-39)
  - `UevrSettings` (uevr_profile.rs:13-26)

  New data goes in sibling structs such as `LinkView`, `HealthTick` or `EnginePaths`.
- **Tests:**
  - **Locked, unchanged:** `crates/engine/tests/cli_json.rs` and `steam_installed_games.rs`.
    - `every_command_with_json_prints_one_object_naming_the_command` (:127) runs `list`/`status` for real. Each must give `ok==(code==0)` and a code from `ERROR_CODES`.
    - `the_binary_prints_one_parseable_object_for_list_json` (:141) uses the real HOME, so a missing Steam must give `ok:true, data:[]`.
  - **New:** `crates/engine/tests/cli_list.rs`, run with a temp HOME and a temp XDG_RUNTIME_DIR.
    1. No Steam gives `[]`.
    2. Two libraries (root plus `/abs/lib2`, using both the new and the old `"<n>" "<path>"` vdf forms), with one game that has a recipe:
       - that game's `status` is the recipe's status and `has_recipe=true`;
       - the other game is `"untested"`/`false`;
       - the order is by name.
    3. A legacy `<id>.recipe` file is found and joined.
    4. A malformed recipe gives `bad-recipe` naming the file.
    5. `library_of` strips `steamapps/common/<dir>`. `compatdata` prefers the game's own library, falls back to the root, and gives None when nothing exists.
    6. `status_at`:
       - no snapshot gives `stopped` with "Link service is not running.";
       - a fresh `ready` snapshot gives `ready`;
       - `updated = now-11` gives `stopped`.
    7. `execute(["list","--json"])` gives one line, `ok:true`, and the array.
- **Inline facts:**
  - `api::Game{appid,name,install_dir:String,status:String,has_recipe}` (api.rs:6)
  - `LinkState` is kebab-case (api.rs:47)
  - `STATUSES` lists works-now, likely, build, not-safe, wont-work, untested
  - The snapshot is `$XDG_RUNTIME_DIR/flat3d/link.json`, else `/tmp/flat3d-<uid>/` (code-map.md:158)
  - `EnginePaths{home,uid,state_dir,config,data,log_dir,bin,recipe_dirs}` (roadmap.md:200)
- **Flags:**
  - The scope gives `catalog(paths)->Catalog`. It needs to be `Result<Catalog,ApiError>`, because a bad vdf and a bad recipe must surface as `io`/`bad-recipe`.
  - The `find_recipe` return type is taken to be `Result<Option<(PathBuf,String)>,String>`. If WP-01 picks something else, this needs a one-line adapter.
  - The inline test `execute_human_not_built_goes_to_stderr` (cli.rs:198) expects `list` to be not-built. It lives in WP-01's cli file, so WP-01 must drop or retarget it once list is wired.
- **Depends on:** WP-01 (wave 2)
- **Owner device:** none (CI)
- **Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test cli_list --test cli_json --test steam_installed_games && tools/check.sh`

### WP-08: Link commands wiring (#120)
- **Closes:** #120 (the T15 link command bodies, roadmap.md:154 and :274)
- **Files (exclusive):**
  - `crates/engine/src/cli/link.rs`
  - `crates/engine/tests/cli_link_run.rs`
- **Signatures:**
  - `pub fn run_link(cmd:LinkCommand, json:bool, p:&EnginePaths) -> Output`
  - `pub fn run_link_with(cmd:LinkCommand, json:bool, p:&EnginePaths, io:&mut LinkIo) -> Output`
  - `pub struct LinkIo<'a>{ now:u64, pid:u32, env:BTreeMap<String,String>, runner:&'a mut dyn exec::CommandRunner, exec:&'a mut dyn FnMut(&ExecSpec)->io::Result<()>, watch:&'a mut dyn FnMut(&EnginePaths)->io::Result<()> }`
  - `pub fn request_3d(r:state::Request, p:&EnginePaths, now:u64) -> Result<api::LinkReport,ApiError>`
  - `pub fn parse_any(args:&[String]) -> Option<(Result<LinkCommand,String>, bool)>`
  - Consumed, frozen in WP-01, bodies from WP-20: `pub enum install::Plan{Install(InstallPlan),Uninstall(UninstallPlan)}` and `pub fn install::execute(plan:&install::Plan, r:&mut dyn exec::CommandRunner, p:&EnginePaths) -> Result<api::InstallReport,ApiError>`
  - Consumed from WP-01: `wrapper::run_with(argv, env, p:&EnginePaths, pid, now, runner, launcher)`. It keeps pid, now and the launcher of `wrapper::run` (wrapper.rs:119-126; critique.md:9).
- **Behaviour:** each command goes through `render(<word>, …, json)` (cli.rs:102).
  - **watch:** calls `service::run(p)`, which never returns.
  - **snapshot:** reads the snapshot with `effective(now)`, or `stopped(now)` when there is none, and prints the full `LinkSnapshot`.
  - **switch get:** reads settings with `switches::read_settings(p.config/settings)` (a missing file gives `Settings::default()`, everything on) and returns `SwitchState{appid,on:game_on}`.
  - **switch set:** calls `set_game` then `write_settings` and returns the new `SwitchState`.
  - **setting get/set:** uses `SettingKey::{Auto3d,Tunnel}` and returns `SettingState{key:as_str(),on}`.
  - **install/uninstall:** call `install::execute(&Plan::Install(install_plan(..))` or `&Plan::Uninstall(uninstall_plan(..)), io.runner, p)` (install.rs:26, :70). They return `InstallReport`. WP-01's stub returns `not-built` until WP-20 lands.
  - **run:**
    - `env` gets `SteamAppId=<appid>` when neither `SteamAppId` nor `SteamGameId` is set (wrapper.rs:98).
    - It calls `wrapper::run_with(argv, &env, p, pid, now, runner, exec)`, passing `p:&EnginePaths` directly. There is no `RunPaths::from`.
    - A launch error gives `command-failed`.
    - The argv after `--` is kept verbatim, including any `--json`.
  - **start-3d/stop-3d (A24):** these only write a request.
    - A missing or stale snapshot (`state::is_stale`, >10 s, state.rs:143) gives `ApiError("link-not-ready","Link service is not running.")`. Nothing is written and nothing is spawned.
    - A fresh snapshot leads to `state::write_request(p, Request::Start3d|Stop3d)` and returns `LinkReport{state:<current>, message:"Requested start-3d."|"Requested stop-3d."}`.
    - `inject 5` with no recipe or game returns `not-found` before touching any prefix.
  - **parse_any:**
    - It strips `--json` only from before the first `--`.
    - It acts only on its own first words: `watch|snapshot|install|uninstall|switch|setting|run` (cli_link.rs:52-58) → `parse_link` (left unchanged). Later WPs register view/rows/launch/setup/doctor sub-parsers for their own first words only.
    - Any other first word, including none (args `--json` or empty), returns `None`, so the caller falls through to `cli::parse` and its error is returned unchanged.
  - **CLI surface (locked cli_json.rs):**
    - `cli::USAGE` (cli.rs:50-51) and `cli::parse` (:57) stay byte-identical. :59-60 pins the USAGE string, and :53-54 pins `""`, `fly`, `list extra`, `list --verbose` and `--json` → `Err(USAGE)`.
    - New usage text goes in separate consts (`LINK_USAGE`, etc.).
    - `execute` reports unknown words with message `USAGE` and `command:""` (:112-123).
  - **Error codes:** only the 8 in `ERROR_CODES` (api.rs:77-86) are used: usage, not-built, not-found, bad-recipe, refused, link-not-ready, command-failed, io. A WP that needs a new code adds it to `ERROR_CODES` in the same WP.
- **Tests:**
  - **Locked, unchanged:**
    - `crates/engine/tests/cli_link_parse.rs`
    - `crates/engine/tests/cli_json.rs`:
      - :127-136 runs `list`, `inject 5`, `start-3d`, `stop-3d` and `status` with --json for real, and every non-zero exit must use an `ERROR_CODES` code. With no snapshot, start-3d/stop-3d give `link-not-ready`, `ok:false` and code 1.
      - :53-60 and :112-123 pin USAGE and the unknown-word behaviour.
  - **New:** `crates/engine/tests/cli_link_run.rs`. Each case uses a temp XDG_RUNTIME_DIR, HOME and config, plus a fake runner, exec and watch.
    1. start-3d with no snapshot gives `link-not-ready`, no `request` file, and an empty runner log.
    2. A stale snapshot (`updated=now-11`) gives the same.
    3. A fresh snapshot leads to the request file containing `start-3d` and `state:"ready"`; stop-3d gives `stop-3d`.
    4. `switch set 5 off` then `switch get 5` gives `on:false`, and the settings file starts with `flat3d-settings 1`.
    5. `setting set tunnel off` round-trips.
    6. `run 5 -- game --json`: exec sees argv `["game","--json"]`, env `SteamAppId=5`, and the run record is written before exec.
    7. `snapshot --json` gives one object.
    8. `watch` calls the watch closure once.
    9. `parse_any` gives None for `fly`, `list extra`, `--json` and `""`. `execute("fly --json")` still gives `command:""`, `code:"usage"` and `message:USAGE`.
    10. `install` with WP-20's body and a fake runner gives an `InstallReport`. Every error from any case is in `ERROR_CODES`.
- **Inline facts:**
  - `ERROR_CODES: [&str; 8]` includes `link-not-ready` (api.rs:77-86)
  - `Request{Start3d,Stop3d}` and `request_text` (state.rs:118)
  - `SettingState{key,on}` and `SwitchState{appid,on}` (api.rs:243, :249)
  - `switches::{read_settings(p:&Path)->Result<Settings,String>, write_settings(p,&Settings)->io::Result<()>}` and `state::{read_snapshot, write_request}` (roadmap.md:210-212)
  - `wrapper::RunPaths` (wrapper.rs:12) stays for the locked or unheadered wrapper tests. It is not used here.
- **Flags:**
  - Start-3d and stop-3d are `cli::Command` variants (cli.rs:19-21), not `LinkCommand` variants (cli_link.rs). WP-01's dispatch must route them to `cli::link::request_3d`, so `run_link` does not handle them.
  - `run_link(cmd, paths)` in the scope has no `json` argument. The signature above adds `json:bool`, or else WP-01 must render the result instead.
  - A bare `cargo test` on a device whose service is live hits cli_json :127 with a fresh snapshot, which writes a real start-3d/stop-3d request. `tools/check.sh` and CI must export a temp `XDG_RUNTIME_DIR`. The self-hosted runner also runs these commands for real.
  - WP-01 must freeze `install::Plan` and the 3-arg `install::execute`, and WP-20 must implement exactly that signature.
- **Depends on:** WP-01, WP-07 (status/`stopped` reuse), WP-20 (install bodies) (wave 3)
- **Owner device:** Q3 (link-smoke)
- **Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test cli_link_run --test cli_link_parse --test cli_json && tools/check.sh`

### WP-09: Watch service correctness
- **Closes:**
  - non-fatal tether failure with retry
  - the allow-debugging event
  - executing every heal action: RestartClient, RestartAdbThenClient, FullReconnect, GiveUp
  - production WiVRn start args
- **Files (exclusive):**
  - `crates/engine/src/watch/machine.rs`
  - `crates/engine/src/watch/service.rs`
  - `crates/engine/tests/watch_service.rs`
- **Signatures:**
  - `pub fn link_commands_with(effect:&Effect, w:&link::WivrnStart) -> Vec<Vec<String>>` (the locked `link_commands`, :262, stays as it is)
  - `impl<R,F> Service<R,F> { pub fn with_sleep(self, s:Box<dyn FnMut(u64)>) -> Self }`
  - `Service.tether_failed: bool`
  - `fn note_event(&mut self, n:Note)`
  - `fn heal(&mut self, now:u64, h:Heal)`
- **Behaviour:**
  - **Tether failure:**
    - `execute(StartTether)` sends an Err to `log(now,"tether: {msg}")` (that much exists). It also now pushes `EventKind::TetherError` with the message and sets `tether_failed=true`.
    - The link start still runs.
    - On the next feed frame that `classify` reads as `Ready`, when `tunnel` is on, the mode is Off and `tether_failed` is set, it calls `connect_tether(false)` once (watch.rs:690-702 today) and clears the flag on Ok.
    - The text "Tether command failed" / "Could not run tether command" is kept.
  - **Unauthorised:** on the move into `Seen::Unauthorised`, it pushes `note_event(Note::AllowUsbDebugging)` once, with kind `allow-debugging` and the text "Put the headset on and allow USB debugging." It does not repeat while the device stays unauthorised.
  - **Heal:** WP-09 owns executing every action. WP-11 only returns them (heal.rs:12-18, :68-70); nothing executes them today.
    - `RestartClient` runs `relay::client_stop_command()` (relay.rs:41), then `relay::client_start_commands()` (relay.rs:20), each through the serial-bound adb runner.
    - `RestartAdbThenClient` runs `adb kill-server`, then `adb start-server` (unprefixed, adb.rs:200), then the `RestartClient` sequence.
    - `FullReconnect` leads to `tether.disconnect()` then `connect_tunnel_only(foreign)`, with `foreign` taken from `stepaside::foreign_relay` (as in watch.rs:693).
    - `GiveUp` leads to one `TetherError` event, after which healing stops.
    - `None` does nothing.
    - A failed action is logged and gives no panic. The healer's next tick decides the next step.
  - **WiVRn start:**
    - StartLink and Reattach use `link_commands_with` with `WivrnStart{app:<detect_quest_app(...).0 or WIVRN_APP>, flags:&["--early-active-runtime","--no-publish-service"]}`.
    - The runner is an `AsLink` adapter (tether `RelayRunner` → `exec::CommandRunner`) wrapped in `AdbRunner::new(inner, adb, serial)` (adb.rs:171).
    - It calls `tether.set_wivrn(app, flags)` before connect.
    - It calls `tether.sync_wifi()` (WP-10) after an Own connect and logs the result through `redact_command`.
  - **Heartbeat:** `with_sleep` gives the tether (`Tether::set_sleep`, WP-10) a sleeper that rewrites the snapshot with `updated=now` every ≤5 s. This stops `WaitTun0` (up to 15×1.5 s plus 8×1.5 s) from making the snapshot stale (10 s).
  - Grace still maps to `Ready` in `state_text` (OD4).
- **Tests:**
  - **Locked, unchanged:** `crates/engine/tests/watch_frames.rs`, `watch_grace.rs`, `watch_manual.rs` (:52-55 pins `link_commands`) and `watch_plug.rs`.
  - **Inline, updated in service.rs:** `plug_authorise_ready_unplug_sequence_sees_the_right_snapshots` (:986).
    - The first snapshot's events now equal `[AllowDebugging seq 1]`.
    - Ready becomes seq 2.
  - **New (locked once merged):** `crates/engine/tests/watch_service.rs`, using `exec::testing::FakeRunner`, a scripted `Feed` and temp paths.
    1. A tether connect fails ("Tether command failed") → a `tether-error` event, the link start still runs, and the next Ready frame retries the connect exactly once.
    2. Unauthorised, then unauthorised again, then ready → one `allow-debugging` event, then `ready`.
    3. Healer `RestartClient`: the runner log equals the `client_stop_command` argv, then the `client_start_commands` argv, all prefixed `<adb> -s <serial>`.
    4. Healer `RestartAdbThenClient`: the runner log starts `[<adb>,"kill-server"]`, `[<adb>,"start-server"]`, then case 3's sequence.
    5. Healer `FullReconnect`: the runner log shows the disconnect commands and then a tunnel-only connect (no `install`, no `am start` for the WiVRn client).
    6. `GiveUp` gives exactly one `tether-error`, and later ticks run nothing.
    7. Detected app `com.x.wivrn` → the `am start` names it and the server argv carries both flags.
    8. The heartbeat sleeper rewrites `link.json` during a 12 s plan sleep, so the snapshot is never stale.
- **Inline facts:**
  - `Heal{None,RestartClient,RestartAdbThenClient,FullReconnect,GiveUp}` (heal.rs:12-18)
  - `MISS_LIMIT=3`, `MIN_HEAL_GAP_SECS=30`, `MAX_RECONNECTS=10` (heal.rs:5-9)
  - WiVRn flags come from uevr-openxr.md:155 and :230
  - `detect_quest_app(r,serial)->Option<(String,String)>` (roadmap.md:278)
  - `GRACE_SECS=120` (watch.rs:90)
  - `EventKind::{AllowDebugging,TetherError}` (api.rs:166)
- **Flags:**
  - The scope names `Heal::Reconnect`, which does not exist. It means `FullReconnect` (heal.rs:16).
  - The inline :986 test must change because allow-debugging is now emitted. It is not locked, so the change is allowed.
  - WP-11's text must drop any claim that it executes `RestartClient` and `RestartAdbThenClient`.
  - Coded only against the landed WP-10, WP-11 and WP-12 APIs: `Tether::{set_wivrn,set_sleep,sync_wifi}`, `run_plan_in`, `start_3d_commands_with`, `detect_quest_app`, `Healer`.
- **Depends on:** WP-01, WP-10 (wave 3), WP-11 (heal decisions), WP-12 (`start_3d_commands_with`, `detect_quest_app`) (wave 4)
- **Owner device:** Q3
- **Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test watch_service --test watch_frames --test watch_grace --test watch_manual --test watch_plug --lib && tools/check.sh`

### WP-10: Tether plans, runner errors, Wi-Fi sync
- **Closes:** executing the plan steps, idempotent APK install, actionable runner errors, wiring the Wi-Fi sync
- **Files (exclusive):**
  - `crates/engine/src/tether.rs`
  - `crates/engine/src/tether/plan.rs`
  - `crates/engine/src/tether/wifi.rs`
  - `crates/engine/src/exec.rs`
  - `crates/engine/tests/tether_plan_run.rs`
  - `crates/engine/tests/tether_runtime.rs` (unheadered, edited with a recorded reason)
- **Signatures:**
  - `pub fn run_plan(steps:&[TetherStep], r:&mut dyn RelayRunner, sleep:&mut dyn FnMut(u64)) -> Result<(),String>`
  - `pub fn run_plan_in(steps, ctx:&PlanCtx, r, sleep, out:&mut PlanRun) -> Result<(),String>`
  - `pub struct PlanCtx{ pub relay:Vec<String> }`
  - `pub struct PlanRun{ pub ran:Vec<Vec<String>>, pub tun0:bool }`
  - `Tether::set_sleep(&mut self, Box<dyn FnMut(u64)>)`, defaulting to `thread::sleep`
  - `Tether::set_wivrn(&mut self, app:String, flags:Vec<String>)`
  - `Tether::sync_wifi(&mut self) -> wifi::SyncDecision`
  - `Tether::last_wifi(&self) -> Option<&wifi::SyncDecision>`
  - `wifi::connect_argv(ssid,sec,psk) -> Vec<String>`
  - **Compat re-export (frozen in WP-01, kept here):** `tether/adb.rs` keeps `pub use crate::exec::RelayRunner as CommandRunner;`.
    - Tether's trait (adb.rs:64-68: `run→Result<String,String>`, `start_relay`, `stop_relay`) moves unchanged to `exec::RelayRunner`.
    - link.rs:110's `run→CommandOutput` trait becomes `exec::CommandRunner`.
    - The existing importers still resolve: tether_runtime.rs:2 and :25-65 (a `Fake` implementing all 3 relay methods) and watch.rs.
- **Behaviour:**
  - **`run_plan_in`** runs every `TetherStep` variant (plan.rs:8) in order:
    - `Run` runs the argv;
    - `SleepMs` calls `sleep`;
    - `IfVpnUp(c)` runs `c` only when `adb shell ip addr show tun0` matches `tun0_present`;
    - `IfNoTun0(cs)` runs `cs` only when tun0 is absent;
    - `WaitTun0{tries,every_ms}` polls tun0 with a sleep between tries and never errors (it sets `out.tun0`);
    - `WaitWifi` polls `quest_ssid_command` → `parse_quest_ssid` until it is Some, and never errors;
    - `KillStaleRelay` runs `ps -eo pid=,args=` and `parse_ps`, then `kill <pid>` for each argv starting with `ctx.relay[0]`;
    - `SpawnRelay` calls `start_relay(ctx.relay)`;
    - `CheckRelayAlive` re-runs ps; if no entry has argv equal to `ctx.relay`, it returns Err "Tether relay exited during startup";
    - `KillRelay` calls `stop_relay`.
  - **`run_plan`** wraps `run_plan_in` with an empty ctx. `SpawnRelay` or `CheckRelayAlive` with an empty ctx returns Err "No relay command".
  - **`Tether` connect, Own mode:**
    1. `reverse --list`;
    2. `start_3d_commands_with(wivrn)`;
    3. reverse localabstract;
    4. `adb shell pm list packages com.genymobile.gnirehtet` → `install -r <apk>` only if the output lacks `package:com.genymobile.gnirehtet`;
    5. `run_plan_in(connect_plan(), {relay: relay_command(relay_path(home), serial)})` (plan.rs:29), which includes the client `am start` through `IfNoTun0`;
    6. `sync_wifi()`.
  - **`connect_tunnel_only`** skips WiVRn and the install.
  - **`disconnect`, Own mode:** `run_plan_in(disconnect_plan())` (plan.rs:49), then `drop_local`.
  - **`sync_wifi`:**
    - `active_ssid_command` → `parse_active_ssid`, `credentials_command` → `parse_credentials`, `quest_ssid_command` → `parse_quest_ssid`;
    - `sync_decision(pc, quest, wifi_attempts)`;
    - `Switch` runs `connect_argv` and `wifi_attempts = next_attempts(prev, joined)`;
    - it logs only `redact_command`;
    - it is best effort: an error never fails the connect.
  - **`ProcessRunner` errors** (adb.rs:72) keep their prefixes:
    - non-zero exit: `Tether command failed: exit <code|signal N>; <last 4 non-empty stderr lines joined " / ">`, or the bare prefix when stderr is empty;
    - spawn failure: `Could not run tether command: <io error>`;
    - the relay messages keep their prefixes.
  - **`exec.rs`:** `tail_lines(stderr,4)` and an `exit_text(status)` helper shared with link's `SystemRunner`.
- **Tests:**
  - **Locked, unchanged:** `crates/engine/tests/tether_plan.rs` (the plan shapes) and the other headered `tether_*` files. The unheadered `tether_tunnel_review.rs` (relay helpers only, 0 connect calls) is also unchanged.
  - **Updated:** `crates/engine/tests/tether_runtime.rs`. It has no `//! Locked` header (line 1 is `use flat3d_inject::api::TetherMode;`), so it is outside the headered locked set. It is edited with this recorded reason in the PR body and CHANGELOG: "connect/disconnect now execute connect_plan/disconnect_plan (WP-10)". The edits:
    - Its imports stay as they are, via the compat re-export.
    - The WIVRN+OWN order adds `pm list packages`, and the plan steps (kill-stale ps, spawn, ps check, the tun0 probe) replace the bare relay start.
    - Disconnect follows `disconnect_plan()`.
    - The `fail_at` indices are renumbered.
    - The sleeper is a no-op.
  - **New:** `crates/engine/tests/tether_plan_run.rs`, using FakeRunner, a recording sleeper and a scripted tun0 / ps / ssid.
    1. Every variant runs once.
    2. `WaitTun0` stops at the first `inet ` and sleeps `every_ms` between tries.
    3. `IfNoTun0` is skipped when tun0 is present.
    4. A missing relay at `CheckRelayAlive` gives Err "Tether relay exited during startup".
    5. When the package is present, no `install` runs.
    6. A failing fake gives `Tether command failed: exit 1; a / b / c / d` (the last 4 of 6 lines).
    7. Wi-Fi: the PC on "home" and the Quest on "other" lead to a connect argv with the PSK redacted in the log; after 3 failures the result is `GiveUp`.
    8. `run_plan` with an empty ctx and `SpawnRelay` gives Err.
    9. An external `impl flat3d_inject::tether::adb::CommandRunner for Fake` (all 3 methods) compiles and drives `Tether::connect`.
- **Inline facts:**
  - `RELAY_PORT=31416` and `CLIENT_PACKAGE="com.genymobile.gnirehtet"` (relay.rs)
  - gnirehtet is v2.5.1
  - the APK is `~/.local/share/flat3d/bin/gnirehtet.apk`
  - `MAX_WIFI_ATTEMPTS=3`
  - `connect_plan`: WaitTun0 15×1500, then 8×1500; `disconnect_plan`: WaitWifi 6×1500
  - `TetherAdb::run` rewrites `adb` to `<path> -s <serial>`
- **Flags:**
  - The frozen 3-arg `run_plan` has no way to know the relay argv, so `run_plan_in` and `PlanCtx` are needed.
  - The WaitTun0 sleeps can reach about 34 s, longer than the 10 s staleness limit. WP-09 must inject a heartbeat sleeper through `set_sleep`.
  - §0/WP-00 must list the locked set exactly (see the review notes). Until they do, WP-10 relies on tether_runtime.rs being unheadered.
- **Depends on:** WP-01, WP-12 (`start_3d_commands_with`) (wave 3)
- **Owner device:** Q3
- **Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test tether_plan_run --test tether_plan --test tether_runtime --test tether_tunnel_review && cargo test --locked -p flat3d-inject tether && tools/check.sh`

Review note: partly rejected the WP-10 `pub use crate::exec::CommandRunner;` in tether/adb.rs, because two different traits share that name.
- adb.rs:64-68 has `run→Result<String,String>`, `start_relay` and `stop_relay`. link.rs:110 has `run→CommandOutput`.
- tether_runtime.rs:25-65 implements the relay shape, so re-exporting the link trait would break it.
- Applied `pub use crate::exec::RelayRunner as CommandRunner;` instead. roadmap.md:82, which re-exports one `CommandRunner` at both old paths, has the same bug and needs the same fix in WP-01.

Review note: rejected the locked-set count of "50 headered + 5 unheadered", because the disk has 49 + 6.
- `inject/tests/tether_tunnel_review.rs` also lacks the header. Its line 1 is `//! Review probes for E6d (#98)…`, and the file never says "Locked".
- §0 should list 49 headered files plus the issue-body tests (#67, #76, #116, #117, #118). The 6 unheadered files are event_ring_wrap, launch_options_quotes, runs_extra_lines, tether_runtime, tether_tunnel_review and wrapper_run; each may change only with a recorded reason.
- The §0/WP-00 edit itself is outside these four blocks.

Review note: rejected "no RunPaths exists" (WP-08), because `wrapper::RunPaths` does exist (wrapper.rs:12-17, built by wrapper_run.rs:214). What does not exist is `RunPaths::from(&EnginePaths)`. The fix (pass `p:&EnginePaths`) is applied anyway.

Review note: amended `install::execute(plan:&Plan, …)`, because no single `Plan` type exists: install.rs:26 returns `InstallPlan` and install.rs:70 returns `UninstallPlan`. WP-01 must freeze `enum install::Plan{Install(InstallPlan),Uninstall(UninstallPlan)}`, and WP-20 must implement that.

### WP-11: Health loop (#125)

- **Closes:** #125 (the code half; the soak run needs the owner's device), A6.
- **Files (exclusive):** `crates/engine/src/watch/health_loop.rs`, `crates/engine/src/tether/heal.rs`, `crates/engine/tests/tether_health.rs`.

**Signatures**

Checked against `inject/src/tether/heal.rs:5-101`, `health.rs:6-104`, `stats.rs:4-151`, `tether/relay.rs:73-116`, `tether/stepaside.rs:4-33` and `api.rs:141-190` (`TetherMode{Off,Own,QuestTether}` at `api.rs:141-145`, `EventKind` at `api.rs:166`). This WP implements **WP-01's frozen shape exactly**. WP-01 is not changed.

```rust
// heal.rs: additions only. The locked Healer/Heal/consts stay byte-identical.
impl Heal { pub fn word(self) -> &'static str } // "none" "restart-client" "restart-adb-then-client" "full-reconnect" "give-up"
impl Healer { pub fn tripped(&self) -> bool }
// watch/health_loop.rs: types and signatures as frozen by WP-01's bones; WP-11 fills the bodies
pub const TUN0_EVERY_SECS: u64 = 5;
pub const HEALTH_EVERY_SECS: u64 = 30;
pub struct HealthTick {
  pub health: Option<Health>, pub stats: Option<Stats>, pub tunnel: bool,
  pub heal: Option<Heal>, pub toast: Option<(EventKind, String)>,
} // exactly WP-01's bones
pub struct HealthLoop { /* private: healer, gate: ToastGate, meter: RateMeter, soak: Soak, soak_file: Option<PathBuf>, last_tun0, last_health, health, up_since, gave_up */ }
impl HealthLoop {
  pub fn new(start_unix: u64, mem_start_kb: u64, soak_file: Option<PathBuf>) -> Self; // args as frozen by WP-01 (see Flags)
  pub fn tick(&mut self, now: u64, mode: TetherMode, playing: bool, r: &mut dyn RelayRunner) -> HealthTick;
  pub fn reconnects(&self) -> u32;
  pub fn finish(self, now: u64, mem_end_kb: u64, orphans: u32) -> SoakSummary;
}
// No take_toast(), no pub playing: the toast comes back in HealthTick.toast and playing is a tick argument.
```

**Behaviour of `tick`**

1. **Tunnel probe.** It runs only when `mode == TetherMode::Own`, and then only when `last_tun0` is None or `now - last >= 5`. In `Off` or `QuestTether`, nothing runs for the tunnel and the healer is not fed. The result then has `tunnel: false`, `heal: None` and `stats: None`, and `up_since` is cleared.
   - `ps -eo pid=,args=` gives `relay_alive = foreign_relay(&parse_ps(out), None)`. A runner `Err` counts as false.
   - `adb shell ip addr show tun0` gives `tun0 = Ok && tun0_present(out)` (`relay.rs:73-78`).
   - `adb shell cat /proc/net/dev` gives `counters = tun0.then(parse_tun0_bytes)`, then `meter.sample(now*1000, counters)`.
   - `tunnel = tunnel_state(relay_alive, tun0) == Connected`. `up_since` is set on the first Connected and cleared on anything else.
   - `h = healer.observe(now, relay_alive, tun0)`:
     - `Heal::None` gives `heal: None`.
     - `GiveUp` gives `Some(GiveUp)` once (latched by `gave_up`), then None.
     - Anything else gives `Some(h)`.
   - While `tunnel` is up, `soak.sample(rx+tx)`.
   - It appends the soak line `sample\t{now}\t{rx}\t{tx}\t{tun0 0|1}\t{relay 0|1}\t{h.word()}\n`.
2. **Health poll.** It runs in every mode (SPEC M10: with Auto 3D off, plugging in still brings up health), when `last_health` is None or `now - last >= 30`.
   - It runs `r.run(&health_command())` (`adb shell dumpsys battery`). `Ok` gives `Some(parse_dumpsys_battery)`. `Err` gives `None`.
   - `gate.observe(zone, playing)` gives a kind, and then `HealthTick.toast = Some((kind, toast_text(kind, temp)))`. `toast` is None on every other tick.
   - It appends `health\t{now}\t{battery|-}\t{temp|-}\t{zone:?}` (lower-cased).
   - Between polls, the cached health is returned.
3. **Stats.** `stats = tunnel.then(|| Stats{rx_bytes_s, tx_bytes_s, uptime_s: now-up_since, reconnects: healer.reconnects()})`.
4. **Soak file.** Writes are best-effort: `create_dir_all` on the parent, then append. `finish` appends `SoakSummary::to_text()` (locked format, starting `flat3d-soak 1`). A reader takes the block after the last `flat3d-soak 1` line.
5. **Word convention.** Every word list starts with the literal `adb`, the same as `health_command()`. The caller passes a runner that resolves `adb` to the path plus `-s <serial>`, as `AdbRunner` does (`tether/adb.rs:171-205`).
6. **Contract for WP-09** (not code here):
   - Call `tick(now, mode, playing, r)` on every watch-loop tick, with the current `TetherMode` and the game-running flag. The loop gates the tunnel itself.
   - Carry out `tick.heal`:
     - `RestartClient` → `relay::client_start_commands()`;
     - `RestartAdbThenClient` → adb `kill-server`/`start-server`, then the client;
     - `FullReconnect` → `Tether::disconnect` + `connect_tunnel_only`;
     - `GiveUp` → push `EventKind::TetherError`.
   - Push `tick.toast` into the ring when it is Some.
   - Copy health, stats and tunnel into `current()` (`watch.rs:761-781` today).

**Tests**

Locked, unchanged: `tests/tether_heal.rs` (8 tests), `health_toast.rs`, `health_parse.rs`, `stats_soak.rs`, `stats_rate.rs`.

New `crates/engine/tests/tether_health.rs` uses a local `Fake` that implements `flat3d_inject::exec::RelayRunner`. Its replies are keyed by `argv.join(" ")` and it records `ran`. It is keyed, not FIFO, so the order of commands inside a tick doesn't matter. Every call is `tick(t, TetherMode::Own, playing, &mut fake)` unless a case says otherwise. Cases:
- `a_good_tunnel_fills_health_stats_and_tunnel`:
  - Fake replies: `"7 /x/gnirehtet run SER"` (ps), `"    inet 10.0.0.2/30 scope global tun0"` (ip), a `/proc/net/dev` line `tun0: 1000 …`, and the battery dump `level: 80`, `temperature: 300`.
  - Expected: `tunnel` is true, `heal` is None, `toast` is None, `health.zone` is Green, and the second probe gives a non-zero `stats.rx_bytes_s`.
- `three_misses_then_one_reconnect`: ip returns `Device "tun0" does not exist.`. Ticks at t=0,5,10 give heals `[None, None, Some(RestartClient)]`. Exactly one is non-None.
- `ticks_between_probes_run_nothing`: t=0, then t=1..4. `ran` doesn't grow for the tun0 words.
- `battery_is_polled_every_30_s`: the count of `dumpsys battery` over t=0..=60 step 5 is 3.
- `other_modes_skip_the_tunnel_probe`: `TetherMode::Off` and `QuestTether` give no `ip addr`, `ps` or `/proc/net/dev` in `ran`, `tunnel` false and `heal` None, but `dumpsys battery` still runs.
- `at_most_ten_reconnects_then_give_up`:
  - All misses, with ticks every 120 s. This spaces the heals so the flap breaker (6 per 600 s) never trips.
  - Expected: `FullReconnect` appears exactly 10 times, then one `Some(GiveUp)`, then only None. `reconnects()` is 10.
- `fast_misses_trip_the_flap_breaker`: ticks every 5 s give `Some(GiveUp)` before t=400.
- `warm_toasts_once_while_playing`: with `playing=true` and temperature `395`, `tick.toast == Some((Warm, "Headset is warm: 39.5 C. It may slow down."))`. The next poll's `toast` is None.
- `no_toast_while_not_playing`: `playing=false`, `395` → `toast` None.
- `soak_lines_are_written`:
  - `soak_file = CARGO_TARGET_TMPDIR/…/soak-1.txt`.
  - Expected: after 3 probes and `finish`, there are 3 `sample\t` lines and the file ends with the exact `to_text()` output.
- `adb_failure_is_a_miss_not_a_panic`.

**Inline facts**
- Zones (`health.rs:97-104`): Green < 38.0, Yellow 38.0..=42.0, Red > 42.0. Temperature is in tenths (`/10.0`).
- `EventKind::{Warm, Hot}` texts: `health.rs:42-48`. The gate is locked in `tests/health_toast.rs:7-30` ("nothing toasts while not playing").
- Soak path: `stats::soak_path(home, start)` = `{home}/.local/state/flat3d/soak-{start}.txt`.
- Heal constants: `MISS_LIMIT=3`, `MIN_HEAL_GAP_SECS=30`, `FLAP_WINDOW_SECS=600`, `FLAP_MAX=6`, `MAX_RECONNECTS=10`.
- The ladder cycles RestartClient → RestartAdbThenClient → FullReconnect.
- The watch loop ticks at 1 s (`watch.rs:890`).
- Relay detection: an argv word is `run` or `relay`, the command contains `gnirehtet`, and the executable is not adb (`stepaside.rs:20-33`).

**Flags**
- Flag: in #125, "one reconnect" means one heal action. The locked ladder's first action is `RestartClient`, not `FullReconnect`. The "at most 10" counts `FullReconnect`.
- Flag: the toast travels only in `HealthTick.toast`, and `playing` is only a `tick` argument, as frozen by WP-01. Do not add `take_toast()` or a `pub playing`. If WP-01's bones give `toast` another type (for example `Option<LinkEvent>`), follow the bones. Do not change them from here.
- Flag: `new` takes whatever args WP-01's bones froze. If the bones froze `HealthLoop::new()` with no arguments, keep that and add `with_soak(start, mem, file) -> Self`. Do not change the bones.
- Flag: `TetherAdb` (`adb.rs:128-168`) does not implement `RelayRunner`. WP-09 needs a small adapter in `service.rs`, like its `adb_run` (`watch.rs:660-667`).

**Depends on:** WP-01. **Owner device:** Q3 soak (`sessions/link-smoke.toml`).

**Done-check:**
```sh
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject \
  --test tether_health --test tether_heal --test health_toast --test health_parse \
  --test stats_soak --test stats_rate \
&& cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings
```

---

### WP-12: Link stop idempotence + WiVRn detection

- **Closes:** A8 (the command half), A20 (stop), K17, K18.
- **Files (exclusive):** `crates/engine/src/link.rs`, `crates/engine/src/wivrn.rs` (bodies only; WP-01's bones freeze the types and signatures), `crates/engine/tests/link_stop_twice.rs`, `crates/engine/tests/wivrn_detect.rs`.

**Signatures**

Checked against `link.rs:1-213`. Constants are at `link.rs:4-13` (`WIVRN_FLATPAK`, `WIVRN_APP="org.meumeu.wivrn.github"`, `WIVRN_UNIT`). `CommandRunner::run -> CommandOutput{success,stdout,stderr}` is at `link.rs:100-112`. `wivrn.rs` does not exist yet. WP-01 creates it with the frozen set below. This is the **one** API, and WP-09/29/31 use the same names.

```rust
// wivrn.rs: frozen by WP-01's bones; WP-12 fills the bodies
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FlatpakScope { User, System }
impl FlatpakScope { pub fn flag(self) -> &'static str } // "--user" | "--system"
pub const SERVER_FLAGS: [&str; 2] = ["--early-active-runtime", "--no-publish-service"];
pub const QUEST_APPS: [&str; 5] = [/* see Inline facts, in order */];
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WivrnStart { pub scope: Option<FlatpakScope>, pub app: String, pub flags: Vec<String> }
impl Default for WivrnStart { /* scope None, app link::WIVRN_APP, flags SERVER_FLAGS */ }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WivrnDirs { pub scope: FlatpakScope, pub app_dir: PathBuf, pub deploy: PathBuf, pub version: Option<String> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestApp { pub package: String, pub version: Option<String> }
pub fn app_dir_from_location(out: &str) -> Option<PathBuf>;
pub fn version_from_info(out: &str) -> Option<String>;
pub fn version_name(dumpsys: &str) -> Option<String>;
pub fn versions_match(server: &str, app: &str) -> bool; // compare the part before '-'
pub fn detect_server(r: &mut dyn CommandRunner) -> Result<WivrnDirs, String>;
pub fn detect_quest_app(r: &mut dyn CommandRunner, serial: &str) -> Option<QuestApp>;
// link.rs: additions. start_3d_commands/stop_3d_commands/start_3d keep their bodies.
pub use crate::wivrn::{FlatpakScope, QuestApp, WivrnDirs, WivrnStart, QUEST_APPS, SERVER_FLAGS, detect_quest_app, detect_server};
pub fn start_3d_commands_with(s: &WivrnStart) -> Vec<Vec<String>>;
pub fn start_3d_with(runner: &mut dyn CommandRunner, s: &WivrnStart) -> Result<LinkState, LinkError>;
fn unit_already_gone(stderr: &str) -> bool;
// stop_3d: body edited (see Behaviour)
```

The older names are retired: `WIVRN_SERVER_FLAGS`, `QUEST_APP_CANDIDATES`, `link::WivrnStart{app,flags}` (roadmap.md:101,278), `detect_quest_app(r, serial) -> Option<(String,String)>`, and `detect_server -> Option`.

**Behaviour**
- **`stop_3d`.** It still runs all 3 commands, in order. If the last one fails and `unit_already_gone(stderr)` holds, it returns `Ok(Stopped)`. Any other failure is still `Err(LinkError)`, so the locked "Access denied" case stays an error.
  - `unit_already_gone` is true for `Unit flat3d-wivrn.service not loaded` or `Unit flat3d-wivrn.service not found`, and for nothing broader. "Failed to connect to bus: No medium found" stays an error.
- **`start_3d_commands_with`.** It builds the same 4 lists as the locked function, with two changes:
  - The server list is `systemd-run --user --collect --unit=flat3d-wivrn flatpak run [<scope.flag()>] --command=wivrn-server io.github.wivrn.wivrn <s.flags…>`.
  - The `am start` list ends with `s.app`.
- **`start_3d_with`.** Same flow as `start_3d` (`link.rs:176-189`), over the `_with` lists.
- **`detect_server`.** It tries `User`, then `System`:
  1. `flatpak info --show-location <flag> io.github.wivrn.wivrn`. A failure (`success == false`) means "not here", so it tries the next scope.
  2. `deploy` = the trimmed stdout. `app_dir` = `deploy` minus its last 3 components (`/<arch>/<branch>/<commit>`). It must end in `app/io.github.wivrn.wivrn`, otherwise it tries the next scope.
  3. `flatpak info <flag> io.github.wivrn.wivrn` → `version_from_info` (the trimmed `Version:` line). A failure here gives `version: None`.
  4. If neither scope matches, it returns `Err("WiVRn server is not installed as a Flatpak (tried --user and --system)")`.
- **`detect_quest_app`.** For each id in `QUEST_APPS`, in order, it runs `["adb","-s",serial,"shell","dumpsys","package",id]`. The first output with a trimmed `versionName=` line wins. If adb fails or no candidate matches, the result is None. `AdbRunner` resolves `adb` and leaves an explicit `-s` alone (`tether/adb.rs:191-200`), so a runner with or without a serial both work.
- **Callers.** WP-09 and WP-10 switch the production callers, `tether.rs` (was `mod.rs:303-312`) and the service `StartLink`. They use exactly these names.

**Tests**

Locked, unchanged: `link_start.rs`, `link_stop.rs` (including `the_server_stop_failing_is_an_error_naming_it` :62-73 and the code-map line edited by WP-01), `link_commands.rs`.

New `crates/engine/tests/link_stop_twice.rs`, with a local scripted `Fake` (the same shape as `link_stop.rs:10-45`):
- `a_second_stop_with_the_unit_not_loaded_is_stopped`: replies `ok, ok, ok`, then `failed×2, failed("Failed to stop flat3d-wivrn.service: Unit flat3d-wivrn.service not loaded.\n")`. Both stops return `Ok(Stopped)`, and 6 commands ran.
- `unit_not_found_is_stopped`.
- `access_denied_is_still_an_error`.
- `a_bus_error_is_still_an_error` ("Failed to connect to bus: No medium found").
- `start_with_default_adds_both_flags`: with `WivrnStart::default()`, list[1] ends `[…, "io.github.wivrn.wivrn", "--early-active-runtime", "--no-publish-service"]`.
- `custom_flags_replace_the_defaults`: `flags: vec![]` makes list[1] end at `io.github.wivrn.wivrn`.
- `user_scope_puts_the_flag_after_run`.
- `a_custom_app_is_the_last_word_of_am_start`.
- `start_with_runs_four_commands_and_is_ready`.
- `with_lists_never_name_tether_or_dashboard` (the `link_commands.rs:68-77` check).
- `link_reexports_the_wivrn_names` (`flat3d_inject::link::SERVER_FLAGS == flat3d_inject::wivrn::SERVER_FLAGS`).

New `crates/engine/tests/wivrn_detect.rs`:
- `user_install_strips_arch_branch_commit`: the location `~/.local/share/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/61d58e18…332\n` gives `Ok`, `scope User`, `app_dir …/app/io.github.wivrn.wivrn`, `deploy` = the full trimmed path, and `version Some("26.9")`.
- `system_install_after_user_fails`: the user location fails with `error: io.github.wivrn.wivrn/*unspecified*/*unspecified* not installed`, and system gives `/var/lib/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc`. The result is `Ok`, scope `System`.
- `neither_is_an_error` (an `Err` naming both scopes).
- `a_short_path_is_an_error`.
- `store_app_found_second`: the first dumpsys gives `Unable to find package: org.meumeu.wivrn.github`, the second gives `    versionName=26.9`. The result is `QuestApp{package:"org.meumeu.wivrn", version:Some("26.9")}`, and `ran[0]` starts `["adb","-s","SER","shell","dumpsys","package"]`.
- `no_candidate_is_none`.
- `adb_failure_is_none`.
- `versions_match("26.9","26.9-github")` is true; `("26.9","26.8")` is false.

**Inline facts**
- Probed on the Z13:
  - `--user` prints the deploy path above. `--system` exits 1 with the "not installed" text above.
  - `flatpak info --user` prints `Version: 26.9`, `Branch: stable`, `Arch: x86_64`.
  - `systemctl --user stop <missing>.service` prints `Failed to stop X.service: Unit X.service not loaded.` and exits 5.
- wivrn-server flags: `--early-active-runtime` ("forcibly manages the active runtime even if no headset present") and `--no-publish-service` (WiVRn `server/main.cpp:1026-1035`).
- `QUEST_APPS`, in order: `org.meumeu.wivrn.github`, `org.meumeu.wivrn`, `org.meumeu.wivrn.github.nighly` (sic), `org.meumeu.wivrn.github.testing`, `org.meumeu.wivrn.local`.
- The client and server versions must match. The server is 26.9, and `WiVRn-release.apk` is the `.github` flavour.

**Flags**
- Flag: WP-01's bones must declare exactly the frozen set above in `wivrn.rs`. WP-09, WP-29 and WP-31 must be edited to these names, the `serial` argument and the `Result` from `detect_server`. Those edits are outside this block.

**Depends on:** WP-01. **Owner device:** Q3 (`adb shell dumpsys package org.meumeu.wivrn.github | grep versionName`, and Stop 3D twice).

**Done-check:**
```sh
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject \
  --test link_stop_twice --test wivrn_detect --test link_start --test link_stop --test link_commands \
&& cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings
```

---

### WP-13: Pinned fetch (`fetch.rs`)

- **Closes:** the fetch gap for #51/#52.
- **Files (exclusive):** `crates/engine/src/injectors/fetch.rs`, `crates/engine/tests/fetch.rs`, `crates/engine/tests/fixtures/pins-sample`.

**Signatures**

```rust
pub const PINS: &str = include_str!("../../../../packaging/pins"); // injectors→src→engine→crates→root
pub const TBD: &str = "TBD-at-pin";
pub const MARKER: &str = ".flat3d-fetch";
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FetchMode { Auto, Manual }
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pin { pub key: String, pub version: String, pub url: String, pub size: u64, pub sha256: String,
  pub md5: Option<String>, pub license: String, pub fetch: Option<FetchMode>, pub page: String, pub note: String }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError { UnknownKey(String), TbdAtPin(String), Manual { key: String, page: String, note: String },
  BadPin { key: String, field: &'static str }, Scheme(String), Command { command: String, message: String },
  SizeMismatch { expected: u64, got: u64 }, ShaMismatch { expected: String, got: String }, Io(String) }
impl std::fmt::Display for FetchError; impl From<FetchError> for crate::api::ApiError;
pub fn parse_pins(text: &str) -> BTreeMap<String, Pin>;
pub fn pin(key: &str) -> Result<Pin, FetchError>;                       // = pin_in(&parse_pins(PINS), key)
pub fn pin_in(pins: &BTreeMap<String, Pin>, key: &str) -> Result<Pin, FetchError>;
pub fn cache_dir(pin: &Pin, p: &EnginePaths) -> PathBuf;               // p.cache()/<key>/<version>
pub fn fetch(pin: &Pin, p: &EnginePaths, r: &mut dyn CommandRunner) -> Result<PathBuf, FetchError>;
#[doc(hidden)] pub struct FetchOpts { pub allow_file: bool }
#[doc(hidden)] pub fn fetch_with(pin: &Pin, p: &EnginePaths, r: &mut dyn CommandRunner, o: &FetchOpts) -> Result<PathBuf, FetchError>;
```

**Behaviour**
- **`parse_pins`.**
  - It skips blank lines and lines starting with `#`, and strips a ` #…` comment from the end of a value.
  - It splits `<key>.<field>: <value>` at the first `.` and the first `: `.
  - It skips the keys `crate` and `npm`, which are version lists only.
  - It ignores unknown fields, such as `pinned`. If a field repeats, the last one wins.
- **`pin_in`.**
  - A missing key → `UnknownKey`.
  - `fetch: manual` → `Manual`.
  - A sha256 of `TBD-at-pin` → `TbdAtPin`, with the message "`<key>` has no recorded sha256; run packaging/pin.sh `<key>`".
  - A missing url or size, or a sha256 that isn't 64 hex characters → `BadPin`.
- **`fetch` / `fetch_with`.**
  1. Check the pin the same way as `pin_in`. Then check the scheme: `https://` only, plus `file://` when `allow_file` is set. Anything else → `Scheme`, and no command runs.
  2. `dir = cache_dir`. If `dir/.flat3d-fetch` equals `sha256\n`, return `Ok(dir)` and run no commands (idempotent). A dir whose marker is missing or different is removed with `remove_dir_all`.
  3. `tmp = p.cache()/<key>/.<version>.tmp`: remove it, then create it. `name` = the last URL segment, cut before `?`.
  4. Run `curl --proto =https --proto-redir =https --tlsv1.2 -fL --retry 3 --max-filesize <size+1> -o <tmp>/<name> <url>`. With `allow_file`, the protocols are `=https,file`. A failure → `Command{command: argv joined, message: stderr trimmed}`.
  5. Check the size with `fs::metadata` → `SizeMismatch`.
  6. Run `sha256sum <tmp>/<name>` and take the first token → `ShaMismatch`.
  7. If `name` ends in `.zip`, `.7z`, `.tar`, `.tar.xz`, `.txz`, `.tar.gz`, `.tgz` or `.tar.zst`, run `bsdtar -xf <tmp>/<name> -C <tmp>/out`. Otherwise `fs::rename` the file to `<tmp>/out/<name>` (apk, AppImage, runtime).
  8. Write `<tmp>/out/.flat3d-fetch`, then `fs::rename(<tmp>/out, dir)` (atomic), then remove `tmp`.
  9. On any error, remove `tmp`. `dir` is never half-made.
- **ApiError codes.** UnknownKey → `not-found`. TbdAtPin, Manual, BadPin, Scheme → `refused`. Command, SizeMismatch, ShaMismatch → `command-failed`. Io → `io`. All 8 codes stay the locked set (`api_response.rs:10`).
- **Downloads.** Tests never download anything.

**Tests** (new `crates/engine/tests/fetch.rs`; no locked tests)

The fixture `tests/fixtures/pins-sample` holds:
- a header `#` comment;
- `ok.*` (url https, size 5, sha 64 hex, fetch auto);
- `tbd.*` (sha `TBD-at-pin`);
- `hand.fetch: manual`, `hand.page: https://example.invalid/p`, `hand.note: n`;
- `nourl.sha256: <hex>`, `nourl.fetch: auto`;
- `ok.pinned: 2026-11-08 tofu size+md5`;
- `crate.ashpd: 0.13.13   # c`, `npm.@decky/api: 1.1.3`.

Cases:
- `parse_reads_keys_skips_versions_lists` (the key set is `{hand, nourl, ok, tbd}`).
- `inline_comment_is_stripped`.
- `tbd_is_refused_with_pin_sh_hint`.
- `manual_is_refused_with_page`.
- `unknown_key`.
- `missing_url_is_bad_pin`.
- `real_pins`: `pin("uevr")` is Ok with sha `4cd6932d…dc6d8` and size 38974809, and `pin("ac7-prereqs")` is Manual. Both stay true after the owner fills the TBDs.
- `http_and_file_are_refused_without_allowance_and_run_nothing` (local recording fake).
- `curl_argv_is_exact` (the fake fails curl; assert `ran[0]`).
- `curl_failure_names_the_command`.

Live tests use `SystemRunner` and `file://` URLs. Each returns early with `eprintln!("skip: …")` when `curl`, `sha256sum` or `bsdtar` is not on PATH. The temp home is under `CARGO_TARGET_TMPDIR`, and the zip is made at test time with `bsdtar -a -cf sample.zip hello.txt`. Cases:
- `zip_unpacks_into_cache_key_version` (`dir/hello.txt`, marker).
- `second_fetch_runs_no_commands`.
- `plain_file_is_kept_by_name`.
- `sha_mismatch_leaves_no_dir`.
- `size_mismatch`.

**Inline facts**
- The pins format and the TBD rule: the `packaging/pins` header (WP-02).
- Host tools: `/usr/bin/{bsdtar,sha256sum,curl,7z}` are present. bsdtar 3.8.8 has liblzma, so `.7z` works (geo11 `geo-11_v0.7.12.7z?versionId=rTOuqnZ6WrSH5gjEin9v.1tDWIUlt4kX` → name `geo-11_v0.7.12.7z`).
- `EnginePaths::cache()` = `data/cache` (roadmap.md:204).

**Flags**
- Flag: the CI apt line (`check.yml`, WP-01) lacks `libarchive-tools`, so `bsdtar` is absent on ubuntu-24.04 and the live tests skip there. WP-01 should add the package.
- Flag: `EnginePaths::uevr_cache(tag)` (`data/uevr/<tag>`) conflicts with `data/cache/uevr/<version>`. The one rule is `cache_dir`, and WP-14 follows it.

**Depends on:** WP-01, WP-02 (`packaging/pins` must exist, or `include_str!` fails to compile). **Owner device:** Q0 (after `pin.sh --all-tbd`, a real `setup fetch`).

**Done-check:**
```sh
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test fetch \
&& cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings
```

---

### WP-14: UEVR route file half

- **Closes:** #51 (file half), A7 (uevr).
- **Files (exclusive):** `crates/engine/src/injectors/uevr.rs`, `crates/engine/tests/uevr_apply.rs`.

**Signatures** (checked against `uevr.rs:1-140`; `steam::Game{appid,name,install_dir}` is at `steam.rs:17`)

```rust
pub const UEVR_VERSION: &str = "nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d";
pub const UEVR_DLLS: [&str; 2] = ["openxr_loader.dll", "UEVRBackend.dll"]; // load order
pub const INJECTOR_EXE: &str = "flat3d-uevr-inject.exe";
pub const DEFAULT_DELAY_S: u32 = 15;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UevrExtras { pub exe_stem: String, pub delay_s: u32, pub native_stereo_fix: Option<bool>, pub profile: Option<(String, String)> }
pub fn settings_from(s: &RouteSettings) -> Result<UevrSettings, String>;
pub fn extras_from(s: &RouteSettings) -> Result<UevrExtras, String>;
pub fn profile_text_full(settings: &UevrSettings, extras: &UevrExtras) -> String;
pub fn uevr_dir(p: &EnginePaths) -> PathBuf;     // p.cache()/uevr/UEVR_VERSION (WP-13 cache_dir rule)
pub fn injector_path(p: &EnginePaths) -> PathBuf; // p.data/bin/flat3d-uevr-inject.exe
pub fn win_path(unix: &Path) -> String;           // "Z:" + path with every '/' → '\'
pub fn debug_cmd(injector: &Path, stem: &str, delay: u32, dir: &Path) -> String; // takes Unix paths; converts the --dll ones itself
pub fn apply_with(game: &steam::Game, settings: &RouteSettings, p: &EnginePaths) -> Result<(Applied, RouteEnv), String>;
// private: write_all takes `text: &str`; apply() passes profile_text(settings), so its locked behaviour is unchanged
```

**Behaviour**
- **`settings_from`.** `uevr-rendering`: `native` (default) or `synced`. `uevr-2d-screen`: `yes` or `no` (default `no`).
- **`extras_from`.**
  - `uevr-exe` is required (`Err("uevr-exe is missing")`).
  - `uevr-delay` is a u32 from 0 to 120, default 15.
  - `uevr-native-stereo-fix`: `yes` or `no`.
  - `uevr-profile`: `<url> <sha256>`. It is only carried here; WP-27 fetches it.
- **`profile_text_full`.** `profile_text(s)`, then optionally `VR_NativeStereoFix=true|false\n`, then `Frontend_RequestedRuntime=openxr_loader.dll\n`.
- **`apply_with`.**
  1. `compatdata = game.install_dir/../../compatdata` (`<lib>/steamapps/compatdata`).
  2. Require both `UEVR_DLLS` in `uevr_dir(p)`, else `Err("UEVR is not fetched; run flat3d-inject setup fetch <appid>")`. Require `injector_path(p)`, else `Err("flat3d-uevr-inject.exe is not installed; run tools/build-win.sh --install")`. Nothing is written before both checks pass.
  3. `dir = profile_dir(compatdata, appid, exe_stem)`.
  4. The same marker and backup guard as `apply` (`uevr.rs:81-89`). A second apply → `Err("…already set up; undo it first")`.
  5. Write with `profile_text_full`, then roll back on failure. The record has `launch_options_before = ""` (OD5).
  6. `RouteEnv{ env: [("PROTON_REMOTE_DEBUG_CMD", debug_cmd(..))], unset: ["XR_RUNTIME_JSON"], sidecar: None }`. `Applied.env` = `route_env.env.clone()`.
- **`debug_cmd`: the path contract with WP-15.**
  - Every path argument the injector receives is a Windows path, `win_path(..)` = `Z:` + the absolute Unix path with `/` replaced by `\`. Proton's `Z:` maps `/`. WP-15 receives Windows paths only and does no conversion.
  - The one exception is `cmd[0]`, the injector itself. Proton is the one that runs it, not the injector, and it stays the absolute Unix path (see Review note).
  - Every word is single-quoted for Python `shlex.split` (POSIX mode, where backslashes inside `'…'` stay literal). An embedded `'` becomes `'"'"'`. The result is:

  ```
  '/…/data/bin/flat3d-uevr-inject.exe' --exe <stem> --delay <d> --dll 'Z:\…\uevr\<ver>\openxr_loader.dll' --dll 'Z:\…\uevr\<ver>\UEVRBackend.dll'
  ```

**Tests**

Locked, unchanged: `uevr_profile.rs` (`profile_text`, `profile_dir`), `injector_route.rs`, `recipe_check.rs`, `recipe_parse.rs`, `wrapper_plan.rs`.

New `crates/engine/tests/uevr_apply.rs`. Its fake library is `CARGO_TARGET_TMPDIR/<case>/lib/steamapps/common/G`, and its fake home holds `uevr_dir` (two empty DLLs) and `data/bin/flat3d-uevr-inject.exe`. Cases:
- `full_profile_adds_requested_runtime`: starts with `profile_text` and ends with `Frontend_RequestedRuntime=openxr_loader.dll\n`.
- `native_stereo_fix_line`.
- `extras_need_uevr_exe`.
- `delay_defaults_to_15_and_parses_30`.
- `bad_delay_is_an_error`.
- `apply_writes_profile_marker_and_records_them`: the profile is under `compatdata/2075800/pfx/…/UnrealVRMod/SWZeroCompany/config.txt`, and `record.created` holds `[marker, profile]`.
- `existing_profile_is_backed_up_and_recorded_replaced`.
- `second_apply_is_refused_and_changes_nothing`.
- `env_is_exactly_the_debug_cmd`:
  - There is one env var, equal to the `debug_cmd` string. `unset == ["XR_RUNTIME_JSON"]` and `sidecar` is None.
  - `openxr_loader.dll` comes before `UEVRBackend.dll`.
  - Both `--dll` values start with `Z:\`, and token 0 starts with `/`.
- `missing_dll_writes_nothing` (the profile dir is absent).
- `missing_injector_is_an_error`.
- `dll_path_contract_example` (**locked here and mirrored word for word by WP-15's args test**): `win_path(Path::new("~/.local/share/flat3d/cache/uevr/nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/openxr_loader.dll"))` == `r"Z:\home\deck\.local\share\flat3d\cache\uevr\nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d\openxr_loader.dll"`.
- `quotes_survive_shlex`: a path with a space and a `'`, with the expected token list asserted by hand (backslashes intact).
- `uevr_version_matches_pins`: `include_str!("../../../packaging/pins")` contains `uevr.version: {UEVR_VERSION}`.

**Inline facts**
- Route keys (frozen in `recipes.rs` by WP-01): `uevr-exe`, `uevr-rendering`, `uevr-2d-screen`, `uevr-native-stereo-fix`, `uevr-delay`, `uevr-profile`.
- Stems: `SWZeroCompany` (2075800) and `Ace7Game` (502500). Two SWZeroCompany.exe files exist, so the injector (WP-15) picks the image path that contains `\Binaries\Win64\`. The profile folder stays `UnrealVRMod/<stem>/`.
- Proton 11 (`proton:2023-2026,2100-2133`, verified locally at `Proton 11.0/proton:2102-2103`) `shlex.split`s `PROTON_REMOTE_DEBUG_CMD`.
  - If `cmd[0]` fails `os.path.isabs`, Proton rewrites it to `g_proton.path(cmd[0])`.
  - It runs `[wine]+cmd` before the game, with the same env and wineserver, and kills it when the game exits.
- `XR_RUNTIME_JSON` bypasses `active_runtime.json`, so it must be unset (SpidyVR `--unset-env=XR_RUNTIME_JSON`).
- The UEVR frontend defaults to OpenVR. With `Frontend_RequestedRuntime` set and `openxr_loader.dll` loaded first, the backend skips OpenVR (VR.cpp:107-120,251-252).
- uevr.zip: sha256 `4cd6932d1d01168178aef70499d40b101ea6f19868cf76099d8e0087ef8dc6d8`, 38974809 bytes.

**Flags**
- Flag: the scope's `apply_with(game, recipe, …)`. `recipes::Recipe` (`recipes.rs:38-57`) carries no route keys, so the second argument is the `RouteSettings` from `recipes::route_settings(text)`. WP-27 parses that first.
- Flag: `--dll` takes full `Z:\` paths, because `LoadLibraryW` with a bare name searches the game's folder. WP-15's `args.rs` accepts only absolute Windows paths, does no `/`→`\` or `Z:` conversion, and carries the `dll_path_contract_example` test with the same strings.
- Flag: `RouteSettings` is assumed to be WP-01's alias for `BTreeMap<String,String>`, and `RouteEnv` lives in `injectors.rs`.

**Depends on:** WP-01. **Owner device:** Q6 (`sessions/z2-uevr.toml`; UEVR `log.txt` shows `[VR] Requested runtime: openxr_loader.dll`).

**Done-check:**
```sh
CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject \
  --test uevr_apply --test uevr_profile --test injector_route \
&& cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings
```

Review note: rejected the "`Z:` + path everywhere" part of the WP-14 [build] correction for `cmd[0]` (the injector exe). Proton checks `os.path.isabs(remote_debug_cmd[0])` and otherwise rewrites it to `g_proton.path(cmd[0])` (`~/.local/share/Steam/steamapps/common/Proton 11.0/proton:2102-2103`; Proton 10.0 `:1957-1958`). On Linux, `Z:\…` is not absolute, so the injector would be looked up inside the Proton folder and fail to start. Every path the injector itself receives (`--dll`) is `Z:\`, and WP-15 does no conversion, as both corrections asked.

### WP-15: flat3d-uevr-inject.exe + build-win.sh
**Closes:** K20 (msvc + xwin + rust-lld, not windows-gnu: critique.md:30) and K21 (build and `--install`: critique.md:31). Runs on the OD2 default, which is our own injector.
**Files:** `crates/uevr-inject/src/{main,args,pick,win}.rs`, `crates/uevr-inject/tests/args.rs`, `tools/build-win.sh`. `crates/uevr-inject/src/lib.rs` belongs to WP-01: `pub mod args; pub mod pick; #[cfg(windows)] pub mod win;`. WP-15 fills the modules and does not touch the lib.rs `mod` lines.

**Signatures**
```rust
// args.rs
pub enum Target { Exe(String) /*stem, ".exe" stripped (ASCII, any case)*/, Pid(u32) }
pub struct Args { pub target: Target, pub delay_s: u64, pub wait_s: u64, pub dlls: Vec<String> /*Windows paths, order kept*/ }
pub const DEFAULT_DELAY_S: u64 = 15; pub const DEFAULT_WAIT_S: u64 = 120;
pub const USAGE: &str = "usage: flat3d-uevr-inject (--exe <stem> | --pid <n>) [--delay <secs>] [--wait <secs>] --dll <path> [--dll <path>]...";
pub fn parse(argv: &[String]) -> Result<Args, String>;   // argv has no program name
pub fn to_win_path(p: &str) -> String;                   // "/a/b" -> "Z:\\a\\b"; anything else unchanged
// pick.rs (pure)
pub struct Pick { pub pid: u32, pub preferred: bool }
pub const FALLBACK_AFTER_S: u64 = 10;
pub fn stem_of(path: &str) -> String;                    // basename after the last '\\' or '/', minus ".exe"
pub fn choose(procs: &[(u32, String)], stem: &str) -> Option<Pick>;
pub fn has_module(modules: &[String], dll_path: &str) -> bool;
// win.rs  (#[cfg(windows)] pub mod win; in lib.rs)
pub fn run(a: &crate::args::Args) -> std::process::ExitCode;
```
- `main.rs` does `use flat3d_uevr_inject::{args, ..}` and parses `std::env::args().skip(1)`. It does not declare `mod` lines, so no module is compiled twice.
  - Bad arguments: print `flat3d-uevr-inject: <e>\n<USAGE>` to stderr and exit 2.
  - `#[cfg(windows)]`: `flat3d_uevr_inject::win::run(&a)`.
  - `#[cfg(not(windows))]` stub: print `flat3d-uevr-inject: Windows only (started by Proton through PROTON_REMOTE_DEBUG_CMD)` and exit 3.
- All Win32 code lives in `win.rs`, behind `#[cfg(windows)]`. `windows-sys` sits under `[target.'cfg(windows)'.dependencies]` (WP-01), so Linux CI clippy never compiles it. `args.rs` and `pick.rs` are `pub` and used by both the tests and main, which leaves no dead_code under `-D warnings` and needs no `allow(dead_code)`.

**Behaviour**
- **`parse`**
  - Flags take their value from the next argument only. `--k=v` is an error.
  - Exactly one of `--exe` or `--pid` is required. `--pid 0` is an error.
  - At least one `--dll` is required.
  - `--delay` and `--wait` must parse as u64.
  - These are errors: a repeated `--exe`, `--pid`, `--delay` or `--wait`; an unknown flag; a missing value.
  - Each `--dll` goes through `to_win_path`.
- **`choose`**
  - A candidate is a process whose `stem_of(path)` equals `stem`, ignoring ASCII case.
  - A candidate is `preferred` when its path, lowercased and with `/` turned into `\`, contains `\binaries\win64\`. Zero Company has two `SWZeroCompany.exe` (uevr-openxr.md:27).
  - It returns the **last** preferred candidate in list order. With no preferred candidate it returns the last candidate, and with no candidates it returns None.
- **`has_module`** compares basenames, ignoring case.
- **`win::run`**
  - **Wait for the game.** Poll every 500 ms for up to `wait_s`:
    - Take a `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` and walk it with `Process32FirstW`/`Process32NextW`.
    - Read each full path with `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` and `QueryFullProcessImageNameW(PROCESS_NAME_WIN32)`. Fall back to `szExeFile`.
    - Take a preferred pick at once. Take a non-preferred pick only after it has been the only candidate for `FALLBACK_AFTER_S`.
    - `--pid` skips the wait.
    - If nothing is found, exit 4.
  - **Delay:** sleep `delay_s`.
  - **Inject each dll in order:**
    - Skip a dll that `has_module` shows is already loaded. The module list comes from a `TH32CS_SNAPMODULE` snapshot walked with `Module32FirstW`/`Module32NextW`.
    - `OpenProcess(PROCESS_CREATE_THREAD|PROCESS_QUERY_INFORMATION|PROCESS_VM_OPERATION|PROCESS_VM_WRITE|PROCESS_VM_READ)`.
    - `VirtualAllocEx(MEM_COMMIT|MEM_RESERVE, PAGE_READWRITE)` for the path as UTF-16 with a trailing NUL, then `WriteProcessMemory`.
    - Get `LoadLibraryW` with `GetProcAddress(GetModuleHandleW(w!("kernel32.dll")), c"LoadLibraryW".as_ptr().cast())`. The `.cast()` is needed because `PCSTR = *const u8` (windows-sys-0.61.2 core/mod.rs:10; LibraryLoader/mod.rs:33), while `CStr::as_ptr` is `*const c_char`.
    - Cast with an annotated `transmute::<unsafe extern "system" fn() -> isize, unsafe extern "system" fn(*mut c_void) -> u32>` (clippy `missing_transmute_annotations`).
    - `CreateRemoteThread`, then `WaitForSingleObject(h, 60_000) == WAIT_OBJECT_0`, then `VirtualFreeEx(.., 0, MEM_RELEASE)`.
    - Check success with `has_module`, not with the thread exit code. The exit code holds only the low 32 bits of the HMODULE, which can be 0 for a 4 GiB-aligned base.
    - On failure, print the dll and `GetLastError` to stderr and exit 5.
  - Exit 0 after the last dll. Proton kills the injector when the game exits anyway (uevr-openxr.md:129-133).
  - An RAII `struct Handle(HANDLE)` closes handles; its Drop calls `CloseHandle`.
- **`tools/build-win.sh [--debug] [--install] [--dry-run] [--fetch-sdk]`** (`set -euo pipefail`, shellcheck clean)
  - `XWIN=${FLAT3D_XWIN_DIR:-$HOME/.local/share/flat-to-3d/xwin}`. It needs `$XWIN/crt/lib/x86_64`, `$XWIN/sdk/lib/um/x86_64` (kernel32.lib) and `$XWIN/sdk/lib/ucrt/x86_64`.
    - If any are missing, print `$HOME/.local/share/flat-to-3d/tools/xwin --accept-license splat --output "$XWIN"` and exit 3.
    - `--fetch-sdk` runs that command. Running it is the owner accepting the SDK licence (Q2).
  - If `rustup target list --installed` lacks `x86_64-pc-windows-msvc`, print the `rustup target add` hint and exit 3. The script never runs `rustup target add` itself (see WP-00 below).
  - `RUSTFLAGS="-Clinker=rust-lld -Ctarget-feature=+crt-static -Lnative=$XWIN/crt/lib/x86_64 -Lnative=$XWIN/sdk/lib/um/x86_64 -Lnative=$XWIN/sdk/lib/ucrt/x86_64"`. With `--target`, RUSTFLAGS reaches target artifacts only.
  - `CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target/win}`, so a RUSTFLAGS change doesn't rebuild the host target.
  - `cargo build --locked --release -p flat3d-uevr-inject --target x86_64-pc-windows-msvc`.
  - Copy `$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/flat3d-uevr-inject.exe` to `dist/tools/flat3d-uevr-inject.exe`. Check that its first 2 bytes are `MZ`, then print the `sha256sum`.
  - `--install` runs `install -Dm755` to `${XDG_DATA_HOME:-$HOME/.local/share}/flat3d/bin/flat3d-uevr-inject.exe`.
  - `--dry-run` prints the env and argv and runs nothing.

**Tests** (no locked tests). `tests/args.rs` uses `use flat3d_uevr_inject::{args, pick};` against WP-01's lib.rs. There are no `#[path]` includes.
- **Parsing:**
  - `--exe SWZeroCompany --dll /c/openxr_loader.dll --dll /c/UEVRBackend.dll` gives Exe, delay 15, wait 120, and dlls `["Z:\\c\\openxr_loader.dll","Z:\\c\\UEVRBackend.dll"]`.
  - `--exe Ace7Game.EXE` gives the stem `Ace7Game`.
  - `--pid 4242 --delay 0 --dll x` parses.
  - Each of these is an Err: no target; both targets; no `--dll`; `--delay abc`; `--pid 0`; `--attach=x`; a trailing `--dll`; `--exe a --exe b`.
- **Picking:**
  - Bootstrap pid 10 plus `...\SWZeroCompany\Binaries\Win64\SWZeroCompany.exe` pid 12 gives `{12, true}`.
  - The bootstrap alone gives `{10, false}`.
  - The uppercase `C:\G\GAME\BINARIES\WIN64\ACE7GAME.EXE` with stem `Ace7Game` is preferred.
  - A `/`-path containing `/Binaries/Win64/` is preferred.
  - `SWZeroCompanyLauncher.exe` gives None.
  - With two preferred matches, the last one is returned.
  - `has_module(["OPENXR_LOADER.DLL"], "Z:\\x\\openxr_loader.dll")` is true.

**Inline facts**
- windows-sys 0.61.2 signatures (in the cargo cache):
  - `CreateRemoteThread(HANDLE,*const SECURITY_ATTRIBUTES,usize,LPTHREAD_START_ROUTINE,*const c_void,u32,*mut u32)->HANDLE` at Threading/mod.rs:73. It is gated by `#[cfg(feature = "Win32_Security")]` at :72.
  - `GetProcAddress(HMODULE, PCSTR)->FARPROC` at LibraryLoader/mod.rs:33.
  - `PCSTR=*const u8`.
  - `LPTHREAD_START_ROUTINE = Option<unsafe extern "system" fn(*mut c_void)->u32>` (:530).
  - `FARPROC = Option<unsafe extern "system" fn()->isize>` (Foundation:4756).
  - `HANDLE = *mut c_void`.
  - `PROCESSENTRY32W.szExeFile: [u16;260]`.
  - Constants: `TH32CS_SNAPPROCESS=2`, `PROCESS_NAME_WIN32=0`, `MEM_RELEASE=0x8000`, `INVALID_HANDLE_VALUE=-1`.
- Load order is `openxr_loader.dll`, then `UEVRBackend.dll` (uevr-openxr.md:110,188).
- The xwin libs are present: `libcmt.lib`, `libvcruntime.lib`, `libucrt.lib`, `kernel32.lib`.
- The earlier probe linked a 118784-byte PE32+ (buildenv §4a).
- `x86_64-pc-windows-msvc` is already in `rustup target list --installed` on this Deck (checked 2026-11-09). There is no mingw (`x86_64-w64-mingw32-gcc` is absent), so `-gnu` is not a fallback here.

**Flags**
- **Feature (WP-01 owns it):** WP-01 adds `Win32_Security` next to `Win32_Foundation`, `Win32_System_Threading`, `Win32_System_Diagnostics_ToolHelp`, `Win32_System_Memory`, `Win32_System_LibraryLoader` and `Win32_System_Diagnostics_Debug` in the uevr-inject `[target.'cfg(windows)'.dependencies] windows-sys` features. Cargo.lock does not change. WP-15 does not edit Cargo.toml.
- **rust-std download (WP-00 owns it):** WP-00 runs `rustup target list --installed | grep -qx x86_64-pc-windows-msvc`.
  - If the target is missing, WP-00 records `rustup target add x86_64-pc-windows-msvc` as a one-time, owner-approved network step. It fetches a rust-std archive.
  - Fallback: `x86_64-pc-windows-gnu`, but only if that target and mingw are already installed.
  - WP-15 itself never downloads anything. On this host the target is present, so WP-00 passes.
- **argv contract for WP-14:** emit `PROTON_REMOTE_DEBUG_CMD='<data>/bin/flat3d-uevr-inject.exe' --exe <stem> --delay <n> --dll '<cache>/openxr_loader.dll' --dll '<cache>/UEVRBackend.dll'`, with **Unix** paths so `shlex` never sees backslashes. Do not use v1's `--uevr-dir`.
- **crt-static:** `+crt-static` has not been linked here yet. If the link fails, drop it; Wine's builtin vcruntime140 covers that case.

**Depends on:** WP-00 (target check) and WP-01 (lib.rs, features). **Owner device:** Q2 (`tools/build-win.sh --install`).
**Done-check:** `CARGO_TARGET_DIR=<scratch>/target cargo test --locked -p flat3d-uevr-inject && cargo clippy --locked -p flat3d-uevr-inject --all-targets -- -D warnings && cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings && tools/build-win.sh --dry-run`. shellcheck runs in CI, since the host has no shellcheck.

---

### WP-16: Geo-11 route pure half
**Closes:** #52 (code half). **Files:** `crates/engine/src/injectors/geo11.rs`, `crates/engine/tests/geo11.rs`.

**Signatures**
```rust
use flat3d_viewer::spec::{Layout, ViewerSpec};
pub type RouteSettings = std::collections::BTreeMap<String, String>; // or recipes::RouteSettings if WP-01 defines it
pub const KEYS: [&str; 7] = ["geo11-fix","geo11-exe","geo11-layout","geo11-separation","geo11-convergence","geo11-d3dx","geo11-remove-first"];
#[derive(Debug, Clone, PartialEq)]
pub struct Geo11Settings { pub layout: Layout, pub separation: u32, pub convergence: f32, pub d3dx: Vec<(String, String)> }
impl Default for Geo11Settings  // Sbs, 40, 2.0, d3dx = [("force_stereo","2")]
impl Geo11Settings { pub fn from_route(m: &RouteSettings) -> Result<Self, String>; }
pub fn direct_mode(l: Layout) -> &'static str;      // sbs|tab|sbs_reversed|tab_reversed
pub fn d3dxdm_text(s: &Geo11Settings) -> String;    // the 3 managed lines
pub fn d3dx_patch(ini: &str, s: &Geo11Settings) -> Result<String, String>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStep { Remove(PathBuf), Copy { from: PathBuf, to: PathBuf }, Write { to: PathBuf, text: String } }
pub fn plan_files(cache_dir: &Path, game_dir: &Path, m: &RouteSettings) -> Result<Vec<FileStep>, String>;
pub fn viewer_check(viewer_exe: &Path) -> Result<(), ApiError>;   // Err(ApiError::ViewerMissing(path)) unless an executable file
#[derive(Debug, Clone, PartialEq, Eq)] pub struct Applied { pub record: UndoRecord }
pub fn apply(cache_dir: &Path, game_dir: &Path, appid: u32, m: &RouteSettings, viewer_exe: &Path)
    -> Result<(Applied, RouteEnv), String>;   // RouteEnv{env,unset,sidecar} from WP-01
```

**Behaviour**
- **`from_route`**
  - `geo11-layout` must be one of `sbs|tab|sbs_reversed|tab_reversed`. The error lists them.
  - `geo11-separation` is 0..=100. `geo11-convergence` is an f32 > 0.
  - `geo11-d3dx` has the form `k=v;k=v`. A part without `=` is an error. The pairs are appended after `force_stereo=2`.
  - `geo11-remove-first` is `;`-separated relative paths. Absolute paths and `..` are errors.
- **`d3dxdm_text`** returns `direct_mode=<m>\ndm_separation=<n>\ndm_convergence=<c>\n`. `<c>` uses `{:.1}` when the value is whole, otherwise `{}`: 2.0 gives "2.0" and 2.25 gives "2.25".
- **Ini set rule**, shared by `d3dx_patch` and the d3dxdm.ini write:
  - Replace the first active `key<sep>value` line, comparing the key without ASCII case and keeping the original `<sep>` (for example ` = `).
  - Otherwise, insert the line after a commented `;key...` line.
  - Otherwise, insert it after a known section header: `force_stereo`→`[Stereo]`, `get_resolution_from`→`[Device]`. If that section is missing, append it.
  - Otherwise return `Err("d3dx.ini has no <key> line")`.
  - Keep CRLF if the input uses it. Patching twice gives the same text as patching once.
- **`plan_files`**
  - The exe is `game_dir/<geo11-exe>`, which must exist. The exe dir `E` is its parent.
  - **Removes:** `Remove(E/<p>)` for each remove-first path that exists.
  - **Fix root:** `<cache>/<geo11-fix>/` is the unpacked tree, written there by WP-27 `setup fetch`. The fix root is the shallowest directory (depth ≤ 3) that holds `d3dx.ini`, else one that holds `d3d11.dll`. Every file under it becomes a `Copy` to `E/<rel>`, except root-level `uninstall*.bat`, `readme*` and `*.url`.
  - **Geo-11 overlay:** when the fix root has no `d3dxdm.ini`, or there is no fix at all, overlay `d3d11.dll`, `nvapi64.dll` and `d3dxdm.ini` from `<cache>/geo11/**/x64/`. The overlay wins over the fix's copies. `d3dx.ini` comes from x64 only when the fix lacks one.
  - **Missing caches:** `Err("… run flat3d-inject setup fetch <key>")`.
  - **Name case:** if `E` already holds a file whose name matches the destination ignoring case, use that existing name.
  - **ini files:** `d3dx.ini` and `d3dxdm.ini` become `Write` steps with the patched text instead of `Copy`.
  - **Order:** Removes, then Copies sorted by `to`, then Writes.
- **`viewer_check`** is called by `apply` first, before anything else is touched. `ApiError::ViewerMissing(path)` displays as `flat3d-viewer not found at <path>; reinstall flat-to-3d (install, AppImage or Decky zip)`. `apply` returns its `to_string()`. The API/launch layer (WP-31) calls `viewer_check` itself to surface the typed error. A missing viewer is never a silent fallback to flat.
- **`apply`**
  - `marker = E/.flat3d-applied` (`uevr::MARKER_FILE`). If the marker exists, or any `*.flat3d-backup` exists, return `Err("… already set up; undo it first")`.
  - Write the marker and record it in `created`.
  - Remove and overwrite steps rename the old file to `<name>.flat3d-backup` and record `Replaced`. New files go in `created`. Directories are created but **not recorded**.
  - On any error, roll back the way `uevr.rs:133` does and return Err.
  - `launch_options_before = ""` (OD5).
  - `RouteEnv.env = [("WINEDLLOVERRIDES","d3d11,nvapi64=n,b")]`, or `"d3d11,nvapi64,dxgi=n,b"` when a step writes `E/dxgi.dll`. `unset = []`.
  - `sidecar = Some([viewer_exe] ++ ViewerSpec{layout, ..ViewerSpec::default()}.to_args())`. WP-31 appends `--until-pid <pid>`.

**Tests** (`tests/geo11.rs`, fake trees under `env!("CARGO_TARGET_TMPDIR")`, no download)
- `d3dxdm_text`: the default is exact; all 4 layouts map to the right `direct_mode`; convergence 2.25 prints as "2.25".
- `from_route`: an empty map gives the default. Each of these is an error: `interlaced`, separation 101, convergence 0, d3dx `nokey`, remove-first `../x`.
- `d3dx_patch`:
  - `force_stereo = 0` becomes `force_stereo = 2`.
  - A commented line gets the active line inserted after it.
  - A missing `[Stereo]` section is appended.
  - CRLF is kept. Patching is idempotent.
  - An unknown key with no line gives Err.
- **Control:**
  - Cache: `control-fix/{d3dx.ini,d3d11.dll,ShaderFixes/a-ps.txt,uninstall.bat}` and `geo11/geo-11_v0.7.12/{x64,x32}/{d3d11.dll,nvapi64.dll,d3dxdm.ini}`. The game dir holds `Control_DX11.exe` and `dxgi.dll`.
  - Route: `geo11-fix: control-fix`, `geo11-exe: Control_DX11.exe`, `geo11-d3dx: get_resolution_from=swap_chain`, `geo11-remove-first: dxgi.dll`.
  - Expected steps: `Remove(dxgi.dll)` comes first; there is no `uninstall.bat`; `d3d11.dll` comes from x64 (not from the fix or from x32); a `d3dx.ini` Write contains both keys.
- **Lana:** the fix tree has its own `d3dxdm.ini`, so no geo11 cache is needed.
- **Missing pieces:** a missing `geo11` cache gives an Err containing `setup fetch geo11`; a missing exe gives an Err.
- **Missing viewer:** `viewer_check(<tmp>/nope)` is `Err(ApiError::ViewerMissing(_))`. `apply` with that path gives an Err containing `flat3d-viewer not found`, and the tree is unchanged with no marker written.
- **apply → undo:**
  - Write `record.to_text()` to `undo::record_path(data, appid)`, then call `undo::undo_game`.
  - Afterwards every file and its content equal the snapshot taken before, and `dxgi.dll` is restored.
  - The env is exact.
  - The sidecar equals `[viewer] ++ spec.to_args()`. The fake viewer is an executable empty file.
- A second apply gives Err, with the tree unchanged.
- A fix that ships `dxgi.dll` gives `d3d11,nvapi64,dxgi=n,b`.
- Rollback: when `E/d3d11.dll` is a directory, apply gives Err and the tree is unchanged.

**Inline facts**
- Pin keys: `geo11` (0.7.12), `lana-fix`, `control-fix` (`packaging/pins`).
- Use the x64 `d3d11.dll`, `nvapi64.dll` and `d3dxdm.ini`, with `force_stereo=2`.
- Control: `get_resolution_from = swap_chain`, and remove HelixSR's `dxgi.dll` first.
- Proton appends a user `WINEDLLOVERRIDES` after its own (geo11-viewer.md:29-34).

**Flags**
- **Dirs left after undo:** `undo_game` removes entries with `fs::remove_file` (inject/src/undo.rs:226), so recorded directories would break undo. Directories such as `ShaderFixes/` are left empty after undo. WP-27 could add a `RemoveDir` action.
- **WINEDLLOVERRIDES merge:** merging with an inherited value is WP-31's job.
- **Ini section placement** is unverified (Q9).
- **Viewer shipping (cross-WP, not WP-16's files):**
  - WP-20 `install` copies `flat3d-viewer` into `paths.bin`.
  - WP-22 (AppImage) and WP-23 (Decky zip) bundle it in `bin/`, each with the done-check `test -x $BIN/flat3d-viewer`.
  - Without those, `viewer_check` fails on every Geo-11 launch.
- **Assumptions about WP-01:**
  - `ViewerSpec: Default`.
  - `RouteEnv` and `RouteSettings` live where WP-01 put them.
  - `ApiError` has the variant `ViewerMissing(PathBuf)`. `ApiError` is not in the repo today: there is no `enum ApiError` in `inject/src/`. WP-01 must freeze it.

**Depends on:** WP-01. **Owner device:** Q9 (`sessions/g1-geo11.toml`).
**Done-check:** `CARGO_TARGET_DIR=<scratch>/target cargo test --locked -p flat3d-inject --test geo11 && cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings`

---

### WP-17: Viewer pure modules (#44a)
**Closes:** #44 part a. **Files:** `crates/viewer/src/layout.rs`, `src/status.rs`, `src/source/test_pattern.rs`, `tests/layout.rs`, `tests/status.rs`.

**Frozen in WP-01 (`crates/viewer/src/frame.rs`). WP-17, WP-18 and WP-19 use only these names:**
```rust
pub struct Plane { pub fd: std::os::fd::OwnedFd, pub offset: u32, pub stride: u32 }
pub enum FrameData { Mem { stride: u32, bytes: Vec<u8> }, DmaBuf { planes: Vec<Plane>, modifier: u64 } }
pub struct Frame { pub width: u32, pub height: u32, pub fourcc: u32, pub seq: u64, pub data: FrameData }
// Debug only; no Clone (OwnedFd is not Clone). fourcc = DRM fourcc, e.g. u32::from_le_bytes(*b"XR24") = XRGB8888 = BGRx bytes in memory.
```
**Also assumed from WP-01:**
- `spec::{ViewerSpec{layout,swap_eyes,crop_aspect:Option<f32>,screen,source}, Layout{Sbs,Tab,SbsRev,TabRev}}`.
- `source::FrameSource{fn next(&mut self, timeout: Duration) -> Result<Option<Frame>, String>}`.
- If the frozen shapes differ, WP-01 wins.

**Signatures**
```rust
// layout.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub struct Rect { pub x: u32, pub y: u32, pub w: u32, pub h: u32 }
pub fn eye_rects(frame_w: u32, frame_h: u32, spec: &ViewerSpec) -> [Rect; 2];   // [left eye, right eye]
pub fn unsqueeze(r: Rect, layout: Layout) -> Rect;   // half-SBS/TAB display size: w*2 (Sbs*) or h*2 (Tab*)
pub fn quad_size(r: Rect, width_m: f32) -> (f32, f32); // (width_m, width_m*h/w); (width_m,0.0) if w==0
// status.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum State { WaitingHeadset, WaitingSource, Showing, Error }
#[derive(Debug, Clone, Copy, PartialEq)] pub struct Status { pub state: State, pub fps: f32, pub w: u32, pub h: u32 }
impl State { pub fn as_str(self) -> &'static str; }
impl Status { pub fn to_line(&self) -> String; pub fn from_line(s: &str) -> Result<Status, String>; }
// source/test_pattern.rs
pub struct TestPattern { /* w,h,spec,fps,seq,next_at */ }
impl TestPattern { pub fn new(w: u32, h: u32, spec: ViewerSpec, fps: u32) -> Self; }   // fps 0 = unpaced
impl FrameSource for TestPattern
pub const LEFT_MARK: [u8; 4] = [0, 255, 0, 255]; pub const RIGHT_MARK: [u8; 4] = [255, 0, 255, 255]; // BGRx byte order
```

**Behaviour**
- **`eye_rects`**
  - `half = w/2`. Sbs gives `[(0,0,half,h),(half,0,half,h)]`; an odd last column is dropped.
  - Tab splits on `h/2` the same way.
  - SbsRev and TabRev swap the pair. `swap_eyes` then swaps it again.
  - `crop_aspect = Some(a)` with `a > 0` crops each eye to the centred `a` aspect, in pixels:
    - if `w/h > a`: `nw = round(h*a)`, `x += (w-nw)/2`;
    - otherwise `nh = round(w/a)`, `y += (h-nh)/2`.
  - A zero-sized frame gives `[Rect::default(); 2]` and never panics.
- **`to_line`** writes `{"state":"showing","fps":59.9,"w":1920,"h":1080}`:
  - fps uses `{:.1}`, and a value that is not finite becomes `0.0`;
  - there is no trailing newline;
  - it is hand-formatted, because the viewer has no serde.
- **`from_line`** parses exactly that shape: key order is fixed and whitespace is tolerated. An unknown state is an Err.
- **`TestPattern::next`**
  - Returns `Frame{width:w, height:h, fourcc: u32::from_le_bytes(*b"XR24"), seq, data: FrameData::Mem{stride: 4*w, bytes}}`. The bytes start as grey `[64,64,64,255]`.
  - In each eye rect from `eye_rects` it draws:
    - an 8×8 mark at the rect's top-left (+2,+2): `LEFT_MARK` in the left eye, `RIGHT_MARK` in the right;
    - a white vertical bar `max(1,rw/32)` wide at `rx + rw/2 ± rw/64` (+ in the left eye, − in the right), so the disparity is visible;
    - a 2-px row at `y = seq % rh` for liveness.
  - It increments `seq`.
  - With `fps > 0` it sleeps until `next_at`. If that is later than `timeout`, it sleeps for `timeout` and returns `Ok(None)`.

**Tests**
- **`tests/layout.rs`:**
  - Sbs 1920×1080 → `[(0,0,960,1080),(960,0,960,1080)]`.
  - Tab → `[(0,0,1920,540),(0,540,1920,540)]`.
  - SbsRev and TabRev are swapped.
  - `swap_eyes` on Sbs equals SbsRev; on SbsRev it equals Sbs.
  - Odd width 1921 → both eyes are 960 wide.
  - **FlatToDepth vector:** 10240×1440 Sbs with `crop_aspect = 16.0/9.0` → `[(1280,0,2560,1440),(6400,0,2560,1440)]`, which is columns 1280–3839 of each 5120-wide eye. With `swap_eyes` the pair is reversed; that is the inverted-depth fix.
  - Crop of a tall eye: 960×1080 with 16:9 → `(0,270,960,540)`.
  - `crop_aspect` None or 0 → no crop. 0×0 → defaults.
  - `quad_size((0,0,2560,1440),3.0) == (3.0,1.6875)`. `unsqueeze(960×1080, Sbs)` → 1920×1080.
  - **TestPattern** 64×32 Sbs, fps 0:
    - `width==64`, `height==32`, `fourcc==u32::from_le_bytes(*b"XR24")`, `FrameData::Mem` with stride 256 and 8192 bytes;
    - the pixel at (2,2) is `LEFT_MARK` and the one at (34,2) is `RIGHT_MARK`;
    - with SbsRev the two are swapped;
    - `seq` is 0, then 1.
  - fps 1 with a 10 ms timeout: the 2nd call is `Ok(None)` within 200 ms.
- **`tests/status.rs`:**
  - The exact line for each of the 4 states.
  - 59.94 → `59.9`; NaN → `0.0`.
  - Round trip through `from_line`.
  - `{"state":"bogus",...}` → Err; `""` → Err.

**Inline facts**
- FlatToDepth (MIT, Blackskydk/FlatToDepth `docs/assessment.md`):
  - each eye is 5120×1440, and the picture fills columns 1280–3839 ("crops to the centred 16:9 region");
  - `swap_eyes=1` fixed the inverted depth;
  - the quad sits 2.5 m away, and its width is clamped to 0.3–20 m.

**Flags**
- **`unsqueeze` is new.** Geo-11's on-screen SBS is half-width, so quads sized from raw pixel rects would show at half aspect. `ViewerSpec` has no half/full field, so WP-19 calls `unsqueeze` for screen capture.
- **Crop semantics:** `crop_aspect` is a pixel aspect, which is FlatToDepth's meaning.
- **No `PixelFormat` type and no `FrameData::Cpu`:** both were superseded by WP-01's frozen `frame.rs`. `Frame` has no `Clone`, so tests compare fields, never whole frames.

**Depends on:** WP-01. **Owner device:** none.
**Done-check:** `CARGO_TARGET_DIR=<scratch>/target cargo test --locked -p flat3d-viewer && cargo clippy --locked -p flat3d-viewer --all-targets -- -D warnings && cargo clippy --locked -p flat3d-viewer --all-targets --features xr -- -D warnings`

---

### WP-18: Viewer capture (#44b)
**Closes:** #44 part b. **Files:** `crates/viewer/src/source/pipewire.rs`, `src/source/gamescope.rs`, `src/source/portal.rs`.
- All three files are behind `feature = "capture"`. The `#[cfg(feature = "capture")] mod` lines are WP-01's.
- `cargo check --workspace` with no features stays host-only: it does not compile pipewire, libspa or bindgen.
- **Build environment:** the `capture` feature needs `libpipewire-0.3-dev libspa-0.2-dev libclang-dev clang` plus pkg-config. bindgen in libspa-sys/pipewire-sys needs libclang.
  - WP-01 puts these packages in the CI apt line.
  - Local Deck builds run only inside the `craft-build` distrobox, which has `/usr/include/pipewire-0.3` and pkg-config (geo11-viewer.md:96).
  - The same requirement applies to WP-19.

**Signatures**
```rust
// pipewire.rs   (inside: `use ::pipewire as pw;` — leading :: avoids clashing with this module's own name)
use crate::frame::{Frame, FrameData, Plane};
pub enum Remote { Default, Fd(std::os::fd::OwnedFd) }
pub enum Target { NodeName(String) /*stream prop target.object*/, NodeId(u32) /*connect(Some(id))*/ }
pub struct Offer { pub fourccs: Vec<u32> /*DRM fourcc*/, pub modifiers: Vec<u64> /*empty ⇒ MemFd only*/, pub max_w: u32, pub max_h: u32 }
pub struct PwSource { rx: std::sync::mpsc::Receiver<Result<Frame, String>>, stop: pw::channel::Sender<()>, thread: Option<std::thread::JoinHandle<()>> }
pub fn open(remote: Remote, target: Target, offer: &Offer) -> Result<PwSource, String>;
pub fn find_node(remote: Remote, name: &str, wait: Duration) -> Result<Option<u32>, String>;
impl FrameSource for PwSource   // rx.recv_timeout(timeout): Timeout ⇒ Ok(None)
impl Drop for PwSource          // stop.send(()), join
// gamescope.rs
pub const NODE_NAME: &str = "gamescope";
pub fn open(offer: &Offer, wait: Duration) -> Result<Option<PwSource>, String>; // None ⇒ status waiting-source
// portal.rs
pub struct PortalSource { inner: PwSource, keep: std::thread::JoinHandle<()>, stop: std::sync::mpsc::Sender<()> }
pub fn open(token_path: &Path, offer: &Offer) -> Result<PortalSource, String>;
impl FrameSource for PortalSource
```

**Behaviour**
- **`pipewire.rs`**
  - Call `pw::init()` once (`std::sync::Once`).
  - Each source runs on its own thread, because `MainLoopRc` is not Send. The thread does:
    - `MainLoopRc::new(None)`, `ContextRc::new(&ml, None)`, then `context.connect_rc(None)` or `connect_fd_rc(fd, None)` (pipewire-0.10.1 context/rc.rs:91);
    - `StreamBox::new(&core, "flat3d-viewer", properties!{MEDIA_TYPE=>"Video", MEDIA_CATEGORY=>"Capture", MEDIA_ROLE=>"Screen"})`, plus `TARGET_OBJECT` for `NodeName`;
    - `connect(Direction::Input, id, StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS, &mut params)`;
    - a stop receiver from `pw::channel::channel()`, attached to `ml.loop_()`, which calls `ml.quit()`.
  - **fourcc ↔ SPA mapping** (private fn): `XR24↔BGRx`, `AR24↔BGRA`, `XB24↔RGBx`, `AB24↔RGBA`, `NV12↔NV12`. In little-endian memory order, DRM XRGB8888 is B,G,R,X. Unknown values are dropped from the offer.
  - **Params:**
    - Two `EnumFormat` objects.
      - The first carries `VideoModifier` with `PropertyFlags::MANDATORY | DONT_FIXATE` and `ChoiceValue::Long(Choice(ChoiceFlags::empty(), ChoiceEnum::Enum{default, alternatives}))` over `offer.modifiers`. It is built only when that list is not empty.
      - The second has no modifier (the shm fallback).
    - Both use `VideoFormat` ∈ the mapped `offer.fourccs`, `VideoSize` in 1×1..max, and `VideoFramerate` 0/1..1000/1.
    - Serialize with `PodSerializer` the way `examples/streams.rs:123-188` does.
  - **`param_changed(Format)`:**
    - Parse `VideoInfoRaw` (`size()`, `format()`, `modifier()`) and map the format back to a fourcc.
    - If the pod still has a modifier choice, fixate it: take the first offered modifier, rebuild, and call `update_params`.
    - Then send a `Buffers` param: object type `spa::sys::SPA_TYPE_OBJECT_ParamBuffers`, key `SPA_PARAM_BUFFERS_dataType`, value `Int(1<<DmaBuf)` when there is a modifier, otherwise `(1<<MemFd)|(1<<MemPtr)`.
  - **`process`:**
    - Call `dequeue_buffer`. Skip the buffer when `chunk.size()==0` or the chunk is CORRUPTED.
    - DmaBuf: `FrameData::DmaBuf{planes, modifier}`. Each `datas[i]` becomes `Plane{fd: BorrowedFd::borrow_raw(d.fd()).try_clone_to_owned()?, offset: chunk.offset(), stride: chunk.stride() as u32}`, and `modifier` is the negotiated one.
    - MemFd/MemPtr: `FrameData::Mem{stride, bytes: data()[offset..offset+size].to_vec()}`.
    - Build `Frame{width, height, fourcc, seq, data}` and `seq += 1`. Use `sync_channel(2)` and `try_send`; when the channel is full, drop the frame (its fds close on drop).
  - Stream state `Error(msg)` → send `Err(msg)`.
- **`find_node`**: registry `add_listener_local().global(..)`. Match `type_ == ObjectType::Node` and props `node.name == name`. Bound the wait with `core.sync(0)` plus a loop timer.
- **`gamescope::open`**: `find_node(Default, "gamescope", wait)`. If the node is found, `open(Default, NodeName("gamescope"), offer)`; otherwise `Ok(None)`.
- **`portal::open`** (option A; ashpd's `pipewire` feature is off)
  - The thread owns a tokio `Builder::new_current_thread().enable_all()` runtime.
  - In `block_on`:
    - `Screencast::new().await` and `proxy.create_session(Default::default())`.
    - `select_sources(&session, SelectSourcesOptions::default().set_cursor_mode(CursorMode::Hidden).set_sources(SourceType::Monitor | SourceType::Window).set_multiple(false).set_restore_token(token.as_deref()).set_persist_mode(PersistMode::ExplicitlyRevoked))`.
    - `start(&session, None, Default::default()).await?.response()?`.
    - Take the first stream's `pipe_wire_node_id()`.
    - Write `restore_token()` to `token_path` with an atomic tmp file + rename, mode 0600. Tokens are single-use, so rewrite it on every start.
    - `open_pipe_wire_remote(&session, Default::default())` returns an OwnedFd.
    - Hand `(fd, node)` back over a std channel. Then hold the proxy and session by awaiting `tokio::task::spawn_blocking(move || stop_rx.recv())`. The session must outlive the stream.
  - Then call `pipewire::open(Remote::Fd(fd), Target::NodeId(node), offer)`.
  - A missing or unreadable token means the user is asked to pick a source.

**Tests:** no new test file, since capture runs only on the device. CI clippy runs with `--features capture` and `--features xr,capture` on the apt-equipped runner. The pure parts are covered by WP-17.

**Inline facts**
- Crate versions: pipewire/libspa `=0.10.1`; ashpd `=0.13.13` with default-features off and `["screencast","tokio"]` (0.14.0 fails the age rule); tokio rt.
- ashpd 0.13.13 signatures (docs.rs source):
  - `create_session(&self, CreateSessionOptions)`;
  - `select_sources(&self,&Session<_>,SelectSourcesOptions)->Request<()>`;
  - `start(&self,&Session<_>,Option<&WindowIdentifier>,StartCastOptions)->Request<Streams>`;
  - `open_pipe_wire_remote(&self,&Session<_>,OpenPipeWireRemoteOptions)->Result<OwnedFd>`;
  - `Streams::{streams()->&[Stream], restore_token()->Option<&str>}`, `Stream::pipe_wire_node_id()->u32`;
  - `CursorMode{Hidden,Embedded,Metadata}` comes from `ashpd::desktop::screencast`, and `PersistMode` from `ashpd::desktop`.
- Gamescope always creates a `Video/Source` node named `"gamescope"` (BGRx/NV12, DmaBuf or MemFd); see gamescope `src/pipewire.cpp:167-168,700-702`.
- The host has no pipewire headers and no pkg-config, so `capture` builds only in craft-build or CI (geo11-viewer.md:96).

**Flags**
- **Module name:** `source.rs` must not `use pipewire::…` directly, because that is ambiguous with the child module. Use `::pipewire`.
- **Dup'd-fd frames are untested:** a producer may reuse the buffer, so WP-19 must import and copy each `FrameData::DmaBuf` at once.
- **Unverified on the device (Q10):** whether gamescope capture includes the Steam overlay, and window capture under KDE.
- **WP-01 owns:** the CI apt line `libpipewire-0.3-dev libspa-0.2-dev libclang-dev clang pkg-config`, the `capture` feature gate, and the `frame.rs` types.

**Depends on:** WP-01 (frame.rs, feature gate, CI apt). **Owner device:** Q10 (`sessions/g2-viewer.toml`).
**Done-check:** `distrobox enter craft-build -- env CARGO_TARGET_DIR=<scratch>/target cargo clippy --locked -p flat3d-viewer --all-targets --features capture -- -D warnings`, then the same command with `--features xr,capture`. Also run `cargo check --locked --workspace` on the host, with no features, which must pass without the pipewire headers. CI's apt job runs the capture clippy too.

---

Review note: none rejected. All 5 corrections hold against the repo:
- `CreateRemoteThread` is gated by `#[cfg(feature = "Win32_Security")]` (windows-sys-0.61.2 Threading/mod.rs:72).
- `GetProcAddress` takes `PCSTR = *const u8` (LibraryLoader/mod.rs:33; core/mod.rs:10).
- `ApiError` does not exist yet anywhere in `inject/src/`, so WP-01 must add `ViewerMissing`.
- On this Deck, `x86_64-pc-windows-msvc` is already installed, so WP-00's check passes without a download. mingw is absent, so the `-gnu` fallback is not available here.

### WP-19: Viewer OpenXR + main (#44c)
**Closes:** #44 part c. **Files (exclusive):** `crates/viewer/src/xr/session.rs`, `crates/viewer/src/xr/swapchain.rs`, `crates/viewer/src/xr/frame.rs`, `crates/viewer/src/gpu/import.rs`, `crates/viewer/src/main.rs`.

**Signatures** (every file except main is `#![cfg(feature="xr")]`; `use ash::vk::{self, Handle}`; `openxr as xr`):
```rust
// session.rs
pub struct XrContext { pub entry: xr::Entry, pub instance: xr::Instance, pub system: xr::SystemId, pub blend: xr::EnvironmentBlendMode,
  pub vk_entry: ash::Entry, pub vk_instance: ash::Instance, pub physical: vk::PhysicalDevice, pub device: ash::Device, pub queue: vk::Queue, pub queue_family: u32 }
pub enum XrError { NoLoader(String), Retry(String) /*runtime or headset not there*/, Missing(&'static str), Vk(vk::Result), Xr(xr::sys::Result) }
impl std::fmt::Display for XrError {..}
pub fn connect(app: &str) -> Result<XrContext, XrError>;
pub struct Session { pub session: xr::Session<xr::Vulkan>, pub waiter: xr::FrameWaiter, pub stream: xr::FrameStream<xr::Vulkan>, pub local: xr::Space, pub view: xr::Space, pub running: bool }
pub enum Pump { Idle, Running, Recenter, Exit }
pub fn start(ctx: &XrContext) -> Result<Session, XrError>;
pub fn pump_events(ctx: &XrContext, s: &mut Session, buf: &mut xr::EventDataBuffer) -> Result<Pump, XrError>;
// swapchain.rs
pub const PREFERRED: [vk::Format; 2] = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];
pub fn pick_format(offered: &[u32]) -> Option<vk::Format>;          // pure
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct SwapKey { pub width: u32, pub height: u32, pub format: vk::Format }
pub fn needs_recreate(cur: Option<SwapKey>, want: SwapKey) -> bool;  // pure
pub struct QuadSwapchain { pub key: SwapKey, pub handle: xr::Swapchain<xr::Vulkan>, pub images: Vec<vk::Image> }
impl QuadSwapchain { pub fn create(s: &Session, key: SwapKey) -> Result<Self, XrError>; pub fn acquire(&mut self) -> Result<u32, XrError>; pub fn release(&mut self) -> Result<(), XrError> }
pub fn ensure(slot: &mut Option<QuadSwapchain>, s: &Session, want: SwapKey) -> Result<bool, XrError>;
// frame.rs
pub fn yaw_only(p: xr::Posef) -> xr::Posef;                      // pure
pub fn quad_pose(anchor: xr::Posef, distance_m: f32) -> xr::Posef; // pure: anchor + forward(-Z)*distance
pub fn submit(s: &mut Session, state: &xr::FrameState, blend: xr::EnvironmentBlendMode,
              sc: Option<&QuadSwapchain>, eyes: [xr::Rect2Di; 2], size: xr::Extent2Df, pose: xr::Posef) -> Result<(), XrError>;
// gpu/import.rs
pub const DEVICE_EXTENSIONS: [&std::ffi::CStr; 4];  // khr::external_memory_fd, ext::external_memory_dma_buf, ext::image_drm_format_modifier, ext::queue_family_foreign (::NAME)
pub enum ImportError { Unsupported(String), Vk(vk::Result) }
pub fn drm_fourcc_to_vk(fourcc: u32) -> Option<vk::Format>;  // pure
pub struct Importer { /* cmd pool+buffer, fence, staging buffer, upload image, dmabuf cache (≤8, keyed (st_dev,st_ino,modifier)) */ }
impl Importer { pub fn new(ctx: &XrContext) -> Result<Self, ImportError>;
  pub fn copy_to(&mut self, ctx: &XrContext, frame: &crate::source::Frame, dst: vk::Image) -> Result<(), ImportError>; pub fn reset(&mut self); }
// main.rs (bin, required-features xr,capture)
fn main() -> std::process::ExitCode;
fn split_until_pid(args: Vec<String>) -> Result<(Vec<String>, Option<i32>), String>;
struct PidWatch(Option<std::os::fd::OwnedFd>); impl PidWatch { fn open(pid: i32) -> Self; fn gone(&self) -> bool }
```
**Behaviour**
- **connect.** Call `unsafe { xr::Entry::load(&()) }`. In 0.22 it takes a `PlatformInfo`; Linux passes `&()`, which loads `libopenxr_loader.so.1`. A failure gives `NoLoader`.
  - Require `enumerate_extensions()?.khr_vulkan_enable2`.
  - `create_instance(&ApplicationInfo{application_name:"flat3d-viewer",application_version:0,engine_name:"flat3d",engine_version:0,api_version:xr::Version::new(1,0,0)}, &exts, &[], &())`.
  - `system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)`: `ERROR_FORM_FACTOR_UNAVAILABLE`, `ERROR_RUNTIME_UNAVAILABLE` and `RUNTIME_FAILURE` give `Retry`.
  - Check `graphics_requirements::<xr::Vulkan>`.
  - Vulkan: `ash::Entry::load()` at API 1.2, then `create_vulkan_instance(system, std::mem::transmute::<vk::PFN_vkGetInstanceProcAddr, openxr::sys::platform::VkGetInstanceProcAddr>(vk_entry.static_fn().get_instance_proc_addr), &ci as *const _ as _)`.
    - The annotated transmute is required: without it, clippy `missing_transmute_annotations` fails under `-D warnings`.
    - The types: ash `PFN_vkGetInstanceProcAddr` (ash-0.38.0 src/vk/features.rs:6) and openxr-sys 0.14.0 `platform::VkGetInstanceProcAddr = unsafe extern "system" fn(VkInstance, *const c_char) -> Option<unsafe extern "system" fn()>` (https://docs.rs/openxr-sys/0.14.0/openxr_sys/platform/type.VkGetInstanceProcAddr.html).
  - Then `vulkan_graphics_device`, then `create_vulkan_device` with `DEVICE_EXTENSIONS`.
  - Pick one graphics queue family. Wrap raw handles with `vk::Instance::from_raw(raw as u64)`.
  - `blend = enumerate_environment_blend_modes(system, PRIMARY_STEREO)[0]`.
- **start.** `create_session::<xr::Vulkan>(system,&xr::vulkan::SessionCreateInfo{instance,physical_device,device,queue_family_index,queue_index:0})`, then `local = create_reference_space(LOCAL, Posef::IDENTITY)` and `view = create_reference_space(VIEW, IDENTITY)`.
- **pump_events.**
  - `READY`: `begin(PRIMARY_STEREO)`; the session is running.
  - `STOPPING`: `end()`.
  - `EXITING` or `LOSS_PENDING`: `Exit`.
  - `ReferenceSpaceChangePending`: `Recenter`.
- **Swapchain.** One image, the same size as the frame.
  - Format from `pick_format(session.enumerate_swapchain_formats())`. Create flags `EMPTY`, usage `COLOR_ATTACHMENT|TRANSFER_DST|SAMPLED`, `sample_count/face_count/array_size/mip_count = 1`.
  - Recreate when the frame's (w, h, format) changes (`needs_recreate`). Drop the old swapchain only after `stream.end`.
  - `acquire` = `acquire_image()` + `wait_image(xr::Duration::INFINITE)`. The image arrives in `COLOR_ATTACHMENT_OPTIMAL`; the importer moves it to `TRANSFER_DST_OPTIMAL` and back before `release`.
- **Frame.**
  1. `waiter.wait()`, then `stream.begin()`.
  2. Take the newest source frame without blocking (`next(Duration::ZERO)`). If there is one: ensure the swapchain, acquire, `copy_to`, release.
  3. If there is no new frame, re-submit the last released image (quad layers allow this).
  4. Build two `xr::CompositionLayerQuad::new().space(&local).eye_visibility(xr::EyeVisibility::LEFT|RIGHT).sub_image(xr::SwapchainSubImage::new().swapchain(&h).image_rect(eyes[i]).image_array_index(0)).pose(pose).size(size)` and end with `stream.end(state.predicted_display_time, blend, &[&l, &r])`.
  - Leave `layer_flags` empty: the layer is opaque, and BGRx alpha is garbage.
  - With no swapchain yet, end with `&[]`.
  - `eyes` and `size` come from WP-17 `layout::eye_rects`/`quad_size(rect, screen_width_m)`; swap_eyes and crop are handled there.
- **Recenter.** On the first running frame, on `Pump::Recenter`, and on SIGUSR1 (a `libc::sigaction` handler sets a static `AtomicBool`): `view.locate(&local, t)`, then `anchor = yaw_only(pose)`, `pose = quad_pose(anchor, distance_m)`.
- **import.**
  - `drm_fourcc_to_vk`: XR24 `0x34325258` and AR24 `0x34325241` → `B8G8R8A8_SRGB`; XB24 `0x34324258` and AB24 `0x34324241` → `R8G8B8A8_SRGB`; anything else → None. Use SRGB views so the blit does not encode twice.
  - **DmaBuf** (single plane only; more planes → `Unsupported`):
    1. Check `get_physical_device_format_properties2` + `DrmFormatModifierPropertiesListEXT` for `TRANSFER_SRC|BLIT_SRC`.
    2. Create the image with tiling `DRM_FORMAT_MODIFIER_EXT`, usage `TRANSFER_SRC`, `ExternalMemoryImageCreateInfo{DMA_BUF_EXT}` and `ImageDrmFormatModifierExplicitCreateInfoEXT{drm_format_modifier, plane_layouts:[SubresourceLayout{offset,row_pitch:stride,..}]}`.
    3. Memory type = `get_memory_fd_properties(DMA_BUF_EXT, fd)` ∩ the image requirements.
    4. Import with `ImportMemoryFdInfoKHR` using a **`libc::dup`** fd (Vulkan owns it), plus `MemoryDedicatedAllocateInfo`.
    5. Barrier: `src_queue_family_index=vk::QUEUE_FAMILY_FOREIGN_EXT` → ours, `old_layout GENERAL` → `TRANSFER_SRC_OPTIMAL`.
  - **MemFd/shm:** copy row by row (honouring stride) into a host-visible staging buffer, `cmd_copy_buffer_to_image` into the upload image, then continue as for DmaBuf.
  - Finally `cmd_blit_image` 1:1 (`Filter::NEAREST`) into the swapchain image, submit, and wait on the fence. `reset()` drops the cache when the format or size changes.
- **main.**
  - `split_until_pid` strips `--until-pid N`. The rest goes to `ViewerSpec::from_args`; an error prints to stderr and exits 2.
  - `PidWatch::open`: `libc::syscall(libc::SYS_pidfd_open, pid, 0)`.
    - `ESRCH`: exit 0 at once.
    - `ENOSYS`: fall back to `kill(pid,0)`.
    - `gone()` = `poll(POLLIN, 0)` reports readable.
    - The watch is checked every loop and during retries. When the pid is gone: `request_exit`, exit 0.
  - Source from `spec.source`: `TestPattern` (WP-17), `Gamescope` and `Portal` (WP-18). `Auto` tries gamescope, then portal.
  - Status lines use WP-17 `status`: one JSON object per line on stdout, flushed.
    - `waiting-headset` while `Retry` (retry every 2s);
    - `waiting-source`;
    - `showing`, on change and every 5s, with fps/w/h;
    - `error`.
  - Exit codes: 0 when the pid is gone or the session exits; 2 usage; 3 fatal (`NoLoader`, `Missing`, Vk).

**Tests:** CI clippy only; there are no locked tests. Inline `#[cfg(test)]` in each WP-19 file:
- `pick_format`: prefers B8G8R8A8_SRGB, falls back to R8G8B8A8_SRGB, None for UNORM-only.
- `needs_recreate`: None→true, same→false, a w/h/format change→true.
- `yaw_only`: pitch and roll dropped, yaw kept, unit quaternion.
- `quad_pose`: identity + 2.5 → z = −2.5.
- `drm_fourcc_to_vk`: the 4 codes, plus NV12 `0x3231564e` → None.
- `split_until_pid` in main.rs (built only with xr,capture): `--until-pid` in the middle and at the end parse; a missing value → Err; a non-numeric value → Err.

**Inline facts:**
- openxr `=0.22.0` (`loaded`), openxr-sys 0.14.0.
- ash `=0.38.0` (`loaded`). Extension modules: `ash::ext::{external_memory_dma_buf,image_drm_format_modifier,queue_family_foreign}`, `ash::khr::external_memory_fd`.
- **libc** `"0.2"` is a normal (non-optional) dependency of `crates/viewer/Cargo.toml`, used for sigaction, dup, the pidfd syscall, poll and kill. WP-01 must add it. It is absent today (engine sys.rs is deliberately libc-free, roadmap.md:207). Cargo.lock resolves an existing 0.2.x (local registry has up to 0.2.191).
- `FrameStream::end(&mut self, Time, EnvironmentBlendMode, &[&CompositionLayerBase<'_, G>])`; `EyeVisibility::{BOTH,LEFT,RIGHT}`.
- Host loader: `/usr/lib/libopenxr_loader.so.1.1.60`.
- Monado honours quad eyeVisibility (oxr_session_frame_end.c:161-176, comp_render_helpers.h:82-100).
- Do not set `XR_RUNTIME_JSON`.
- Defaults: `--layout sbs --source auto --screen-width-m 3 --distance-m 2.5` (geo11-viewer.md:133).

**Depends on:** WP-01, **WP-17, WP-18**, so this runs in **wave 3**. WP-19 calls `layout`, `status` and `test_pattern` (WP-17), `gamescope` and `portal` (WP-18), and `source::Frame`, so it builds on their merged code and needs no `todo!()` stubs. **Owner device:** Q10 (`sessions/g2-viewer.toml`).

**Done-check:**
1. Host: `CARGO_TARGET_DIR=$T cargo clippy --locked -p flat3d-viewer --all-targets --features xr -- -D warnings && cargo test --locked -p flat3d-viewer --features xr`.
2. Then: `distrobox enter craft-build -- env CARGO_TARGET_DIR=$T cargo clippy --locked -p flat3d-viewer --all-targets --features xr,capture -- -D warnings`.

- **Flag:** WP-17/WP-18's `Frame` must carry `width`, `height`, `fourcc: u32` and `FrameData::{DmaBuf{fd, offset, stride, modifier}, Mem{bytes, stride}}`. Verify this against the merged WP-17/18 code before starting.
- **Flag:** WP-18 must negotiate only BGRx/BGRA/RGBx/RGBA, with `SPA_VIDEO_FORMAT_BGRx` first. There is no NV12 path.
- **Flag:** the tree lists `gpu/import.rs` but no `gpu.rs`. WP-01 must add the `#[cfg(feature="xr")] pub mod gpu;` facade (`gpu.rs` with `pub mod import;`).
- **Flag (WP-01):** add `libc = "0.2"` to `crates/viewer/Cargo.toml` (see Inline facts).

---

### WP-20: Install execute + launch options (#128)
**Closes:** #128 (code), A15, K24. **Files (exclusive):** `crates/engine/src/install.rs`, `crates/engine/src/launch_options.rs`, `packaging/flat3d-link.service`, `crates/engine/tests/install_execute.rs`, `crates/engine/tests/launch_options_at.rs`.

**Signatures** (checked against install.rs:3-91 and launch_options.rs:3-114; `CommandRunner` = `crate::exec::CommandRunner { fn run(&mut self,&[String])->CommandOutput{success,stdout,stderr} }`):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSpec { pub description: String, pub exec_start: String, pub restart_sec: u32,
                      pub start_limit_interval_sec: u32, pub start_limit_burst: u32, pub wanted_by: String }
impl Default for UnitSpec { /* "Flat to 3D headset link", "%h/.local/bin/flat3d-inject watch", 3, 60, 10, "default.target" */ }
pub fn unit_text_with(spec: &UnitSpec) -> String;
// InstallPlan / UninstallPlan: add #[derive(Debug, Clone)] and the new fields (locked tests only read fields)
pub struct InstallPlan { /* locked: copies, unit_path, unit_text, commands */ pub dirs: Vec<String>, pub trees: Vec<(String, String)>, pub manifest: String }
pub struct UninstallPlan { /* locked: before, removes, after */ pub home: String, pub manifest: String }
pub fn execute(plan: &InstallPlan, runner: &mut dyn CommandRunner) -> Result<InstallReport, ApiError>;
pub fn execute_uninstall(plan: &UninstallPlan, runner: &mut dyn CommandRunner) -> Result<InstallReport, ApiError>;
// launch_options.rs
pub fn add_wrapper_at(options: &str, wrapper_abs: &str) -> String;
```
**Behaviour**
- `unit_text_with(&UnitSpec::default())` is exactly:
  `"[Unit]\nDescription=Flat to 3D headset link\nStartLimitIntervalSec=60\nStartLimitBurst=10\n\n[Service]\nType=simple\nExecStart=%h/.local/bin/flat3d-inject watch\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n"`
  - `StartLimit*` belong in `[Unit]`.
  - No `KillMode`: the default `control-group` is documented by WP-02 in packaging/README.
  - No linger.
  - `unit_text`, `install_plan` and `uninstall_plan` stay as they are (locked install_plan.rs:14-19,29-72).
- `install_plan` also fills:
  - `dirs = [bin_dir, tools_dir, {home}/.local/share/flat3d/recipes, {home}/.config/systemd/user]`;
  - `trees = [({src}/tools/lib, {tools}/lib), ({src}/recipes, {home}/.local/share/flat3d/recipes)]`: files only, one level deep; recipes filtered to `*.recipe`;
  - `manifest = {home}/.local/share/flat3d/installed.txt`.
- execute:
  1. `fs::create_dir_all` on each dir.
  2. Copy each binary. A missing `{src}/flat3d-inject|flat3d-run` → `Err(ApiError::new("io", "missing <path>"))`, before any command runs.
  3. Copy each tool. A missing tool source is skipped and noted in the summary as "not bundled: adb, …" (ruling 34: prefer the running adb).
  4. Copy the trees when the source dir exists. `chmod 0755` the binaries, adb, gnirehtet and lib files; 0644 for the .apk and recipes.
  5. Write the unit to `plan.unit_path` atomically (tmp + rename) with `unit_text_with(&UnitSpec::default())`, **not** `plan.unit_text` (critique:34).
  6. Write the manifest: every file written, one absolute path per line.
  7. Run `plan.commands` in order. A non-success → `Err(ApiError::new("command-failed", "<argv joined>: <stderr last line>"))`, leaving the files in place.
  8. Return `InstallReport{written, ran: argv joined by " ", summary}`.
- execute_uninstall:
  1. Run `before`. A failure is tolerated and noted, since the unit may not exist.
  2. Remove `removes`, the manifest entries and the manifest itself. NotFound is ignored. Refuse any path not under `{home}/.local/` or `{home}/.config/systemd/user/`.
  3. Run `after`; a failure → command-failed.
  - Never touch `~/.config/flat3d` or `~/.local/state`.
- `add_wrapper_at`:
  - If `wrapper_abs` contains whitespace or `"` → `add_wrapper(options)` (bare fallback).
  - If `has_wrapper`: replace the first wrapper word before `%command%` with `wrapper_abs` and drop any further ones. This upgrades bare to absolute and is idempotent.
  - Otherwise behave like `add_wrapper` with `wrapper_abs` in place of `WRAPPER`: after any leading env words, or in the form `wrapper_abs %command% <rest>` when `%command%` is absent.
  - Removing either form already works: `is_wrapper` matches the last path segment (launch_options.rs:142-144; locked launch_options.rs:50), so `remove_wrapper` is unchanged.

**Tests:** The locked tests stay green unchanged: `tests/install_plan.rs`, `tests/launch_options.rs`, and the unlocked `launch_options_quotes.rs`. New tests:
- `install_execute.rs`: temp HOME via `env!("CARGO_TARGET_TMPDIR")`, and a local `Fake` implementing `CommandRunner` that records argv.
  - `default_unit_text_is_exactly_this`;
  - `packaging_unit_file_equals_unit_text_with_default` (`include_str!("../../../packaging/flat3d-link.service")`);
  - `custom_exec_start`;
  - `execute_copies_chmods_writes_unit_then_reloads_and_enables`: argv order is `systemctl --user daemon-reload`, then `systemctl --user enable --now flat3d-link.service`; mode `0o755`;
  - `missing_tools_are_skipped_and_named`;
  - `missing_binary_is_io_and_runs_nothing`;
  - `systemctl_failure_is_command_failed`;
  - `execute_twice_is_fine`;
  - `uninstall_tolerates_missing_unit_and_removes_manifest_files`;
  - `uninstall_keeps_config_and_state`;
  - `nothing_mentions_linger`: no argv contains `loginctl` or `enable-linger`.
- `launch_options_at.rs`, with `A="~/.local/bin/flat3d-run"`:
  - `""`→`A %command%`; `"%command%"`→`A %command%`;
  - `"mangohud %command% -novid"`→`A mangohud %command% -novid`;
  - `"FOO=1 BAR=2 %command%"`→`FOO=1 BAR=2 A %command%`;
  - `"-novid"`→`A %command% -novid`;
  - `"flat3d-run %command%"`→`A %command%`; `"FOO=1 flat3d-run %command%"`→`FOO=1 A %command%`; `"/opt/x/flat3d-run %command%"`→`A %command%`;
  - idempotent: `f(f(x))==f(x)`;
  - `remove_wrapper(add_wrapper_at(x,A))` gives the locked remove-test pairs;
  - `%command% -arg "a b"` keeps its quotes; `"%command% flat3d-run"`→`A %command% flat3d-run`;
  - an abs path with a space → the bare form.

**Inline facts:**
- `UNIT_NAME="flat3d-link.service"`.
- `tools_dir = {home}/.local/share/flat3d/bin` (tether/adb.rs:30).
- `api::InstallReport{written,ran,summary}` (api.rs:255); `ApiError::new(&str,&str)` (api.rs:95).
- The user manager has `XDG_RUNTIME_DIR=/run/user/1000`.
- Ruling 34: prefer the running adb.

**Depends on:** WP-01. **Owner device:** Q3 (`sessions/link-smoke.toml`: `flat3d-inject install`, then `systemctl --user is-active flat3d-link.service`).

**Done-check:** `CARGO_TARGET_DIR=$T cargo test --locked -p flat3d-inject --test install_execute --test launch_options_at --test install_plan --test launch_options --test launch_options_quotes`

- **Flag:** WP-08 calls install/uninstall in the same wave, so WP-01 must stub `install::{UnitSpec, unit_text_with, execute, execute_uninstall}` and `launch_options::add_wrapper_at` with these signatures.
- **Flag:** the scope says `execute(..)->InstallReport`. Here it returns `Result<InstallReport, ApiError>`, because a copy or systemctl failure needs an error code.

---

### WP-21: Decky scaffold (#130)
**Closes:** #130, A13. **Files (exclusive):** `decky/main.py`, `decky/plugin.json`, `decky/package.json`, `decky/pnpm-lock.yaml`, `decky/rollup.config.js`, `decky/tsconfig.json`, `decky/src/index.tsx`, `decky/src/engine.ts`, `decky/src/types.ts`, `decky/tests/test_run_engine.py`, `decky/README.md`.

**Signatures**
```python
ENGINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", "flat3d-inject")  # tests patch main.ENGINE
def _failure(command: str, code: str, message: str) -> dict
def _env() -> dict          # setdefault XDG_RUNTIME_DIR=/run/user/<uid>, DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/<uid>/bus, FLAT3D_SRC_DIR=dirname(ENGINE)
def _fix_modes() -> None    # chmod 0o755 ENGINE, bin/flat3d-run, bin/tools/{adb,gnirehtet}; OSError ignored; once per process
async def run_engine(*args: str, timeout: float = 30.0) -> dict
class Plugin:  # list_games(), inject(appid=None), start_3d() kept (#67); engine(self, args: list) -> dict; _main(); _unload()
```
```ts
// engine.ts
export type ErrorCode = "usage"|"not-built"|"not-found"|"bad-recipe"|"refused"|"link-not-ready"|"command-failed"|"io";
export interface ApiError { code: ErrorCode; message: string }
export type Envelope<T> = { ok: true; command: string; data: T } | { ok: false; command: string; error: ApiError };
export async function engine<T>(...args: string[]): Promise<Envelope<T>>;
export const view = () => engine<LinkView>("view"); export const rows = () => engine<GameRow[]>("rows");
export const start3d = () => engine<LinkReport>("start-3d"); export const stop3d = () => engine<LinkReport>("stop-3d");
export const setSwitch = (appid: number, on: boolean) => engine<SwitchState>("switch","set",String(appid),on?"on":"off");
export const setSetting = (key: "auto-3d"|"tunnel", on: boolean) => engine<SettingState>("setting","set",key,on?"on":"off");
export const launch = (verb: "add"|"remove", line: string) => engine<LaunchText>("launch", verb, line);
export const launchSync = (opts: string) => engine<LaunchText>("launch-sync","--json","--options",opts);
export const doctor = () => engine<DoctorReport>("doctor");
export const collection = (current?: number[]) => engine<CollectionPlan>(...(current ? ["collection","--current",current.join(",")] : ["collection"]));
export const toasts = (lastSeq: number) => engine<ToastBatch>("toasts", String(lastSeq));
export const logs = (limit: number) => engine<LogLines>("logs", "--limit", String(limit));
```
**Behaviour**
- main.py follows #67's brief exactly:
  - argv is `[ENGINE,*args,"--json"]`, with stdout and stderr piped;
  - `OSError` → `io "cannot run the engine: {e}"`;
  - `asyncio.wait_for(proc.communicate(), timeout)`; on timeout, `kill`+`wait`, then `command-failed "timed out after {timeout:g}s"`;
  - a dict with a bool `ok` is returned unchanged, whatever the exit code;
  - anything else → `command-failed "engine printed invalid JSON (exit code {rc}): {detail[:200]}"`.
- `_env()` is passed as `env=`. `_fix_modes()` runs in `_main`, and lazily before the first exec. Guard the import as `try: import decky except ImportError: decky = None`; the logger falls back to `logging.getLogger("flat3d")`.
- Delete `_ = (asyncio, json)` and the "not wired up" docstrings. `Plugin.engine` is the only generic door, with no rules: `run_engine(*[str(a) for a in args])`.
- plugin.json keeps its current keys and adds `"api_version": 1`; without it Decky uses the legacy API (plugin.py:43). `flags` stays `[]`, so the plugin runs as deck.
- **launch-sync argv.** `launchSync` passes `['launch-sync','--json','--options',opts]` so that `--options` and its value come last, as WP-28's parser expects.
  - main.py still appends its own trailing `--json`; its argv shape is locked by #67's order test and must not change.
  - The duplicate is harmless, because the CLI drops every `--json` wherever it appears (`args.iter().any(|a| a == "--json")` and `.filter(|a| *a != "--json")`, inject/src/cli.rs:58-62).
- index.tsx is a scaffold; WP-32 replaces it.
  - `definePlugin(() => ({ name: "Flat to 3D", titleView: <div className={staticClasses.Title}>Flat to 3D</div>, content: <Content/>, icon: <FaVrCardboard/> }))`.
  - `Content` polls `view()` every 1s while mounted and shows `Field label={v.label} description={v.hint}`, or `error.message`.
- types.ts holds hand copies of the code-map API table (code-map.md:128-150), plus what WP-32 imports:
  - `LinkState` = `"no-headset"|"unauthorised"|"starting"|"ready"|"stopped"`; `TetherMode` = `"off"|"own"|"quest-tether"`; `Zone` = `"green"|"yellow"|"red"|"unknown"`;
  - `LinkEvent{seq,kind,text}`, `HealthView{battery,temp,zone,colour}`;
  - `LinkView{state,label,hint,start,stop,health,rx,tx,uptime,tether,tunnel,auto_3d,adb_source}`;
  - `GameRow{appid,name,status,status_label,status_colour,switch_on,plays_3d,flat_reason:string|null}`;
  - `LaunchText{options}`, `CollectionPlan{name,wanted,add,remove}`, `ToastBatch{toasts,last_seq}`, `LogLines{lines,text}`;
  - `LinkReport{state,message}`, `SwitchState{appid,on}`, `SettingState{key,on}`;
  - **`export interface DoctorCheck{name:string; ok:boolean; detail:string; fix:string}`**, **`export interface DoctorReport{checks:DoctorCheck[]; ok:boolean}`**, **`export const TOAST_TITLE = "Flat to 3D"`** (mirrors roadmap.md:221-222 `api.rs`; WP-32 imports all three).
  - If WP-01's `api.rs` differs, `api.rs` wins.
- README covers:
  - the build: `pnpm i --frozen-lockfile --offline && pnpm run build`, using pnpm 9.15.9;
  - the shim's role (OD1);
  - the `bin/` layout = `FLAT3D_SRC_DIR` (WP-22).

**Files verbatim:**
- package.json:
  - `{"name":"flat-to-3d","version":"0.1.0","type":"module","license":"GPL-3.0","private":true,"scripts":{"build":"rollup -c","typecheck":"tsc --noEmit"}}`;
  - devDependencies `@decky/rollup 1.0.2`, `@decky/ui 4.16.0`, `@types/react 19.1.1`, `@types/react-dom 19.1.1`, `rollup 4.53.3`, `typescript 5.9.3`;
  - dependencies `@decky/api 1.1.3`, `tslib 2.7.0`, `react-icons 5.3.0`;
  - `"pnpm":{"peerDependencyRules":{"ignoreMissing":["react","react-dom"]}}`;
  - **no** `packageManager`, which clashes with `pnpm/action-setup` `version:`. Any CI `pnpm/action-setup` uses `version: 9.15.9`.
- rollup.config.js: `import deckyPlugin from "@decky/rollup"; export default deckyPlugin({});`
- tsconfig.json: `{"compilerOptions":{"outDir":"dist","module":"ESNext","target":"ES2020","jsx":"react-jsx","declaration":false,"moduleResolution":"node","noUnusedLocals":true,"noUnusedParameters":true,"esModuleInterop":true,"noImplicitReturns":true,"noImplicitThis":true,"noImplicitAny":true,"strict":true,"allowSyntheticDefaultImports":true,"skipLibCheck":true},"include":["src"],"exclude":["node_modules"]}`
- pnpm-lock.yaml is produced by the **WP-00 network pre-step** (below) and committed by WP-21 unchanged. Its lockfileVersion is `'9.0'`, and its integrity values must equal the pins research, e.g. `@decky/api` `sha512-XsPCZxfx…MEOOBA==`.

**Network (declared):** WP-21 needs the npm registry, once, and only in a WP-00 pre-step.
- Use the locally installed **pnpm 9.15.9** (`~/.nvm/versions/node/v25.9.0/bin/pnpm`; store `~/.local/share/pnpm/store/v3`). There is no `npx pnpm@10…` and no corepack (not installed).
- The local cache cannot serve this install offline:
  - pnpm's metadata cache holds only `pnpm.json`, so `--lockfile-only --offline` cannot resolve;
  - the store lacks `@decky/ui 4.16.0` (has 4.11.6/4.12.0/4.16.4), `rollup 4.53.3` (has 4.52.5/4.62.2), `react-icons 5.3.0` (has 5.7.0) and `tslib 2.7.0` (has 2.8.1/2.9.1).
- Pre-step, in a scratch dir holding the verbatim package.json: `pnpm install --lockfile-only`, then `pnpm fetch` (warms the store). Hand `pnpm-lock.yaml` to WP-21.
- After that, WP-21 itself and its done-check run `--offline`.

**Locked tests and facade checklist** (owner answer 1: full reshape, "even if locked tests need updating"):
- `decky/tests/test_run_engine.py` lands **with WP-21**, as the **first** commit, byte-identical to the #67 body's "Tests (locked)" block. Its path is `decky/tests/test_run_engine.py`; it does `sys.path.insert(0, <tests>/..)` then `import main`. There are 9 cases: success, order, nonzero-envelope, nonzero-no-json, bad-json, not-envelope, timeout 0.3s, missing engine, no args. It is Python, so there is no fmt pass; it stays byte-identical. #67 is CLOSED (not planned), but its body is still the contract.
- Rust issue tests (#76, #116, #117, #118; landed by their own WPs) follow the same procedure:
  1. Commit the test byte-identical to the issue body.
  2. Make a separate rustfmt-only commit.
  3. Check that commit with `git diff --stat -w HEAD~1 HEAD -- <test>`. It must show nothing; any non-whitespace rustfmt change, such as import reordering, gets a review note.
- The facade must keep these paths resolvable (lib name `flat3d_inject`, inject/Cargo.toml:9-11; `flat3d_app`, app/Cargo.toml:9-10):
  - #76 `inject/tests/rows.rs` → `crates/engine/tests/rows.rs`: `flat3d_inject::api::{Game, LinkState}`, **`flat3d_inject::rows::game_rows`**, `flat3d_inject::switches::Settings`.
  - #116 `app/tests/headset_page.rs`: `flat3d_app::headset_page::{update, Action, HeadsetState, Msg}`, `flat3d_inject::api::{LinkSnapshot, LinkState}`.
  - #117 `app/tests/games_page.rs`: `flat3d_app::games_page::{update, Action, GamesState, Msg}`, **`flat3d_inject::rows::GameRow`**.
  - #118 `app/tests/setup_page.rs`: `flat3d_app::setup_page::{all_done, steps}`, `flat3d_inject::api::{AdbSource, LinkSnapshot, LinkState}`, `flat3d_inject::setup::Packages`.
  - So WP-01's `lib.rs` keeps `pub use face::rows;` at the crate root, because `rows` moves to `face/rows.rs` (roadmap.md:24,232,269), and keeps `pub mod api`, `pub mod switches` and `pub mod setup`. These exist today at lib.rs:8,21,28: `Game` api.rs:6, `LinkState` :47, `AdbSource` :149, `LinkSnapshot` :200, `Settings` switches.rs:9, `Packages` setup.rs:18.

**Signatures check (TS ↔ Python):** `Plugin.engine(args)` calls `run_engine(*args)` → argv `[ENGINE,*args,"--json"]`.

**Tests:**
- Locked: `decky/tests/test_run_engine.py`, as above.
- No new Python test files, since the file list is exclusive. The `_env` setdefault behaviour is checked in the Q14 session.

**Inline facts:**
- Decky marks a plugin with no `main.py` "passive" (decky-loader plugin.py:46,118-130).
- The plugin process has no `XDG_RUNTIME_DIR` (sandboxed_plugin.py:69-95; plugin_loader-release.service).
- `ZipFile.extractall` drops mode bits, hence `_fix_modes`.
- The error codes are the 8 in api_response.rs:11.
- CLI forms come from cli_link.rs: `switch set <appid> on|off`, `setting set auto-3d|tunnel on|off`.
- The CLI strips `--json` anywhere in argv (cli.rs:58-62).

**Depends on:** WP-01, plus the WP-00 network pre-step (lockfile and warm store). **Owner device:** Q14 (`sessions/decky.toml`).

**Done-check:** `python3 -m py_compile decky/main.py && python3 -m unittest discover -s decky/tests -v && (cd decky && pnpm i --frozen-lockfile --offline && pnpm run typecheck && pnpm run build && test -f dist/index.js)`

- **Flag:** the scope's `run_engine(args)` must be `run_engine(*args, timeout=30.0)`, because the locked test calls `run_engine("list")` and `run_engine()`.
- **Flag:** the view, launch, launch-sync, doctor, collection, toasts and logs commands land in WP-28 (wave 3). Until then they return `not-built`, and the scaffold shows that message.

---

### WP-22: Packaging scripts (#127/#129/#134)
**Closes:** #127, #129 and #134 (script halves). **Files (exclusive):** `packaging/build-tools.sh`, `packaging/build-appimage.sh`, `packaging/AppRun`, `packaging/flat3d.desktop`, `packaging/build-decky.sh`, `packaging/tests/test_packaging.py`, `.github/workflows/appimage.yml`.

**Interfaces.** Every script is bash with `set -euo pipefail`, clean under shellcheck, and has:
- `pin(){ v=$(grep -m1 "^$1: " "$PINS" | cut -d' ' -f2-); [ -n "$v" ] && [ "$v" != TBD-at-pin ] || { echo "pin $1 missing or TBD-at-pin: run packaging/pin.sh" >&2; exit 2; }; printf '%s\n' "$v"; }`
- `PINS=${PINS:-$ROOT/packaging/pins}`
```
packaging/build-tools.sh    [--dry-run] [--no-box] [--pins FILE] [--out DIR=dist/tools]
packaging/build-appimage.sh [--dry-run] [--no-tools] [--tools DIR=dist/tools] [--skip-build] [--pins FILE] [--out FILE=dist/Flat_to_3D-x86_64.AppImage]
packaging/build-decky.sh    [--dry-run] [--no-tools] [--tools DIR] [--bin-dir DIR] [--root DIR] [--skip-build] [--out FILE=dist/flat-to-3d.zip]
```
**Behaviour**
- **build-tools.sh** (#127, ruling 34).
  - `--dry-run` prints every step below as `+ <argv>` and exits 0 with no network, container or writes.
  - Unless `--no-box`, or already inside a container (`/run/.containerenv`), re-exec via `distrobox enter "${FLAT3D_BUILD_BOX:-craft-build}" -- env FLAT3D_IN_BOX=1 bash "$0" "$@"`.
  - Preflight: `cmake ninja pkg-config patchelf unzip sha256sum curl ldd`. List what is missing and exit 3; it never installs anything.
  1. Fetch `android-tools.url` and `gnirehtet.url`, each into its own fresh `mktemp -d "${XDG_CACHE_HOME:-$HOME/.cache}/flat3d/dl.XXXXXX"`, using `curl -fL --proto '=https' --tlsv1.2 --retry 3 -o`. Then `echo "<sha>  <file>" | sha256sum -c`; a mismatch exits 4.
  2. `tar -xJf` the tarball, then `cmake -GNinja -S android-tools-<version> -B build -DCMAKE_BUILD_TYPE=Release && ninja -C build adb`. Find the binary with `find build -type f -name adb -perm -u+x`.
  3. Copy it to `$out/adb`. Copy its `ldd` closure into `$out/lib/`, minus `linux-vdso ld-linux-x86-64 libc libm libpthread libdl librt libudev`. Run `patchelf --set-rpath '$ORIGIN/lib' $out/adb` and `--set-rpath '$ORIGIN'` on each lib.
  4. `unzip -j -o <zip> '*/gnirehtet' '*/gnirehtet.apk' -d $out`, then chmod 755 adb/gnirehtet/lib and 644 the apk.
  5. Write `$out/SHA256SUMS`. The APK has no published hash, so its hash is recorded here rather than in pins.
  6. Verify: `$out/adb version` matches `Android Debug Bridge`, and `ldd $out/adb` has no `not found`.
- **build-appimage.sh** (#129).
  - Unless `--skip-build`: `cargo build --release --locked -p flat3d-inject -p flat3d-app`.
  - Build the AppDir `$tmp/AppDir`:
    - `usr/bin/{flat3d-app,flat3d-inject,flat3d-run}`;
    - **symlinks** `usr/flat3d-inject -> bin/flat3d-inject` and `usr/flat3d-run -> bin/flat3d-run`: install_plan reads `{src}/flat3d-inject` (install.rs:34), and `fs::copy` follows links;
    - `usr/tools/` (from `--tools`, unless `--no-tools`) and `usr/recipes/*.recipe`;
    - `AppRun`, `flat3d.desktop`, `flat3d.svg` (written from a heredoc) and `.DirIcon -> flat3d.svg`.
  - Fetch and verify `appimagetool.*` and `appimage-runtime.*` the same way as build-tools.sh, `chmod +x` the tool, then run `APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 appimagetool --runtime-file <runtime> AppDir <out>`.
  - Bundle **no** wayland/xkbcommon/vulkan/GL/X11/drm/gbm/fontconfig libs (AppImage excludelist). Assert that no `*.so*` exists outside `usr/tools/lib`.
- **AppRun:**
  ```sh
  #!/bin/sh
  HERE=$(dirname "$(readlink -f "$0")"); APPDIR=${APPDIR:-$HERE}
  export FLAT3D_SRC_DIR="$APPDIR/usr"; export PATH="$APPDIR/usr/bin:$PATH"
  exec "$APPDIR/usr/bin/flat3d-app" "$@"
  ```
  Mode 0755. The installed unit never points into `$APPDIR`, because WP-20 copies the files out.
- **flat3d.desktop:** `[Desktop Entry]`, `Type=Application`, `Name=Flat to 3D`, `Comment=Play flat PC games in 3D on a Quest`, `Exec=flat3d-app`, `Icon=flat3d`, `Terminal=false`, `Categories=Game;Utility;`.
- **build-decky.sh** (#134; no Decky CLI, because it needs Docker).
  - Unless `--skip-build`: `pnpm -C decky i --frozen-lockfile && pnpm -C decky run build` (pnpm 9.15.9, matching WP-21's lockfile), and `cargo build --release --locked -p flat3d-inject`.
  - Stage `flat-to-3d/`:
    - `dist/index.js`, `package.json`, `plugin.json`, `main.py`, `README.md` (from `decky/`), and `LICENSE` (repo root);
    - `bin/flat3d-inject`, `bin/flat3d-run`, `bin/tools/…` (unless `--no-tools`) and `bin/recipes/*.recipe`. `bin/` is the `FLAT3D_SRC_DIR` that main.py exports, so `install_plan(home, bin)` matches.
  - chmod 755 the executables, then zip with `python3 -m zipfile -c "$out" flat-to-3d` (stdlib; keeps mode bits).
- **appimage.yml:** `on: workflow_dispatch`; job `appimage` with `runs-on: ubuntu-24.04`. Steps:
  1. `actions/checkout@v4`.
  2. `sudo apt-get update && sudo apt-get install -y --no-install-recommends build-essential pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev curl ca-certificates desktop-file-utils`.
  3. `id: pin`: `echo "channel=$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)" >> "$GITHUB_OUTPUT"`.
  4. **`dtolnay/rust-toolchain@master`** with `toolchain: ${{ steps.pin.outputs.channel }}`. This is WP-00's pinned `rust-toolchain.toml`, never `@stable`.
  5. `packaging/build-appimage.sh --no-tools` with env `APPIMAGE_EXTRACT_AND_RUN: "1"`.
  6. `actions/upload-artifact@v4` (`name: Flat_to_3D-x86_64.AppImage`, `path: dist/Flat_to_3D-x86_64.AppImage`).

**Tests:** `packaging/tests/test_packaging.py` (stdlib unittest, `subprocess` bash, temp dirs):
- **tools dry-run:** prints the android-tools URL and sha `2c6de1ab…6e`, the gnirehtet URL and sha `dee55499…275`, `cmake -GNinja`, `ninja -C build adb`, `patchelf --set-rpath $ORIGIN/lib` and `dist/tools/gnirehtet.apk`. It creates no out dir, and a fake `curl` on `PATH` that writes a marker is never called.
- **TBD refusal:** a pins copy with `android-tools.sha256: TBD-at-pin` → exit 2.
- **appimage dry-run:** contains `--runtime-file`, the `appimagetool-x86_64.AppImage` URL and `usr/flat3d-inject -> bin/flat3d-inject`; contains no `libwayland`, `libvulkan` or `libGL`.
- **desktop keys:** read with `configparser(interpolation=None)` and `optionxform=str`; checks the keys above, and that `Categories` ends with `;`.
- **AppRun:** is executable and contains `FLAT3D_SRC_DIR="$APPDIR/usr"`.
- **workflow toolchain:** `appimage.yml` contains `dtolnay/rust-toolchain@master` and `steps.pin.outputs.channel`, and does not contain `@stable`.
- **decky zip,** on a fake tree (`--root T --bin-dir B --tools D --skip-build`): `namelist()` equals the expected set under `flat-to-3d/`; `bin/flat3d-inject` has `external_attr>>16 & 0o111`; `--no-tools` leaves out `bin/tools/`.
- **shellcheck** of the 3 scripts plus AppRun, run only `if shutil.which("shellcheck")`.

**Inline facts:**
- `android-tools` 37.0.0p1: `https://github.com/nmeum/android-tools/releases/download/37.0.0p1/android-tools-37.0.0p1.tar.xz`, sha256 `2c6de1abe16b211c7d9f59276217e14fd9e0e18b20e331d3c84358f5dffd576e`. Skip the vendor tarball.
- `gnirehtet` 2.5.1: `…/Genymobile/gnirehtet/releases/download/v2.5.1/gnirehtet-rust-linux64-v2.5.1.zip`, sha256 `dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275`. The relay and the APK must come from the same tag (relay main.rs:548-556).
- `appimagetool` 1.9.1: sha256 `ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0`.
- `appimage-runtime` 20251108: `https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64`, sha256 `2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d`. Never `continuous`.
- `dist/` is already in .gitignore:2.
- Today the repo has no `rust-toolchain.toml`, and check.yml uses `dtolnay/rust-toolchain@stable` (.github/workflows/check.yml:24,41). WP-00 adds the toml and owns check.yml's switch.

**Depends on:** WP-00 (`rust-toolchain.toml`), WP-01, WP-02 (`packaging/pins`). **Owner device:** Q1 (`sessions/tools-build.toml`), Q16 (`sessions/appimage.toml`).

**Done-check:** `python3 -m unittest discover -s packaging/tests -p 'test_*.py' -v && bash -n packaging/*.sh packaging/AppRun`

- **Flag:** iced-packaging.md:96 puts the tools in `usr/share/flat3d/tools`. But `FLAT3D_SRC_DIR=$APPDIR/usr`, together with install_plan's `{src}/tools/…`, needs `usr/tools/` plus the binary symlinks above. That is resolved here.
- **Flag:** CI cannot build adb (nmeum 37 on Ubuntu noble is unverified), so appimage.yml ships `--no-tools`. WP-20 skips missing tools ("not bundled"). Q16 decides whether to build a tools-bundled AppImage locally from Q1's `dist/tools`.
- **Flag:** no WP's file list has an icon file, so build-appimage.sh writes an SVG from a heredoc.
- **Flag:** `runs-on: ubuntu-24.04` is GitHub-hosted, while check.yml uses `[self-hosted, linux]` (also Ubuntu 24.04). If hosted minutes matter, switch the label. `APPIMAGE_EXTRACT_AND_RUN=1` already covers the no-FUSE case.

Review note: accepted all six corrections, with three choices of route:
- WP-19 moves to wave 3 (depends on WP-17 and WP-18) instead of using WP-01 `todo!()` stubs.
- WP-21 declares the npm registry and moves the network step to WP-00, because an offline local-pnpm install can't work: the metadata cache is empty and four pinned versions are missing from the store.
- WP-21's `--options` fix goes in the TS `launchSync` helper, not in main.py, because #67's locked order test fixes main.py's argv. The CLI's `--json` stripping (cli.rs:58-62) makes the duplicate flag harmless.

### WP-23: App Games page (#117)

**Closes:** #117 (SPEC M24). The device check comes later, through #122 and Q15.

**Files (exclusive):** `crates/app/src/games_page.rs` (WP-01 leaves a stub here and adds `pub mod games_page;` to `crates/app/src/lib.rs`; WP-23 does not edit lib.rs), `crates/app/tests/games_page.rs`.

**Signatures** (from the #117 brief, checked with `gh issue view 117`; `GameRow` fields and derives from #76 brief line 10: `Debug, Clone, PartialEq, Eq, Serialize`):
```rust
//! The Games page: state, messages, update. docs/code-map.md, Parts, crates/app/src.
use flat3d_inject::rows::GameRow;
#[derive(Debug, Clone, PartialEq)]
pub struct GamesState { pub rows: Vec<GameRow>, pub filter: String, pub error: Option<String>, pub note: Option<String> }
impl Default for GamesState { fn default() -> Self { Self { rows: vec![], filter: "all".into(), error: None, note: None } } } // hand-written (brief)
#[derive(Debug, Clone, PartialEq)]
pub enum Msg { Rows(Result<Vec<GameRow>, String>), Filter(String), Switch { appid: u32, on: bool }, UndoPressed(u32), UndoDone(Result<String, String>) }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action { LoadRows { filter: String }, SetSwitch { appid: u32, on: bool }, Undo(u32) }
pub fn update(state: &mut GamesState, msg: Msg) -> Option<Action>;
```
- The test's `assert_eq!` on `Option<Action>` needs `Action: Debug + PartialEq`.
- iced needs `Msg: Clone`. That works because `GameRow` is `Clone`.
- The struct-update syntax in the tests needs every field to be `pub`.

**Behaviour** (from the brief, one-to-one):
- `Rows(Ok(r))`: replace `rows`, clear `error`, return `None`.
- `Rows(Err(e))`: set `error = Some(e)`, keep the rows, return `None`.
- `Filter(f)`: store `f`, return `Some(LoadRows{filter:f})`.
- `Switch{appid,on}`: set `switch_on = on` on the matching row only, and change nothing else on it. If no row matches, change nothing. Either way, return `Some(SetSwitch{appid,on})`.
- `UndoPressed(a)`: clear `note` and `error`, return `Some(Undo(a))`.
- `UndoDone(Ok(s))`: set `note = Some(s)`, return `Some(LoadRows{filter: state.filter.clone()})`.
- `UndoDone(Err(e))`: set `error = Some(e)`, return `None`.
- Pure: no I/O and no iced.
- WP-33 (`crates/app/src/engine.rs`) turns the actions into engine calls, not this WP: `LoadRows` → the `rows` view command (WP-28), `SetSwitch` → `switch`, `Undo` → `undo <appid>`.
- The page has no Inject button.

**Tests:**
- Locked: `crates/app/tests/games_page.rs` is the #117 "Tests (locked)" block, copied byte-for-byte. Its 9 tests:
  - `a_new_page_shows_all_games_and_nothing_else`
  - `rows_replace_the_old_ones_and_clear_the_error`
  - `a_failed_load_keeps_the_rows`
  - `the_filter_asks_for_a_reload`
  - `a_switch_shows_at_once_and_is_asked_for`
  - `a_switch_for_an_unknown_game_is_still_asked_for`
  - `undo_clears_old_messages_and_asks`
  - `undo_done_shows_the_summary_and_reloads_with_the_filter`
  - `undo_failed_shows_the_error`
- Imports: `flat3d_app::games_page::{update, Action, GamesState, Msg}`, `flat3d_inject::rows::GameRow`.
- Commit the tests first (R28), then the code.
- No new tests.

**Inline facts:**
- Fixture row: `status "works-now"`, `status_label "Works now"`, `status_colour "#2e9e5b"`.
- The import paths in the test body need no edits (reshape §1).
- Issue prose for the PM to fix later (§6): `app/...` → `crates/app/...`, and Done-when → `cargo test -p flat3d-app`.

**Depends on:** WP-01 (the frozen `rows::GameRow` and the `games_page` stub).

**Owner device:** none.

**Done-check:** `cargo fmt --all --check && CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-app --test games_page && cargo clippy --locked -p flat3d-app --all-targets -- -D warnings`

**Flag (applies to every locked issue test that lands in wave 2: WP-03/04/05/23):**
- The verbatim #117 body fails `cargo fmt --check`. I checked with `rustfmt --emit stdout`: lines 8-9 (the `GameRow{..}` literal) and 22-23 get reflowed.
- Land it in two commits:
  1. the verbatim file;
  2. `cargo fmt` of that file only, format-only.
- Owner answer 1 sanctions rustfmt on locked tests. Without commit 2, CI's `cargo fmt --all --check` goes red.

---

### WP-24: Session runner, stereo gates, device sessions

**Closes:** T9-T12. Gives every Q row (§5) its driver, including Q0, Q2 and Q12.

**Files:** `tools/session/{session.py,test_session.py}`, `tools/stereo/{fps.py,disparity.py,test_stereo.py}`, `sessions/README.md`, and the **23** `sessions/*.toml` files in the index (20 game/app sessions plus `q0`, `q2`, `q12`).

Does not touch `docs/STEP0-RUNSHEET.md` (WP-25b hand-scrubs it, WP-34 generates it).

**Rules:** Python ≥3.11, stdlib only (`tomllib argparse subprocess shlex json zlib struct glob time os pathlib`). No numpy and no Pillow (buildenv §4d: neither exists anywhere).

**Signatures:**
```
session.py check [FILES...]            # default sessions/*.toml; exit 0 ok, 1 schema errors ("file: field: msg")
session.py render FILE [--out PATH]    # markdown runsheet to stdout/PATH; deterministic
session.py run FILE [--script ANSWERS.json] [--state-dir DIR] [--now UNIX] [--dry-run]
  # exit 0 PASS, 1 FAIL, 3 INCOMPLETE (a skip), 2 usage/IO
fps.py CSV [--min 30] [--skip-rows 0] [--json]                # exit 0 pass, 1 fail, 2 bad input
disparity.py IMG [--layout sbs|tab] [--block 16] [--step 64] [--max-shift 48] [--min-texture 20] [--min 1.0] [--json]
```
Python API (the tests import these):
- `load(path)->dict`, `validate(doc, path)->list[str]`, `render(doc)->str`
- `run(doc, answers:dict|None, state_dir:Path, now:int)->dict`
- `to_markdown(result)->str`
- `fps.summarize(rows)->dict(n,p50,p1)`, `fps.read_mangohud(path)->list[tuple[float,float|None]]`
- `disparity.decode(path)->(w,h,luma:bytearray)`, `disparity.measure(w,h,luma,layout,...)->dict(n,mean,p90)`

**TOML schema (`schema = 1`):**
- Top level:
  - `name` (must equal the file stem), `title`, `issues=[int]` (≥1; the issues the result .md is pasted into), `spec=["M1"]`.
  - `evidence_only=[int]` (optional, must be a subset of `issues`): issues that a PASS only gathers evidence for. These close only with the later works-now recipe PR, never by the session.
  - `device` ⊆ {z13, quest2, craft-build, ci}.
  - `done_when` (copied verbatim from the issue).
  - `after=[session names]` (each must exist), `needs=[str]` (prerequisites).
  - Optional `[vars]`: string constants.
  - `[[trouble]] symptom/cause`, rendered as the "If it fails" table.
- `[[step]]` fields:
  - `id` (unique, `[a-z0-9-]+`), `kind` = cmd | ask | check | measure, `who` = you | script, `text`, `section` = setup | each-run | check (default each-run).
  - cmd: `run=[argv]` (no shell), `ok_rc=[0]`, `timeout=120`, `expect` (stdout regex), `background=false` (Popen, killed at session end), `optional=false`.
  - check: `gate=true`, answered y/n/s.
  - measure: `tool` = fps | disparity, `args=[...]`, `input` (glob; the newest match is offered as the default answer), `optional=false`.
- Placeholders:
  - `{repo}`, `{home}`, `{state}` = `${XDG_STATE_HOME:-~/.local/state}/flat3d`.
  - `{re_home}` = `$FLAT3D_RE_HOME` (default `~/.local/share/flat-to-3d`).
  - `{build_box}` = `os.environ.get("FLAT3D_BUILD_BOX", "craft-build")`. Keep this on one line, because the docs check allow-lists lines containing FLAT3D_BUILD_BOX.
  - `{<var>}`, and `{ans:<id>}` (an earlier answer).
  - An unknown placeholder is a `check` error.
- `check` also rejects a literal `/home/<user>` or `sudo` in `run`.

**Run behaviour:**
- Walks the steps in order and prints each `text` with its placeholders filled.
  - A cmd records rc and the last 20 lines of stdout and stderr.
  - A measure runs `python3 {repo}/tools/stereo/<tool>.py <input> --json <args>` and records its JSON.
  - `--script` answers by step id. A missing answer is exit 2. cmds still run.
- Verdict:
  - PASS: every non-optional cmd ok, every gate check `y`, every non-optional measure pass.
  - INCOMPLETE: any `s`.
  - FAIL: anything else.
  - An optional measure that fails, or has no input, is recorded but does not change the verdict.
- Writes `{state-dir}/<name>-<now>.json`. The default state-dir is `{state}/sessions` = `~/.local/state/flat3d/sessions/`.
  - Contents: `{schema, session, issues, evidence_only, started, ended, os: PRETTY_NAME from /etc/os-release, verdict, steps:[{id,kind,status,answer,rc,tail,metrics}]}`.
  - No hostname, because machine names are private.
- Also writes `<name>-<now>.md`, a paste-ready issue comment (header, `done_when`, step table), and prints it.
  - For each `evidence_only` issue it adds the line: "Evidence for #N. This does not close it: #N closes with the works-now recipe PR (`recipes/<appid>-<slug>.recipe`)."
  - The tool makes no GitHub writes.
- render:
  - Header `<!-- generated by tools/session/session.py render sessions/<name>.toml; edit the TOML -->`.
  - Sections: "One-time setup" / "Each run, in this order" / "What to check (pass = done)" (`- [ ]`) / "If it fails".
  - Roles render as **You:**/**Script:**. cmds render as fenced `shlex.join`.

**fps.py:**
- MangoHud CSV layout: a preamble `os,cpu,gpu,...`, then a values line, then the column header whose first cell is `fps` (`frametime` is in ms).
- p50 = median fps. p1 = `1000/pct(frametime,99)` when `frametime` exists, else `pct(fps,1)` (nearest rank).
- pass = `p50 >= --min`.
- No header or no rows → exit 2.

**disparity.py:**
- Decoders:
  - PNG: signature, IHDR/PLTE/IDAT/IEND, `zlib.decompress`, unfilter types 0-4. Colour types 0/2/3/4/6 at depth 8; depth 16 keeps the high byte. Interlace 1 → exit 2.
  - PPM/PGM: P5/P6, maxval ≤255, `#` comments.
- Luma = `(299R+587G+114B)//1000`. Split by layout: sbs = left/right halves, tab = top/bottom halves. Disparity is horizontal in both.
- Blocks are `block`² on a `step` grid, skipping a `max_shift` border. A block counts when its std-dev ≥ `min-texture`.
- For each block, `d` = the SAD argmin over ±`max_shift`; keep `|d|`.
- Reports `n`, `mean`, `p90`. pass = `n>0 and mean > --min` (">1 px", T12).

**Tests:**
- `tools/session/test_session.py`:
  - `check` passes on every repo `sessions/*.toml`. Repo root is `Path(__file__).parents[2]`.
  - `check` rejects:
    - a duplicate id
    - an unknown kind
    - an unknown placeholder
    - `name`≠stem
    - a cmd without `run`
    - a literal `/home/x`
    - an `evidence_only` issue that is not in `issues`
  - render is deterministic and contains every step text plus the generated header.
  - `run --script` on a temp TOML:
    - cmds `python3 -c "print('ok')"` (expect `ok`) and `false` (optional) → PASS;
    - check `n` → FAIL; `s` → INCOMPLETE;
    - the JSON and .md land in a temp `--state-dir`, named `<name>-<now>.json`;
    - a measure with a synthetic CSV records p50;
    - an optional measure with no input file → still PASS;
    - an `evidence_only=[36]` session's .md contains "Evidence for #36" and no "closes".
  - A missing answer → exit 2.
- `tools/stereo/test_stereo.py`:
  - CSV with median 60 → pass; median 25 → fail (exit 1); no `fps` header → exit 2; p1 computed from frametime.
  - A PNG built in the test: chunk CRC via `zlib.crc32`, rows cycling filters 0-4, RGB and gray.
    - Random texture (`random.Random(1)`); right eye = left shifted 4 px → mean ≈4, pass.
    - Identical halves → mean 0, exit 1.
  - A PPM round-trip gives the same luma as the PNG.
  - An interlaced or truncated PNG → exit 2.

**Session index** (23 files. Each TOML carries its issue's Done-when verbatim. `needs` lists WPs and sessions. "Evidence" marks `evidence_only` issues):

| file | issues | key facts / gates |
|---|---|---|
| q0 | Q0 (pins) | **Owner**, one-time, on a networked machine. `who=you` cmds: `bash {repo}/packaging/pin.sh <key>` for `geo11`, `lana-fix`, `control-fix`, `ac7-profile`. The owner runs the downloads; the script records url/size/sha256 in `packaging/pins`. <br>• Gate cmd: `grep -E '^(geo11\|lana-fix\|control-fix\|ac7-profile)\.sha256: TBD-at-pin' {repo}/packaging/pins` with `ok_rc=[1]` (no TBD left). <br>• needs: WP-02. Must come before a2 and g1 (`after = ["q0"]` in a2-uevr, g1-geo11, g4-control). |
| q2 | Q2 (health check, v1 roadmap.md:404) | **Owner**, Z13 with Quest 2 on USB and WiVRn running. `flat3d-inject doctor --json`, expect `"ok":\s*true`. Check gate: "every doctor line is ok or explained". needs: the doctor WP. |
| step0 | #5 (M1) | Spider-Man **Remastered appid 1817070**, Spidy v0.3.8 (submodule `aed5c0c`). Steps from STEP0-RUNSHEET:18-62, with roles You/Script. <br>• Launch option, written out in full with `{home}` filled in at run time (spidy-linux.sh:4-6 says the path must be absolute): `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 PRESSURE_VESSEL_FILESYSTEMS_RW={home}/.local/share/flatpak/app/io.github.wivrn.wivrn STEAM_COMPAT_LAUNCHER_SERVICE=proton MANGOHUD=1 MANGOHUD_CONFIG=output_folder={state}/mangohud,autostart_log=30,log_duration=120 %command%`. A system Flatpak uses `/var/lib/flatpak/app/io.github.wivrn.wivrn`. Optional cmd `flat3d-inject setup launch-env --json` (WP-29). <br>• cmds: `mkdir -p {state}/mangohud`; background `flatpak run io.github.wivrn.wivrn --early-active-runtime --no-publish-service`; `adb reverse tcp:9757 tcp:9757`; `adb shell am start -a android.intent.action.VIEW -d wivrn+tcp://localhost org.meumeu.wivrn.github`; `env SPIDY_DIR={re_home}/spidy-0.3.8/Spidy-0.3.8 bash {repo}/third_party/SpidyVR/tools/spidy-wivrn.sh`. <br>• Game settings: frame generation off, vsync off, RT reflections off, aspect Auto. VR VIEW: GAME SCREEN, GAME SCREEN 3D at max. <br>• 5 gate checks. fps measure, input `{state}/mangohud/*.csv`; Shift_L+F2 toggles logging. <br>• Trouble rows: `-51`, "Proton session was not found", "version is not supported" (needs 4.630.0.0), menus ignore controllers, eye drops. |
| sm2-offline | #7 | **SM2 appid 2651280**, exe pin `sm2-2.810.0.0`. cmd `sha256sum {ans:exe}` with expect `c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9` (194262992 B). You turn the network off. Gate: gameplay reached with no PSN sign-in block. |
| link-smoke | #26, #39, #125 soak, #128 (Q3/Q4) | `flat3d-inject install --json`; `systemctl --user is-active flat3d-link` (expect `active`); `status --json` expect `"ready"`; `start-3d`/`stop-3d`/`start-3d --json` all ok (#39); `doctor --json`; 30-minute soak, then `ls {state}/soak-*.txt`; `uninstall --json`, then check nothing is left. |
| tools-build | #127 | `distrobox enter {build_box} -- {repo}/packaging/build-tools.sh`; `dist/tools/adb version` expect `37.0.0`; `dist/tools/gnirehtet` prints usage; `sha256sum` of the zip = `dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275`. |
| z1-offline | #31 | 2075800. One-time EA link, then a mission with the network off. |
| z2-uevr | #32, #51 | after z1. Pin `uevr` nightly-01143, sha `4cd6932d…dc6d8`. <br>• Steps: `flat3d-inject inject 2075800 --json`; launch through Steam; then `flat3d-inject undo 2075800 --json`, clean. <br>• **Depth gate is owner-judged:** check `gate=true` "in the headset the game screen shows depth (near objects stand out), and the HUD sits flat". `ask` step `capture-method` records how the eyes were captured, if at all. <br>• Disparity measure is **`optional=true`**, with `input` `{state}/captures/2075800-*.png` (an SBS image). It runs only when a two-eye capture exists. No known method dumps UEVR's two eyes: UEVR submits straight to the OpenXR runtime, so the SBS viewer's capture can't see it, and a WiVRn per-eye dump is unconfirmed. A missing capture never blocks PASS. |
| z3-screen / z4-mission / z5-recipe | #33 / #35 / #36 (evidence) | z3: head turn does not move the camera, and depth is visible. z4: fps ≥30, no dropped eyes or crash. z5 **gathers evidence**: the fps/depth results plus the proposed recipe text. `evidence_only=[36]`. #36 closes only with the works-now `recipes/2075800-*.recipe` PR (WP-26's `works_now_recipes_are_injectable` then guards it). |
| a1-install / a2-uevr / a3-sortie | #40 / #41 / #42, #47 (evidence) | 502500. <br>• a2 `after=["q0","a1-install"]`. It needs (OD8, manual): Nexus mods 2179 and 2274, Konan's VR Planes + `AC7_VR_Patch_P.pak` (Flat2VR Discord), the campaign beaten once, and **not** kosnag's `~~~UEVR_Compatibility_Mod_P.pak`. Disable decoupled pitch. <br>• a3: fps ≥30. `evidence_only=[47]`: #47 closes with the works-now recipe PR. |
| g1-geo11 | #43, #52 | 1608230. `after=["q0"]`. Pins `geo11` 0.7.12 and `lana-fix`. `d3dxdm.ini` `direct_mode=sbs`. <br>• Disparity measure on an SBS **desktop screenshot**. Geo-11 direct mode draws both eyes into one flat frame, so an ordinary screenshot holds both eyes. Gate >1 px. <br>• fps measure. Confirms `geo11-exe: Planet of Lana.exe` (WP-26). |
| g2-viewer | #44 | after g1. `distrobox enter {build_box} -- cargo build --locked --release -p flat3d-viewer --features xr,capture`. <br>• Gates: a still screen with depth, and head movement does not steer it. <br>• Optional disparity measure on the viewer's `--dump-eyes` output, if the viewer WP ships that flag. |
| g3-lana / g4-control | #45, #48 (evidence) / #46, #49 (evidence) | fps ≥30 with depth. `evidence_only=[48]` / `[49]`. Each closes with its works-now recipe PR. <br>• Control 870780, `Control_DX11.exe`. g4 `after=["q0"]`. Remove HelixSR's `dxgi.dll` first. <br>• Trouble row: Geo-11 SBS crash after a game update (2025-03-13 helix comment). |
| app-window | #78, #122, #123 | `cargo run --locked -p flat3d-app`: the window opens; 4 pages render; Start/Stop works; the Headset page follows within 2 s. |
| appimage | #129 | `chmod +x` and run `Flat_to_3D-x86_64.AppImage` (CI artifact); the window starts. |
| decky | #130-#134 | Script part: `packaging/build-decky.sh`; `unzip -l dist/flat-to-3d.zip`; install it. |
| q12 | Q12 (Decky in Game Mode, v1 roadmap.md:414), #130-#133 | **Owner**, Z13 in Game Mode. `after=["decky"]`. Check gates: the panel renders in the QAM; launch options change on a real library entry; the collection and the toast show; the plugin survives a Game Mode restart. |
| spidy-070 | #155 | after step0 passes. Remastered still in 3D at 30+ fps on the 0.7.0 pin (fps measure). README/SPEC mention the pin. |

#29 (SM2 recipe at works-now) has no session in this pass. Like #36/#47-#49, it closes only with its works-now recipe PR, after the port track.

`sessions/README.md` covers:
- the schema, including `evidence_only`;
- the three commands and where results land;
- "paste the .md into the issue; an evidence-only issue stays open until its recipe PR".

**Depends on:** WP-01 and WP-25a (`tools/docs/check.py`, used in this WP's done-check).

**Owner device:** none for the code. Every Q row, Q0/Q2/Q12 included, runs these sessions.

**Done-check:** `for d in tools/session tools/stereo; do python3 -m unittest discover -s $d -p 'test_*.py' || exit 1; done && python3 tools/session/session.py check sessions/*.toml && test $(ls sessions/*.toml | wc -l) -eq 23 && python3 -m py_compile tools/session/*.py tools/stereo/*.py && python3 tools/docs/check.py sessions/README.md`

**Flags:**
- The scope's "step0 has SM2 appid 2651280 and pin sm2-2.810.0.0" is wrong:
  - Step 0 is Spider-Man **Remastered 1817070** (STEP0-RUNSHEET.md:14; spidy-linux.sh:12 `BUS=com.steampowered.App1817070`).
  - 2651280 and `sm2-2.810.0.0` go in `sm2-offline.toml`.
- The text of the Q0/Q2/Q12 rows comes from v1 roadmap.md:404/414 and the a2 row's "pin.sh ac7-profile (Q0)". If §5 words them differently, the TOMLs follow §5. The file names stay the same.

---

### WP-25a: Docs check (wave 1)

**Closes:** T2 (the checker). Runs in **wave 1**, in parallel with WP-01/WP-02, so that every docs-writing WP in wave 2 can use it as a done-check.

**Files (exclusive):** `tools/docs/check.py`, `tools/docs/test_check.py`, `tools/docs/private-allow.txt`.

**check.py** (`python3 tools/docs/check.py [--root DIR] [--only status,links,mermaid,usage,private] [PATHS...]`):
- Output: `path:line: rule: msg`. Exit 0 clean, 1 findings, 2 usage/IO error.
- Files:
  - `PATHS` (repo-relative) limits the check to those files. Docs-writing WPs check only their own files, before WP-25b has scrubbed the rest.
  - With no `PATHS`, it checks `git -C root ls-files`, or walks the directory when there is no `.git`.
- Always skipped: `third_party/`, `**/tests/**`, `Cargo.lock`, `pnpm-lock.yaml`, anything not UTF-8.

Rules:
- **status:** in code-map's Parts table, when the status cell starts with `present`, `partly present` or `stub`, every backticked path in cell 1 must exist.
  - A bare name resolves against the previous path's directory.
  - `*` globs must match something. A trailing `/` means a directory.
  - `planned` rows are not checked.
- **links:** for each `[..](t)` outside code fences:
  - skip `http(s):`, `mailto:` and `#…`;
  - strip `#`/`?`;
  - resolve against the file's directory (a leading `/` means the repo root);
  - the target must exist.
- **mermaid:** inside ```` ```mermaid ````:
  - remove `"…"`, `|…|`, the bodies of `[..]`/`(..)`/`{..}`, and ` & `;
  - split on `-->|==>|-.->|---`;
  - a whitespace-separated token inside an inner segment is a finding ("2 statements on one line").
- **usage:** every literal `usage: flat3d-inject …` in `crates/engine/src/cli.rs` and `cli/*.rs` is split on `|`.
  - Each alternative's first word must appear in column 1 of code-map's `## Commands`.
  - One-way only: K11, and the locked USAGE at cli.rs:50-51 omits `undo`.
- **private:** `\bI[a]n\b`, `(?i)hyperion`, `B:/`, `/home/[a-z_][a-z0-9_-]*`, `(?i)qwen`, `FF8S5`, `(?i)sandisk`, `\bMDT\b`, `(?i)craft-build`, `(?<![\w/.])iannitram(?!/)`, and e-mail `[\w.+-]+@[\w-]+\.[a-z]{2,}`. The allow-list can lift any of these.

`private-allow.txt` lines are `<glob>` (whole file) or `<glob>\t<substring>` (only the lines containing it). Initial contents:
```
docs/notes/DECISIONS.md
docs/notes/**
tools/docs/check.py
tools/docs/test_check.py
tools/docs/private-allow.txt
decky/plugin.json	"author"
.github/ISSUE_TEMPLATE/issue.md	Qwen brief
AGENTS.md	Qwen brief
docs/workbench.md	craft-build
*	FLAT3D_BUILD_BOX
```
- `docs/notes/**`: the research notes carry "sandisk" drive names (research-report.md:73,84,129; games_group_a.md:40). Their archive move is a follow-up.
- `decky/plugin.json` `"author"`: the O-9 default keeps the publishing handle.

**Tests** (`tools/docs/test_check.py`): each test builds a temp tree with no `.git`. Forbidden words are assembled at runtime (`"Hyp"+"erion"`), so the file itself stays clean.
- status: a `present` row with a missing file → finding; `planned` → none; a bare-name cell resolves.
- links: a dead link → finding; an anchor-only link and an http link → none; a link inside a code fence is ignored.
- mermaid: the code-map:21 shape → finding; a chain `A --> B --> C` → none; a quoted label holding `-->` → none.
- usage: USAGE `list | inject <appid>` with only `list` in the table → finding; an extra table row → none.
- private: a hit → finding; a whole-file glob or substring allow → none; a `tests/` path is exempt; `iannitram/flat-to-3d` passes.
- `PATHS`: a finding in an unlisted file is not reported.
- Exit codes are 0/1/2.

**Done-check for wave-2 docs writers** (WP-24, WP-25b, and any other WP that writes `.md`): `python3 tools/docs/check.py <the .md files it wrote or changed>`. A full-repo `check.py` with no paths is expected to fail until WP-25b lands.

**Depends on:** none. Its tests use temp trees, so it does not wait for WP-01's `crates/` paths.

**Owner device:** none.

**Done-check:** `python3 -m unittest discover -s tools/docs -p 'test_*.py' && python3 -m py_compile tools/docs/check.py && python3 tools/docs/check.py --only private tools/docs/check.py tools/docs/test_check.py tools/docs/private-allow.txt`

---

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

### WP-26: Recipe seeds + repo recipe lint

**Closes:** T21, K25.

**Files:** the 5 seeds `crates/engine/tests/fixtures/recipe-seeds/{2075800-star-wars-zero-company,502500-ace-combat-7,1608230-planet-of-lana,870780-control,2651280-spider-man-2}.recipe`, plus `crates/engine/tests/recipes_repo.rs` and `crates/engine/tests/recipe_seeds.rs`.

**Signatures used** (checked against inject/src):
- `recipes::load_recipes(&Path)->Result<BTreeMap<u32,Recipe>,RecipeError>` (recipes.rs:199). Only `<appid>-*.recipe` files load; they are parsed and checked.
- `recipes::parse_recipe(appid,&str)` (:113):
  - ignores keys outside `KEYS` (:127-129), so route keys can sit in the same file;
  - strips one pair of surrounding double quotes from each value (:137-140).
- `recipes::STATUSES` (:9): `works-now likely build not-safe wont-work untested`.
- `injectors::route_of(&str)->Route{Uevr,Geo11,Port,NoRoute}` (injectors.rs:20). "build…" → Port, "uevr" → Uevr, "geo-11" → Geo11.
- `injectors::check_injectable(&Recipe)->Result<(),String>` (:35): checks safe, then works-now ("is not works-now"), then route.
- Frozen by WP-01: `recipes::route_settings(text:&str)->BTreeMap<String,String>` and `recipes::check_route_settings(route:&str, m:&BTreeMap<String,String>)->Result<(),String>`.

**Seeds:** `key: value` lines. The first line is the comment `# Seed (OD6): untested until sessions/<x>.toml passes; real recipe lands in recipes/ at works-now.`. All five share:
- `stereo-status: untested`
- `stereo-free: yes`
- `stereo-safe: yes` (SM2's carries `# pending #7` on the line above, because the parser does not strip trailing comments)
- `stereo-checked: 2026-11-08`

The values are unquoted. This matches the fixtures in inject/tests/wrapper_run.rs:9. The quoted form in recipes/README.md:23 also parses.

| file | name / route / engine / api / mod | route keys |
|---|---|---|
| 2075800 | STAR WARS Zero Company / `uevr 2d screen mode` / `unreal 5.6` / dx12 / `https://github.com/praydog/UEVR-nightly` | `uevr-exe: SWZeroCompany`, `uevr-rendering: native`, `uevr-2d-screen: yes`, `uevr-delay: 15` |
| 502500 | ACE COMBAT 7: SKIES UNKNOWN / `uevr 2d screen mode + keton profile` / `unreal 4` / dx11 / `https://github.com/keton/ace-combat-uevr` | as above with `uevr-exe: Ace7Game`, plus `uevr-profile: ac7-profile` |
| 1608230 | Planet of Lana / `geo-11 + sbs viewer` / `unity` / dx11 / `https://helixmod.blogspot.com/2025/12/planet-of-lana-geo-11-fix.html` | `geo11-fix: lana-fix`, `geo11-exe: Planet of Lana.exe`, `geo11-layout: sbs` |
| 870780 | Control / `geo-11 + sbs viewer` / `northlight` / dx11 / `https://helixmod.blogspot.com/2019/10/control-3d-vision-ready-fix-losti-v100.html` | `geo11-fix: control-fix`, `geo11-exe: Control_DX11.exe`, `geo11-layout: sbs`, `geo11-remove-first: dxgi.dll` |
| 2651280 | Marvel's Spider-Man 2 / `build: port spidy stereo screen` / `custom` / dx12 / `https://github.com/IlyaMez/SpidyVR` | none (the port route is #27b, not in this pass) |

**Shared pin parsing (test-local, duplicated in both test files, ~15 lines):**
- `fn pins(root:&Path)->BTreeMap<String,BTreeMap<String,String>>` reads `root/packaging/pins`:
  - skip `#` lines and blanks;
  - strip a trailing ` # …`;
  - split on `": "`, then split the left side on the first `.`.
- A pin key is one with `url`, `size` and `sha256` (64 hex or `TBD-at-pin`).
- `fn pin_refs(route:Route, m:&BTreeMap<String,String>)->Vec<String>`:
  - Uevr ⇒ `uevr` plus `m["uevr-profile"]`;
  - Geo11 ⇒ `geo11` plus `m["geo11-fix"]`;
  - Port ⇒ none.
- Root is `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")`.

**Tests:**
- `recipe_seeds.rs`:
  1. `the_seeds_are_the_five_games`: the key set is exactly {502500, 870780, 1608230, 2075800, 2651280}.
  2. `no_seed_is_works_now`: status is `untested`, and `check_injectable` gives an Err containing "is not works-now".
  3. `each_seed_names_its_route`: 2075800/502500 → Uevr, 1608230/870780 → Geo11, 2651280 → Port.
  4. `route_keys_pass_the_frozen_check`: `check_route_settings(&r.route, &route_settings(&text))` is Ok for each file.
  5. `pin_refs_name_real_pins`: the union of refs is exactly {uevr, ac7-profile, geo11, lana-fix, control-fix}, and each one is a pin key in `packaging/pins`.
  6. `an_unknown_pin_is_refused`: text with `uevr-profile: no-such-pin` → the cross-check names `no-such-pin`.
  7. `seeds_stay_out_of_recipes_until_works_now`: any `recipes/<seed appid>-*.recipe` must be works-now.
- `recipes_repo.rs`:
  1. `every_repo_recipe_loads`: `load_recipes(root/recipes)` is Ok. Zero entries is fine today.
  2. `file_names_are_appid_slug`: each `*.recipe` matches `^[0-9]+-[a-z0-9-]+\.recipe$`.
  3. `works_now_recipes_are_injectable`: `check_injectable` Ok, `check_route_settings` Ok, and the pin refs exist. This is the gate on the later works-now recipe PRs that close #29/#36/#47-#49 (WP-24 sessions only gather evidence for them).
- Locked tests stay untouched: `recipe_parse.rs`, `recipe_check.rs`, `recipe_load.rs` (`the_repo_recipes_folder_loads` passes on the empty folder).

**Inline facts:**
- Pin keys in `packaging/pins` (WP-02): `uevr` (sha `4cd6932d1d01168178aef70499d40b101ea6f19868cf76099d8e0087ef8dc6d8`), `geo11`, `lana-fix`, `control-fix`, `ac7-profile`. The last four are TBD-at-pin until session `q0` passes.
- `ac7-prereqs` has no url, so it is not a pin key for this check.

**Depends on:** WP-01, WP-02 (`packaging/pins`, read at test time). The WP index row must list both.

**Owner device:** none.

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test recipe_seeds --test recipes_repo --test recipe_load && cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings`

**Flags:**
- The scope says status `testing`, but `STATUSES` (recipes.rs:9-16) has no `testing`: `check_recipe` would reject it and `load_recipes` would fail. Use `untested`, which DECISIONS "Recipe keys" (:60) allows.
- WP-01 must freeze these points, or this WP's test 4 picks the wrong form:
  - `check_route_settings`' `route` argument is the raw `stereo-route` text (the test passes `&recipe.route`);
  - `uevr-profile` and `geo11-fix` take a **pin key**. v1's facts (roadmap.md:306) had `uevr-profile: <url> <sha256>`, which conflicts.
- `geo11-exe: Planet of Lana.exe` is not confirmed. Session `g1-geo11` checks it.

Review note: rejected "WP-26 `stereo-free: \"yes\"` parses as the literal `\"yes\"` because the parser has no quote trimming", because `parse_recipe` strips one pair of surrounding double quotes (inject/src/recipes.rs:137-140), and recipes/README.md:23 documents the quoted form. The seeds use the unquoted form anyway, for consistency with inject/tests/wrapper_run.rs:9. The WP-02 dependency was already in WP-26's own Depends line; it is now stated as required in the index too.

Review note: WP-24's z2 capture fix does not use the viewer's `--dump-eyes` for UEVR. UEVR submits straight to the OpenXR runtime, so the SBS viewer (geo11-viewer.md:3,65) never sees its frames, and no WiVRn per-eye dump is confirmed in the reports. z2's depth gate is owner-judged and its disparity measure is optional. `--dump-eyes` is used only, optionally, in g2-viewer.

Review note: Q0/Q2/Q12 have no definition in the available reports beyond v1 roadmap.md:404 (Q2 = health check, doctor), :414 (Q12 = Decky in Game Mode) and the a2 row's "pin.sh ac7-profile (Q0)". q0/q2/q12.toml are written to those meanings.

### WP-27: inject, undo and setup fetch commands + ensure_applied (#27a)

**Closes:** #27 (route dispatch, the `inject <appid>` dev hook, `undo`) and `setup fetch <key>`. The Port route stays `not-built` until #27b.

**Files (exclusive; the index must list exactly these):**
- `crates/engine/src/injectors.rs`
- `crates/engine/src/undo.rs`
- `crates/engine/src/cli/inject.rs`
- `crates/engine/src/cli/undo.rs`: WP-01 stubs this file.
- `crates/engine/src/cli/setup_fetch.rs`: WP-01 stubs this file. WP-29's `cli/setup.rs` sends `setup fetch` here.
- `crates/engine/tests/inject_fake_prefix.rs`
- `crates/engine/tests/ensure_applied.rs`
- `crates/engine/tests/cli_undo.rs`
- `crates/engine/tests/cli_setup_fetch.rs`

**Signatures:**
```rust
// injectors.rs (replaces the stub `inject(game,recipe)->Result<String,NotBuilt>` at inject/src/injectors.rs:59)
pub fn inject(game:&steam::Game, recipe:&Recipe, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::InjectResult, api::ApiError>;
// WP-01's signature with `game` in place of appid. The error stays String. WP-27 and WP-31 both call exactly this.
pub fn ensure_applied(game:&steam::Game, recipe:&Recipe, settings:&BTreeMap<String,String>, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<RouteEnv, String>;
pub fn env_file_text(route:Route, settings:&BTreeMap<String,String>, env:&RouteEnv)->String;
pub fn parse_env_file(text:&str)->Result<(String /*route*/, BTreeMap<String,String>, RouteEnv), String>;
// undo.rs
pub fn save_record(data_dir:&Path, rec:&UndoRecord)->Result<PathBuf, ApiError>; // atomic; refuses if a record exists
pub fn put_back(rec:&UndoRecord)->Vec<String>; // split out of undo_game's loop (inject/src/undo.rs:201)
// cli/setup_fetch.rs
pub fn setup_fetch(key:&str, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::SetupReport, api::ApiError>;
```
Keep these unchanged: `Route`, `route_of`, `check_injectable` (injectors.rs; used by wrapper.rs:4 and the locked tests).

**Behaviour:**
- **inject:**
  1. `check_injectable`. Err → `refused`.
  2. Get the settings with `catalog::find_recipe(&p.recipe_dirs, appid)`, then `recipes::route_settings`, then `check_route_settings`. `None` → `not-found`. Err → `bad-recipe`.
  3. Call `ensure_applied(game, recipe, &settings, p, r)` and map its String error (see "Error mapping"). Report "already set up" when the record was already there and nothing was written.
  4. Return `InjectResult{appid, route, launched:false, summary}`.
  - Routes never write Steam launch options and never start the game (#27).
- **ensure_applied:**
  1. Record (`undo::record_path`), marker `.flat3d-applied` and env file `p.env_file(appid)` all present, with route and settings equal → `parse_env_file`. No fetch, no writes, no runner calls.
  2. Present but mismatched → `undo_game(p.data, appid)`, then go on to step 3.
  3. Choose pins:
     - Uevr: `uevr`, plus `uevr-profile` when the recipe names one.
     - Geo11: `geo11` and `geo11-fix`.
     - Port: Err `"not-built: port route: #27b"`.
  4. For each pin, skip the fetch when the cache sentinel under `p.cache().join(key).join(ver)` is present. Otherwise run WP-13 `fetch::fetch`, then `unpack`.
  5. Apply the route:
     - Uevr: `let (applied, env) = uevr::apply_with(game, &route_settings, p)?` (WP-14, 3 args, `Result<(Applied,RouteEnv),String>`). The DLL dir is `p.cache().join("uevr").join(ver)`.
     - Geo11: `geo11::apply(..)` (WP-16). Its `RouteEnv.sidecar` comes from WP-16's `spec.to_args()` and is not built here. WP-31 appends `--until-pid`.
  6. `save_record` with `launch_options_before: ""`.
  7. `fsx::write_atomic(p.env_file(appid), env_file_text(..))`.
  - If step 6 or 7 fails, run `put_back(&applied.record)` and return Err `"io: …"`.
  - The wrapper (WP-31) plays flat on Err.
- **Env file format:**
  - First line `flat3d-env 1`, then `route\t<name>`, `setting\tK\tV`, `set\tK\tV`, `unset\tK`, `sidecar\targ…`.
  - Use undo's escaping for tab, newline and backslash.
- **cli `undo <appid>`** (`cli/undo.rs`):
  - Calls `undo_game(p.data, appid)`.
  - Usage errors exit 2 with `UNDO_USAGE` (cli.rs:53). Other errors exit 1 (`exit_code`, cli.rs:93).
- **cli `setup fetch <key> [--json]`** (`cli/setup_fetch.rs`):
  - Looks up the pin by key in `p.pins()`, then fetches and unpacks into `p.cache()/<key>/<ver>`. Skips the fetch if the sentinel is present.
  - Returns `SetupReport{steps,summary}`. Unknown key → `not-found`. Bad argv → usage, exit 2.
- **Error mapping:**
  - WP-13 `FetchError::Unpinned` → `refused` "run packaging/pin.sh <key>";
  - `FetchError::Failed` (curl or sha256) → `command-failed`;
  - `FetchError::Io` and fs failures → `io`;
  - missing game or recipe → `not-found`.
  - `ensure_applied` Strings carry a `<code>: ` prefix from `ERROR_CODES` (api.rs:77), and `inject` splits on it.
- **Cross-WP edit:** the missing-pin hints in WP-14 and WP-16 both read "run `flat3d-inject setup fetch <key>`". `<appid>` is wrong.

**Tests:**
- **Locked, honour verbatim:**
  - `tests/cli_json.rs`: `inject 5 --json` must print one object with `ok==(code==0)` and a code in `ERROR_CODES`. Bad lines: "inject", "inject abc", "inject -1 --json", "inject 1 2".
  - `tests/injector_route.rs`, `wrapper_exec.rs`, `wrapper_plan.rs`.
- **New `inject_fake_prefix.rs`:**
  - temp HOME, prefix and cache;
  - seeds `p.cache()/uevr/<ver>/UEVRBackend.dll` and `openxr_loader.dll`;
  - FakeRunner asserts no `curl`;
  - Ok, with the record's `launch_options_before==""` and the env file written;
  - a second call returns "already set up";
  - a `TBD-at-pin` pin → `refused` with the prefix byte-identical;
  - curl exit 22 → `command-failed` with the prefix untouched;
  - an unwritable env dir → rolled back with no marker left.
- **New `ensure_applied.rs`:**
  - a second call returns an identical RouteEnv and makes no runner calls;
  - after a settings change, the record is replaced and the env reflects the new setting;
  - Port → Err starting `not-built`.
- **New `cli_undo.rs`:** uses `env!("CARGO_BIN_EXE_flat3d-inject")` with `.env("XDG_DATA_HOME",tmp)`.
  - no record → exit 1, `not-found`;
  - a record with a created file → file removed, `ok`, record deleted;
  - a repeat → `not-found`;
  - `undo abc` → exit 2 and stderr contains `UNDO_USAGE`;
  - a corrupt record → `command-failed`.
- **New `cli_setup_fetch.rs`:**
  - sentinel present → no `curl`, `ok`;
  - TBD pin → `refused` naming the key;
  - unknown key → `not-found`;
  - `setup fetch` with no key → exit 2.

**Inline facts:**
- Undo record path is `<data>/undo/<appid>.txt` (undo.rs:176-178).
- `data_dir` is `$XDG_DATA_HOME/flat3d`, falling back to `~/.local/share/flat3d`.
- Undo summary: "{n} change(s) put back for {appid}; the launch option is left to the Decky plugin".
- UEVR refuse text is "the UEVR profile for {} is already set up; undo it first" (injectors/uevr.rs:87).
- UEVR env (WP-14 builds it) is `PROTON_REMOTE_DEBUG_CMD` = `"{inj} --dll {win(cache)} --exe {stem} --delay {d}"`, with `unset=["XR_RUNTIME_JSON"]`.
- Geo11 env is `WINEDLLOVERRIDES="d3d11,nvapi64=n,b"`.
- compatdata is `<library>/steamapps/compatdata/<appid>`.

**Flags:**
- `Flag:` `EnginePaths` has no pins dir. Add `pins()` = `$FLAT3D_SRC_DIR/packaging/pins`, falling back to `data/pins` (WP-01 owner). `packaging/` does not exist in the repo yet.
- `Flag:` WP-01 must stub both `cli/undo.rs` and `cli/setup_fetch.rs`, because only WP-01 may add mods.

**Depends on:** WP-01, WP-07, WP-13, WP-14, WP-16, WP-20. The index must add WP-20.

**Owner device:** any Linux box (fakes only).

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test inject_fake_prefix --test ensure_applied --test cli_undo --test cli_setup_fetch --test cli_json && tools/check.sh`

---

### WP-28: View commands (#121)

**Closes:** #121.

**Files (exclusive; the index must list exactly these):**
- `crates/engine/src/cli/view.rs`: argv parsing and IO only.
- `crates/engine/src/face/link_model.rs`: the pure `link_view(snap, now)` model, moved out of `cli/view.rs` so it can be tested without IO.
- `crates/engine/tests/cli_view.rs`

**Signatures:**
```rust
// face/link_model.rs
pub fn link_view(snap:Option<LinkSnapshot>, now:u64)->api::LinkView;
// cli/view.rs
pub fn rows_cmd(p:&EnginePaths, filter:&str, now:u64)->Result<Vec<GameRow>,ApiError>;
pub fn launch_cmd(op:LaunchOp, options:&str, p:&EnginePaths)->api::LaunchText; // LaunchOp{Add,Remove}
pub fn collection_cmd(games:&[api::Game], current:Option<&[u32]>)->api::CollectionPlan;
pub fn toasts_cmd(snap:Option<LinkSnapshot>, last_seq:u64, now:u64)->serde_json::Value;
pub fn logs_cmd(p:&EnginePaths, limit:usize)->Result<api::LogLines,ApiError>;
// used from WP-06 (one signature in the bones, WP-06 and here):
//   face::logs::log_lines(text:&str, limit:usize)->LogLines            // pure
//   face::logs::read_log_text(p:&EnginePaths)->Result<Option<String>,ApiError>
```

**Behaviour:**
- **`view`** (`link_model::link_view`):
  - `None` → `LinkSnapshot::stopped(now)`, else `.effective(now)` (stale after `STALE_SECS=10`).
  - `label` and `hint` come from `link_view::{link_label, link_hint}`; `start` and `stop` from `headset_buttons(state)`.
  - `health` = `health_view(s.health.as_ref())`.
  - `rx` and `tx` = `throughput_text(stats.rx_bytes_s / tx_bytes_s)`; `uptime` = `uptime_text`. With no stats, all three are `"—"`.
  - `tether`, `tunnel`, `auto_3d` and `adb_source` are copied from the snapshot.
- **`rows [--filter <status>|all]`:** default `all`. Calls `game_rows(&games, filter, &switches::read_settings(p.settings())?, effective_state)`.
- **`launch add|remove [--json] --options <string>`:**
  - The options are exactly one argv element, the one after `--options`. It may be `""` and may contain `--` or `%command%`.
  - `--json` must come before `--options`. Nothing may follow the options value.
  - `--options` missing, or extra elements → usage.
  - `add` → `launch_options::add_wrapper_at(options, &p.wrapper_bin())`.
  - `remove` → `remove_wrapper(options)`.
- **`collection [--current <csv>]`:**
  - `--current ""` → `Some(&[])` (WP-32 sends this). Without the flag → `None`.
  - `wanted` = `three_d_ready_appids`.
  - `add` and `remove` come from `collection_diff`, and are empty when `None`.
  - `name` = `"3D Ready"`.
- **`toasts <last_seq>`:** returns the `ToastBatch` from `toasts_since(events, last_seq)` (oldest first), plus an extra key `"title": TOAST_TITLE`.
- **`logs [--limit n]`:**
  - default `DEFAULT_LOG_LINES=500` (headset_view.rs:21);
  - `read_log_text(p)`; `None` → `LogLines{lines:[],text:""}`;
  - otherwise `face::logs::log_lines(text, limit)`: newest first, blanks dropped, the same rule as `log_lines_newest_first` (headset_view.rs:23).
- **Output rules:**
  - Every command prints JSON; `--json` is implied and accepted.
  - Bad args → `usage`, exit 2.

**Tests:**
- **Locked:** `tests/link_view.rs`, `tests/cli_json.rs`, `tests/headset_view.rs` (unchanged).
- **New `cli_view.rs`:**
  - parse table:
    - `view`; `rows`; `rows --filter works-now`;
    - `launch add --options ""`;
    - `launch add --json --options "%command% -dx11"`;
    - `launch add --json --options "FOO=1 %command% -- -dx11"` (keeps `--` inside the value);
    - `launch add --json --options "-- %command%"`;
    - `launch remove --options "flat3d-run %command%"`;
    - `collection --current 1,2`;
    - `collection --current ""` (→ `Some([])`);
    - `toasts 7`; `logs --limit 3`.
  - bad lines → usage:
    - `rows --filter`;
    - `launch`; `launch add`; `launch frob --options x`;
    - `launch add --options a b`;
    - `launch add --options x --json`;
    - `collection --current a`;
    - `toasts`; `toasts -1`;
    - `logs --limit 0`.
  - shapes:
    - `view` with `None` → `state:"stopped"`, `adb_source:"missing"`, `auto_3d:false`, `rx:"—"`;
    - a stale snapshot (updated now-11) → stopped;
    - stats `rx_bytes_s:1536` → `"1.5 KB/s"`, `uptime_s:185` → `"3m 5s"`;
    - `launch add --options ""` contains the absolute wrapper path followed by `%command%`;
    - `remove` round-trips, including on `"-- %command%"`;
    - `collection --current ""` → `add==wanted`, `remove==[]`;
    - `toasts` returns `title=="Flat to 3D"` and keeps seq order oldest first;
    - `logs` on a missing file → empty; on 5 lines with limit 3 → newest 3.

**Inline facts:**
- DTO fields per docs/code-map.md:110-149. `LinkView{state,label,hint,start,stop,health,rx,tx,uptime,tether,tunnel,auto_3d,adb_source}`.
- kebab-case enums: `TetherMode{off,own,quest-tether}`, `AdbSource{running,bundled,missing}`.
- `WRAPPER="flat3d-run"`. `add_wrapper` inserts after leading env words (launch_options.rs:77-92).
- Decky calls `engine<LaunchText>("launch","add","--json","--options",cur)` (replaces decky.md:173's bare `cur`) and removes the wrapper with `launch remove --options cur` (code-map:188).

**Flags:**
- `Flag:` the scope says `launch <appid>`, but #121, code-map:115 and decky.md:173 say `launch add|remove`. This WP keeps `add|remove`.
- `Flag:` `ToastBatch` has no `title`. This WP adds it as an extra JSON key; no test builds a literal, so this is safe.
- `Flag:` WP-21's `main.py` must put `--json` before `--options <cur>` and not append it after (WP-21 owner).

**Depends on:** WP-01, WP-03, WP-06, WP-20.

**Owner device:** any.

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test cli_view --test link_view --test headset_view --test cli_json && tools/check.sh`

---

### WP-29: doctor + setup commands

**Closes:** the doctor and setup parts of the CLI (`AnyCommand::{Doctor,Setup}`).

**Files (exclusive; the index must list exactly these):**
- `crates/engine/src/doctor.rs`: the pure checks and `doctor()`.
- `crates/engine/src/cli/doctor.rs`: argv and printing.
- `crates/engine/src/cli/setup.rs`: `packages`, `install-apps` and `launch-env`. It sends `fetch` to WP-27's `cli::setup_fetch`.
- `crates/engine/tests/cli_doctor.rs`: replaces the index's `tests/doctor.rs`.
- `crates/engine/tests/cli_setup.rs`

**Signatures:**
```rust
// doctor.rs
pub fn doctor(p:&EnginePaths, r:&mut dyn CommandRunner, quest_serial:Option<&str>)->api::DoctorReport;
// cli/setup.rs
pub fn setup_packages(r:&mut dyn CommandRunner)->Result<serde_json::Value,ApiError>;
pub fn setup_install_apps(p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::SetupReport,ApiError>;
pub fn launch_env(appid:Option<u32>, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::LaunchText,ApiError>;
```

**Behaviour:** `doctor` runs these checks in order. Each one is a `DoctorCheck{name,ok,detail,fix}`, and `ok` is the AND of all of them.
1. **`wivrn-server`:** `wivrn::detect_server` (user flatpak first, then system). Fix: "flatpak install --user flathub io.github.wivrn.wivrn".
2. **`quest-app`:** `detect_quest_app(r,serial)` gives the versionName. Skip with ok:true and detail "no headset" when there is no serial.
3. **`adb`:** `tether::adb::choose_adb(running, bundled_adb(home))`. The detail is the source (running, bundled or missing).
4. **`gnirehtet`:** two parts.
   - The relay exists in `tools_dir(home)` = `{home}/.local/share/flat3d/bin`.
   - The APK exists at its path inside the extracted `gnirehtet` pin (WP-22): `p.cache().join("gnirehtet").join(ver)` + `gnirehtet-rust-linux64/gnirehtet.apk`.
   - Fix: "flat3d-inject setup fetch gnirehtet".
5. **`link-service`:** `systemctl --user is-active flat3d-link.service`. Fix: "systemctl --user enable --now flat3d-link.service".
6. **`wrapper`:** `p.wrapper_bin()` (`~/.local/bin/flat3d-run`) exists and is executable.
7. **`steam`:** `~/.local/share/Steam/steamapps/libraryfolders.vdf` exists.
8. **`pins`:** no pin carries `TBD-at-pin`. The detail lists the keys; fix "run packaging/pin.sh <key>".
9. **`injector`:** `p.data/bin/flat3d-uevr-inject.exe` exists.

Setup commands:
- **`setup packages`:**
  - `adb shell pm list packages` → `parse_packages`;
  - emits `{"wivrn":bool,"gnirehtet":bool,"missing":[..]}`, where `missing()` gives `["WiVRn","gnirehtet"]`.
- **`setup install-apps`:**
  - for each missing package, take its APK:
    - WiVRn: pin `wivrn-apk`;
    - gnirehtet: `gnirehtet.apk` inside the extracted `gnirehtet` pin. There is no `gnirehtet-apk` pin.
  - Fetch through `setup_fetch` when the sentinel is absent, then `adb install -r <apk>`;
  - returns `SetupReport{steps,summary}`;
  - nothing missing → summary "nothing to install".
- **`setup launch-env [<appid>]`** (prints `LaunchText`):
  - No appid → the base Pressure Vessel line `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=… PRESSURE_VESSEL_FILESYSTEMS_RW=<wivrn folder> %command%`, with the same pairs `exec_spec` builds (wrapper.rs:239-242). The folder comes from `detect_server`, so user and system installs both work. WP-24 step0 uses this form.
  - With an appid → `"<abs flat3d-run> %command%"`.
- **`setup fetch <key>`:** delegates to `cli::setup_fetch::setup_fetch` (WP-27).

Doctor never writes anything.

**Tests (FakeRunner only):**
- **New `cli_doctor.rs`:**
  - all checks green in a temp HOME with the files seeded (including `cache/gnirehtet/<ver>/gnirehtet-rust-linux64/gnirehtet.apk`) and the runner scripted → `ok:true`, 9 checks, names in order;
  - systemctl prints `inactive` → only `link-service` is red, and its fix has the enable line;
  - a TBD pin → `pins` is red and names the key;
  - APK missing from the pin dir → `gnirehtet` is red;
  - no serial → `quest-app` ok and skipped;
  - JSON round-trips through `DoctorReport`.
- **New `cli_setup.rs`:**
  - `pm list packages` output `package:org.meumeu.wivrn.github\n` → `missing:["gnirehtet"]`;
  - install-apps with a seeded gnirehtet sentinel makes no `curl` call, and one `adb install -r <cache>/gnirehtet/<ver>/gnirehtet-rust-linux64/gnirehtet.apk`;
  - the runner fails the install → `command-failed`;
  - `launch-env` with no appid → contains `PRESSURE_VESSEL_FILESYSTEMS_RW=` and ends `%command%`;
  - `launch-env 5` → wrapper line;
  - `launch-env abc` → usage exit 2.

**Inline facts:**
- `WIVRN_PACKAGE="org.meumeu.wivrn.github"` and `GNIREHTET_PACKAGE="com.genymobile.gnirehtet"` (setup.rs; locked `tests/setup_packages.rs`).
- `WIVRN_FLATPAK="io.github.wivrn.wivrn"` and `WIVRN_PORT=9757` (link.rs).
- `UNIT_NAME="flat3d-link.service"` (install.rs).
- `DEFAULT_STEAM_ROOT=".local/share/Steam"` (steam.rs).
- The gnirehtet pin is `gnirehtet-rust-linux64-v2.5.1.zip`, sha256 `dee55499…c275`. The relay and the APK come from the same tag (roadmap.md:339).
- The server pin is 26.9, and the Quest versionName must match it.

**Flags:**
- `Flag:` `setup::Packages` derives only `Debug, Clone, Copy, PartialEq, Eq` (setup.rs:17) and setup.rs is not in this WP's files, so the JSON is built with `json!`. Alternatively WP-01 adds `Serialize`.
- `Flag:` wrapper.rs:229 hard-codes the user flatpak path `{home}/.local/share/flatpak/app/io.github.wivrn.wivrn`. The doctor and `launch-env` use `detect_server`, so the two may disagree on system-flatpak installs (WP-31 owner).
- `Flag:` the extracted top folder `gnirehtet-rust-linux64/` assumes WP-13's `unpack` keeps the zip's top folder. If WP-13 strips it, drop that segment in both checks.

**Depends on:** WP-01, WP-12, WP-13, WP-20, WP-22, WP-27 (`setup_fetch`).

**Owner device:** any (fakes); a manual Deck run later.

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test cli_doctor --test cli_setup --test setup_packages --test cli_json && tools/check.sh`

---

### WP-30: RE tooling + address map (#19 tools, #37 column)

**Closes:** the #19 tooling half and the #37 SM2-column generator. The address values themselves stay pending until a VT run.

**Files (exclusive; the index must list exactly these):**
- `tools/re/analyze.sh`: T5. Ghidra analyse and VT driver; replaces `vt.sh` and `pipeline.sh`.
- `tools/re/exe_id.py`: T7. Exe → build slug.
- `tools/re/fill_map.py`: T6. Merge, render and splice.
- `tools/ghidra/Flat3dVtOptions.java`
- `tools/ghidra/ExportVtMatches.java`
- `games/spiderman2/map.toml`: the single source of truth. There is no `addresses.toml`.
- `tools/re/test_fill_map.py`
- `tools/re/test_analyze_sh.py`
- `tools/re/test_exe_id.py`
- Not touched: `tools/ghidra/headless.sh` (WP-25 owns it), because the folder goes in the project-name argument.

**Signatures:**
- `analyze.sh analyze <projdir> <exe> [--dry-run]`
- `analyze.sh vt <projdir> <src-slug> <dst-slug> <out.csv> [--dry-run]`
- `analyze.sh all <projdir> <src-exe> <dst-exe> <out.csv> [--dry-run]`
- `exe_id.py <exe> [--json]` → `slug sha256 version`. Exit 3 on an unknown hash, which prints the sha so it can be added.
- `fill_map.py --map games/spiderman2/map.toml --vt out.csv --targets tools/xmatch/targets.py --doc docs/spiderman2/address-map.md [--check]`
- Python:
  - `exe_id`: `def identify(path)->tuple[str,str,str]`.
  - `fill_map`: `def load_vt(path)->dict[int,tuple[int,str,float,float,str]]`, `def render_table(prio:int, items, vt)->str`, `def splice(doc:str, prio:int, table:str)->str`, `def emit_toml(d)->str`.

**Behaviour:**
- **T5 `analyze.sh analyze`:**
  - the slug comes from `exe_id.py`;
  - runs `headless.sh <projdir> spiderman/<slug> -import <exe> -max-cpu 16 -log <log>`;
  - the folder goes in the project-name argument, never `-folderPath`.
- **T5 `analyze.sh vt`:**
  - runs `headless.sh <projdir> spiderman -process <dst> -preScript Flat3dVtOptions.java -postScript AutoVersionTrackingScript.java "/" "<session>" "/spiderman/<src>" -postScript ExportVtMatches.java "/<session>" <out.csv> -scriptPath tools/ghidra`;
  - no `-readOnly`;
  - the session name is fresh, `vt-<unix>`, because AutoVT refuses an existing session (AutoVersionTrackingScript.java:141-146);
  - both programs must be analysed first;
  - `all` = analyze src, analyze dst, vt, then `fill_map.py`;
  - `--dry-run` prints each argv and exits 0;
  - honours exit 75 (busy) and 137 (low memory) from headless.sh unchanged.
- **T7 `exe_id.py`:**
  - stdlib only;
  - streams sha256, reads the PE version resource, and maps the hash through `KNOWN_BUILDS`, copied from tools/research/inspect_game.py:17-21, plus slugs `remastered-4.630` and `sm2-2.810`.
- **Flat3dVtOptions.java:**
  - copies SetAutoVersionTrackingOptionsScript, using a `GhidraValuesMap` and `VTOptionDefines.*_OPTION_TEXT`;
  - ends with `state.addEnvironmentVar("autoVTOptionsMap", map)` (read at :207-208);
  - values:
    - CREATE_IMPLIED_MATCHES true;
    - RUN_EXACT_SYMBOL, RUN_EXACT_DATA, RUN_EXACT_FUNCTION_BYTES, RUN_EXACT_FUNCTION_INST, RUN_DUPE_FUNCTION and RUN_REF_CORRELATORS all true;
    - DATA_CORRELATOR_MIN_LEN 5, SYMBOL_CORRELATOR_MIN_LEN 3, FUNCTION_CORRELATOR_MIN_LEN 10, DUPE_FUNCTION_CORRELATOR_MIN_LEN 10;
    - APPLY_IMPLIED_MATCHES true, MIN_VOTES 2, MAX_CONFLICTS 0;
    - REF_CORRELATOR_MIN_SCORE 0.95d, REF_CORRELATOR_MIN_CONF 10.0d.
- **ExportVtMatches.java:**
  - extends `GhidraScript`;
  - `args[0]` = session path, `args[1]` = csv;
  - opens with `getProjectData().getFile(path).getDomainObject(this,true,true,monitor)`;
  - iterates `getMatchSets()` → `getMatches()`;
  - writes the header `src_addr,dst_addr,status,similarity,confidence,correlator`;
  - releases the session in `finally`. Never call `saveVersionTrackingSession`: its null check is inverted and it throws.
- **T6 `fill_map.py` (render):**
  - stdlib only, Python 3.12+ (`tomllib` to read; it emits TOML itself, because there is no writer);
  - merges VT rows into `map.toml` by Remastered RVA;
  - a target becomes `found` only when a VT match is computed for it;
  - `render_table` uses the header and separator of tools/xmatch/mkmap.py:14-15 verbatim;
  - `splice` replaces only the text between the literal markers `<!-- BEGIN GENERATED: fill_map prio-N -->` and `<!-- END GENERATED: fill_map prio-N -->`. WP-25 inserts these markers, and WP-25's `check.py` matches the same literal;
  - missing markers → exit 2 and refuse (no guessing);
  - `--check` exits 1 on drift.
- **Lints (exit 1):**
  - an `evidence` field equal to `herovr`;
  - any `[0-9a-fA-F]{5,}` inside `[[herovr]]` text;
  - `found` without a computed match.
- **One owner per fact:** `games/spiderman2/map.toml` holds the addresses. The doc tables are generated from it.

**Tests:**
- **New `test_fill_map.py`:**
  - a 3-row CSV fixture produces the right table, the splice is idempotent, the markers stay intact, and text outside the markers is byte-identical;
  - missing markers → refuse;
  - each lint trips;
  - TOML emit round-trips through `tomllib`.
- **New `test_analyze_sh.py`:**
  - the `vt --dry-run` and `analyze --dry-run` argv equal the expected lists;
  - `exe_id.py` is stubbed through `FLAT3D_EXE_ID` or a fixture;
  - shellcheck clean.
- **New `test_exe_id.py`:** a fixture file whose sha is patched into a temp `KNOWN_BUILDS` → slug; unknown → exit 3.
- **Locked:** none.

**Inline facts:**
- Ghidra lives at `~/.local/share/flat-to-3d/ghidra_12.1.4_PUBLIC` (symlink `.../ghidra`), JDK 21 at `.../jdk/current`.
- `GHIDRA_HEADLESS_MAXMEM=24G`. headless.sh exits 75 when busy (qwen.lock or pgrep) and 137 when MemAvailable < 2048 MiB.
- **Builds:**
  - Remastered 4.630.0.0: sha256 `e297d4d94f1ffe4febf289745e79e7b6fa233a788e7a00f480fc77c55db81ad1`, Steam build 23986256, appid 1817070;
  - SM2 2.810.0.0: sha256 `c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9`, size 194262992, Steam build 24651585, appid 2651280, exe `Spider-Man2.exe`;
  - the local SM2 copy is at `~/.local/share/flat-to-3d/exe/sm2-2.810.0.0`.
  - Source for both: tools/research/inspect_game.py:12-21.
- `targets.py` has 85 rows `(name,rva,kind,prio,hex)`.
- **map.toml:**
  - `[meta] herovr_rev="82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859"`;
  - `check` ∈ `found|not-found|pending|runtime|off-path|no-row`;
  - seed rows: H1 runtime; H2-H5 and H7-H10 pending; H6 no-row; H11 off-path.
- HEROVR source: https://github.com/lecutis7of9/HEROVR/blob/82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859/BUILD-SM2.md
- The doc is `docs/spiderman2-address-map.md` today, with its prio tables under the headings at :62, :83 and :111. WP-25 moves it to `docs/spiderman2/address-map.md` and adds the markers.

**Flags:**
- `Flag:` `analyze.sh` calls WP-25's `headless.sh` and must use WP-25's `FLAT3D_RE_HOME`, `FLAT3D_HEAVY_LOCK` and `FLAT3D_BUILD_BOX`. headless.sh is no longer edited here.
- `Flag:` WP-25's `check.py` and its marker literal must equal `<!-- BEGIN GENERATED: fill_map prio-N -->` / `<!-- END GENERATED: fill_map prio-N -->`. This replaces WP-25's `BEGIN GENERATED: tools/re/fill_map.py render` (WP-25 owner).

**Depends on:** WP-02, WP-25.

**Owner device:** the build box (VT run, 24G heap). Tests run on any machine.

**Done-check:** `python3 -m unittest discover -s tools/re -p 'test_*.py' && shellcheck tools/re/analyze.sh && FLAT3D_EXE_ID=stub tools/re/analyze.sh vt /tmp/p remastered-4.630 sm2-2.810 /tmp/o.csv --dry-run`

Review note: rejected WP-28's "use WP-06's signature `log_lines(state_dir,runs_dir,max)`" because the [locked] correction rewrites WP-06 to the pure `log_lines(text,limit)` plus `read_log_text(p)`, which wraps the existing `log_lines_newest_first(text, limit)` (inject/src/headset_view.rs:23). WP-28 uses that new WP-06 form.

### WP-31: Wrapper route end-to-end
**Closes:** A5 (complete), A7, A8 (wrapper half), gap D1.
**Files (exclusive):** `crates/engine/src/wrapper.rs`, `crates/engine/src/bin/flat3d-run.rs`, `crates/engine/tests/wrapper_route.rs`.

**Signatures** (checked against `inject/src/wrapper.rs` @2812f64). These items stay byte-for-byte: `prepare` (:29), `run<T>` (:119), `RunPaths` (:12, which the test at wrapper_run.rs:214 builds), `exec_spec` (:219), `plan` (:259). WP-01 froze the new items. If its frozen parameter lists differ from the ones below, keep WP-01's lists; the behaviour here still applies.
```rust
pub struct ExecSpec { pub argv: Vec<String>, pub env: Vec<(String,String)>, pub unset: Vec<String>, pub sidecar: Option<Vec<String>> } // no test builds a literal; legacy exec_spec sets vec![]/None
pub fn exec_spec_with(plan:&Plan, argv:&[String], route:&RouteEnv, wivrn:&WivrnDirs, existing_fs_rw:Option<&str>) -> ExecSpec;
pub fn prepare_with_route(argv:&[String], env:&BTreeMap<String,String>, snapshot:Result<Option<&LinkSnapshot>,String>,
    settings:Result<&Settings,String>, recipe:Result<Option<&Recipe>,String>,
    route:&mut dyn FnMut(u32,&Recipe)->Result<(RouteEnv,WivrnDirs),String>, pid:u32, now:u64) -> Launch;
pub fn run_with<T>(argv:&[String], env:&BTreeMap<String,String>, paths:&EnginePaths, pid:u32, now:u64,
    runner:&mut dyn CommandRunner, launcher: impl FnOnce(&Launch)->io::Result<T>) -> (Launch, io::Result<T>);
pub fn note_run(paths:&EnginePaths, launch:&Launch, now:u64, key:&str, value:&str);          // appends a record line + run.log line, ignores IO errors
pub fn flat_after_sidecar_error(paths:&EnginePaths, launch:&Launch, now:u64, err:&str) -> ExecSpec;
```
`RouteEnv{env,unset,sidecar}` is imported from wherever WP-01 put it (`crate::injectors`). `WivrnDirs` and `wivrn::detect_server(&mut dyn CommandRunner)->Result<WivrnDirs,String>` come from WP-12. `injectors::ensure_applied(&EnginePaths, u32, &Recipe)->Result<RouteEnv,String>` comes from WP-27.

**Behaviour**
- **`run_with`:**
  - appid comes from the private `appid()` (:98).
  - snapshot: `state::read_snapshot(paths)`.
  - settings: `switches::read_settings(&paths.settings())`. NotFound gives Default (K5).
  - recipe: `catalog::find_recipe(&paths.recipe_dirs, id)` (`{id}.recipe` or `{id}-*.recipe`, first dir wins; this fixes A5), then `recipes::parse_recipe` and `check_recipe`.
  - Then `prepare_with_route` with the closure `|id,r| { let e = injectors::ensure_applied(paths,id,r).map_err(|e| format!("route: {e}"))?; let d = wivrn::detect_server(runner).map_err(|e| format!("wivrn: {e}"))?; Ok((e,d)) }`.
- **`prepare_with_route` order:**
  - appid error, then `Ok(None)` snapshot ("no link snapshot"), then `state::is_stale(s.updated,now)` ("snapshot is stale").
  - Then `plan(s.state, settings.game_on(id), recipe)`. The `LinkState::Ready` gate stays in `plan` (:270). OD4 is unchanged: Grace counts as Ready.
  - Only when the plan is `ThreeD`: require a non-empty HOME, then call `route` exactly once.
  - Any `Err(m)` gives `Plan::Flat{NotInjectable(m)}` and the message `"flat: {m}"`. On a Flat plan, `route` is never called and the runner sees no argv.
  - `Launch.command = exec_spec_with(..)`. The message formats match legacy: `"3d: {route:?}"` and `"flat: {reason.text()}"`.
- **`exec_spec_with`:**
  - Flat: `argv` only, empty env and unset, `sidecar: None`.
  - ThreeD: the same 3 PV vars as :237-247, but the WiVRn folder is `wivrn`'s stripped app root, so a system install gives `/var/lib/flatpak/app/io.github.wivrn.wivrn`. FS_RW merging is the same as :230-236.
  - Then `route.env`. A key already present is replaced in place, so there is one entry per key and the last value wins.
  - `unset = route.unset` minus any key that is also in env, deduped.
  - `sidecar = route.sidecar.clone()`, verbatim. The bin appends `--until-pid`.
- **Record:** written atomically to `state::run_record_path(&paths.state_dir.to_string_lossy(), pid)`.
  - Content: `record.to_text()`, plus `note\t<message>` when the launch is flat for an error. Extra lines are legal (runs_extra_lines.rs).
  - Then `fsx::append_line(paths.run_log(), "{now} appid={} pid={pid} {msg}")`, with newlines replaced as at :172.
  - IO failures append `; run record: {e}`. The launcher is always called exactly once (same guarantee as :117-118).
- **`flat3d-run.rs`:**
  - `EnginePaths::from_env(&|k| env.get(k).cloned(), sys::current_uid())`. On Err, exec argv untouched and print `flat3d-run: paths: {e}` to stderr. Delete the local `uid()` (:7-19) in favour of `sys::current_uid`.
  - Runner: `exec::SystemRunner`.
  - Launcher: if `command.sidecar` is `Some(sc)`, resolve `sc[0]`. When it is not absolute, try `current_exe().parent()/sc[0]`, then `paths.bin/sc[0]`. Spawn it with `.args(&sc[1..]).args(["--until-pid", &std::process::id().to_string()])`, stdin null, stdout/stderr appended to `paths.log_dir/viewer.log`.
    - On Ok: `note_run(.., "sidecar", "pid {id} {argv}")`.
    - On Err: `flat_after_sidecar_error`, which rewrites the record to `mode flat` with `note\tsidecar: {e}` and logs `flat: sidecar: {e}`, then exec the flat spec.
  - Then `Command::new(argv[0]).args(..).envs(env)`, plus `env_remove(k)` for each `unset`, then `exec()` (unix), as at :53-66.
  - The viewer exits when the game pid exits, so it is re-parented and reaped and leaves no zombie.

**Tests**
- **Locked and unchanged:** `wrapper_run.rs` (WP-01 already changed :206-207 to `CARGO_TARGET_TMPDIR`), `wrapper_plan.rs`, `wrapper_exec.rs` (`FOLDER` is still the user path through legacy `exec_spec`) and `runs_extra_lines.rs`. Legacy `run` keeps reading `{id}.recipe`.
- **New `wrapper_route.rs`.** Temp root `env!("CARGO_TARGET_TMPDIR")/wrapper-route/<pid>-<n>`. Uses `testing::FakeRunner`. Reuses wrapper_run's RECIPE and SETTINGS text.
  1. `42-test-game.recipe` in a recipe dir with no env file gives flat with "route:", not "no recipe" (A5).
  2. A missing settings file is not "invalid settings".
  3. Starting, Stopped and Error give flat "headset link is not ready", the route closure is called 0 times and FakeRunner records no argv.
  4. A stale snapshot (updated 100, now 111) gives "stale".
  5. With `Ok(None)`, the message contains "snapshot".
  6. A route `Ok` with system dirs gives ThreeD, FS_RW `/games:/var/lib/flatpak/app/io.github.wivrn.wivrn`, `PROTON_REMOTE_DEBUG_CMD` present, and unset `["XR_RUNTIME_JSON"]`.
  7. A route `Err("env file missing")` gives the record mode Flat and the message `flat: route: env file missing`.
  8. A wivrn `Err` gives "wivrn:".
  9. An empty HOME gives flat, and the closure is not called.
  10. `exec_spec_with` Flat drops the sidecar.
  11. A sidecar is copied verbatim, with no `--until-pid`.
  12. A route env `PRESSURE_VESSEL_FILESYSTEMS_RW` replaces the base entry, leaving one entry.
  13. A key in both env and unset is kept only in env.
  14. `run_with`: the record parses with `parse_run_record`, the `note\t` line is present, and run.log contains `appid=42 pid=7 flat: route:`.
  15. The launcher runs once when the log dir is a file.
  16. `flat_after_sidecar_error` gives a Flat record and an argv-only spec.

**Inline facts**
- Flatpak id: `io.github.wivrn.wivrn`.
- Detect: `flatpak info --user --show-location io.github.wivrn.wivrn`, then `--system`. Strip `/<arch>/<branch>/<commit>`. A non-zero exit means "not here".
- PV env: `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1`, `PRESSURE_VESSEL_FILESYSTEMS_RW`, `STEAM_COMPAT_LAUNCHER_SERVICE=proton`.
- UEVR route: `PROTON_REMOTE_DEBUG_CMD`, plus unset `XR_RUNTIME_JSON` (uevr-openxr.md:126).
- Geo-11 route: `WINEDLLOVERRIDES="d3d11,nvapi64=n,b"`. Sidecar `flat3d-viewer --layout sbs --source auto --screen-width-m 3 --distance-m 2.5` + `--until-pid N` (geo11-viewer.md:133,138).
- Env file: `data/env/<appid>.env` (`EnginePaths::env_file`).
- Server flags `--early-active-runtime --no-publish-service` belong to WP-12, not here.
- `STALE_SECS=10` (state.rs:83).

**Depends on:** WP-27, WP-12, WP-16 (and everything WP-01 froze).
**Owner device:** Q6 `sessions/z2-uevr.toml`, Q9 `g1-geo11.toml`, Q10 `g2-viewer.toml`.
**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test wrapper_route --test wrapper_run --test wrapper_plan --test wrapper_exec --test runs_extra_lines && cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings && tools/check.sh`

**Flags**
- **Flag:** after a Geo-11 inject, Geo-11's `d3d11.dll` in the game folder may load even on a "flat" launch. Proton puts its own `d3d11=n` (DXVK) ahead of ours (geo11-viewer.md:38). If so, a flat fallback can still render SBS. WP-16/27 should move the DLL aside, or we accept it. Verify in Q9.
- **Flag:** recording sidecar success and failure needs the two helpers above in wrapper.rs, because `launcher` only gets `&Launch`. No frozen signature changes.

### WP-32: Decky panel, launch sync, collection, toasts (#131-#133)
**Closes:** #131, #132, #133 (code).
**Files (exclusive):** `decky/src/index.tsx` (rewrites WP-21's scaffold), `decky/src/panel.tsx`, `decky/src/launch.ts`, `decky/src/collection.ts`, `decky/src/toasts.ts`.

**Signatures**
```ts
// panel.tsx
export function Content(): JSX.Element;
// launch.ts
export async function syncLaunchOptions(plan: CollectionPlan): Promise<string[]>; // returns error messages
// collection.ts
export async function syncCollection(): Promise<CollectionPlan | null>;
// toasts.ts
export function startToasts(everyMs?: number /*2000*/): () => void; // returns the stop fn
// index.tsx
export default definePlugin(() => ({ name: "Flat to 3D", titleView, content: <Content/>, icon: <FaVrCardboard/>, onDismount: stop }));
```
Imports:
- `engine<T>(...args)`, `Envelope<T>` from `./engine` (WP-21).
- `LinkView, GameRow, LaunchText, CollectionPlan, ToastBatch, DoctorReport, TOAST_TITLE` from `./types` (WP-21).
- `definePlugin, toaster` from `@decky/api`.
- `PanelSection, PanelSectionRow, ButtonItem, ToggleField, Field, staticClasses` from `@decky/ui`. Importing `@decky/ui` types the globals `SteamClient`, `appStore`, `appDetailsStore` and `collectionStore` (decky.md:41).

**Behaviour**
- **`panel.tsx`:**
  - While mounted, a 1 s `setInterval` calls `engine<LinkView>("view")` and `engine<GameRow[]>("rows")`. Clear it on unmount.
  - Render order (#131):
    1. `Field label={v.label} description={v.hint}`.
    2. `ButtonItem` "Start 3D" (`disabled={!v.start}`) calls `engine("start-3d")`. "Stop 3D" (`!v.stop`) calls `engine("stop-3d")`. Both are requests; the CLI follows the A24 one-actor rule.
    3. Health: `v.health.battery`, `v.health.temp` (colour `v.health.colour`), `v.rx`/`v.tx`/`v.uptime`.
    4. Toggles: `ToggleField` "Tunnel" (`checked={v.tunnel}`) calls `engine("setting","set","tunnel",on?"on":"off")`. "Auto 3D" does the same with `auto-3d`.
    5. One `ToggleField` per row (`label=r.name`, `description=r.flat_reason ?? r.status_label`, `checked=r.switch_on`). It calls `engine("switch","set",String(r.appid),on?"on":"off")`, then refreshes.
    6. A "Check setup" `ButtonItem` calls `engine<DoctorReport>("doctor")` on demand only (it runs adb), then lists the checks where `!ok` as `Field label={name} description={detail+" — "+fix}`.
  - On any `!ok` envelope, show one `Field` with `error.message` verbatim. There is no wording of our own and no logic.
- **`collection.ts`:**
  - `id = collectionStore.GetCollectionIDByUserTag("3D Ready")`. Then `c = id ? collectionStore.GetCollection(id) : collectionStore.NewUnsavedCollection(name, undefined, [])`. If the filter param's type rejects `undefined`, cast via `undefined as unknown as Parameters<typeof collectionStore.NewUnsavedCollection>[1]` (no `any`).
  - `cur = Array.from(c.apps.keys()).join(",")`. This works whether `apps` is a Set or a Map.
  - `plan = engine<CollectionPlan>("collection","--current",cur)`. Use `plan.name` for the lookup on later runs.
  - `ov = ids => ids.map(i => appStore.GetAppOverviewByAppID(i)).filter((o): o is NonNullable<typeof o> => !!o)`.
  - `dd = c.AsDragDropCollection()`, then `dd.AddApps(ov(add))`, `dd.RemoveApps(ov(remove))`, then `await c.Save()` only if `add.length + remove.length > 0`. Return the plan.
- **`launch.ts`:**
  - For `id` of `plan.wanted`: `await appDetailsStore.RequestAppDetails(id)`, then `cur = appDetailsStore.GetAppDetails(id)?.strLaunchOptions ?? ""`, then `r = engine<LaunchText>("launch","add",cur)`.
  - If `r.ok && r.data.options !== cur`, call `SteamClient.Apps.SetAppLaunchOptions(id, r.data.options)`. It is fire-and-forget (void).
  - For `id` of `plan.remove`, do the same with `"remove"`.
  - `cur` goes in as one argv element. The engine does all the parsing, adds our absolute-path word, and strips the bare or absolute word without touching the rest (OD5: no saved line is ever restored).
- **`toasts.ts`:** `lastSeq = 0` in memory. Every 2 s, `b = engine<ToastBatch>("toasts", String(lastSeq))`. If `b.ok`, call `toaster.toast({ title: TOAST_TITLE, body: t.text })` for each toast (oldest first), then `lastSeq = b.data.last_seq`.
- **`index.tsx`:**
  - Inside `definePlugin`, call `stop = startToasts()`. The interval lives outside the QAM (decky.md:44).
  - Then `void syncCollection().then(p => p && syncLaunchOptions(p))`. Panel mount re-runs the same sync.
  - `titleView = <div className={staticClasses.Title}>Flat to 3D</div>`, icon `FaVrCardboard` from `react-icons/fa`.

**Tests:** none (R-ts: "no logic, no tests, typecheck only"). The locked `decky/tests/test_run_engine.py` (WP-21) must stay green. The CI `decky` job runs `pnpm i --frozen-lockfile && pnpm run typecheck && pnpm run build`.

**Inline facts**
- `SteamClient.Apps.SetAppLaunchOptions(appId:number, launchOptions:string): void` (@decky/ui 4.16.0 `App.d.ts:92`).
- `appDetailsStore` (`appdetailsstore.d.ts:63,76`; `strLaunchOptions` at `App.d.ts:443`).
- `collectionStore`: `GetCollectionIDByUserTag`, `GetCollection`, `NewUnsavedCollection(strName, filter, rgApps)`, `AsDragDropCollection().AddApps/RemoveApps`, `Save(): Promise<void>`. Pattern: https://github.com/OpenCloudGaming/OpenNOW/blob/bec4a0b11fc77fb5bfe9584f012296333718e84b/decky/src/steam.ts#L73-L79
- `toaster.toast({title, body})` (`@decky/api@1.1.3/dist/types.d.ts:12-37`).
- Names and versions: `COLLECTION_NAME="3D Ready"` (collection.rs:8), `TOAST_TITLE="Flat to 3D"`. Pins: @decky/api 1.1.3, @decky/ui 4.16.0, react-icons 5.3.0, typescript 5.9.3, pnpm 10.34.9 in CI (9.15.9 locally reads lockfile v9).
- `ErrorCode`s are the 8 in api_response.rs:11.

**Depends on:** WP-21, WP-28.
**Owner device:** Q14 `sessions/decky.toml`. It checks that `SetAppLaunchOptions` persists, that `Save()` persists, that a toast shows, and that `XDG_RUNTIME_DIR` is set by the `main.py` `_env`.
**Done-check:** `cd decky && pnpm i --frozen-lockfile && pnpm run typecheck && pnpm run build && python3 -m unittest discover -s tests -p 'test_*.py'`

**Flags**
- **Flag:** #131 says `decky/src/Panel.tsx`. The WP index says `panel.tsx`, and that is the name to use. Fix #131 in §6.
- **Flag:** `types.ts` (WP-21) must export `DoctorReport`/`DoctorCheck` and `TOAST_TITLE`. If it does not, WP-32 cannot add them, which blocks WP-32 or needs a WP-21 follow-up.
- **Flag:** WP-28's `collection` parser must accept `--current ""` (empty collection). Otherwise `plan.add` is empty on the first run.

### WP-33: App views + main wiring (#122/#123)
**Closes:** #122, #123 (code).
**Files (exclusive):** `crates/app/src/{lib,main,engine,view_headset,view_games,view_setup,view_logs}.rs`, `crates/app/tests/app_wiring.rs`.
**Build environment:** the `craft-build` distrobox (archlinux base-devel, running on this machine) or CI. iced needs system libraries (`pkg-config`, xkbcommon, wayland, fontconfig) that the SteamOS host does not have, so every `-p flat3d-app` build, test and clippy runs inside `distrobox enter craft-build`. In CI they come from WP-01's apt line (see Depends on).

**Signatures.** Page types come from WP-04/05/06/23 as frozen. Where a variant name below differs from the frozen one, the frozen name wins and the behaviour stays.
```rust
// lib.rs: keep Tab/ALL/title/placeholder and the 4 unit tests (lib.rs:5-38, :61-90)
pub mod headset_page; pub mod games_page; pub mod setup_page; pub mod logs_page; pub mod engine;
pub mod view_headset; pub mod view_games; pub mod view_setup; pub mod view_logs;
#[derive(Debug, Default, Clone)] pub struct App { pub active: Tab, pub now: u64, pub ticks: u64,
  pub headset: headset_page::HeadsetState, pub games: games_page::GamesState,
  pub setup: setup_page::SetupState, pub logs: logs_page::LogsState }   // packages + setup note live in SetupState (WP-05)
#[derive(Debug, Clone)] pub enum Message { Select(Tab), Tick(u64), Headset(headset_page::Msg), Games(games_page::Msg),
  Setup(setup_page::Msg), Logs(logs_page::Msg), SwitchSaved(Result<(),String>) }   // no lib-local SetupMsg / Packages / SetupDone
#[derive(Debug, Clone, PartialEq, Eq)] pub enum Effect { ReadSnapshot, Request(Request), SetSetting{key:&'static str,on:bool},
  LoadRows{filter:String}, SetSwitch{appid:u32,on:bool}, Undo(u32), ReadPackages, LoadLogs, Setup(Vec<String>), Copy(String) }
pub fn update(app:&mut App, m:Message) -> Vec<Effect>;   // pure
// engine.rs (the only fs/process code)
pub fn paths() -> Result<EnginePaths,String>;             // EnginePaths::from_env with the BONES (WP-01) argument list, process env + sys::current_uid()
pub fn cli_path(exe:&Path, src_dir:Option<&str>, bin:&Path, exists:&dyn Fn(&Path)->bool) -> Option<PathBuf>;
pub fn snapshot_or_stopped(r:Result<Option<LinkSnapshot>,String>, now:u64) -> Result<LinkSnapshot,String>;
pub fn read_snapshot(p:&EnginePaths, now:u64) -> Result<LinkSnapshot,String>;   // state::read_snapshot + snapshot_or_stopped
pub fn request(p:&EnginePaths, r:Request) -> Result<(),String>;                 // state::write_request (request_text → request_path)
pub fn parse_envelope(stdout:&str) -> Result<serde_json::Value,String>;          // data | error.message
pub fn row_from_json(v:&serde_json::Value) -> Option<GameRow>;                 // GameRow is Serialize-only (#76)
pub fn run_cli(args:&[&str]) -> Result<serde_json::Value,String>;              // <cli> args.. --json
// view_*.rs
pub fn view_headset::view(s:&HeadsetState, now:u64) -> Element<'_, headset_page::Msg>;
pub fn view_games::view(s:&GamesState) -> Element<'_, games_page::Msg>;
pub fn view_setup::view<'a>(s:&'a SetupState, snap:Option<&'a LinkSnapshot>) -> Element<'a, setup_page::Msg>;
pub fn view_logs::view(s:&LogsState) -> Element<'_, logs_page::Msg>;
```

**Behaviour**
- **`update` routing:**
  - `Select(t)` sets `active` and returns `[ReadSnapshot]`, plus `LoadRows{filter}` for Games, `ReadPackages` for Setup, and `LoadLogs` for Logs.
  - `Tick(n)` sets `now=n`, `ticks+=1`, and returns `[ReadSnapshot]`. Add `LoadLogs` when Logs is active, and `ReadPackages` when Setup is active and `ticks%10==0`.
  - `Headset(m)` calls `headset_page::update`, which maps `Start3d`/`Stop3d` to `Request(..)`, `SetSetting` to `SetSetting`, and `Refresh` to `ReadSnapshot`.
  - `Games(m)` maps `LoadRows`, `SetSwitch` and `Undo` 1:1.
  - `Logs(m)` maps `Load` to `LoadLogs` and `Copy(t)` to `Copy(t)`.
  - `Setup(m)` calls `setup_page::update(&mut app.setup, m)` (WP-05). Its action maps to an `Effect`: recheck gives `ReadPackages`, install-apps gives `Setup(["setup","install-apps"])`, and doctor gives `Setup(["doctor"])`. Result messages (packages loaded, setup done) go through the same `update` and set `SetupState` fields. `lib.rs` keeps no setup state of its own.
  - `SwitchSaved(Ok)` gives `LoadRows{current filter}`. `Err(e)` sets `games.error`.
- **`main.rs`:**
  - `iced::application(App::default, step, view).title("Flat to 3D").subscription(|_| iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick(flat3d_inject::sys::unix_now()))).run()`. The map closure captures nothing.
  - `step` = `Task::batch(update(..).into_iter().map(run_effect))`.
  - Every effect except `Copy` (`iced::clipboard::write(t)`) runs through `Task::perform(off_thread(..), map)`. `off_thread` is a oneshot from `iced::futures::channel::oneshot` fed by `std::thread::spawn` (iced-packaging.md §1).
  - Effects map back as:
    - `ReadSnapshot` → `Headset(Msg::Snapshot)`;
    - `Request` and `SetSetting` → `Headset(Msg::Done)`;
    - `LoadRows` → `Games(Msg::Rows)` via `run_cli(["rows","--filter",f])` + `row_from_json`;
    - `SetSwitch` → `SwitchSaved` via `["switch","set",id,"on"|"off"]`;
    - `Undo` → `Games(Msg::UndoDone)` via `["undo",id]` with `data.summary`;
    - `ReadPackages` → `Setup(<WP-05 packages-result Msg>)` via `["setup","packages"]`, reading `data.wivrn` and `data.gnirehtet` into `flat3d_inject::setup::Packages` (setup.rs:18);
    - `LoadLogs` → `Logs(logs_page::Msg::Loaded)` via `["logs","--limit","200"]` with `data.lines` (WP-06's variant);
    - `Setup(args)` → `Setup(<WP-05 done-result Msg>)` with `data.summary` or the error text.
  - `SetSetting` goes through `["setting","set",key,on]`.
  - The app never runs adb or `link::start_3d`; Start/Stop only writes the request file.
  - Tabs are a button row as today (main.rs:13-19), with the page below `.map(Message::X)`. Closing the window leaves `flat3d-link.service` running.
- **`paths()`:** calls `EnginePaths::from_env` with exactly the argument list WP-01 froze in BONES, fed from `std::env` and `sys::current_uid()`. WP-31's `flat3d-run.rs` call and this one must use the same form. If BONES's form is not roadmap.md:201's `(&dyn Fn(&str)->Option<String>, u32) -> Result<Self,String>` (engine-a.md:80 proposed `(&BTreeMap<String,String>, u32) -> EnginePaths`), follow BONES and wrap a non-Result return in `Ok`.
- **`cli_path` order:** `exe.parent()/flat3d-inject`, then `$FLAT3D_SRC_DIR/bin/flat3d-inject`, then `bin/flat3d-inject`.
- **Views:**
  - `view_headset` takes its labels, hint, start/stop and health text from `face::link_model::link_view(snapshot or LinkSnapshot::stopped(now), now)`. `button.on_press` is attached only when `start`/`stop` is true and `!busy`. `toggler(auto_3d).label("Auto 3D").on_toggle(Msg::SetAuto3d)` and the same for Tunnel. The error appears as text.
  - `view_games`: `pick_list(["all"]+status::STATUSES)` gives `Filter`. A `scrollable` list of rows shows name, `text(status_label).color(Color::parse(status_colour))`, a toggler (`Switch`), `flat_reason`, and an Undo button. Note and error appear as text.
  - `view_setup`: `setup_page::steps(s.packages, snap)` (WP-05) gives one line each, with ✓ or ○, title and hint. It also shows the setup note or error from `SetupState` and 3 buttons that emit WP-05's recheck, install-apps and doctor `Msg`s.
  - `view_logs`: a `scrollable` of lines and a Copy button (`CopyPressed`).

**Tests**
- **Locked, unchanged:** `crates/app/tests/{headset_page,games_page,setup_page,logs_page}.rs` (#116, #117, #118, WP-06) and the lib.rs unit tests.
- **New `app_wiring.rs`:**
  1. `Select(Games)` gives `[ReadSnapshot, LoadRows{"all"}]`.
  2. `Tick` on Setup gives `ReadPackages` only on every 10th tick.
  3. `Tick` on Logs gives `LoadLogs`.
  4. `StartPressed` with no snapshot gives `Request(Start3d)`, and a second press gives nothing.
  5. `Done(Ok)` gives `ReadSnapshot`.
  6. `SetTunnel(false)` gives `SetSetting{"tunnel",false}`.
  7. `Games(UndoPressed(7))` gives `Undo(7)`.
  8. `SwitchSaved(Err)` sets `games.error`.
  9. `SwitchSaved(Ok)` reloads with the current filter.
  10. `Logs(CopyPressed)` gives `Copy(text)`.
  11. The setup page's doctor `Msg` gives `Setup(["doctor"])`, and its recheck `Msg` gives `ReadPackages`. A packages-result `Msg` lands in `app.setup` (through `SetupState::update`, not a lib field).
  12. `Logs(Msg::Loaded(Ok(..)))` fills `app.logs.lines` and returns no effect.
  13. `cli_path` prefers the sibling, then `FLAT3D_SRC_DIR/bin`, then `None`.
  14. `snapshot_or_stopped(Ok(None),5)` is `stopped(5)`. A stale snapshot is `Stopped`. An `Err` passes through.
  15. `parse_envelope` returns `data` on ok, `error.message` on an error, and Err on non-JSON.
  16. `row_from_json` round-trips `serde_json::to_value(GameRow)` and gives `None` on a missing key.

**Inline facts**
- iced 0.14.0 (app/Cargo.toml:21) with `features=["smol"]`; `every` exists only with smol or tokio (iced-packaging.md G1).
- `request_text` gives `start-3d`/`stop-3d` (state.rs:135).
- `STALE_SECS=10`.
- AppImage layout: `usr/bin/{flat3d-app,flat3d-inject,flat3d-run}`, with `FLAT3D_SRC_DIR=$APPDIR/usr`.
- Status filters: `works-now likely build not-safe wont-work untested` (status.rs:5).
- Today's CI already installs iced's libraries for one job (`.github/workflows/*.yml:40`: `pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev`). The other job (:23) does not.

**Depends on:** WP-04, WP-05, WP-06, WP-23, WP-28, WP-29, plus WP-01's CI apt line. Every Rust CI job that builds the workspace must install `pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev libarchive-tools`. `libarchive-tools` provides `bsdtar` for WP-13's `unpack` (roadmap.md:279). If WP-01 left any of these out, WP-01 needs a follow-up before WP-33's CI can go green.
**Owner device:** Q15 `sessions/app-window.toml` (and Q16 `appimage.toml` later).
**Done-check:** `distrobox enter craft-build -- env CARGO_TARGET_DIR=$SCRATCH/target sh -c 'cd the repo root && cargo test --locked -p flat3d-app && cargo clippy --locked -p flat3d-app --all-targets -- -D warnings'`. Do not run it on the host: the app's system libraries are present only in craft-build or CI.

**Flags**
- **Flag:** the scope says tabs "Headset/Games/Setup/Logs", but `lib.rs:17` and the unit test `the_four_tabs_are_in_order` fix Setup, Headset, Games, Logs, and the window opens on Setup. Keep the existing order.
- **Flag:** #123 says a 2 s tick (docs/code-map.md:74 also says 2 s), but the scope says 1 s. 1 s is used here; align #123 and code-map in §6.
- **Flag:** `crates/app/Cargo.toml` needs `serde_json` (workspace) from WP-01, because Cargo.lock is frozen. If WP-01 left it out, that blocks WP-33.
- **Flag:** the page `Msg` types need `Debug, Clone` (owned by WP-04/05/06/23) for `Message: Clone`. `SetupState` also needs `Default, Clone, Debug` for `App`'s derive.
- **Flag:** if WP-05's `setup_page::Msg` has no variants that carry the packages and setup-done results, WP-05 needs a follow-up. WP-33 must not add setup state to `lib.rs` to get around it.
- **Flag:** `setup packages` runs adb from the CLI, which makes it a second adb client next to the service. It is accepted because both talk to the one adb server.

Review note: I could not check WP-05/WP-06's frozen names (`Loaded`, `SetupState::update`) or BONES's `from_env` against the repo. The page modules don't exist yet (app/src has only lib.rs and main.rs), and the older roadmap.md:272 still says `Msg{Lines(..)}`. I applied those corrections as stated, with "the frozen name wins" fallbacks. Verified: craft-build exists, the CI apt line is at workflows:23/40, and WP-13 uses bsdtar (roadmap.md:279).

### WP-34: Docs sync after the pass

**Closes:** the doc remainder of A2-A4 (README and code-map text still describing the old `inject/` crate and its stub commands). A14 in code-map. OD3 (the SPEC viewer line). The STEP0-RUNSHEET generation. Shrinking the private-word allow-list (OD7).

**Files (exclusive):** `docs/code-map.md`, `docs/ROADMAP.md`, `CHANGELOG.md`, `SPEC.md`, `recipes/README.md`, `docs/STEP0-RUNSHEET.md`, `tools/docs/private-allow.txt`, `README.md`.

**Signatures:** no Rust, TS or Python code changes. The doc text must match what these produce:
- `cli::parse_any` → `AnyCommand{Engine,Link,View,Setup,Doctor}`. WP-01 froze it.
- The `pub const` key lists for route settings in `crates/engine/src/recipes.rs`, also frozen by WP-01. Today the file has `KEYS: [&str; 9]` at `inject/src/recipes.rs:24`, plus `STATUSES:9`, `APIS:18`, `FREES:20` and `SAFES:22`.
- `tools/session/session.py render <toml>` (WP-24).
- `tools/docs/check.py` (WP-25).

**Behaviour**
1. **docs/code-map.md: status flip**
   - Every Parts row whose file was merged in WP-01…33 gets exactly `present` in the Status cell, with nothing after it. code-map.md:50 says the locked tests check for that exact text.
   - The locked cells must read verbatim:
     - `` | `crates/engine/src/api.rs` | present | ``
     - `` | `crates/engine/src/link.rs` | present | ``
     - `` | `crates/engine/src/recipes.rs` | present | ``
     
     These come from WP-01's path edits to `api_response.rs:69`, `link_stop.rs:112` and `recipe_load.rs:150`.
   - Only `crates/engine/src/injectors/port.rs` stays `stub` (#27b, not in this pass).
   - Rows whose code is built but has not been tested on the device stay `present`. Put "device check: Qn" in the Owns column, never in Status.
   - Today `inject/src/tether/` (code-map.md:66) and `install.rs` (:68) say "partly present (…)". Both become `present`, at their new paths.
   - Viewer (A14):
     - code-map.md:85 now says "Likely a Windows binary run in the game's Proton session…". Change that row to `` | `crates/viewer/` | present | Linux-native PipeWire→OpenXR quad viewer on WiVRn: `flat3d-viewer` (features `xr`,`capture`); layout/status/spec pure; device check Q10 | M21 | ``.
     - In the mermaid block (code-map.md:42), change the `Viewer[...]` label to `crates/viewer — SBS screen viewer (Linux, PipeWire→OpenXR)`. One statement per line (C19).
   - Mermaid :13 currently reads `inject/src/bin/flat3d-run.rs`. Change it to `crates/engine/src/bin/flat3d-run.rs`.
   - Update the section headers `## API shape (inject/src/api.rs)` (:94) and others to the `crates/engine/src/` paths. Keep the section names "Parts", "API shape", "Interlinks", "Packaging", "State, IPC and files" and "LinkEvent": 51 `//!` headers in the src files cite them.
2. **docs/code-map.md: command table.** Add a `## Commands` table under Parts. It has one row per word `parse_any` accepts. Columns: `| Command | Group | Module | JSON (api.rs type) | WP |`.

   | Group | Commands | Module | WP |
   |---|---|---|---|
   | Engine | `list`, `status`, `inject <appid>`, `undo <appid>` | `cli/engine.rs` (list, status); `cli/inject.rs` (inject, undo) | WP-07; WP-27 |
   | Link | `watch`, `snapshot`, `switch get\|set <appid> [on\|off]`, `setting get\|set <key> [on\|off]`, `install`, `uninstall`, `run …`, `start-3d`, `stop-3d` | `cli/link.rs` | WP-08 |
   | View | `view`, `rows`, `launch`, `collection`, `toasts`, `logs` | `cli/view.rs` | WP-28 |
   | Setup | `setup packages\|install-apps\|launch-env\|fetch` | `cli/setup.rs` (fetch: `cli/setup_fetch.rs`) | WP-29, WP-27 |
   | Doctor | `doctor [--only games]` | `cli/doctor.rs` | WP-29 |

   - The link verbs today come from `parse_link` (cli_link.rs:52-58). The engine verbs today come from cli.rs:65-78.
   - Add the non-`parse_any` binaries in a separate small list:
     - `flat3d-run %command%`;
     - `flat3d-viewer` (args from `ViewerSpec::to_args`);
     - `flat3d-uevr-inject.exe` (`crates/uevr-inject/src/args.rs`).
   - check.py enforces USAGE ⊆ table, one way (K11). USAGE today is cli.rs:50-51.
3. **docs/ROADMAP.md.**
   - Rewrite "Where it stands (2026-11-07)" (:25-35) as "Where it stands (<merge date>)". It holds one bullet per wave (0-5) with the WPs merged, and "device work pending: Q0-Q16". Drop the ":27 commands mostly don't" and ":34 #183" bullets.
   - Replace the "What's next" section (:159-212) with two parts:
     - a **Waves** table (WP-00…34, wave, issue, owner device);
     - the **Owner-device queue**, the bones §5 Q0-Q16 table verbatim, with Driver set to `tools/session/session.py run sessions/<x>.toml`.
   - "Decisions for you" (:213) keeps only:
     - OD1-OD8, each marked defaulted or ruled;
     - REA registration (ruling 22);
     - AER (ruling 3);
     - contacting the SpidyVR author;
     - library drives.
   - Section 6 housekeeping stays out: it goes to `tools/pm/issues_report.py`.
   - Add a "Not in this pass" list: the bones list, as one line per item.
4. **CHANGELOG.md.** The file's rule (:3) is "Newest first. One line per change". Insert above :5 one line per WP, WP-34 first and WP-00 last. Format: `- <YYYY-MM-DD> WP-NN <title> (closes …; PR #n, main <sha7>)`. The sha and PR come from `git log --merges --oneline`. Leave history lines as they are, including their `inject/` paths.
5. **SPEC.md (OD3).** Replace only line 77 with:
   `- Geo-11 games need an SBS screen viewer that puts Geo-11's side-by-side picture on the fixed screen: a Linux-native PipeWire→OpenXR quad viewer on WiVRn (\`crates/viewer\`, \`flat3d-viewer\`), capturing the gamescope node.`
   - Do not touch M21 (SPEC:41), which already says "through an SBS screen viewer".
   - Do not touch the approval line or any other line.
6. **recipes/README.md.** Between the markers WP-25 defines (expected: `<!-- BEGIN GENERATED: route-keys -->` … `<!-- END GENERATED: route-keys -->`), write a table: `| Route | Key | Allowed values |`. Each row comes from one of `recipes.rs`'s frozen route key-list consts.
   - Generate it with the generator WP-25 shipped (expected `python3 tools/docs/check.py --write recipe-keys`).
   - Keep the strings `<appid>-<slug>.recipe` and `key: value`, which `recipe_load.rs` reads.
   - Keep the C2/C17 text that WP-25 wrote.
7. **docs/STEP0-RUNSHEET.md.** Run `python3 tools/session/session.py render sessions/step0.toml > docs/STEP0-RUNSHEET.md`.
   - The first line must be `<!-- generated; edit sessions/step0.toml -->`.
   - The path stays, because SPEC M1 (SPEC.md:21) and README.md:69 link it.
8. **tools/docs/private-allow.txt.** Exactly one non-comment line: `docs/notes/DECISIONS.md`.
   - `crates/*/tests/**` stays exempt through check.py's built-in `tests/` exemption (T2), not through this list.
   - Before shrinking, `git grep -nE '\bI[a]n\b|Hyperion|B:/|/home/[a-z]+|qwen|FF8S5|sandisk|MDT'` outside `tests/` and DECISIONS must print nothing.
9. **README.md.**
   - Rewrite the layout table so its rows are the real top level: `crates/{engine,app,viewer,uevr-inject}`, `decky/`, `packaging/`, `sessions/`, `tools/`, `games/`, `recipes/`, `docs/`, `.github/`, `cmake/`, `third_party/`.
   - Build/test section:
     - `tools/check.sh`;
     - `cargo test --workspace --locked`;
     - `cargo test -p flat3d-app`;
     - viewer device build `cargo build -p flat3d-viewer --features xr,capture` inside the distrobox build container;
     - `tools/build-win.sh`.
   - Drop every mention of `inject/` and `app/` as top-level folders (A2-A4 remainder).

**Tests:** no new test files.
- Locked tests honoured: `crates/engine/tests/api_response.rs`, `link_stop.rs` and `recipe_load.rs` (code-map `present` cells; recipes/README strings).
- Gates: `tools/docs/test_check.py` (WP-25) must still pass, and `tools/docs/check.py` must run clean on the real tree.

**Inline facts**
- SPEC.md:15: "A Must or a Won't changed after approval sends the line above back to 'not yet'." Line 77 is under "Implementation decisions", so editing it does not reset approval.
- OD3 text: "Linux-native PipeWire→OpenXR quad viewer on WiVRn".
- Viewer features: `xr=["dep:openxr","dep:ash"]`, `capture=["dep:pipewire","dep:ashpd","dep:tokio"]`; bin `flat3d-viewer` has `required-features=["xr","capture"]`.
- Pins: openxr 0.22.0, ash 0.38.0, pipewire 0.10.1, ashpd 0.13.13.
- `ERROR_CODES`: 8 locked codes (api_response.rs:11). Do not list counts in docs (A18).
- Q table and OD table: the bones §5 and §7.

**Depends on:** WP-00 to WP-33, all merged.

**Owner device:** none.

**Done-check:**
```
export CARGO_TARGET_DIR=$SCRATCH/target
tools/check.sh && python3 tools/docs/check.py \
&& python3 tools/session/session.py render sessions/step0.toml | diff - docs/STEP0-RUNSHEET.md \
&& cargo test --locked -p flat3d-inject --test api_response --test link_stop --test recipe_load \
&& [ "$(grep -v '^#' tools/docs/private-allow.txt | grep -c .)" = 1 ] \
&& git diff --exit-code main -- SPEC.md | grep -c '^[-+][^-+]' | grep -qx 2
```

**Flags:**
- **OD3 names the wrong section.** OD3 says "Must section", but SPEC.md:77 is an Implementation-decisions line. The Must for the viewer is M21 at SPEC.md:41, and it needs no change.
- **WP-25 must ship two things this WP uses.** It owns `check.py`, so it must ship (a) the route-keys markers and a `--write` (or similar) generator entry, and (b) the exact column format of the `## Commands` table that check.py parses. If it ships neither, WP-34 hand-writes the block. check.py then has to compare that block against `recipes.rs`.
- **Who removes the old paths from code-map.** code-map.md:42, :66 and :85 still name `inject/` and `viewer/`. WP-01 does the path rename of code-map rows, but if any `inject/`/`app/` row is left, WP-34 removes it. check.py's rule "`present`/`stub` ⇒ path exists" catches a leftover row.
- **`session.py render` output.** It is assumed to print to stdout. If WP-24 writes the file through `-o`, use `render sessions/step0.toml -o docs/STEP0-RUNSHEET.md`.
