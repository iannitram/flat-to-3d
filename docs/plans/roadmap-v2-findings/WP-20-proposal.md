# Proposed replacement text for split/wp/WP-20.md

Plan mode blocked the edit to WP-20.md. Copy everything below the line over that file.

---

### WP-20: Install execute + launch options (#128)
**Wave:** 2. **Closes:** #128 (code), A15, K24. **Files (exclusive):** `crates/engine/src/install.rs`, `crates/engine/src/launch_options.rs`, `packaging/flat3d-link.service`, `crates/engine/tests/install_execute.rs`, `crates/engine/tests/launch_options_at.rs`.

**Contract (WP-01 "Install and launch options" stubs, WP-01.md:171-177; this WP fills the bodies and does not change any frozen signature):**
```rust
// install.rs (WP-01 wrote these with stub bodies)
pub struct UnitSpec { pub exec_start: String, pub start_limit_interval_sec: u32, pub start_limit_burst: u32 }
pub fn unit_text_with(spec: &UnitSpec) -> String;
pub struct InstallReport { pub steps: Vec<String> }
pub fn execute(plan: &InstallPlan, runner: &mut dyn CommandRunner, paths: &EnginePaths) -> Result<InstallReport, ApiError>;
pub fn execute_uninstall(plan: &UninstallPlan, runner: &mut dyn CommandRunner, paths: &EnginePaths) -> Result<InstallReport, ApiError>;
// launch_options.rs
pub fn add_wrapper_at(options: &str, wrapper_abs: &str) -> String;
```
- `CommandRunner`/`CommandOutput` = `crate::exec::{CommandRunner, CommandOutput}`, the link shape: `fn run(&mut self, &[String]) -> CommandOutput{success, stdout, stderr}` (base link.rs:98-113). `EnginePaths` = `crate::paths::EnginePaths`. `ApiError::new(&str,&str)` (base api.rs:95).
- Additions inside this WP's own file (allowed, §0): 
  - `#[derive(Debug, Clone, PartialEq, Eq)]` on `UnitSpec` and `impl Default for UnitSpec` = `{exec_start: "%h/.local/bin/flat3d-inject watch", start_limit_interval_sec: 60, start_limit_burst: 10}`. If WP-01 already derived `Default`, drop it from the derive list and keep this impl.
  - `#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]` on `InstallReport` (WP-08 serialises it).
  - `#[derive(Debug, Clone)]` and new pub fields: `InstallPlan { dirs: Vec<String>, trees: Vec<(String, String)>, manifest: String }`, `UninstallPlan { home: String, manifest: String }`. Locked tests only read the old fields (install_plan.rs:30-72), and nothing outside this file builds these structs with a literal.
  - Every other helper is private. No new pub item in `install` may be named `json`, `c`, `p`, `H` or `full` (bones glob guard; install_plan.rs:3 globs `install::*`).
- `paths` is unused in the bodies (name it `_paths`); the plans carry the home.
- **If** WP-01 replaces `install::InstallReport{steps}` with `pub use crate::api::InstallReport;` (`{written, ran, summary}`, base api.rs:255; see the cross note), fill `written` = the "wrote" paths, `ran` = the "ran" argv lines, `summary` = the notes joined by "; ". The step lines below stay the source.

**Behaviour**
- `unit_text_with(&UnitSpec::default())` is exactly:
  `"[Unit]\nDescription=Flat to 3D headset link\nStartLimitIntervalSec=60\nStartLimitBurst=10\n\n[Service]\nType=simple\nExecStart=%h/.local/bin/flat3d-inject watch\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n"`
  - `Description`, `Type`, `Restart`, `RestartSec=3` and `WantedBy` are fixed text; only `ExecStart` and the two `StartLimit*` values come from the spec. `StartLimit*` belong in `[Unit]`.
  - No `KillMode` (the default `control-group` is documented by WP-02 in packaging/README). No linger.
  - `unit_text`, `unit_path`, `bin_dir`, `UNIT_NAME` and the existing fields of `install_plan`/`uninstall_plan` stay as they are (locked install_plan.rs:14-72).
- `packaging/flat3d-link.service` is byte-equal to `unit_text_with(&UnitSpec::default())`.
- `install_plan(home, src)` also fills:
  - `dirs = [{home}/.local/bin, {home}/.local/share/flat3d/bin, {home}/.local/share/flat3d/recipes, {home}/.config/systemd/user]`;
  - `trees = [({src}/tools/lib, {home}/.local/share/flat3d/bin/lib), ({src}/recipes, {home}/.local/share/flat3d/recipes)]`;
  - `manifest = {home}/.local/share/flat3d/installed.txt`.
- `uninstall_plan(home)` also fills `home` and the same `manifest`.
- `execute`, in this order:
  1. Check that both binary sources exist (`copies` whose file name is `flat3d-inject` or `flat3d-run`). A missing one → `Err(ApiError::new("io", "missing <src path>"))`, before anything is written or run.
  2. `fs::create_dir_all` on each of `dirs`. An io error → `Err(ApiError::new("io", "<path>: <err>"))` (same for every fs error below).
  3. Copy the binaries (`fs::copy`), mode `0o755`. Step `"wrote <dst>"`.
  4. Copy the other `copies` (tools). A missing source is skipped. The skipped names go in one step `"not bundled: adb, gnirehtet"` (file names, plan order; ruling 34: the running adb is preferred). `adb` and `gnirehtet` get `0o755`, `gnirehtet.apk` `0o644`.
  5. For each tree whose source dir exists: create the destination dir, copy the regular files one level deep (no recursion). For the recipes tree copy only `*.recipe`. Lib files `0o755`, recipes `0o644`.
  6. Write `unit_text_with(&UnitSpec::default())` to `plan.unit_path` atomically: write `{unit_path}.tmp`, then `fs::rename`. Never `plan.unit_text` (critique:34).
  7. Write the manifest: every destination file written in steps 3-6, one absolute path per line, `\n`-terminated. Step `"wrote <manifest>"`.
  8. Run `plan.commands` in order. Step `"ran <argv joined by ' '>"` per command. A non-success → `Err(ApiError::new("command-failed", "<argv joined>: <last non-empty stderr line>"))`; the files stay in place and later commands do not run.
  9. Return `InstallReport{steps}`.
- `execute_uninstall`:
  1. Run each of `before`. A non-success is tolerated (the unit may not exist): step `"tolerated <argv joined>: <last stderr line>"`; otherwise `"ran <argv>"`.
  2. The removal set is `removes`, then each manifest line (if the manifest exists), then the manifest itself. Each path is removed with `fs::remove_file`; `NotFound` is ignored; another error → `io`. Step `"removed <path>"` per actual removal.
  3. A path is removed only when it has no `..` component and starts with one of `{home}/.local/bin/`, `{home}/.local/share/flat3d/`, `{home}/.config/systemd/user/`. Any other path (this includes `{home}/.config/flat3d/…` and `{home}/.local/state/…`) is skipped with step `"refused <path>"`.
  4. Run each of `after`; a non-success → `command-failed` as in execute.
  5. Return `InstallReport{steps}`.
- `add_wrapper_at(options, wrapper_abs)`:
  - If `wrapper_abs` contains whitespace or `"`, or its last `/` segment is not `WRAPPER` (`flat3d-run`) → return `add_wrapper(options)` (bare fallback, so `remove_wrapper` still finds it).
  - If `has_wrapper(options)`: using the same `words()` split, replace the first wrapper word before `%command%` (or before the end when there is no `%command%`) with `wrapper_abs`, drop any further wrapper words before that point, and join with `" "`. This upgrades bare to absolute and is idempotent.
  - Otherwise do exactly what `add_wrapper` does, with `wrapper_abs` in place of `WRAPPER`: after any leading env words, or `wrapper_abs %command% <rest>` when `%command%` is absent.
  - `remove_wrapper` is unchanged: `is_wrapper` already matches the last path segment (base launch_options.rs:51-53; locked tests/launch_options.rs:50-52).
  - `add_wrapper`, `has_wrapper`, `remove_wrapper`, `WRAPPER` keep their bodies (locked launch_options.rs, unlocked launch_options_quotes.rs).

**Tests.** The locked `crates/engine/tests/install_plan.rs` and `launch_options.rs` and the unlocked `launch_options_quotes.rs` stay green and unedited. New:
- `crates/engine/tests/install_execute.rs`. Each test makes its own dir `{env!("CARGO_TARGET_TMPDIR")}/install_execute/<test name>` (remove it first), with `home/` and `src/` inside. `src/` holds fake files `flat3d-inject`, `flat3d-run`, and (unless the test says otherwise) `tools/adb`, `tools/gnirehtet`, `tools/gnirehtet.apk`, `tools/lib/libusb.so`, `recipes/1-a.recipe`, `recipes/notes.txt`. A local `struct Fake { calls: Vec<Vec<String>>, fail_on: Option<String> }` implements `flat3d_inject::exec::CommandRunner`; it records argv and returns `success=false, stderr="boom\n"` when the joined argv contains `fail_on`. `EnginePaths` comes from the §2 frozen `EnginePaths::from_env(&BTreeMap::from([("HOME".to_string(), home)]), 1000).unwrap()`.
  - `default_unit_text_is_exactly_this` (the string above);
  - `packaging_unit_file_equals_unit_text_with_default` (`include_str!("../../../packaging/flat3d-link.service")`);
  - `custom_exec_start`: `UnitSpec{exec_start:"/x/y watch".into(),..Default::default()}` gives `ExecStart=/x/y watch`, and `start_limit_burst: 5` gives `StartLimitBurst=5`;
  - `execute_copies_chmods_writes_unit_then_reloads_and_enables`: calls are exactly `[systemctl --user daemon-reload]`, then `[systemctl --user enable --now flat3d-link.service]`; `{home}/.local/bin/flat3d-run` mode `& 0o777 == 0o755`; `gnirehtet.apk` is `0o644`; the unit file equals `unit_text_with(&UnitSpec::default())`; `lib/libusb.so` and `recipes/1-a.recipe` are copied and `notes.txt` is not; the manifest lists every copied file;
  - `missing_tools_are_skipped_and_named`: no `tools/adb`, no `tools/gnirehtet` → Ok, a step equals `"not bundled: adb, gnirehtet"`;
  - `missing_binary_is_io_and_runs_nothing`: no `flat3d-run` → `Err` with `code == "io"`, `calls` empty, unit file absent;
  - `systemctl_failure_is_command_failed`: `fail_on = Some("enable".into())` → `Err` with `code == "command-failed"` and message ending `": boom"`; the unit file is still there;
  - `execute_twice_is_fine`: run twice → both Ok, same file set;
  - `uninstall_tolerates_missing_unit_and_removes_manifest_files`: after an execute, `fail_on = Some("disable".into())` → Ok; every manifest path, the manifest and the unit are gone; the last call is `[systemctl --user daemon-reload]`;
  - `uninstall_keeps_config_and_state`: create `{home}/.config/flat3d/settings` and `{home}/.local/state/flat3d/x`, append both paths to the manifest → Ok, both files still exist, steps contain `"refused <path>"` for each;
  - `nothing_mentions_linger`: over install and uninstall, no recorded argv contains `loginctl` or `enable-linger`, and the unit text has no `Linger`.
- `crates/engine/tests/launch_options_at.rs`, `const A: &str = "~/.local/bin/flat3d-run";` (input → output of `add_wrapper_at(x, A)`):
  - `""` → `A %command%`; `"%command%"` → `A %command%`;
  - `"mangohud %command% -novid"` → `A mangohud %command% -novid`;
  - `"FOO=1 BAR=2 %command%"` → `FOO=1 BAR=2 A %command%`;
  - `"-novid"` → `A %command% -novid`;
  - `"flat3d-run %command%"` → `A %command%`; `"FOO=1 flat3d-run %command%"` → `FOO=1 A %command%`; `"/opt/x/flat3d-run %command%"` → `A %command%`; `"flat3d-run flat3d-run %command%"` → `A %command%`;
  - idempotent: `add_wrapper_at(&add_wrapper_at(x, A), A) == add_wrapper_at(x, A)` for every input above;
  - inverse: for each `before` in the locked PAIRS inputs (`""`, `"%command%"`, `"mangohud %command%"`, `"-novid"`, `"FOO=1 BAR=2 %command%"`, `"FOO=1 mangohud %command% -x"`, `"gamemoderun %command% -windowed"`), `remove_wrapper(&add_wrapper_at(before, A))` equals `""` for `"%command%"` and `before` otherwise;
  - quotes: `%command% -arg "a b"` → `A %command% -arg "a b"`; `"%command% flat3d-run"` → `A %command% flat3d-run`;
  - fallbacks: `add_wrapper_at("%command%", "/home/a b/.local/bin/flat3d-run")` → `flat3d-run %command%`; `add_wrapper_at("%command%", "/usr/bin/other")` → `flat3d-run %command%`.

**Inline facts:**
- `UNIT_NAME = "flat3d-link.service"` (install.rs:3).
- `tools_dir(home) = {home}/.local/share/flat3d/bin` (`crate::tether::adb::tools_dir`, base tether/adb.rs:30-32).
- The user manager has `XDG_RUNTIME_DIR=/run/user/1000`.
- Ruling 34: prefer the running adb.
- No network. No device in the tests; they never run real `systemctl`.

**Depends on:** WP-01 (wave 1). **Owner device:** Q3 (`sessions/link-smoke.toml`: `flat3d-inject install --json`, then `systemctl --user is-active flat3d-link.service`).

**Done-check** (`T=<agent scratch>/target`):
- `CARGO_TARGET_DIR=$T cargo test --locked -p flat3d-inject --test install_execute --test launch_options_at --test install_plan --test launch_options --test launch_options_quotes`
- `CARGO_TARGET_DIR=$T cargo clippy --locked -p flat3d-inject --all-targets -- -D warnings` and `cargo fmt --all --check`
- `git diff --exit-code Cargo.lock`

- **Note:** WP-08 (wave 3) and WP-28/WP-29 (wave 3) call these bodies after this WP merges. They must use the WP-01 forms above: `execute(&InstallPlan, …, &EnginePaths)` and `execute_uninstall(&UninstallPlan, …, &EnginePaths)`, no `install::Plan` enum.
- **Note:** the old scope said `execute(..)->InstallReport`. It returns `Result<InstallReport, ApiError>` because a copy or systemctl failure needs an error code (`io`, `command-failed`, both in `ERROR_CODES`).

---
