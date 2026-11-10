# WP-10 recheck: replacement text for scratchpad/split/wp/WP-10.md

Plan mode blocked the edit. Paste the block below over the whole of WP-10.md.

---

### WP-10: Tether plans, runner errors, Wi-Fi sync
- **Closes:** executing the plan steps, idempotent APK install, actionable runner errors (K15), the Wi-Fi sync entry point
- **Files (exclusive):**
  - `crates/engine/src/tether.rs` (was `tether/mod.rs`; WP-01 git-mv'd it. No new `mod` lines.)
  - `crates/engine/src/tether/plan.rs`
  - `crates/engine/src/tether/wifi.rs` (owned; no new pub item is needed, the existing fns are used)
  - `crates/engine/tests/tether_plan_run.rs` (new)
  - `crates/engine/tests/tether_runtime.rs` (unheadered, edited with a recorded reason)
- **Body-only edit in a frozen file:** `crates/engine/src/exec.rs`. Change only the body of `impl RelayRunner for ProcessRunner { fn run }` and add private (non-`pub`) helpers `tail_lines(stderr:&str, n:usize)->String` and `exit_text(status:&ExitStatus)->String`. No signature, trait, re-export or `SystemRunner` change.
- **Read-only uses (do not edit):** `tether/adb.rs` (`TetherAdb`, `reverse_list_command`, `install_command`, `remove_reverse_command`), `tether/relay.rs` (`relay_command`, `relay_path`, `apk_path`, `client_start_commands`, `client_stop_command`, `CLIENT_PACKAGE`, `tun0_present`), `tether/stepaside.rs` (`parse_ps`, `tether_mode`), `tether/wifi.rs` fns, `link::{start_3d_commands, start_3d_commands_with, headset_from_adb}`, `link::wivrn::WivrnStart` (§2 frozen; body WP-12).
- **Signatures (all new items live in WP-10's own files):**
  - `plan.rs`: `#[derive(Debug,Clone,Default,PartialEq,Eq)] pub struct PlanCtx{ pub relay:Vec<String>, pub adb:String }`
  - `plan.rs`: `#[derive(Debug,Clone,Default,PartialEq,Eq)] pub struct PlanRun{ pub ran:Vec<Vec<String>>, pub tun0:bool, pub spawned:bool }`. Callers build it with `PlanRun::default()`.
  - `plan.rs`: `pub fn run_plan_in(steps:&[TetherStep], ctx:&PlanCtx, r:&mut dyn RelayRunner, sleep:&mut dyn FnMut(u64), out:&mut PlanRun) -> Result<(),String>`
  - `plan.rs`: `pub fn run_plan(steps:&[TetherStep], r:&mut dyn RelayRunner, sleep:&mut dyn FnMut(u64)) -> Result<(),String>`
  - `plan.rs`: `pub fn tun0_command() -> Vec<String>` = `["adb","shell","ip","addr","show","tun0"]`; `pub fn ps_command() -> Vec<String>` = `["ps","-eo","pid=,args="]`.
  - `tether.rs`: `Tether::set_sleep(&mut self, sleep:Box<dyn FnMut(u64)>)`; the default (set in `new`) is `|ms| std::thread::sleep(Duration::from_millis(ms))`.
  - `tether.rs`: `Tether::set_wivrn(&mut self, start:&WivrnStart)`. It stores `link::start_3d_commands_with(start)` (a `Vec<Vec<String>>`, the same shape as the locked `start_3d_commands()`), so `WivrnStart` needs no `Clone`. Until it is called, connect uses the locked `link::start_3d_commands()`.
  - `tether.rs`: `Tether::sync_wifi(&mut self) -> wifi::SyncDecision`, `Tether::last_wifi(&self) -> Option<&wifi::SyncDecision>`, `Tether::last_wifi_log(&self) -> Option<&str>` (the `redact_command` line of the last connect attempt).
  - `RelayRunner` is imported as `crate::exec::RelayRunner` (in `tether.rs` the existing `adb::CommandRunner` alias still names it). `Tether<R: adb::CommandRunner>` keeps its bound, `new` signature and `pub adb` field.
- **Behaviour:**
  - **`run_plan_in`** runs the steps in order and appends every argv it passes to `r.run` (before any adb rewrite) to `out.ran`. A tun0 probe is one `r.run(&tun0_command())`; present = `Ok(o)` with `relay::tun0_present(&o)`, and an `Err` counts as absent. Per `TetherStep` (plan.rs:8):
    - `Run(c)`: `r.run(c)?`.
    - `SleepMs(ms)`: `sleep(ms)`.
    - `IfVpnUp(c)`: one probe; runs `c` (`?`) only when present.
    - `IfNoTun0(cs)`: one probe; runs each of `cs` (`?`) only when absent.
    - `WaitTun0{tries,every_ms}`: up to `tries` probes, `sleep(every_ms)` between two probes (not after the last, not after a hit); sets `out.tun0`; never errors.
    - `WaitWifi{tries,every_ms}`: same loop with `wifi::quest_ssid_command()` until `wifi::parse_quest_ssid` is `Some`; never errors.
    - `KillStaleRelay`: when `ctx.relay` is empty, does nothing. Else `r.run(&ps_command())?`, `stepaside::parse_ps`, and for each `(pid, args)` whose first whitespace word equals `ctx.relay[0]`, `r.run(["kill", pid])`, ignoring its error.
    - `SpawnRelay`: empty `ctx.relay` → `Err("No relay command")`. Else `r.start_relay(&ctx.relay, &ctx.adb)?` and `out.spawned = true`.
    - `CheckRelayAlive`: empty `ctx.relay` → `Err("No relay command")`. Else `r.run(&ps_command())?`; if no `parse_ps` entry has `args == ctx.relay.join(" ")`, `Err("Tether relay exited during startup")`.
    - `KillRelay`: `r.stop_relay()?`.
  - **`run_plan`** is `run_plan_in(steps, &PlanCtx::default(), r, sleep, &mut PlanRun::default())`.
  - **Inside `Tether`**, the plan runs through a private adapter in `tether.rs` (`struct Via<'a,R>(&'a mut TetherAdb<R>)`, `impl<R: adb::CommandRunner> RelayRunner for Via<'_,R>`): `run` → `TetherAdb::run` (so `adb` becomes `<path> -s <serial>`), `start_relay(c,_)` → `TetherAdb::start_relay(c)` (it supplies the real adb path), `stop_relay` → `TetherAdb::stop_relay`. The ctx is `PlanCtx{relay: relay::relay_command(&relay::relay_path(&home), &serial), adb: String::new()}`. `&mut *self.sleep` and `&mut self.adb` are disjoint field borrows.
  - **`connect` / `connect_tunnel_only`** keep their signatures, the "already active" and "No adb" checks, and the reverse bookkeeping in `run_connect_command`. Steps:
    1. `reverse --list` (as today);
    2. the WiVRn lists: the stored `set_wivrn` lists, else `link::start_3d_commands()`; `connect_tunnel_only` keeps only the first list (`adb devices`), as today; each goes through `run_connect_command`;
    3. Own only (`!foreign_relay`): `run_connect_command(client_start_commands()[0])` (reverse localabstract);
    4. Own only: `adb shell pm list packages com.genymobile.gnirehtet` (`?`); `install -r <apk_path(home)>` (`?`) only if no output line trims to `package:com.genymobile.gnirehtet`. `connect_tunnel_only` runs this too (today it installs too);
    5. Own only: `run_plan_in(&connect_plan(), ctx, Via, sleep, &mut out)`. The client `am start` now runs only through `IfNoTun0`. After the call (Ok or Err): `client_started = out.ran` contains `client_start_commands()[1]`.
    - Connect never calls `sync_wifi`; the caller (WP-09) does, after an Own connect.
  - **Connect rollback** (on any Err), in this order: client stop (and force-stop if that fails) when `client_started`; `adb.stop_relay()` when `out.spawned` (ignore its error); `cleanup_reverses()` (appending its error as today). Mode stays Off.
  - **`disconnect`, Own mode:** `run_plan_in(&disconnect_plan(), …)`. On Ok: reset like `drop_local` but without a second `stop_relay` (mode Off, `client_started=false`, `added_reverses` cleared). On Err: best effort, run `stop_relay` and `remove_reverse_command()` if `out` shows the plan did not reach them, return the first error, and leave the mode Own (as today). Off and QuestTether branches are unchanged.
  - **`sync_wifi`** (best effort; never returns Err, never touches the mode):
    1. `pc = run(active_ssid_command()).ok().and_then(parse_active_ssid)`; `quest = run(quest_ssid_command()).ok().and_then(parse_quest_ssid)`;
    2. `d = sync_decision(pc, quest, self.wifi_attempts)`;
    3. only on `Switch`: `run(credentials_command(pc))` → `parse_credentials`; `None` or an Err counts as a failed attempt. Else `argv = connect_network_command(ssid, sec, psk)`, `self.wifi_log = Some(redact_command(&argv))`, and `joined = run(&argv).is_ok()`; `wifi_attempts = next_attempts(prev, joined)`;
    4. store `d` for `last_wifi` and return it. The PSK never appears in a stored string or an error.
  - **`ProcessRunner::run` errors** (exec.rs, body only), prefixes kept for watch/service.rs and its tests:
    - non-zero exit: `Tether command failed: exit <code>` (or `exit signal <N>` via `std::os::unix::process::ExitStatusExt::signal`), then `; <the last 4 non-empty stderr lines, trimmed, joined " / ">` when stderr has any; the bare `Tether command failed: exit 1` when it has none;
    - spawn failure: `Could not run tether command: <io error>`;
    - `start_relay`/`stop_relay` messages are unchanged.
- **Tests:**
  - **Locked, unchanged:** every headered `crates/engine/tests/tether_*.rs` (`tether_adb_choice`, `tether_adb_runner`, `tether_heal`, `tether_plan`, `tether_relay_cmds`, `tether_stepaside`, `tether_tunnel`, `tether_wifi_cmds`, `tether_wifi_parse`, `tether_wifi_sync`) and the unheadered `tether_tunnel_review.rs`.
  - **Updated:** `crates/engine/tests/tether_runtime.rs`. It has no `//! Locked` header (line 1 is `use flat3d_inject::api::TetherMode;`), so R28 does not lock it. Record this reason in the PR body and CHANGELOG: "connect/disconnect now execute connect_plan/disconnect_plan (WP-10)". The edits:
    - imports stay (`tether::adb::{choose_adb, CommandRunner}` is the WP-01 alias of `RelayRunner`);
    - the `Fake` gains: `ps -eo pid=,args=` answers `4242 /fixture/.local/share/flat3d/bin/gnirehtet run S1` after `start_relay` and before `stop_relay`, else empty; `ip addr show tun0` answers `    inet 10.0.0.2/24` while a `tun0: bool` field (default true) holds, set false when the client STOP is recorded; `cmd wifi status` answers `Wifi is connected to "home"`; `pm list packages` answers empty;
    - `Tether::set_sleep(Box::new(|_| {}))` in `tether()`;
    - the Own connect order becomes WIVRN, then `reverse localabstract:gnirehtet tcp:31416`, `shell pm list packages com.genymobile.gnirehtet`, `install -r …gnirehtet.apk`, `ps -eo pid=,args=`, `…/gnirehtet run S1`, `ps -eo pid=,args=`, then 3 × `shell ip addr show tun0` (WaitTun0 hit, IfNoTun0 probe, WaitTun0 hit);
    - disconnect follows `disconnect_plan()`: STOP, tun0 probe (now absent, no force-stop), `svc wifi disable`, `svc wifi enable`, `cmd wifi status`, `stop-owned-relay`, REMOVE;
    - the `fail_at` indices are renumbered: the failing steps are 1..=11 (the tun0 probes never fail the connect), and the relay-start failure no longer expects STOP (the client has not started yet).
  - **New:** `crates/engine/tests/tether_plan_run.rs`. It defines its own local `Fake` implementing `flat3d_inject::tether::adb::CommandRunner` (all 3 methods; scripted replies by argv prefix, a call log) and does **not** depend on `testing::FakeRunner`'s API. Sleeps are recorded in a `Vec<u64>`.
    1. A plan holding every `TetherStep` variant once runs each once, in order (`out.ran` and the fake's log).
    2. `WaitTun0{tries:5,every_ms:1500}` with tun0 appearing on the 3rd probe: 3 probes, sleeps `[1500,1500]`, `out.tun0 == true`; never appearing: 5 probes, 4 sleeps, Ok, `out.tun0 == false`.
    3. `IfNoTun0` is skipped when tun0 is present and runs when absent.
    4. A ps without the relay line at `CheckRelayAlive` gives `Err("Tether relay exited during startup")`; `KillStaleRelay` kills only pids whose first word is `ctx.relay[0]`.
    5. `Tether::connect(false)` with `pm list packages` answering `package:com.genymobile.gnirehtet` runs no `install`.
    6. `ProcessRunner::default().run(&["sh","-c","printf 'x\\ny\\na\\nb\\nc\\nd\\n' >&2; exit 1"])` gives `Err("Tether command failed: exit 1; a / b / c / d")`; `exit 1` with no stderr gives `Err("Tether command failed: exit 1")`; a missing program gives an Err starting `Could not run tether command: `. (Uses `/bin/sh`; no network, no device.)
    7. Wi-Fi: PC on `home` (`yes:home`), Quest on `other`, credentials `wpa-psk\nsecret pass`: `sync_wifi()` is `Switch`, the connect argv ran, and `last_wifi_log()` holds `home` but not `secret pass`. With the connect failing, the 4th call is `GiveUp` and runs no connect.
    8. `run_plan(&[TetherStep::SpawnRelay], …)` and `run_plan(&[TetherStep::CheckRelayAlive], …)` give `Err("No relay command")`.
    9. The local `impl flat3d_inject::tether::adb::CommandRunner for Fake` (all 3 methods) drives `Tether::connect(false)` to `Ok(TetherMode::Own)`.
    10. `set_wivrn(&start)` with a `WivrnStart` built from the §2 fields: the calls after `reverse --list` start with `link::start_3d_commands_with(&start)` rewritten to `/tools/adb -s S1 …` (compared against the function's own output, so the test does not depend on WP-12's exact lists).
- **Inline facts:**
  - `RELAY_PORT=31416`, `CLIENT_PACKAGE="com.genymobile.gnirehtet"` (relay.rs:5-6); gnirehtet v2.5.1.
  - The APK is `relay::apk_path(home)` = `<home>/.local/share/flat3d/bin/gnirehtet.apk`.
  - `MAX_WIFI_ATTEMPTS=3`; Quest SSID text is `Wifi is connected to "NAME"`.
  - `connect_plan`: KillStale, Spawn, Sleep 1500, Check, WaitTun0 15×1500, IfNoTun0(client start), WaitTun0 8×1500. `disconnect_plan`: STOP, Sleep 1500, IfVpnUp(force-stop), wifi disable, Sleep 4000, wifi enable, WaitWifi 6×1500, KillRelay, remove reverse.
  - `TetherAdb::run` rewrites `adb` to `<path> -s <serial>`; other argv (ps, kill, nmcli) pass through.
- **Flags:**
  - A fresh connect can sleep about 34 s (WaitTun0), longer than the 10 s snapshot staleness. WP-09 injects a heartbeat sleeper through `set_sleep`.
  - `last_wifi_log` is new so WP-09 can log the Wi-Fi attempt without ever holding the PSK.
- **Depends on:** WP-01, WP-12 (body of `link::start_3d_commands_with`, `link::wivrn::WivrnStart`) → **wave 3**. Same-wave WPs (27, 28, 29, 30) own no file listed here.
- **Owner device:** Q3 (link smoke). The pass itself needs no network and no device.
- **Done-check** (run from the WP branch checkout, not the read-only repo):
  ```
  export CARGO_TARGET_DIR=$SCRATCH/target
  export XDG_RUNTIME_DIR=$(mktemp -d)
  cargo test --locked -p flat3d-inject --test tether_plan_run --test tether_plan --test tether_runtime \
    --test tether_tunnel_review --test tether_adb_runner --test tether_wifi_sync --test tether_wifi_cmds \
  && cargo test --locked -p flat3d-inject tether \
  && git diff --exit-code main -- crates/engine/tests/tether_adb_choice.rs crates/engine/tests/tether_adb_runner.rs \
       crates/engine/tests/tether_heal.rs crates/engine/tests/tether_plan.rs crates/engine/tests/tether_relay_cmds.rs \
       crates/engine/tests/tether_stepaside.rs crates/engine/tests/tether_tunnel.rs crates/engine/tests/tether_tunnel_review.rs \
       crates/engine/tests/tether_wifi_cmds.rs crates/engine/tests/tether_wifi_parse.rs crates/engine/tests/tether_wifi_sync.rs \
  && tools/check.sh
  ```
  Plus a reviewer read of `git diff main -- crates/engine/src/exec.rs`: only `ProcessRunner::run`'s body and new private fns change.
