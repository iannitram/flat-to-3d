# WP-33 recheck: proposed replacement text

Plan mode was active, so `wp/WP-33.md` was NOT edited. Below is the full corrected WP-33, ready to replace
roadmap-v2.md (WP-33 section) verbatim.

---

### WP-33: App views + main wiring (#122/#123)
**Closes:** #122, #123 (code).
**Files (exclusive):** `crates/app/src/{lib,main,engine,view_headset,view_games,view_setup,view_logs}.rs`, `crates/app/tests/app_wiring.rs`. WP-01 created `engine.rs` and `view_*.rs` as stubs and wrote every `pub mod` line in `lib.rs`; this WP fills the bodies and adds no `mod` line and no dependency (`crates/app/Cargo.toml` already has `flat3d-inject`, `iced` with `smol`, and `serde_json`, WP-01).
**Build environment:** the `craft-build` distrobox (archlinux base-devel, running on this machine) or CI. iced needs `pkg-config`, xkbcommon, wayland and fontconfig dev libraries that the SteamOS host lacks, so every `-p flat3d-app` build, test and clippy runs inside `distrobox enter craft-build`. CI gets them from WP-01's check.yml apt line (`pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev`, present in both bones §2 and WP-01).

**Signatures.** Page types are WP-04/05/06/23's, as frozen (quoted in "Page types" below). Engine items are WP-01's frozen forms.
```rust
// lib.rs: keep Tab / Tab::ALL / title / placeholder and the 4 unit tests (lib.rs:5-38, :61-90) unchanged.
// Keep WP-01's mod lines: headset_page games_page setup_page logs_page engine view_headset view_games view_setup view_logs.
// Fix the module doc (lib.rs:1-3): the view_* modules use iced; update stays pure.
use flat3d_inject::state::Request;
use setup_page::SetupKind;
#[derive(Debug, Default, Clone)] pub struct App { pub active: Tab, pub now: u64, pub ticks: u64,
  pub headset: headset_page::HeadsetState, pub games: games_page::GamesState,
  pub setup: setup_page::SetupState, pub logs: logs_page::LogsState }
#[derive(Debug, Clone)] pub enum Message { Select(Tab), Tick(u64), Headset(headset_page::Msg), Games(games_page::Msg),
  Setup(setup_page::Msg), Logs(logs_page::Msg), SwitchSaved(Result<(),String>) }
#[derive(Debug, Clone, PartialEq, Eq)] pub enum Effect { ReadSnapshot, Request(Request), SetSetting{key:&'static str,on:bool},
  LoadRows{filter:String}, SetSwitch{appid:u32,on:bool}, Undo(u32), ReadPackages, RunSetup(SetupKind), LoadLogs, Copy(String) }
pub fn update(app:&mut App, m:Message) -> Vec<Effect>;   // pure, no IO
// engine.rs (the only fs/process code)
pub fn paths() -> Result<EnginePaths,String>;
pub fn cli_path(exe:&Path, src_dir:Option<&str>, bin:&Path, exists:&dyn Fn(&Path)->bool) -> Option<PathBuf>;
pub fn snapshot_or_stopped(r:Result<Option<LinkSnapshot>,String>, now:u64) -> Result<LinkSnapshot,String>;
pub fn read_snapshot(p:&EnginePaths, now:u64) -> Result<LinkSnapshot,String>;   // snapshot_or_stopped(state::read_snapshot(p), now)
pub fn request(p:&EnginePaths, r:Request) -> Result<(),String>;                 // state::write_request(p,r).map_err(|e| e.to_string())
pub fn parse_envelope(stdout:&str) -> Result<serde_json::Value,String>;          // ok → data (Null if absent) | error.message
pub fn row_from_json(v:&serde_json::Value) -> Option<GameRow>;                 // GameRow is Serialize-only (#76): build it field by field
pub fn run_cli(args:&[&str]) -> Result<serde_json::Value,String>;              // <cli> args.. [--json unless already in args]
// view_*.rs
pub fn view_headset::view(s:&HeadsetState, now:u64) -> Element<'_, headset_page::Msg>;
pub fn view_games::view(s:&GamesState) -> Element<'_, games_page::Msg>;
pub fn view_setup::view<'a>(s:&'a SetupState, snap:Option<&'a LinkSnapshot>) -> Element<'a, setup_page::Msg>;
pub fn view_logs::view(s:&LogsState) -> Element<'_, logs_page::Msg>;
```
Imports: `flat3d_inject::paths::EnginePaths`, `flat3d_inject::api::LinkSnapshot`, `flat3d_inject::rows::GameRow`, `flat3d_inject::{state, sys, setup::Packages}`, `flat3d_inject::face::link_model::link_view`, `flat3d_inject::face::logs::read_logs`, `flat3d_inject::headset_view::DEFAULT_LOG_LINES` (=500, headset_view.rs:21), `flat3d_inject::status::STATUSES` (status.rs:5).

**Page types (frozen, do not redefine):**
- WP-04 `headset_page`: `HeadsetState{snapshot:Option<LinkSnapshot>,busy,error}`; `Msg{Snapshot(Result<LinkSnapshot,String>),StartPressed,StopPressed,Done(Result<(),String>),SetAuto3d(bool),SetTunnel(bool)}`; `Action{Start3d,Stop3d,SetSetting{key,on},Refresh}`; `update(&mut,Msg)->Option<Action>`.
- WP-23 `games_page`: `GamesState{rows,filter:String,error,note}` (hand-written Default, filter `"all"`); `Msg{Rows(Result<Vec<GameRow>,String>),Filter(String),Switch{appid,on},UndoPressed(u32),UndoDone(Result<String,String>)}`; `Action{LoadRows{filter},SetSwitch{appid,on},Undo(u32)}`.
- WP-05 `setup_page`: `SetupState{packages:Option<Packages>,snapshot,busy,error}`; `Msg{Packages(Result<Packages,String>),Snapshot(..),RunPressed(SetupKind),Done(Result<(),String>)}`; `Action{RunSetup(SetupKind),Refresh}`; `SetupKind{Packages,InstallApps}` with `args()` = `["setup","packages","--json"]` / `["setup","install-apps","--json"]`; `steps(Option<&Packages>,Option<&LinkSnapshot>)->Vec<Step>`; `adb_source_text(Option<&LinkSnapshot>)`.
- WP-06 `logs_page`: `LogsState{lines,filter,loading,error}`; `Msg{Refresh,Lines(Vec<String>),Failed(String),SetFilter(Filter),CopyPressed}`; `Action{Load,Copy(String)}`; `visible(&LogsState)->Vec<&str>`.

**Behaviour**
- **`update` routing:**
  - `Select(t)` sets `active` and returns `[ReadSnapshot]`, plus `LoadRows{filter: games.filter}` for Games, `ReadPackages` for Setup, and `LoadLogs` for Logs.
  - `Tick(n)` sets `now=n`, then `ticks+=1`, and returns `[ReadSnapshot]`, plus `LoadLogs` when Logs is active, and `ReadPackages` when Setup is active and `ticks%10==0` (checked after the increment, so ticks 10, 20, …).
  - `Headset(m)` → `headset_page::update`; `Start3d`/`Stop3d` → `Request(Request::Start3d|Stop3d)`, `SetSetting{key,on}` → `SetSetting{key,on}`, `Refresh` → `ReadSnapshot`.
  - `Games(m)` → `games_page::update`; `LoadRows`, `SetSwitch`, `Undo` map 1:1.
  - `Logs(m)` → `logs_page::update`; `Load` → `LoadLogs`, `Copy(t)` → `Copy(t)`.
  - `Setup(m)` → `setup_page::update(&mut app.setup, m)`; `RunSetup(k)` → `RunSetup(k)`, `Refresh` → `ReadPackages`. Results (`Packages(..)`, `Done(..)`) come back through the same `update` and land in `SetupState`. `lib.rs` keeps no setup state of its own. The "Check again" button sends `RunPressed(SetupKind::Packages)` so `busy` is set and cleared by `Done` (WP-05 clears `busy` only on `Done`).
  - `SwitchSaved(Ok)` → `[LoadRows{filter: games.filter.clone()}]`. `SwitchSaved(Err(e))` sets `games.error=Some(e)` and returns `[]`.
- **`main.rs`:**
  - `iced::application(App::default, step, view).title("Flat to 3D").subscription(|_| iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick(flat3d_inject::sys::unix_now()))).run()`. The map closure captures nothing (iced hashes subscriptions by closure type).
  - `step(app,m) -> Task<Message>` = `Task::batch(update(app,m).into_iter().map(run_effect))`.
  - `Copy(t)` → `iced::clipboard::write(t)`. Every other effect → `Task::perform(off_thread(move || ..), |r| r)`, where `off_thread` sends the closure's result through `iced::futures::channel::oneshot` from a `std::thread::spawn` thread (a dropped sender maps to an `Err("worker stopped")`-style message for that effect).
  - `engine::paths()` is called inside each worker; its `Err(e)` becomes that effect's error message.
  - Effects map back as:
    - `ReadSnapshot` → `Headset(Msg::Snapshot(engine::read_snapshot(&p, sys::unix_now())))`;
    - `Request(r)` → `Headset(Msg::Done(engine::request(&p, r)))`. The app never runs adb or `link::start_3d`; Start/Stop only writes the request file;
    - `SetSetting{key,on}` → `Headset(Msg::Done(..))` via `run_cli(["setting","set",key, if on {"on"} else {"off"}])` (WP-08), data ignored;
    - `LoadRows{filter}` → `Games(Msg::Rows(..))` via `run_cli(["rows","--filter",&filter])` (WP-28): `data` is an array, each element through `row_from_json`; an element that gives `None` makes the whole result `Err("bad row: <json>")`;
    - `SetSwitch{appid,on}` → `SwitchSaved(..)` via `run_cli(["switch","set",&appid.to_string(), "on"|"off"])` (WP-08), data ignored;
    - `Undo(id)` → `Games(Msg::UndoDone(..))` via `run_cli(["undo",&id.to_string()])` (WP-27), `Ok(data["summary"] as string)`;
    - `ReadPackages` → `Setup(Msg::Packages(..))` via `run_cli(["setup","packages"])` (WP-29, data `{"wivrn":bool,"gnirehtet":bool,"missing":[..]}`) → `Packages{wivrn,gnirehtet}` (pub fields, setup.rs:17-21); a missing or non-bool key is `Err`;
    - `RunSetup(k)` → `Setup(Msg::Done(run_cli(k.args()).map(|_| ())))`;
    - `LoadLogs` → `Logs(Msg::Lines(read_logs(&p, DEFAULT_LOG_LINES)))`, or `Logs(Msg::Failed(e))` when `paths()` fails (WP-06 §Msg; no CLI call).
  - `view`: tabs are a button row as today (main.rs:13-19); below it the active page: Setup `view_setup::view(&app.setup, app.headset.snapshot.as_ref()).map(Message::Setup)`, Headset `view_headset::view(&app.headset, app.now).map(Message::Headset)`, Games `…map(Message::Games)`, Logs `…map(Message::Logs)`. Closing the window leaves `flat3d-link.service` running (the app starts and stops nothing).
- **`paths()`:** `EnginePaths::from_env(&|k: &str| std::env::var(k).ok(), flat3d_inject::sys::current_uid())`, which is WP-01's frozen form `from_env(get:&dyn Fn(&str)->Option<String>, uid:u32)->Result<Self,String>`. Return it as is. WP-31's `flat3d-run.rs` uses the same form.
- **`cli_path` order:** `exe.parent()/flat3d-inject`, then `<src_dir>/bin/flat3d-inject` when `src_dir` is `Some`, then `bin` (the caller passes `Path::new("bin/flat3d-inject")`); the first one where `exists(p)` is true wins, else `None`. `run_cli` calls it with `std::env::current_exe()`, `std::env::var("FLAT3D_SRC_DIR").ok().as_deref()`, and `&|p| p.exists()`; `None` → `Err("flat3d-inject not found")`. It runs the binary with `std::process::Command`, appends `--json` only when `args` does not already contain it (`SetupKind::args()` already does), and passes stdout to `parse_envelope` (the engine prints the envelope on stdout for both ok and error exits).
- **`parse_envelope`:** the `api::Response` shape (api.rs:110-116): `{"ok":true,"data":..}` → `Ok(data or Null)`; `{"ok":false,"error":{"code","message"}}` → `Err(message)`; not JSON → `Err("bad engine output: <first 200 chars>")`.
- **`snapshot_or_stopped`:** `Ok(None)` → `Ok(LinkSnapshot::stopped(now))`; `Ok(Some(s))` with `state::is_stale(s.updated, now)` (state.rs:143, `STALE_SECS=10`) → `Ok(LinkSnapshot::stopped(now))`; fresh → `Ok(s)`; `Err` passes through.
- **Views:**
  - `view_headset`: `let v = link_view(s.snapshot.clone(), now);` (WP-28 frozen form `link_view(Option<LinkSnapshot>, now:u64)->api::LinkView`). Show `v.label`, `v.hint`, the health line `v.health.battery` / `v.health.temp` coloured with `Color::parse(&v.health.colour).unwrap_or(Color::WHITE)`, and `v.rx`, `v.tx`, `v.uptime`. Start/Stop buttons get `.on_press(Msg::StartPressed|StopPressed)` only when `v.start`/`v.stop` is true and `!s.busy`. `toggler(v.auto_3d).label("Auto 3D").on_toggle(Msg::SetAuto3d)` and `toggler(v.tunnel).label("Tunnel").on_toggle(Msg::SetTunnel)`. `s.error` shows as text.
  - `view_games`: `pick_list(["all"] + STATUSES as Vec<String>, Some(s.filter.clone()), Msg::Filter)`. A `scrollable` column of rows: name, `text(&row.status_label).color(Color::parse(&row.status_colour).unwrap_or(Color::WHITE))`, `toggler(row.switch_on).on_toggle(move |on| Msg::Switch{appid,on})`, `flat_reason` when `Some`, and an `Undo` button (`UndoPressed(appid)`). `s.note` and `s.error` show as text.
  - `view_setup`: `setup_page::steps(s.packages.as_ref(), snap)` gives one line each: `✓` or `○`, title, and the hint when not done. Then `adb_source_text(snap)`, `s.error` as text, and two buttons, "Check again" → `RunPressed(SetupKind::Packages)` and "Install headset apps" → `RunPressed(SetupKind::InstallApps)`, each without `on_press` while `s.busy`. There is no doctor button (not in #122; `flat3d-inject doctor` stays CLI-only).
  - `view_logs`: a `scrollable` of `logs_page::visible(s)` lines, an "All / Errors" filter (`SetFilter`), Refresh (`Refresh`) and Copy (`CopyPressed`). `s.error` shows as text.

**Tests**
- **Locked, unchanged:** `crates/app/tests/{headset_page,games_page,setup_page,logs_page}.rs` (#116, #117, #118, WP-06) and the 4 `lib.rs` unit tests (`select_moves_to_the_tab` calls `update(..)` as a statement; the returned `Vec` is not `#[must_use]`, so it still compiles clean).
- **New `crates/app/tests/app_wiring.rs`** (imports `flat3d_app::{update, App, Effect, Message, Tab, engine, headset_page, games_page, setup_page, logs_page}`):
  1. `Select(Games)` on a default `App` gives `[ReadSnapshot, LoadRows{filter:"all"}]`.
  2. With `active=Setup`, ticks 1-9 give `[ReadSnapshot]` and tick 10 gives `[ReadSnapshot, ReadPackages]`.
  3. With `active=Logs`, `Tick` gives `[ReadSnapshot, LoadLogs]`.
  4. `Headset(StartPressed)` with no snapshot gives `[Request(Request::Start3d)]`; a second press gives `[]`.
  5. `Headset(Done(Ok(())))` gives `[ReadSnapshot]`.
  6. `Headset(SetTunnel(false))` gives `[SetSetting{key:"tunnel",on:false}]`.
  7. `Games(UndoPressed(7))` gives `[Undo(7)]`.
  8. `SwitchSaved(Err("x"))` sets `games.error=Some("x")` and gives `[]`.
  9. After `Games(Filter("works-now"))`, `SwitchSaved(Ok(()))` gives `[LoadRows{filter:"works-now"}]`.
  10. With `logs.lines=["a","b"]`, `Logs(CopyPressed)` gives `[Copy("a\nb")]`.
  11. `Setup(RunPressed(InstallApps))` gives `[RunSetup(InstallApps)]` and a second press gives `[]`; `Setup(Done(Ok(())))` gives `[ReadPackages]`; `Setup(Packages(Ok(Packages{wivrn:true,gnirehtet:false})))` sets `app.setup.packages` and gives `[]`.
  12. `Logs(Lines(vec!["x".into()]))` sets `app.logs.lines` and gives `[]`.
  13. `cli_path` with an `exists` fake: sibling present → sibling; only `<src>/bin/flat3d-inject` present → that; nothing present → `None`.
  14. `snapshot_or_stopped(Ok(None),5)` equals `LinkSnapshot::stopped(5)`; a snapshot with `updated=0` at `now=100` gives state `Stopped`; `Err("e")` passes through.
  15. `parse_envelope` returns `data` for `{"ok":true,"data":{"a":1}}`, `Err("boom")` for `{"ok":false,"error":{"code":"io","message":"boom"}}`, and `Err` for `not json`.
  16. `row_from_json(&serde_json::to_value(row).unwrap())` equals `Some(row)` for a full `GameRow` literal; removing the `"name"` key gives `None`.

**Inline facts**
- iced 0.14.0 with `features=["smol"]` (workspace dep, WP-01); `iced::time::every` exists only with smol or tokio.
- `request_text` gives `start-3d`/`stop-3d` (state.rs:135). `Request` is `Copy, Eq` (state.rs:117-118).
- `STALE_SECS=10`, `is_stale(updated, now)` (state.rs:83,143).
- `GameRow{appid,name,status,status_label,status_colour,switch_on,plays_3d,flat_reason:Option<String>}` (WP-03).
- `HealthView{battery:String,temp:String,zone,colour:String}` (view.rs:15).
- AppImage layout: `usr/bin/{flat3d-app,flat3d-inject,flat3d-run}`, `FLAT3D_SRC_DIR=$APPDIR/usr`, so the sibling rule finds the CLI.
- Status filters: `works-now likely build not-safe wont-work untested` (status.rs:5).
- Setting keys are exactly `"auto-3d"` and `"tunnel"`; the CLI takes `on`/`off` (WP-08).

**Wave:** 4.
**Depends on (merge order):** WP-04, WP-05, WP-06, WP-23 (wave 2, page types and bodies) and WP-28 (wave 3, `link_view` body). Everything else it uses is frozen by WP-01.
**Runtime only (not merge-blocking; needed before Q15):** WP-08 (`setting set`, `switch set`), WP-27 (`undo`), WP-29 (`setup packages|install-apps`). No WP-33 test runs the CLI. WP-29 is not a merge dependency because it itself depends on WP-27 (wave 3) and so cannot sit in an earlier wave than this one.
**Network/device:** no network. Device: Q15 `sessions/app-window.toml` (and Q16 `appimage.toml` later), run by the owner after WP-08/27/29/33 are merged.
**Done-check (in your WP-33 checkout, never on the host):** `distrobox enter craft-build -- env CARGO_TARGET_DIR=<agent scratch>/target sh -c 'cd <your WP-33 checkout> && cargo fmt --all --check && cargo test --locked -p flat3d-app && cargo clippy --locked -p flat3d-app --all-targets -- -D warnings'`, then `git diff --exit-code Cargo.lock`.

**Flags**
- **Flag:** the scope says tabs "Headset/Games/Setup/Logs", but `lib.rs:17` and the unit test `the_four_tabs_are_in_order` fix Setup, Headset, Games, Logs, and the window opens on Setup. Keep the existing order.
- **Flag:** #123 says a 2 s tick (docs/code-map.md:74 also says 2 s); the scope says 1 s. 1 s is used here; align #123 and code-map in §6 / WP-34.
- **Flag:** WP-05's `SetupState` has no note field and `Done` carries `()`, so the install-apps summary is not shown; only errors are. Changing that is a WP-05 follow-up, not a lib.rs field.
- **Flag:** `SetupState.snapshot` is not fed; `view_setup` takes the Headset page's snapshot instead (one reader, one source).
- **Flag:** `setup packages` runs adb from the CLI, a second adb client next to the service. Accepted: both talk to the one adb server.
