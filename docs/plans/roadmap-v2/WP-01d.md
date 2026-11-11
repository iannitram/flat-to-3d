<!-- split from WP-01.md (roadmap-v2.md lines 755-1385 at c0e6874), part d of four -->
### WP-01d: Frozen stubs later WPs fill, and the wiring to them
> Note (2026-11-10, ruling 42): the tether parts of this WP are removed by WP-35..37; this file stays as history.

**Closes:** A5 (the bug, new issue), K4, K5, K6, K8. It writes every remaining frozen signature as a compiling stub (wivrn, health_loop, injectors, wrapper, install, launch_options, face, the app pages) with its stub tag (rulings 2026-11-09; Craft review A10), makes `wrapper::run_with` real (the A5 fix), and wires the watch service and `flat3d-run` to the new forms. Most bodies stay stubs, and later WPs fill them in. It is the last WP-01 part: after it, nothing in §0 "Frozen after wave 1" changes except through its named WP.

**One of four parts.** WP-01 is built as WP-01a → WP-01b → WP-01c → WP-01d, in that order inside wave 1, one agent and one branch per part. Each part leaves the tree compiling and passes its own done-check. "Depends on WP-01" in another file means all four parts merged. Every frozen form below is copied from bones §2 ("Frozen signatures" and "Bones WP-01 freezes"). If this file and bones §2 disagree, bones wins. In this file, "Craft review Ax" means that review's item, not the audit ID of the same name. The `questions.md` entries for all four parts sit under its `## WP-01` heading.

**Depends on:** WP-01c. **Wave:** 1 (last of the four WP-01 parts). **Owner device:** none (host). **Network:** none. `Cargo.lock` is frozen: build with `--locked`, and never run `cargo fetch`, `cargo update` or `cargo generate-lockfile`.

**Do not touch:** anything outside the Files list below. In particular `tools/**`, `packaging/**`, `decky/**`, `.gitignore`, `recipes/**`, every doc except `docs/code-map.md`, and the 52 moved test files that get no sanctioned edit (they move byte-identical). No GitHub writes. No `cargo update` and no `cargo generate-lockfile`.

**Order of work** (each step ends with a build that compiles; commit after each):
1. Write the engine stubs under "Frozen signatures" (wivrn, health_loop, injectors, wrapper, install, launch_options, face) with their `mod` and re-export lines.
2. Make the "Behaviour wired now" changes (watch service, `flat3d-run`).
3. Write the app page stubs and the 9 `mod` lines.
4. Add the stub rows to `docs/code-map.md`.
5. Run the done-check.

**Files (exclusive):** `crates/engine/src/{lib,face,link,watch,injectors,wrapper,install,launch_options,api}.rs`, `crates/engine/src/face/{rows,link_model,logs}.rs`, `crates/engine/src/link/wivrn.rs`, `crates/engine/src/watch/**`, `crates/engine/src/injectors/{geo11,port,fetch}.rs`, `crates/engine/src/bin/flat3d-run.rs`, `crates/app/src/{lib,engine,headset_page,games_page,setup_page,logs_page,view_headset,view_games,view_setup,view_logs}.rs`, `docs/code-map.md`. In detail:
- **New:**
  - Engine `crates/engine/src/`: `face/{rows,link_model,logs}.rs`; `link/wivrn.rs`; `watch/health_loop.rs`; `injectors/{geo11,port,fetch}.rs`.
  - App `crates/app/src/`: `engine.rs`, `headset_page.rs`, `games_page.rs`, `setup_page.rs`, `logs_page.rs`, `view_headset.rs`, `view_games.rs`, `view_setup.rs`, `view_logs.rs`.
- **Edited:**
  - Engine `src/`: `lib.rs` (two re-export changes), `face.rs` and `watch.rs` (facade lines), `watch/service.rs` (or whichever `watch/` file holds `Service`, `publish()` and `current()`), `link.rs` (facade lines plus the `start_3d_commands_with` stub), `injectors.rs` (mod lines, `RouteEnv`, `ensure_applied`), `wrapper.rs`, `bin/flat3d-run.rs`, `install.rs`, `launch_options.rs`, `api.rs` (one re-export line).
  - `crates/app/src/lib.rs` (the 9 mod lines).
  - `docs/code-map.md` (stub rows only).
- **Not edited:** `Cargo.*`, `check.yml`, `cli.rs`, any moved test file. **There is no `mod.rs` anywhere and no `x/x.rs`.**
- **Ownership after this WP:**
  - `exec.rs`: WP-01 owns it because it holds the trait split. WP-36 later folds the relay methods out of it (WP-10 is dropped, ruling 42).
  - `link.rs`: WP-12 owns the bodies, and the WP-01 facade lines stay.
  - `crates/app/src/lib.rs`: WP-33 owns it from wave 4.
  - Everything else in §0 "Frozen after wave 1" (`00-run-the-pass.md`) is frozen.

---

#### Module facades (explicit re-exports, never `*`)

Engine `lib.rs`: add `rows` to the `face` re-export and add the `wivrn` line, so the re-export block reads in its final form:
```rust
  pub mod face; pub mod paths; pub mod exec; pub mod catalog; pub mod fsx; pub mod sys;
  #[doc(hidden)] pub mod testing;
  pub use cli::link as cli_link;
  pub use face::{collection, headset_view, link_view, rows, toasts, view};
  pub use link::wivrn;
```
  - `pub use link::wivrn;` makes `crate::wivrn` and `crate::link::wivrn` the same module.

`face.rs`: `pub mod collection; pub mod headset_view; pub mod link_view; pub mod toasts; pub mod view; pub mod rows; pub mod link_model; pub mod logs;`

`watch.rs` (facade only): add `mod health_loop;` and the last line, so the final facade is:
```rust
mod frames; mod machine; mod files; mod service; mod run; mod health_loop;
pub use frames::{parse_track_frames, Device};
pub use machine::{step, link_commands, note_text, note_kind, note_event, Effect, Event, Inputs, Note, Phase, Watch, GRACE_SECS};
pub use files::{write_snapshot, Paths};
pub use service::{Feed, Service, TrackDevices};
pub use run::run;
pub use health_loop::{HealthLoop, HealthTick};
```
- The re-export list is exactly this one.

`link.rs`, at the top:
```rust
pub mod wivrn;
pub use crate::exec::{CommandOutput, CommandRunner, SystemRunner};
pub use wivrn::{FlatpakScope, QuestApp, WivrnDirs, WivrnStart, QUEST_APPS, SERVER_FLAGS, detect_quest_app, detect_server};
```
- `WIVRN_APP` (link.rs:10) stays where it is.
- Also add `pub fn start_3d_commands_with(_s: &WivrnStart) -> Vec<Vec<String>>`. Its stub body is `start_3d_commands()`, and WP-12 writes the real one.

`injectors.rs`: `pub mod uevr; pub mod geo11; pub mod port; pub mod fetch;`. The 3 new files are empty stubs.

App `crates/app/src/lib.rs`: add these lines above the existing `Tab` code:
```rust
pub mod engine; pub mod headset_page; pub mod games_page; pub mod setup_page; pub mod logs_page;
pub mod view_headset; pub mod view_games; pub mod view_setup; pub mod view_logs;
```

`api.rs`: add `pub use crate::face::rows::GameRow;`. The re-export is a named one, which the glob guard allows. Nothing else in `api.rs` changes.

**Glob-import guard:** no new pub item in `api`, `install` or `tether::relay` may be named `json`, `c`, `p`, `H` or `full`, and none of those three modules gets a glob re-export. Locked tests glob-import them (install_plan.rs:3, api_snapshot.rs:3, tether_tunnel.rs:3, tether_relay_cmds.rs:3, tether_tunnel_review.rs:3). (Repeated from WP-01c.)

---

#### Frozen signatures (stubs compile)

Stub rules (repeated from WP-01c, which wrote the `cli::…` stubs; this part writes the rest):
- A stub that returns `Result` gives `Err(NotBuilt("<path> (WP-xx)").into())` / `ApiError{code:"not-built"}` or `Err("not-built: <fn> (WP-xx)".into())`.
- **Every not-built stub names the WP that fills it** (Craft review A10). The tags are:
  - `cli::engine::run (WP-07)`, `cli::inject::run (WP-27)`, `cli::link::run (WP-08)`, `cli::link::request_3d (WP-08)`, `cli::view::run (WP-28)`, `cli::setup::run (WP-29)`, `cli::doctor::run (WP-29)`, `cli::setup_fetch::setup_fetch (WP-27)`;
  - `install::execute (WP-20)`, `install::execute_uninstall (WP-20)`, `injectors::ensure_applied (WP-27)`, `route (WP-31)`;
  - today's `NotBuilt("injectors::inject")` (injectors.rs:61) is retagged `injectors::inject (WP-27)` (WP-01b already did this);
  - any other stub that returns not-built takes the tag of the WP this file names as its body owner.
- `tests/architecture.rs` checks that every tag names a real WP (see Tests).
- Any other stub returns a neutral value, or delegates to the legacy fn as stated.
- Parameters take a `_` prefix. There is no `todo!()`, `unimplemented!()` or panic.

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
- `QUEST_APPS`, in this order: `"org.meumeu.wivrn.github"`, `"org.meumeu.wivrn"`, `"org.meumeu.wivrn.github.nighly"` (sic, the upstream spelling), `"org.meumeu.wivrn.github.testing"`, `"org.meumeu.wivrn.local"`.
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

**`face`:**
- `rows::GameRow` has the fields and derives exactly as in issue #76: `#[derive(Debug, Clone, PartialEq, Eq, Serialize)] pub struct GameRow { pub appid: u32, pub name: String, pub status: String, pub status_label: String, pub status_colour: String, pub switch_on: bool, pub plays_3d: bool, pub flat_reason: Option<String> }`. `rows::game_rows(games: &[api::Game], filter: &str, settings: &switches::Settings, link: LinkState) -> Vec<GameRow>` is a stub returning `Vec::new()`; WP-03 writes the body.
- `link_model::link_view(snap: Option<LinkSnapshot>, now: u64) -> api::LinkView` is a stub that ignores both parameters and returns a Stopped view; WP-28 writes the body. The stub's fields: `state: LinkState::Stopped`, `label: link_label(Stopped)`, `hint: link_hint(Stopped)`, `start`/`stop` from `headset_buttons(Stopped)`, `health: health_view(None)`, `rx` and `tx`: `throughput_text(0)`, `uptime: uptime_text(0)`, `tether: TetherMode::Off`, `tunnel: false`, `auto_3d: true`, `adb_source: AdbSource::Missing`.
- `logs::log_lines(text: &str, max: usize) -> Vec<String>`, `logs::read_logs(p: &EnginePaths, max: usize) -> Vec<String>` and `logs::is_error_line(line: &str) -> bool` are stubs returning `Vec::new()`, `Vec::new()` and `false`. WP-06 writes the bodies.
- The single log limit is `headset_view::DEFAULT_LOG_LINES` (500). There is no `LOG_LIMIT`.

**App page stubs** (types exactly as in #116, #117 and #118, so their test-first commits compile). The types are written out here, so WP-01 needs no issue read. Every `update` stub returns `None`, takes `_`-prefixed parameters and is not `#[must_use]`.
- `headset_page.rs`:
  ```rust
  use flat3d_inject::api::LinkSnapshot;
  #[derive(Debug, Clone, Default, PartialEq)]
  pub struct HeadsetState { pub snapshot: Option<LinkSnapshot>, pub busy: bool, pub error: Option<String> }
  #[derive(Debug, Clone, PartialEq)]
  pub enum Msg { Snapshot(Result<LinkSnapshot, String>), StartPressed, StopPressed, Done(Result<(), String>), SetAuto3d(bool), SetTunnel(bool) }
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Action { Start3d, Stop3d, SetSetting { key: &'static str, on: bool }, Refresh }
  pub fn update(_state: &mut HeadsetState, _msg: Msg) -> Option<Action> { None }
  ```
- `setup_page.rs`. `steps` returns `Vec::new()` and `all_done` returns `false`:
  ```rust
  use flat3d_inject::api::LinkSnapshot;
  use flat3d_inject::setup::Packages;
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct Step { pub title: &'static str, pub done: bool, pub manual: bool, pub hint: &'static str }
  pub fn steps(_packages: Option<&Packages>, _snapshot: Option<&LinkSnapshot>) -> Vec<Step> { Vec::new() }
  pub fn all_done(_steps: &[Step]) -> bool { false }
  ```
  WP-05 adds the rest of the page (`SetupKind`, `SetupState`, `Msg`, `Action`, `update`) in this file.
- `games_page.rs`. `GamesState::default()` has filter `"all"`, so `Default` is written by hand:
  ```rust
  use flat3d_inject::rows::GameRow;
  #[derive(Debug, Clone, PartialEq)]
  pub struct GamesState { pub rows: Vec<GameRow>, pub filter: String, pub error: Option<String>, pub note: Option<String> }
  impl Default for GamesState { fn default() -> Self { Self { rows: vec![], filter: "all".into(), error: None, note: None } } }
  #[derive(Debug, Clone, PartialEq)]
  pub enum Msg { Rows(Result<Vec<GameRow>, String>), Filter(String), Switch { appid: u32, on: bool }, UndoPressed(u32), UndoDone(Result<String, String>) }
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum Action { LoadRows { filter: String }, SetSwitch { appid: u32, on: bool }, Undo(u32) }
  pub fn update(_state: &mut GamesState, _msg: Msg) -> Option<Action> { None }
  ```
- `logs_page.rs`, with no limit constant of its own:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub enum Filter { #[default] All, Errors }
  #[derive(Debug, Clone, Default, PartialEq, Eq)]
  pub struct LogsState { pub lines: Vec<String>, pub filter: Filter, pub loading: bool, pub error: Option<String> }
  #[derive(Debug, Clone, PartialEq)] pub enum Msg { Refresh, Lines(Vec<String>), Failed(String), SetFilter(Filter), CopyPressed }
  #[derive(Debug, Clone, PartialEq, Eq)] pub enum Action { Load, Copy(String) }
  pub fn update(_state: &mut LogsState, _msg: Msg) -> Option<Action> { None }
  pub fn visible(_state: &LogsState) -> Vec<&str> { Vec::new() }
  ```
- `engine.rs` and `view_*.rs` are empty, and WP-33 fills them.

---

#### Behaviour wired now

- **watch:**
  - `Service` builds `HealthLoop::new(sys::unix_now(), 0, None)`.
  - `publish()` (watch.rs:784) calls `tick(sys::unix_now(), mode, playing, &mut Shared(Rc::clone(&self.runner)))`, pushes `toast` to the event ring, and keeps `heal` in a private field for WP-09 to execute (K8). `current()` takes `&HealthTick`.
- **flat3d-run:**
  - builds `EnginePaths::from_env(&env, sys::current_uid())`;
  - calls `run_with` with a `SystemRunner`;
  - in the launcher, applies `env_remove` for each `unset` and spawns the `sidecar` (when Some) with `--until-pid <pid>` appended, where `<pid>` is `std::process::id()` (the exec keeps the pid, so it is the game's pid). The sidecar is always `None` until WP-31, which owns this file from wave 4.
- **`docs/code-map.md`:**
  - Rows for the new stubs say `stub`.
    These are the new stub files of all four parts: the viewer and uevr-inject stubs (WP-01a), the `cli/*.rs` stubs (WP-01c) and this part's files. WP-01a already rewrote the paths.

---

#### Tests

No new test file. The moved test files stay byte-identical to what WP-01a left (the done-check's `locked-14` step proves it). The stubs are held by tests the earlier parts wrote: `tests/architecture.rs` checks every new stub tag, `dispatch_routes_each_family` still routes, and the stub `tick` keeps today's snapshot behaviour, so no locked watch test changes.

---

**Done-check** (run after committing on this part's branch, `v2/wp-01d-*`). Each step is labelled. A failing step prints `FAIL <step>: fix: <hint>` and stops (Craft review A2).
```sh
export CARGO_TARGET_DIR=$SCRATCH/target
T=v2-wp00
fail() { echo "FAIL $1: fix: $2"; exit 1; }
git rev-parse -q --verify "refs/tags/$T^{commit}" >/dev/null \
  || fail tag "the local tag v2-wp00 is missing: report blocked; the runner places it on the commit where WP-00 landed (§0); do not guess a commit"
git merge-base --is-ancestor "$T" HEAD || fail tag-ancestor "v2-wp00 is not an ancestor of this branch: report blocked; the runner re-places it on the commit where WP-00 landed (§0)"
git show "$T:inject/Cargo.toml" 2>/dev/null | grep -q field_reassign_with_default \
  || fail tag-wp00 "v2-wp00 must sit on a commit that holds WP-00 and still has inject/: report blocked; the runner re-places it (§0)"
{ test ! -e inject && test ! -e app; } || fail moved "git mv inject/ and app/ under crates/"
test -z "$(git ls-files '*mod.rs')" || fail no-mod-rs "use the facade-file form (x.rs + x/)"
cargo metadata --locked --offline --format-version 1 >/dev/null || fail lock "re-seed Cargo.lock as in Fetching dependencies"
cargo fmt --all --check || fail fmt "cargo fmt --all"
cargo clippy --locked --workspace --all-targets -- -D warnings \
  || fail clippy "run cargo clippy --workspace --all-targets -- -D warnings and fix each lint it names"
cargo clippy --locked -p flat3d-viewer --all-targets --features xr -- -D warnings || fail clippy-xr "fix the viewer lints it names"
XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked --workspace || fail test "fix the failing test it names"
XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test tether_adb_runner --test tether_runtime --test cli_json \
  || fail runners "keep the exec split re-exports (tether::adb::CommandRunner)"
cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings || fail clippy-win "fix the uevr-inject lints it names"
git diff --exit-code Cargo.lock || fail lock-frozen "restore the committed Cargo.lock; never run cargo update"
test "$(git ls-tree --name-only "$T" inject/tests/ | grep -c '\.rs$')" = 55 || fail tests-55 "v2-wp00 must hold 55 inject/tests files: report blocked; the runner re-places the tag (§0)"
for f in $(git ls-tree --name-only "$T" inject/tests/); do
  test -f "crates/engine/tests/${f#inject/tests/}" || fail tests-moved "git mv $f"
done
for t in paths fsx catalog_find switches_io state_io route_settings cli_any architecture; do
  test -f "crates/engine/tests/$t.rs" || fail tests-new "add crates/engine/tests/$t.rs"
done
test -f crates/viewer/tests/spec.rs || fail tests-new "add crates/viewer/tests/spec.rs"
test "$(for f in $(git ls-tree --name-only "$T" inject/tests/); do git show "$T:$f" | diff -u - "crates/engine/tests/${f#inject/tests/}"; done | grep -c '^[-+][^-+]')" = 14 \
  || fail locked-14 "undo every edit to a moved test file except the 7 sanctioned ones"
if [ -x tools/check.sh ]; then tools/check.sh || fail check.sh "fix the step check.sh names"
else echo "SKIP check.sh: WP-02 not merged yet"; fi
```
- `$T` is the fixed local tag `v2-wp00` (§0 "Git references in done-checks"): the integration-line commit where WP-00 landed, so `inject/tests/` as WP-00's rustfmt left it. It is the same commit in both run modes and for all four WP-01 parts, so these steps do not depend on where `main` or the integration line is now, or on which parts have already merged.
- `tests-55`, `tests-moved` and `locked-14` read the old path `inject/tests/<f>` from the tag and the new path `crates/engine/tests/<f>` from the working tree. `inject/` is never looked for in this branch's own base.
- This part never places the tag. `inject/` is already gone from this branch's base (WP-01a moved it), so no commit here can stand in for it; a missing or wrong tag is `blocked`.
- The `locked-14` step proves that exactly the 7 sanctioned edits (14 diff lines) touched the 55 moved test files.
- The `architecture` test runs inside the `test` step; `tests-new` checks that its file exists.
- Python, shellcheck and decky are not run here. Their CI steps are guarded until WP-02, WP-21 and WP-25b land.
- The `capture` clippy is not run here (no pipewire, spa or clang headers on the host); CI runs it.

**PR and merge** (rulings D1, D5):
- The PR body has `## Verified`: each done-check step and its exit status, quoted from the run. It never claims a run that was not done.
- It also has `## Not verified`: each skip the run hit. For WP-01d that is the `capture` clippy (CI only), the `py` and `decky` jobs, `check.sh` if WP-02 has not merged, and the CI run itself if it was not seen to finish.
- The merge takes only the head commit the review passed.
- WP-01d has no device Q row, so it keeps `Closes` for its issues (D5 applies only to WPs with a device half).
