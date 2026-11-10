<!-- split from roadmap-v2.md lines 138-476 at c0e6874 -->
## 2. New bones

### Tree

**Tree** (`*` new; `→` git mv). Facade-file form only: a module with children is `x.rs` + `x/`. No `mod.rs` anywhere, so there is no E0761. `mod_module_files` is off by default, so WP-01 turns it on as `deny` in the workspace lints below; `module_inception` stays clean too.
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
  tests/{paths,fsx,catalog_find,switches_io,state_io,route_settings,cli_any,architecture}.rs* tests/fixtures/*
crates/app/ (pkg flat3d-app) ← app/   src/{lib,main,engine*,headset_page*,games_page*,setup_page*,logs_page*,view_{headset,games,setup,logs}*}.rs tests/*
crates/viewer/* (pkg flat3d-viewer) src/{lib,frame,spec,layout,status,source,main}.rs source/{test_pattern,pipewire,gamescope,portal}.rs gpu.rs gpu/import.rs xr.rs xr/{session,swapchain,frame}.rs tests/spec.rs
crates/uevr-inject/* (pkg flat3d-uevr-inject, lib flat3d_uevr_inject) src/{lib,main,args,pick,win}.rs
decky/{main.py,plugin.json,tests/*} packaging/{pins*,pin.sh*,build-*.sh*,AppRun*,flat3d.desktop*,flat3d-link.service*,tests/*} sessions/*.toml* games/spiderman2/map.toml*
```

### Root Cargo.toml

**Root Cargo.toml:** `[workspace] resolver="2" members=["crates/*"] default-members=["crates/engine","crates/viewer","crates/uevr-inject"]`. Then:
- `[workspace.package] edition="2021" license="GPL-3.0-only"`;
- `[workspace.dependencies]` serde(derive), serde_json, `iced={version="0.14.0",features=["smol"]}`, `flat3d-inject={path="crates/engine"}`, `flat3d-viewer={path="crates/viewer",default-features=false}`, `windows-sys="=0.61.2"`;
- `[workspace.lints.clippy] field_reassign_with_default="allow"`, `mod_module_files="deny"`; each crate sets `[lints] workspace=true`.

Viewer:
- `[features] default=[]`, `xr=["dep:openxr","dep:ash"]`, `capture=["dep:pipewire","dep:ashpd","dep:tokio"]`.
- `openxr={version="=0.22.0",features=["loaded"]}`, `ash="=0.38.0"`, `pipewire="=0.10.1"`, `ashpd={version="=0.13.13",default-features=false,features=["screencast","tokio"]}`, tokio (features `rt`, `time`; optional), libc (not optional).
- `[[bin]] name="flat3d-viewer" required-features=["xr","capture"]`.

uevr-inject:
- windows-sys goes under `[target.'cfg(windows)'.dependencies]` with the features `Win32_System_Diagnostics_ToolHelp`, `Win32_System_Memory`, `Win32_System_Threading`, `Win32_System_LibraryLoader`, `Win32_Foundation`, `Win32_System_Diagnostics_Debug` and `Win32_Security`. `Win32_Security` is there because `CreateRemoteThread` is gated on it and takes `*const SECURITY_ATTRIBUTES`.
- `src/lib.rs` (WP-01) is `pub mod args; pub mod pick; #[cfg(windows)] pub mod win;`.
- WP-01 stubs are an empty `args.rs`/`pick.rs`, an empty `win.rs`, and a `main.rs` that exits 3 on non-Windows. WP-15 overwrites all four bodies.
- The final exit codes are WP-15's: 2 bad args, 3 not Windows, 4 no game found, 5 inject failure.

### git-mv map

**git-mv map** (`<!-- mv-map -->` block, the one check.py exemption):
- `inject/**`→`crates/engine/**` and `app/**`→`crates/app/**`;
- `cli_link.rs`→`cli/link.rs`;
- `{collection,headset_view,link_view,toasts,view}.rs`→`face/`;
- `tether/mod.rs`→`tether.rs`;
- `watch.rs`→`watch/service.rs`, then frames/machine/files/run are split out; the new `watch.rs` is facade-only.

`cli/undo.rs` is new. Today's working undo dispatch body moves into it, so it is not a git mv.

Untracked `inject/Cargo.lock`, `app/Cargo.lock` and per-crate `target/` are deleted.

### Facade re-exports

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

### Glob-import guard

**Glob-import guard:** locked tests glob `install::*` (install_plan.rs:3), `api::*` (api_snapshot.rs:3) and `tether::relay::*` (tether_tunnel.rs:3, tether_relay_cmds.rs:3, tether_tunnel_review.rs:3).
- No new pub item in `api`, `install` or `tether::relay` may be named `json`, `c`, `p`, `H` or `full`. These are the local or explicit names those tests use: install_plan.rs:5-11, api_snapshot.rs:4,6.
- No new glob re-export goes into those three modules. The existing items stay as they are.

### Exec trait split

**Exec trait split (`exec.rs`).** Two distinct traits stay re-exported under their locked names:
- Link shape: `CommandRunner { fn run(&mut self, argv:&[String]) -> CommandOutput }`, plus `CommandOutput` and `SystemRunner`. Re-exported as `link::{CommandOutput, CommandRunner}` (tether_adb_runner.rs:3).
- Relay shape: `RelayRunner { fn run(&mut self,&[String])->Result<String,String>; fn start_relay(&mut self,&[String],&str)->Result<(),String>; fn stop_relay(&mut self)->Result<(),String> }` (verbatim from adb.rs:64-68), plus `ProcessRunner`. Re-exported as `tether::adb::CommandRunner`. `tether_runtime.rs:2` imports it under that name; the file is unheadered, and the import is kept for compile, not because the file is locked.
- `AdbRunner<R: link::CommandRunner>` keeps its bound and constructor (adb.rs:171-191) and still implements `link::CommandRunner`. The alias in `tether::adb` does not touch `AdbRunner`. `TetherAdb<R: RelayRunner>` (adb.rs:134) is unchanged too.
- No blanket impl. `testing::FakeRunner` implements both traits; its API is frozen below.
- Every new fn names its trait: `&mut dyn CommandRunner` (link/wivrn/doctor/fetch/setup/injectors) or `&mut dyn RelayRunner` (tether/health).
- WP-01's compile check includes `cargo test -p flat3d-inject --test tether_adb_runner --test tether_runtime`.
- Only `exec.rs` spawns. WP-01's `crates/engine/tests/architecture.rs` fails on `Command::new` anywhere else in `crates/engine/src`, except a tagged allowlist. Each allowlist entry names a file and a tag. Today's sites are in four files: link.rs, watch.rs (two), tether/adb.rs and bin/flat3d-run.rs. The link.rs and tether/adb.rs sites are `SystemRunner` and `ProcessRunner`, which WP-01 moves into `exec.rs`. So the list it lands holds the watch sites after the split (tagged `follow-up`; no WP in this pass moves them), bin/flat3d-run.rs (WP-31) and cli/link.rs (WP-08's `exec_launch`). The last two exec the game in place of their own process, so they are permanent entries, not runner calls. A repo-wide rule with no allowlist would fail on day one.

### Legacy `*_with` rule

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

### Bones WP-01 freezes

**Bones WP-01 freezes.** Stub bodies return `ApiError{code:"not-built"}`/`Err`/neutral values. Parameters take a `_` prefix. No `todo!()` or panics.
- Every not-built stub names the WP that fills it: `NotBuilt("<path> (WP-xx)")`. `tests/architecture.rs` checks that each tag names a real WP.
- `cli::parse_any`→`AnyCli{command:AnyCommand{Engine,Link,View,Setup,Doctor},json}` + dispatch, `AnyCommand::id`, the `cli::COMMANDS` registry and the `cli::ok_exit` hook, all full (shapes under Frozen signatures, `cli.rs`).
- `api` DTOs (fields under Frozen signatures), `api::error_help` (full), `TOAST_TITLE="Flat to 3D"` and the `GameRow` re-export; Serialize on `HealthView` (face/view.rs) and `ToastBatch` (face/toasts.rs).
- `state::{read_snapshot,write_request}` and `switches`, full. `catalog::find_recipe`, full: a dir that does not exist is skipped, and the result is `Ok(None)` when no dir has a match.
- `recipes::{UEVR_KEYS,GEO11_KEYS,PORT_KEYS,RouteSettings,route_settings,check_route_settings}`, full (rules below).
- `paths.rs`, full.
- `link::wivrn::WivrnDirs::user_default`, full.
- `testing::FakeRunner`, full.
- `viewer::spec` (ViewerSpec, `to_args`/`from_args`), full.
- `viewer::frame`, full; `viewer::source::FrameSource`, full (re-exports `frame::Frame`).

### Frozen signatures

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
    | `pins()` | `data/pins` (kept, but unused in this pass: no WP writes or reads that file; pins come from the compiled `fetch::PINS`, WP-13) |
    | `run_paths()` | `->RunPaths` |
    | `watch_paths()` | `->watch::Paths` |

    `log_dir` is `home/.local/state/flat3d` (watch.rs:323, flat3d-run.rs:34).
  - Locked `state::*_path(&str)` callers pass `p.state_dir.to_string_lossy()`.
  - Each accessor has a case in `crates/engine/tests/paths.rs`.
- **`fsx.rs` / `sys.rs`** (full): `fsx::{write_atomic(&Path,&[u8])->io::Result<()>, append_line, read_opt(&Path)->io::Result<Option<String>>}`, `sys::{current_uid()->u32, unix_now()->u64}`.
- **`link/wivrn.rs`** (WP-12 body; `crate::wivrn` is the same module):
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FlatpakScope{User,System}` + `pub fn flag(self)->&'static str` (`--user`/`--system`);
  - `pub const SERVER_FLAGS:[&str;2]=["--early-active-runtime","--no-publish-service"]` and `pub const QUEST_APPS:[&str;5]`;
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct WivrnDirs{pub scope:FlatpakScope, pub app_dir:PathBuf, pub deploy:PathBuf, pub version:Option<String>}` + `pub fn user_default(home:&Path)->WivrnDirs`. It is full in WP-01 (part WP-01d): scope User, app_dir `home/.local/share/flatpak/app/io.github.wivrn.wivrn`, deploy `app_dir/current/active`, version None;
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
  - Known interim, not a bug to fix early: `publish()` passes the raw runner to `tick`. WP-09 (wave 4) swaps in its adb adapter in `watch/service.rs`. So in waves 1 to 3 the tick's probes run a bare `adb` with no path and no serial. WP-11 lists this under `## Not verified` and does not change `publish()`; no other WP touches it before WP-09.
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
- **`recipes.rs`** (full in WP-01, part WP-01b):
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

    The stub body is `Err(NotBuilt("cli::<m>::run (WP-xx)").into())`, the same form as today's dispatch fallback (cli.rs:147) plus the WP tag. `cli/undo.rs` gets today's working undo body.
  - The only other pub items under `cli/`:
    - `cli::link::request_3d(r:state::Request, p:&EnginePaths, now:u64)->Result<api::LinkReport,ApiError>`. Dispatch sends `Command::Start3d/Stop3d` here with `sys::unix_now()` and serialises the result with `serde_json::to_value`;
    - `cli::view::parse(args:&[String])->Option<Result<ViewCommand,String>>`. None means "not a view verb". The stub accepts `view` and `logs` (`logs` gives `Logs{limit: DEFAULT_LOG_LINES}`, as `cli_any.rs` expects), and WP-28 fills the rest of the grammar;
    - `cli::setup_fetch::setup_fetch(key:&str, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::SetupReport,ApiError>` (WP-27 body; WP-29's `setup fetch` calls it);
    - `parse_link`, `LinkCommand` and `SettingKey` stay pub, because they are locked.
  - **Command registry** (in `cli.rs`, full in WP-01, part WP-01c):
    - `pub struct CommandSpec{pub id:&'static str, pub group:&'static str, pub args:&'static str, pub output:&'static str, pub effects:&'static [SideEffect]}` and `pub static COMMANDS:&[CommandSpec]`, one row per command, stubbed ones included.
    - `id` is the command words as typed (`list`, `inject`, `setup fetch`, `doctor`); `group` is the `AnyCommand` family; `args` is the argument form after the id (`<appid>`); `output` is the `api` DTO name in `data`.
    - `pub enum SideEffect{ReadOnly, Probes, WritesFiles, RunsCommands}`. The name is not `Effect`, which `watch` already uses. `ReadOnly` means the runner ran nothing and the file tree is byte-identical; `Probes` means only read-only probe argv ran.
    - `impl AnyCommand { pub fn id(&self)->&'static str }`. For `Engine(c)` it returns the locked `c.name()` (cli.rs:30).
    - `tests/cli_any.rs` checks the registry: every row's `id`, with a sample value for each `args` placeholder, parses through `parse_any`; every `AnyCommand` variant's `id()` has exactly one row; and every command word in the locked usage strings has a row. The `view` rows other than `view` itself join the parse check when WP-28 fills the view grammar.
    - The other faces are checked against the registry by their own WPs: the Decky wrappers in WP-28's `decky_face.rs`, the app's effect args in WP-33's `engine::effect_args` test, and the code-map `## Commands` table in WP-34.
    - A command WP that fills a body keeps its row's `effects` true and adds a test for it.
  - **Exit hook:** `pub fn ok_exit(cmd:&AnyCommand, data:&serde_json::Value)->i32`, full in WP-01 (part WP-01c). Dispatch calls the locked `render` (cli.rs:102) unchanged, then, on `Ok`, sets `Output.code = ok_exit(&cmd, &data)`. It returns 1 for `Doctor(DoctorCommand::Run)` when `data["ok"]==false` (a red report), else 0. stdout still carries the full report. Errors keep the locked `exit_code`.
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
  - `pub fn error_help(code:&str)->(bool, &'static str)` returns `(retryable, fix)` for each of the 8 codes, full in WP-01 (part WP-01c). Where one command fixes the error, `fix` names it. A test checks that every `ERROR_CODES` entry has an entry. The `ApiError` JSON stays `{code,message}`.
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

### Wave-owned pub APIs

**Wave-owned pub APIs.** The owning WP adds these inside its own file after wave 1. Later WPs call exactly these forms.
- WP-10 (`tether.rs`, `tether/plan.rs`, `tether/wifi.rs`):
  - `run_plan(steps:&[TetherStep], r:&mut dyn RelayRunner, sleep:&mut dyn FnMut(u64))->Result<(),String>`;
  - `run_plan_in(steps, ctx:&PlanCtx, r, sleep, out:&mut PlanRun)->Result<(),String>`, with `PlanCtx{pub relay:Vec<String>}` and `PlanRun{pub ran:Vec<Vec<String>>, pub tun0:bool}`. `PlanCtx` has no `adb` field (questions.md 24); WP-10 puts a doc comment on `run_plan_in` instead: "Call it through `Tether`. A bare `ProcessRunner` would start the relay with `ADB` empty.";
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

    WP-27 uses this mapping; inside `ensure_applied` (String errors) it writes `format!("{}: {}", a.code, a.message)` with `a = ApiError::from(e)`, so the String still starts with the code (WP-27).
- WP-14 (`injectors/uevr.rs`): `apply_with(game:&steam::Game, settings:&RouteSettings, p:&EnginePaths)->Result<(uevr::Applied,RouteEnv),String>`, `profile_text_full(&UevrSettings,&UevrExtras)->String` and `uevr_dir(p)->PathBuf` (`p.uevr_cache()/UEVR_VERSION`).
- WP-16 (`injectors/geo11.rs`):
  - `apply(cache_dir:&Path, game_dir:&Path, appid:u32, m:&RouteSettings, viewer_exe:&Path)->Result<(geo11::Applied,RouteEnv),String>`. Its `RouteEnv.sidecar` comes from `ViewerSpec::to_args`;
  - `viewer_check(viewer_exe:&Path)->Result<(),ApiError>`, which returns code `not-found` unless the path is an executable file;
  - callers pass `p.viewer_bin()`.
- WP-20 (`launch_options.rs`): `add_wrapper_at(options:&str, wrapper_abs:&str)->String`, called with `p.wrapper_bin()`.

### ERROR_CODES

`ERROR_CODES` stays the 8 locked codes (api_response.rs:10: `usage not-built not-found bad-recipe refused link-not-ready command-failed io`). The set is closed for this pass: no WP adds a code. A new failure maps to an existing code with a specific message.

### Exit contract

**Exit contract** (stated once, here): `usage` and `not-built` exit 2, every other error exits 1, and success exits 0 unless `cli::ok_exit` says otherwise (a red doctor report exits 1). With `--json`, stdout is one envelope line. The locked `cli::exit_code` (cli.rs:93) and the `cli_json` tests enforce the error exits; a WP-01 test checks `ok_exit`. Tool exit codes (pin.sh, `flat3d-uevr-inject`, the packaging scripts) stay per tool; there is no repo-wide exit table, because those tools already give the same numbers different meanings.

### PR and issue rules

**PR and issue rules:**
- The PR body (`## Verified` / `## Not verified`) and merge rules live in §0 "PR body" (ruling D1).
- A WP that has a device Q row writes `Refs #N (code half)`, never `Closes #N`. The issue is closed when its device session passes.
- A WP with no device Q row writes `Refs #N` when another WP still owes work on the issue (WP-06, WP-07), and `Closes #N` otherwise (WP-03, WP-04, WP-05, WP-23).

### CI final shape

**CI final shape (`check.yml`, WP-01).** Every job uses `runs-on: [self-hosted, linux]` (check.yml:18,35) and `dtolnay/rust-toolchain@1.95.0` (host rustc 1.95.0). WP-22's `appimage.yml` uses the same runner and pin. Neither file uses `ubuntu-24.04`, `@stable` or `@master`.
- `tools/check.sh` (WP-02) is the only list of CI steps. Each job does its setup, then runs `tools/check.sh --ci --only <job>` (`rust`, `py` or `decky`). `--ci` turns every skip (a tool or target missing on this machine) into a failure. A step whose input file has not landed yet (the file guards below) prints `n/a`, which `--ci` leaves alone.
- The inline steps below are the wave-1 fallback only. Each sits behind `if: hashFiles('tools/check.sh')==''`, because WP-01 and WP-02 share wave 1; a step's own file guard is added to that one. check.sh runs each of these steps with the same flags and file guards, so the fallback cannot drift from it. check.sh may add steps of its own (WP-02).
- Setup, never skipped: the apt line, the toolchain pin and `rustup target add x86_64-pc-windows-msvc` (rust); setup-node 22.23.4 and pnpm 10.34.9 (decky). The setup steps carry no `if:`.
- Actions, as WP-01 writes them: every job starts with `actions/checkout@v4`. The `py` job keeps `actions/setup-python@v5` with `python-version: "3.12"`. The `decky` job uses `actions/setup-node@v4` with `node-version: 22.23.4` and `pnpm/action-setup@v4` with `version: 10.34.9`.
- Job `rust`:
  - apt via the runner's `SUDO=$(command -v sudo || true)` pattern (check.yml:23): `build-essential curl ca-certificates pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev libpipewire-0.3-dev libspa-0.2-dev libclang-dev clang shellcheck libarchive-tools`. `libarchive-tools` provides bsdtar for WP-13's zip unpack;
  - `cargo fmt --all --check`;
  - `cargo clippy --locked --workspace --all-targets -- -D warnings`;
  - `cargo clippy --locked -p flat3d-viewer --all-targets --features xr,capture -- -D warnings`;
  - `cargo test --locked --workspace` with `XDG_RUNTIME_DIR=$(mktemp -d)`;
  - `cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings` (no SDK, no link).
- Job `py`:
  - `python3 -W error -m py_compile $(git ls-files '*.py')`;
  - the per-folder loop `for d in $(git ls-files '*test_*.py'|xargs -n1 dirname|sort -u|grep -vx 'decky/tests'); do python3 -m unittest discover -s "$d" -p 'test_*.py'||exit 1; done`;
  - step `if: hashFiles('decky/tests/test_*.py')!=''` → `cd decky && python3 -m unittest discover -s tests -p 'test_*.py'`. The cwd puts `decky/` on sys.path for the locked `import main`, with no `__init__.py` and no edit to locked files;
  - `shellcheck $(git ls-files '*.sh')` runs only inside check.sh. Its guard was check.sh's own existence, so it has no inline fallback;
  - step `if: hashFiles('docs/workbench.md')!=''` → `python3 tools/docs/check.py`. The guard is WP-25b's new file, not check.py: check.py lands in wave 1 (WP-25a), but the repo passes a full run only after WP-25b. `tools/check.sh` uses the same guard (`[ -f docs/workbench.md ]`).
- Job `decky`: setup-node 22.23.4 and pnpm 10.34.9 (setup, no `if:`). Every step after the setup steps is guarded with `if: hashFiles('decky/package.json')!=''`, never at job level. Steps: `pnpm i --frozen-lockfile && pnpm run typecheck && pnpm run build`, run with `working-directory: decky`. CI installs without `--offline`; on a builder's machine check.sh's `decky` step adds `--offline` (WP-02, WP-21).

### One owner per fact

**One owner per fact:**

| Fact | Owner |
|---|---|
| part status (`present`/`stub`/`planned`) | `docs/code-map.md` (check.py: `present`/`stub` ⇒ path exists; no `inject/` path tokens) |
| JSON shapes, error codes | `api.rs` |
| command names, args, effects | `cli::COMMANDS` (usage text, Decky wrappers, app effect args and the code-map Commands table are checked against it) |
| cross-WP signatures | §2 "Frozen signatures" (WP-01) and "Wave-owned pub APIs" |
| issue state, waves | ROADMAP.md + `tools/pm/issues_report.py` |
| rulings | `docs/notes/DECISIONS.md` (untouched) |
| URLs, versions, hashes | `packaging/pins` (`<key>.<field>: <value>`) |
| SM2 addresses | `games/spiderman2/map.toml` (WP-30; doc block generated) |
| route keys | `recipes.rs` key lists (recipes/README generated block) |
| device procedures | `sessions/*.toml` (STEP0-RUNSHEET generated) |
| test counts | CI only |

A generated block sits between the BEGIN/END GENERATED markers that WP-25b defines. A stale block fails `tools/check.sh`. Two steps share the work: check.py's `generated` rule (step `docs`) checks the marker pairs, the route-keys block and the runsheet, and prints the regenerate command; `fill_map.py --check` (step `fill-map`, WP-30) checks the address-map block. check.py does not run `fill_map.py`.

### Final doc set

**Final doc set:**
- README.md, SPEC.md, AGENTS.md, PIPELINE.md, CHANGELOG.md;
- docs/{ROADMAP,code-map,STEP0-RUNSHEET(generated),workbench}.md, docs/notes/DECISIONS.md;
- docs/spiderman2/{plan,spidy-core-map,address-map}.md (← PORT-SPIDERMAN2, spidy-stereo-core-map, spiderman2-address-map);
- docs/archive/uncharted3-stereo.md;
- recipes/README.md, packaging/README.md, sessions/README.md.
