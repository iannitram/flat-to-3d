# Proposed replacement for split/wp/WP-29.md (plan mode blocked the edit)

Apply by replacing the whole of
roadmap-v2.md (WP-29 section)
with the block below.

---

### WP-29: doctor + setup commands (wave 4)

**Closes:** T13, T14, gap D8: the doctor and setup parts of the CLI (`AnyCommand::{Doctor,Setup}`).

**Wave:** 4. It depends on WP-27 (wave 3, `cli::setup_fetch::setup_fetch`), so it cannot be in wave 3. Same-wave WPs: WP-31 (`wrapper.rs`, `bin/flat3d-run.rs`, `tests/wrapper_route.rs`) and WP-32 (`decky/src/*`). No file overlap. WP-29 only *reads* wrapper.rs:239-247 for the env pairs. WP-33 depends on WP-29, so it moves to wave 5 (cross item).

**Files (exclusive; the index must list exactly these):**
- `crates/engine/src/cli/doctor.rs`: the `run` body, the 9 checks and printing. All helpers are private to this file. There is no `src/doctor.rs`, because WP-01 adds no `doctor` module at the crate root and only WP-01 may add a `mod` line.
- `crates/engine/src/cli/setup.rs`: the `run` body for `packages`, `install-apps` and `launch-env`. It hands `fetch` to `crate::cli::setup_fetch::setup_fetch` (WP-27).
- `crates/engine/tests/cli_doctor.rs` (new)
- `crates/engine/tests/cli_setup.rs` (new)

Nothing else. `setup.rs` (crate root), `wrapper.rs`, `api.rs`, `cli.rs`, `Cargo.*` and locked tests are not touched.

**Signatures (frozen by WP-01, bones §2; WP-29 fills the bodies only):**
```rust
// cli/doctor.rs
pub fn run(cmd:&DoctorCommand, ctx:&mut Ctx<'_>)->Result<serde_json::Value,ApiError>;
// cli/setup.rs
pub fn run(cmd:&SetupCommand, ctx:&mut Ctx<'_>)->Result<serde_json::Value,ApiError>;
// Ctx (cli.rs, bones §2): Ctx<'a>{paths:&'a EnginePaths, runner:&'a mut dyn CommandRunner, relay:&'a mut dyn RelayRunner, json:bool}
```
Enums as WP-01 freezes them (parsing is WP-01's, in `parse_any`): `DoctorCommand::Run` (argv `doctor [--json]`); `SetupCommand::{Packages, InstallApps, LaunchEnv{appid:Option<u32>}, Fetch{key:String}}`. Bad argv (`launch-env abc`, `fetch` with no key) is a `usage` ApiError → exit 2.

Private helpers (names are free; keep them private, bones §2 "Helpers stay private"):
- `fn doctor(p:&EnginePaths, r:&mut dyn CommandRunner, serial:Option<&str>)->api::DoctorReport`
- `fn adb_serial(r)->Option<String>`: runs `["adb","devices"]`; the first line `<serial>\tdevice` gives the serial.
- `fn packages(r)`, `fn install_apps(p,r)->Result<api::SetupReport,ApiError>`, `fn launch_env(appid,p,r)->Result<api::LaunchText,ApiError>`.

Calls into frozen or existing code:
- `crate::link::wivrn::{detect_server, detect_quest_app}` (bones §2: `detect_server(r)->Result<WivrnDirs,ApiError>`, `detect_quest_app(r, serial:Option<&str>)->Option<QuestApp>`).
- `crate::tether::adb::{tools_dir, bundled_adb, choose_adb, proc_exe_path}` (adb.rs:26-63), `crate::api::AdbSource`.
- `crate::setup::{packages_command, parse_packages, missing, WIVRN_PACKAGE, GNIREHTET_PACKAGE}` (setup.rs:4-45).
- `crate::install::UNIT_NAME`, `crate::steam::DEFAULT_STEAM_ROOT`, `crate::link::WIVRN_FLATPAK`.
- `crate::injectors::fetch::{parse_pins, pin_in, cache_dir, PINS, MARKER}` (WP-13).
- `crate::cli::setup_fetch::setup_fetch(key:&str, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::SetupReport,ApiError>` (WP-27).
- `EnginePaths` accessors `cache()`, `pins()`, `wrapper_bin()` and fields `home`, `data` (WP-01).

**DTO shapes** (WP-01 `api.rs`; they must match WP-21 `decky/src/types.ts`):
- `DoctorCheck{name:String, ok:bool, detail:String, fix:String}`, where `fix` is `""` when ok;
- `DoctorReport{ok:bool, checks:Vec<DoctorCheck>}`;
- `SetupReport{steps:Vec<String>, summary:String}`;
- `LaunchText{options:String}`.

**Pins source:** `pins_text(p)` = `fs::read_to_string(p.pins())` when that file exists, else `fetch::PINS`. Then `fetch::parse_pins`. The doctor `pins` check and the APK paths both use it.

**Behaviour:** `cli::doctor::run` gets `serial = adb_serial(ctx.runner)` and calls `doctor(ctx.paths, ctx.runner, serial.as_deref())`. It returns `serde_json::to_value(report)`. Without `--json` it prints one line per check (`✓|✗ name: detail`, plus `  fix: <fix>` when red). `doctor` runs these checks in this order. Each check is a `DoctorCheck`, and the report's `ok` is the AND of all of them:
1. **`wivrn-server`:** `detect_server(r)`. If Ok, the detail is the dirs root. If Err, the detail is the message. Fix: `flatpak install --user flathub io.github.wivrn.wivrn`.
2. **`quest-app`:** with no serial, ok:true and detail `no headset`. Otherwise `detect_quest_app(r, Some(serial))`: Some → ok, detail `<package>`; None → red, fix `install WiVRn on the headset: flat3d-inject setup install-apps`.
3. **`adb`:** `running` = `pgrep -x adb` through the runner, first pid, then `fs::read_link(proc_exe_path(pid))` (as in watch.rs:849); `bundled` = `bundled_adb(home)` only if that file exists. Then `choose_adb(running, bundled)`. The detail is the source (`running`/`bundled`/`missing`). Red only on `Missing`, fix `install the bundled tools (packaging/build-tools.sh) or start adb`.
4. **`gnirehtet`:** two parts, both must exist:
   - the relay `{tools_dir(home)}/gnirehtet`;
   - the APK `fetch::cache_dir(&pin_in(&pins,"gnirehtet")?, p)/gnirehtet-rust-linux64/gnirehtet.apk`. WP-13 unpacks with `bsdtar -xf` and keeps the zip's top folder.
   - Fix: `flat3d-inject setup fetch gnirehtet`.
5. **`link-service`:** `["systemctl","--user","is-active",UNIT_NAME]`. Ok when the trimmed stdout is `active`. Fix: `systemctl --user enable --now flat3d-link.service`.
6. **`wrapper`:** `p.wrapper_bin()` exists and has a `0o111` mode bit. Fix: `flat3d-inject install`.
7. **`steam`:** `{home}/{DEFAULT_STEAM_ROOT}/steamapps/libraryfolders.vdf` exists. Fix: `start Steam once`.
8. **`pins`:** no pin's `sha256` equals `TBD-at-pin`. The detail lists those keys, comma-separated, in BTreeMap order. Fix: `run packaging/pin.sh <key>` (first key).
9. **`injector`:** `p.data.join("bin/flat3d-uevr-inject.exe")` exists (WP-15 `--install` target). Fix: `tools/build-win.sh --install`.

Doctor never writes anything and never fails as a whole: each check catches its own error into `detail`.

Setup commands (`cli::setup::run`):
- **`setup packages`:** `packages_command()` → stdout → `parse_packages`. It returns `json!({"wivrn":p.wivrn,"gnirehtet":p.gnirehtet,"missing":missing(&p)})`. `Packages` has no `Serialize` (setup.rs:17), so the JSON is built with `json!`. A runner failure (`success==false`) is `command-failed` with the stderr.
- **`setup install-apps`:** runs `packages` first. Then, for each missing name in order:
  - `WiVRn` → key `wivrn-apk`; the APK is the single `*.apk` file directly inside `cache_dir(pin)`;
  - `gnirehtet` → key `gnirehtet`; the APK is `cache_dir(pin)/gnirehtet-rust-linux64/gnirehtet.apk`. There is no `gnirehtet-apk` pin.
  - It calls `setup_fetch(key, p, r)?`, which runs no command when `<cache_dir>/.flat3d-fetch` holds `<sha256>\n`. Then it runs `["adb","install","-r",<apk>]`. A failure is `command-failed` with the last stderr line.
  - Each fetch and install adds one line to `steps`. The summary is `installed WiVRn, gnirehtet` (the names done); when nothing is missing, steps is empty and the summary is `nothing to install`.
- **`setup launch-env`** (returns `LaunchText`):
  - With no appid: `detect_server(r)?` gives `root`. `options` = `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 PRESSURE_VESSEL_FILESYSTEMS_RW=<root> STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%`. These are the same 3 pairs as `exec_spec` (wrapper.rs:238-247), and the folder comes from `detect_server`, so user and system installs both work. WP-24 step0 uses this form.
  - With an appid: `options` = `"<p.wrapper_bin() absolute> %command%"`.
- **`setup fetch <key>`:** `serde_json::to_value(setup_fetch(key, ctx.paths, ctx.runner)?)`.

**Tests (FakeRunner only, no network, no device).**
Harness: a temp HOME under `env!("CARGO_TARGET_TMPDIR")`, with `EnginePaths::from_env(&env, 1000)` (bones §2 form). `ctx = Ctx{paths:&p, runner:&mut fake, relay:&mut fake_relay, json:true}`, calling `flat3d_inject::cli::{doctor,setup}::run`. Then `serde_json::from_value::<DoctorReport|SetupReport|LaunchText>`. The runner is `flat3d_inject::testing::FakeRunner`, scripted by argv prefix with its argv log. Seeded files: `p.pins()` with `gnirehtet.version: 2.5.1`, `gnirehtet.url: file:///nonexistent/gnirehtet-rust-linux64-v2.5.1.zip`, `gnirehtet.size: 1`, `gnirehtet.sha256: dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275`.

- **`cli_doctor.rs`:**
  1. `all_green`: these are seeded:
     - the relay, the bundled adb, `cache/gnirehtet/2.5.1/gnirehtet-rust-linux64/gnirehtet.apk` + marker;
     - the wrapper (mode 0o755), `libraryfolders.vdf`, `data/bin/flat3d-uevr-inject.exe`.
     
     These are scripted:
     - `adb devices` → `List of devices attached\nSERIAL1\tdevice\n`;
     - `flatpak info --show-location --user io.github.wivrn.wivrn` → `<home>/.local/share/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc\n`;
     - `adb -s SERIAL1 shell dumpsys package org.meumeu.wivrn.github` → `    versionName=26.9\n`;
     - `systemctl --user is-active flat3d-link.service` → `active\n`;
     - `pgrep` fails.
     
     The result is `ok:true` and 9 checks named in order `wivrn-server quest-app adb gnirehtet link-service wrapper steam pins injector`.
  2. `link_service_inactive`: as 1, but systemctl prints `inactive\n` and exits 3. Only `link-service` is red, and its `fix` contains `systemctl --user enable --now flat3d-link.service`.
  3. `tbd_pin_is_red`: as 1, plus `geo11.sha256: TBD-at-pin` (with version/url/size) in the pins file. `pins` is red and its detail contains `geo11`.
  4. `apk_missing_is_red`: as 1, without the APK file. `gnirehtet` is red, and its fix is `flat3d-inject setup fetch gnirehtet`.
  5. `no_serial_skips_quest`: `adb devices` → header only. `quest-app` is ok with detail `no headset`, and no `dumpsys` argv is in the log.
  6. `report_round_trips`: `serde_json::to_string(&report)` → `from_str::<DoctorReport>` equals the report.
  7. `doctor_writes_nothing`: the list of files under HOME is the same before and after.
- **`cli_setup.rs`:**
  1. `packages_missing_gnirehtet`: `adb shell pm list packages` → `package:org.meumeu.wivrn.github\n` → `{"wivrn":true,"gnirehtet":false,"missing":["gnirehtet"]}`.
  2. `install_apps_uses_cached_apk`: the same packages output, plus a seeded gnirehtet dir and its marker (`<sha>\n`). The log has no argv starting with `curl`, and exactly one `["adb","install","-r","<cache>/gnirehtet/2.5.1/gnirehtet-rust-linux64/gnirehtet.apk"]`. The summary is `installed gnirehtet`.
  3. `install_fails_command_failed`: as 2, but `adb install` returns success=false with stderr `Failure [INSTALL_FAILED]`. The result is `Err` with code `command-failed`.
  4. `nothing_to_install`: both packages present → steps empty, summary `nothing to install`, and no `adb install`.
  5. `launch_env_base`: the scripted user flatpak location. `options` contains `PRESSURE_VESSEL_FILESYSTEMS_RW=<home>/.local/share/flatpak/app/io.github.wivrn.wivrn` and ends with `%command%`.
  6. `launch_env_appid`: `LaunchEnv{appid:Some(5)}` → `options == format!("{} %command%", p.wrapper_bin().display())`.
  7. `launch_env_bad_appid_usage`: `Command::new(env!("CARGO_BIN_EXE_flat3d-inject")).args(["setup","launch-env","abc","--json"])` → exit 2, and the stdout JSON has `error.code == "usage"`. Exit 2 alone would also pass for `not-built` (cli.rs:93-97), so the code is asserted too.
  8. `setup_fetch_no_key_usage`: binary `setup fetch --json` → exit 2, `error.code == "usage"`. No network: parsing fails before any runner call.

**Inline facts:**
- `WIVRN_PACKAGE="org.meumeu.wivrn.github"` and `GNIREHTET_PACKAGE="com.genymobile.gnirehtet"` (setup.rs:4,6; locked `tests/setup_packages.rs`).
- `WIVRN_FLATPAK="io.github.wivrn.wivrn"` (link.rs:7); `UNIT_NAME="flat3d-link.service"` (install.rs:3); `DEFAULT_STEAM_ROOT=".local/share/Steam"` (steam.rs:10).
- `tools_dir(home)="{home}/.local/share/flat3d/bin"`, `bundled_adb = tools_dir/adb` (adb.rs:30-36).
- The gnirehtet pin is `gnirehtet-rust-linux64-v2.5.1.zip`, sha256 `dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275`. The relay and the APK come from the same tag.
- `ERROR_CODES` stays the 8 locked codes; WP-29 uses `usage`, `command-failed`, `not-found`, `refused`, `io` only.

**Flags:**
- `Flag:` the server pin is 26.9 and the Quest versionName must match it. Bones §2 `QuestApp{package,label}` and `WivrnDirs{scope,root}` carry no versions, so the doctor does not compare them. The match is checked by hand in Q4 (`adb shell dumpsys package org.meumeu.wivrn.github | grep versionName`). If the owner reconciles to WP-12's `QuestApp.version` + `versions_match`, add it to check 2.
- `Flag:` wrapper.rs:229 hard-codes the user flatpak path. The doctor and `launch-env` use `detect_server`, so they may disagree on system-flatpak installs until WP-31 lands (same wave, different file).

**Depends on:** WP-01, WP-12 (`link::wivrn`), WP-13 (`fetch`), WP-15 (injector install path, text only), WP-20 (unit/wrapper install), WP-22 (gnirehtet layout), WP-27 (`setup_fetch`). All are in waves ≤ 3, so this is **wave 4**.

**Owner device:** none for the tests (fakes); Q4 runs `flat3d-inject doctor --json` on the Deck with the Quest on USB.

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test cli_doctor --test cli_setup --test setup_packages --test cli_json && tools/check.sh`

---
