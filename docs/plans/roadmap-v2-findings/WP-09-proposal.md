# Proposed replacement text for split/wp/WP-09.md

Plan mode blocked the edit to WP-09.md. This is the full corrected text, ready to paste over the file.

---

### WP-09: Watch service correctness
- **Wave:** 4 (it calls WP-10's `Tether::{set_wivrn,set_sleep,last_wifi}`, and WP-10 is wave 3 because it depends on WP-12).
- **Closes:**
  - non-fatal tether failure with retry
  - the allow-debugging event
  - executing every heal action: RestartClient, RestartAdbThenClient, FullReconnect, GiveUp
  - production WiVRn start args
- **Files (exclusive):**
  - `crates/engine/src/watch/machine.rs`
  - `crates/engine/src/watch/service.rs`
  - `crates/engine/tests/watch_service.rs`
- **Uses (frozen by WP-01, bodies from WP-10/11/12; call exactly these shapes):**
  - `crate::wivrn::{WivrnStart{scope:Option<FlatpakScope>,app:String,flags:Vec<String>}, SERVER_FLAGS:[&str;2], QuestApp{package:String,version:Option<String>}, detect_quest_app(r:&mut dyn exec::CommandRunner, serial:&str)->Option<QuestApp>}` (WP-01 "WiVRn", WP-12 lines 14-31).
  - `crate::link::{start_3d_commands_with(&WivrnStart)->Vec<Vec<String>>, WIVRN_APP}`.
  - `Tether::set_wivrn(&mut self, app:String, flags:Vec<String>)`, `Tether::set_sleep(&mut self, Box<dyn FnMut(u64)>)` (milliseconds), `Tether::last_wifi(&self)->Option<&wifi::SyncDecision>` (WP-10). `Tether::connect` already runs `sync_wifi()` as its step 6, so WP-09 does **not** call `sync_wifi`.
  - `HealthTick.heal: Option<tether::heal::Heal>`. WP-01's `publish()` stores it. There is no `HealthAction`; read `tick.heal`.
  - `crate::tether::adb::AdbRunner::new(inner, adb:&str, serial:Option<&str>)` (adb.rs:171), `tether::relay::{client_stop_command (relay.rs:41), client_start_commands (relay.rs:20)}`, `stepaside::{foreign_relay, parse_ps}`.
  - `machine::note_event(n:Note)->EventKind` stays the frozen free `pub fn` (bones §2). WP-09 adds no method with that name.
- **New items (all inside WP-09's own files; the frozen `watch` facade is not touched):**
  - machine.rs: `pub(crate) fn link_commands_with(effect:&Effect, w:&crate::wivrn::WivrnStart) -> Vec<Vec<String>>`. StartLink gives `start_3d_commands_with(w)`, Reattach gives `start_3d_commands_with(w)[2..]`, and every other effect gives the same result as `link_commands`. The locked `link_commands` (watch.rs:262 today) stays byte-identical. It is not re-exported.
  - service.rs:
    - `impl<R,F> Service<R,F> { pub fn with_sleep(self, s:Box<dyn FnMut(u64)>) -> Self }`. It replaces only the raw sleeper, so it must be called before the first tick.
    - `pub fn heal(&mut self, now:u64, h:Heal)`. It is pub so tests can call it.
    - Private fields `tether_failed:bool`, `retry_at:u64`, `gave_up:bool`, `quest_app:Option<(String,String)>` (serial, package), `sleep:Rc<RefCell<Box<dyn FnMut(u64)>>>`, `last:Rc<RefCell<Option<LinkSnapshot>>>`, `slept_ms:Rc<Cell<u64>>`.
    - A private `struct AsLink<R>(Shared<R>)` that implements `exec::CommandRunner`. `Ok(s)` gives `CommandOutput{success:true,stdout:s,stderr:""}` and `Err(e)` gives `{success:false,stdout:"",stderr:e}`.
- **Behaviour:**
  - **Tether failure:**
    - In `execute`, `Err(message)` from `StartTether` keeps today's `log(now,"tether: {message}")`. It also pushes `ring.push(EventKind::TetherError, &message)` and sets `tether_failed=true`.
    - The link start in the same batch still runs (`wivrn_by_tether` stays false).
    - Retry: in `tick`, on a feed frame that `classify` reads as `Ready` while `self.ready` is already true, retry only when `settings.tunnel` is on, the tether mode is Off (or there is no tether), `tether_failed` is set and `now >= retry_at`. The retry calls `connect_tether(false)` once. Ok clears the flag. Err logs, pushes nothing, and sets `retry_at = now + MIN_HEAL_GAP_SECS` (30).
    - The error prefixes "Tether command failed" / "Could not run tether command" are kept.
  - **Unauthorised:** in the `Seen::Unauthorised` arm, when `self.unauthorised` was false (the move into the state), push `ring.push(note_event(Note::AllowUsbDebugging), note_text(Note::AllowUsbDebugging))`, which gives kind `allow-debugging` and the text "Put the headset on and allow USB debugging.". The push comes before `self.unauthorised = true`. There is no repeat while the device stays unauthorised.
  - **Heal.** At the end of `tick`, after `publish`, take the `heal` that WP-01's `publish()` stored. If it is Some, call `self.heal(now, h)`. If `gave_up` is set, `heal` returns at once.
    - `RestartClient`: `run_steps(now, &[client_stop_command()], false)`, then `run_steps(now, &client_start_commands(), false)`. Both go through `adb_run`/`TetherAdb`, which gives `<adb> -s <serial> …`.
    - `RestartAdbThenClient`: `self.runner.borrow_mut().run(&[adb.path,"kill-server"])`, then `[adb.path,"start-server"]`, run directly. `TetherAdb::run` always inserts `-s` (adb.rs:148-157 today), and only the link-shape `AdbRunner` exempts server commands (adb.rs:194-197). Then the `RestartClient` sequence.
    - `FullReconnect`: if there is a tether, call `tether.disconnect()` (Err is logged), then `connect_tether(false)`. `connect_tether` reads `foreign` via `stepaside::foreign_relay` (watch.rs:693 today). With no tether it only logs.
    - `GiveUp`: one `ring.push(EventKind::TetherError, "The USB tunnel kept dropping. Unplug the headset and plug it back in.")`, then `gave_up=true`. `gave_up` is cleared when `connect_tether` next returns Ok.
    - `None`: nothing.
    - Every failure is logged. No panic, no `unwrap` on runner output.
  - **WiVRn start:**
    - Before `StartLink`/`Reattach` and before `connect_tether(true)`, build `w = WivrnStart{scope:None, app, flags:SERVER_FLAGS.map(String::from).to_vec()}`.
    - `app` is `detect_quest_app(&mut AdbRunner::new(AsLink(Shared(Rc::clone(&self.runner))), &self.adb.path, Some(&self.serial)), &self.serial).map(|q| q.package).unwrap_or_else(|| link::WIVRN_APP.to_string())`, cached in `quest_app` per serial, so the 5 dumpsys probes run once per headset.
    - StartLink/Reattach run `run_steps(now, &link_commands_with(&effect,&w), true)`.
    - `connect_tether` calls `tether.set_wivrn(w.app.clone(), w.flags.clone())` before `connect`/`connect_tunnel_only`.
    - After an Own connect, log `wifi: {:?}` of `tether.last_wifi()`. `SyncDecision` is a fieldless enum (wifi.rs:163-170), so no SSID or psk reaches the log. Commands are logged only through `wifi::redact_command`.
  - **Heartbeat:**
    - `Service::new` installs the default raw sleeper `|ms| thread::sleep(Duration::from_millis(ms))`, so production gets the heartbeat with no change to `watch/run.rs`.
    - `tether_for`, when it creates a `Tether`, calls `set_sleep` with a closure that captures `Rc` clones of `sleep`, `last` and `slept_ms` plus `paths.snapshot()`. Each call runs the raw sleeper, adds `ms` to `slept_ms`, and, if `last` is Some, writes it with `updated = last.updated + slept_ms/1000` through `write_snapshot` (errors ignored).
    - `publish` stores its snapshot in `last` and resets `slept_ms` to 0.
    - This keeps `WaitTun0` (up to 15×1.5 s, then 8×1.5 s) from making the snapshot stale (10 s).
  - Grace still maps to `Ready` in `state_text` (OD4).
- **Tests:**
  - **Locked, unchanged:** `crates/engine/tests/watch_frames.rs`, `watch_grace.rs`, `watch_manual.rs` (:52-55 pins `link_commands`, which stays as it is) and `watch_plug.rs`. None of them builds a `Service`.
  - **Inline, updated in service.rs (not locked):**
    - The `service()` helper adds `.with_sleep(Box::new(|_| {}))`, so no inline test sleeps for real once WP-10's plans sleep.
    - The inline `Fake` (watch.rs:904 today) answers the tun0 probe that WP-10's `connect_plan` runs as "up", so the tether tests still reach `TetherMode::Own`.
    - `plug_authorise_ready_unplug_sequence_sees_the_right_snapshots` (watch.rs:986 today):
      - `first.events` becomes one `AllowDebugging` with seq 1 (it was empty).
      - `second`: `events.len()==2` and `events[1]` is `Ready` with seq 2.
      - `third`, `fourth` and `written`: `events.len()==2` (watch.rs:1005-1033 today).
  - **New (locked once merged):** `crates/engine/tests/watch_service.rs`.
    - Imports: `flat3d_inject::watch::{Service, Feed, Device, Paths}`, `flat3d_inject::switches::Settings`, `flat3d_inject::tether::adb::AdbChoice`, `flat3d_inject::tether::heal::Heal`, `flat3d_inject::api::{EventKind, LinkState, TetherMode}`.
    - A local scripted `Feed`. Temp `Paths{home,state_dir}` under `env!("CARGO_TARGET_TMPDIR")`. Always `.with_sleep(..)`.
    - The runner is `flat3d_inject::testing::FakeRunner` (WP-01). If it cannot script a reply per argv substring or record `start_relay`, the file uses a local `Fake` that implements the relay `CommandRunner` (the same shape as `tether_runtime.rs:25-65`), recording `ran:Vec<String>` and failing lines that contain a set word.
    1. Tunnel on, auto-3D on, and the fake fails `reverse --list` with "Tether command failed" on the first try only. The plug tick shows one `TetherError` event, and `ran` has one `systemd-run`, so the link start ran. The next Ready frame at `now+1` shows `ran` gaining exactly one `ps -eo pid=,args=` and one `reverse --list`, and `tether == Own`.
    2. Frames unauthorised, unauthorised, ready. After the ticks the events are `[AllowDebugging seq 1, Ready seq 2]`.
    3. Plug with tunnel on, mark `ran.len()`, then `heal(now, Heal::RestartClient)`. The new lines equal `client_stop_command()`, then each `client_start_commands()` argv, with `adb` replaced by `<adb> -s SER`.
    4. `heal(now, Heal::RestartAdbThenClient)`. The new lines start `<adb> kill-server`, `<adb> start-server`, then case 3's lines.
    5. `heal(now, Heal::FullReconnect)`. The new lines include `relay stop` (the disconnect) and a later `relay …` start. No new line contains `systemd-run`, `install` or `wivrn+tcp`.
    6. `heal(now, Heal::GiveUp)` twice, then `heal(now, Heal::RestartClient)`. There is exactly one `TetherError` event, and nothing is added to `ran` after the first call.
    7. The fake answers `dumpsys package org.meumeu.wivrn.github` with `Unable to find package` and `dumpsys package org.meumeu.wivrn` with `    versionName=26.9`. A Start 3D request then gives an `am start` line whose last word is `org.meumeu.wivrn`, and a `systemd-run` line that ends `--early-active-runtime --no-publish-service`. Use a `QUEST_APPS` member: an arbitrary id is never probed.
    8. Tunnel on. The fake reports tun0 down for the first 8 probes, then up, which gives 8×1.5 s = 12 s of plan sleep. The test's raw sleeper keeps a virtual clock. At every call it reads `link.json` and asserts `virtual_now_s - updated <= 5`. After the tick, `updated >= tick_now + 10`.
- **Inline facts:**
  - `Heal{None,RestartClient,RestartAdbThenClient,FullReconnect,GiveUp}` (heal.rs:12-18)
  - `MISS_LIMIT=3`, `MIN_HEAL_GAP_SECS=30`, `MAX_RECONNECTS=10` (heal.rs:5-9)
  - `EventKind::{AllowDebugging,TetherError}` (api.rs:170,173)
  - `GRACE_SECS=120` (watch.rs:90)
  - `Service<R: tether::adb::CommandRunner (relay shape), F: Feed>`, `Service::new(paths, Rc<RefCell<R>>, feed, AdbChoice)` (watch.rs:481-498)
  - `TetherAdb::run` prefixes `<path> -s <serial>` on every `adb` word list (adb.rs:148-157)
- **Flags:**
  - The scope names `Heal::Reconnect`, which does not exist. It means `FullReconnect` (heal.rs:16).
  - WP-11's "Contract for WP-09" lists only `client_start_commands` for `RestartClient`. WP-09 runs stop then start, as above.
  - The test files must use `crates/...` paths only, never `inject/`.
- **Depends on:** WP-01 (wave 1), WP-11, WP-12 (wave 2), WP-10 (wave 3).
- **Owner device:** Q3 (`sessions/link-smoke.toml`). The tests need no network and no device.
- **Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test watch_service --test watch_frames --test watch_grace --test watch_manual --test watch_plug --lib && tools/check.sh` (run from the worktree root).
