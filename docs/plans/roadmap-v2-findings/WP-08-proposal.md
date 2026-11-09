# WP-08 recheck: proposed replacement text

Plan mode was active, so `split/wp/WP-08.md` was not edited. Replace its whole body with the block below.

---

### WP-08: Link commands wiring (#120)
- **Closes:** #120 (T15 link command bodies), A24
- **Files (exclusive):**
  - `crates/engine/src/cli/link.rs` (the parse half from `cli_link.rs` stays byte-identical: `SettingKey`, `LinkCommand`, `parse_link` and their private helpers; locked `cli_link_parse.rs` imports them via `flat3d_inject::cli_link`)
  - `crates/engine/tests/cli_link_run.rs` (new, not locked)
- **Does not edit:** `cli.rs` (frozen: `USAGE`, `parse`, `parse_any`, `Ctx`, dispatch, `render` belong to WP-01), `install.rs` (WP-20), `wrapper.rs` (WP-31), `state.rs`, `switches.rs`, `api.rs`, any locked test.
- **Public signatures (frozen by WP-01; fill in bodies only, add no other pub item):**
  - `pub fn run(cmd:&LinkCommand, ctx:&mut Ctx<'_>) -> Result<serde_json::Value, ApiError>` (bones §2 `cli.rs`; `Ctx{paths,runner,relay,json}`)
  - `pub fn request_3d(r:state::Request, p:&EnginePaths, now:u64) -> Result<api::LinkReport, ApiError>`. WP-01's dispatch routes `cli::Command::{Start3d,Stop3d}` here with `sys::unix_now()` and serialises the report.
- **Private (no `pub`):**
  - `struct LinkIo<'a>{ now:u64, pid:u32, env:BTreeMap<String,String>, src_dir:PathBuf, exec:&'a mut dyn FnMut(&wrapper::Launch)->io::Result<()>, watch:&'a mut dyn FnMut(&EnginePaths) }`
  - `fn run_with(cmd:&LinkCommand, ctx:&mut Ctx<'_>, io:&mut LinkIo<'_>) -> Result<Value, ApiError>`
  - `fn exec_launch(l:&wrapper::Launch, appid_added:Option<u32>, pid:u32) -> io::Result<()>`
  - `run` builds the real `LinkIo` and calls `run_with`: `now = sys::unix_now()`, `pid = std::process::id()`, `env = std::env::vars().collect()`, `src_dir = current_exe()`'s parent (error → `io`), `exec = exec_launch`, `watch = |p| crate::watch::run(&p.watch_paths())` (returns `!`).
- **Consumed (exact forms):**
  - `install::execute(&InstallPlan, &mut dyn CommandRunner)` and `install::execute_uninstall(&UninstallPlan, &mut dyn CommandRunner)`, both `-> Result<api::InstallReport, ApiError>` (WP-20 lines 14-15; WP-01 stubs return not-built). `install::{install_plan(home:&str, src_dir:&str), uninstall_plan(home:&str)}` (install.rs:26, :70).
  - `wrapper::run_with<T>(argv:&[String], env:&BTreeMap<String,String>, p:&EnginePaths, pid:u32, now:u64, runner:&mut dyn CommandRunner, launcher:impl FnOnce(&Launch)->io::Result<T>) -> (Launch, io::Result<T>)` (WP-01). `Launch{command:ExecSpec{argv,env,unset,sidecar},record,message}`.
  - `state::{read_snapshot(&EnginePaths)->Result<Option<LinkSnapshot>,String>, write_request(&EnginePaths, Request)->io::Result<()>, request_path, is_stale}`; `switches::{read_settings(&Path)->Result<Settings,String> (NotFound → Default), write_settings(&Path,&Settings)->io::Result<()>}`; `Settings{auto_3d,tunnel,off}` + `game_on`/`set_game` (switches.rs:9-38).
  - `api::{LinkSnapshot::stopped(now), LinkSnapshot::effective(self,now), LinkReport{state,message}, SwitchState{appid,on}, SettingState{key,on}, InstallReport{written,ran,summary}, ApiError::new}` (api.rs:56, 216, 233, 243, 249, 255, 95). Not from WP-07.
  - `EnginePaths::{settings(), watch_paths(), home, state_dir}` (WP-01).
- **Behaviour (`run_with`, one arm per `LinkCommand`):**
  - **Watch:** `(io.watch)(ctx.paths)`; if it returns (tests only), `Ok(Value::Null)`.
  - **Snapshot:** `read_snapshot(p)`: `Ok(Some(s))` → `s.effective(io.now)`; `Ok(None)` → `LinkSnapshot::stopped(io.now)`; `Err(e)` → `io`. Returns the full `LinkSnapshot` as JSON.
  - **SwitchGet{appid}:** `read_settings(&p.settings())` (Err → `io`) → `SwitchState{appid, on:game_on(appid)}`.
  - **SwitchSet{appid,on}:** read, `set_game`, `write_settings` (Err → `io`) → `SwitchState{appid,on}`.
  - **SettingGet/SettingSet{key,..}:** `Auto3d` ↔ `settings.auto_3d`, `Tunnel` ↔ `settings.tunnel`; set writes the file. Returns `SettingState{key:key.as_str().into(), on}`.
  - **Install:** `install::execute(&install_plan(&home, &src_dir), ctx.runner)` with `home = p.home.to_string_lossy()`, `src_dir = io.src_dir.to_string_lossy()`. **Uninstall:** `install::execute_uninstall(&uninstall_plan(&home), ctx.runner)`. Returns `InstallReport`; WP-20's errors pass through unchanged.
  - **Run{appid,argv}:**
    - `env = io.env.clone()`; insert `SteamAppId=<appid>` when neither `SteamAppId` nor `SteamGameId` is set (wrapper.rs:98), and remember that it was added.
    - `wrapper::run_with(&argv, &env, p, io.pid, io.now, ctx.runner, |l| (io.exec)(l))`. `argv` is passed verbatim (it may contain `--json`; `parse_any` only strips `--json` before the first `--`).
    - Launcher `Err(e)` → `ApiError::new("command-failed", &e.to_string())`. `Ok(())` (tests only) → `json!({"message": launch.message})`.
    - `exec_launch`: `Command::new(argv[0]).args(&argv[1..])`, `.envs(spec.env)`, `env_remove` each `spec.unset`, `.env("SteamAppId", appid)` when it was added; spawn `spec.sidecar` (if any) with `--until-pid <pid>` appended, as `bin/flat3d-run.rs` does; then `CommandExt::exec` (returns only on error).
  - **request_3d (A24 one-actor rule):** `read_snapshot(p)`:
    - `Ok(None)`, `Err(_)`, or `Ok(Some(s))` with `state::is_stale(s.updated, now)` → `Err(ApiError::new("link-not-ready", "Link service is not running."))`. Nothing is written, nothing is spawned (the fn has no runner).
    - Fresh → `write_request(p, r)` (Err → `io`) → `Ok(LinkReport{state:s.state, message:"Requested start-3d."|"Requested stop-3d."})` (text from `state::request_text(r)`).
  - **Error codes:** only the 8 in `ERROR_CODES` (api.rs:77): this WP uses `io`, `command-failed`, `link-not-ready`, plus whatever `install::*` returns. No new code.
- **Tests:**
  - **Locked, unchanged, must pass:** `cli_link_parse.rs`, `cli_json.rs` (`:127-136` runs `start-3d/stop-3d --json` for real: with the temp `XDG_RUNTIME_DIR` from check.sh/CI there is no snapshot, so they give `link-not-ready`, `ok:false`, exit 1), `install_plan.rs`.
  - **No new dev-dependency** (Cargo.lock is frozen). Temp dirs: integration tests use `env!("CARGO_TARGET_TMPDIR")/cli_link_run/<test name>`; unit tests use `std::env::temp_dir()/flat3d-cli-link-<pid>-<test name>`. Remove the dir first. Build paths with `EnginePaths::from_env(&env, 1000)` where `env` = `{HOME: <tmp>/home, XDG_RUNTIME_DIR: <tmp>/run}` (BTreeMap form, bones §2). Runners are `flat3d_inject::testing::FakeRunner` (read its constructor and log accessor with `grep -n 'pub fn' crates/engine/src/testing.rs`); `Ctx` needs two of them (`runner`, `relay`).
  - **New `crates/engine/tests/cli_link_run.rs` (public API only):**
    1. `request_3d(Start3d, &p, 1005)` with no snapshot → `Err` code `link-not-ready`, message `Link service is not running.`; `state::request_path(&p.state_dir.to_string_lossy())` does not exist.
    2. Snapshot written with `updated=1000` (`LinkSnapshot::stopped(1000)` with `state=Ready`, `serde_json::to_string`, written to `state::snapshot_path(&p.state_dir.to_string_lossy())`, dir created), `request_3d(Start3d,&p,1011)` → same as 1.
    3. Same snapshot, `now=1005`: `Start3d` → `Ok(LinkReport{state:Ready, message:"Requested start-3d."})`, request file trims to `start-3d`; then `Stop3d` → file trims to `stop-3d`.
    4. `run(&SwitchSet{appid:5,on:false})` then `run(&SwitchGet{appid:5})` → `{"appid":5,"on":false}`; `p.settings()` starts with `flat3d-settings 1`.
    5. `run(&SettingSet{key:Tunnel,on:false})` then `SettingGet{key:Tunnel}` → `{"key":"tunnel","on":false}`; `Auto3d` still reads `on:true`.
    6. `run(&Snapshot)` with no snapshot → an object with `"state":"stopped"`; after writing a snapshot with `updated = sys::unix_now()` and `state=Ready` → `"state":"ready"`.
    7. `run(&Install)` with the temp HOME (the test binary's dir has no `flat3d-inject` to copy): result is `Ok` with an `InstallReport` shape or `Err` whose code is in `ERROR_CODES`; the FakeRunner log never contains `enable`.
    8. `cli::execute(&["fly","--json"])` → `command:""`, `code:"usage"`, `message:USAGE` (guards that link routing did not change the fallback).
  - **New unit tests, `#[cfg(test)] mod tests` in `crates/engine/src/cli/link.rs` (private seam via `run_with`):**
    9. `Run{appid:5, argv:["game","--json"]}` with `env` lacking SteamAppId: the fake exec sees `l.command.argv == ["game","--json"]`, the run record file `state::run_record_path(state_dir, pid)` already exists when exec is called, and `l.record.appid == 5` (proves `SteamAppId=5` reached `run_with`; `RunRecord{appid,pid,mode}`, runs.rs:14); fake exec returning `Err` → code `command-failed`.
    10. `Watch` calls the fake watch closure exactly once and returns `Ok(Null)`.
    11. `Install` with `io.src_dir` = a temp dir holding empty `flat3d-inject`, `flat3d-run`, `tools/{adb,gnirehtet,gnirehtet.apk}` and a FakeRunner → `Ok`, deserialises as `InstallReport`; the runner log has `systemctl --user daemon-reload` then `systemctl --user enable --now flat3d-link.service`. `Uninstall` afterwards → `Ok(InstallReport)`.
- **Flags:**
  - `request_3d` must be frozen by WP-01 (stub `Err(NotBuilt("cli::link::request_3d"))`) and dispatch must call it for `Start3d/Stop3d`, because `cli.rs` is frozen after wave 1 and `run` takes `&LinkCommand`, which has no start/stop variants.
  - A bare `cargo test` on a device whose link service is live finds a fresh snapshot in cli_json `:127` and writes a real request; always run with a temp `XDG_RUNTIME_DIR` (check.sh and CI do).
- **Depends on:** WP-01, WP-20 (install/uninstall bodies for test 11) (wave 3)
- **Network/device:** none for the pass. Owner device: Q3 (`sessions/link-smoke.toml`).
- **Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --lib cli::link --test cli_link_run --test cli_link_parse --test cli_json --test install_plan && tools/check.sh`
