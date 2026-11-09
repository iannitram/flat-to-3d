# WP-28 recheck: proposed replacement text

Plan mode blocked editing `split/wp/WP-28.md`. This is the corrected text to put in its place.

---

### WP-28: View commands (#121, #75)

**Wave:** 3. **Closes:** #121, #75.

**Depends on:** WP-01 (frozen `ViewCommand`, `Ctx`, `cli::view::run`, `face::link_model::link_view`, api DTOs, `EnginePaths`), WP-03 (`game_rows`), WP-06 (`face::logs::read_logs`), WP-07 (`catalog::catalog`), WP-20 (`launch_options::add_wrapper_at`). All of them are in wave 1 or 2.

**Files (exclusive):**
- `crates/engine/src/cli/view.rs`: the body of the frozen `run`, the view parser body, and private helpers. No new `pub` items.
- `crates/engine/src/face/link_model.rs`: the body of the frozen `link_view`.
- `crates/engine/tests/cli_view.rs` (new).

Do not edit `cli.rs`, `face.rs`, `api.rs`, `lib.rs` or `Cargo.*` (frozen).

**Signatures (frozen by WP-01; fill in the bodies only):**
```rust
// face/link_model.rs
pub fn link_view(snap: Option<LinkSnapshot>, now: u64) -> api::LinkView;
// cli/view.rs
pub fn run(cmd: &ViewCommand, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError>;
// ViewCommand{View, Rows{filter}, Launch{add, options}, Collection{current:Option<Vec<u32>>}, Toasts{last_seq}, Logs{limit}}
// plus the view parser that parse_any calls (signature as WP-01 freezes it)
```
Private helpers in `cli/view.rs` (not `pub`, per bones §2 "Helpers stay private"): `rows`, `launch`, `collection`, `toasts`, `logs`. `now` = `sys::unix_now()` (Ctx has no `now`).

**Behaviour:**
- **`view`:** `link_view(state::read_snapshot(p).map_err(io)?, now)`.
  - `None` → `LinkSnapshot::stopped(now)`, else `.effective(now)`.
  - `label`/`hint` from `link_view::{link_label, link_hint}`; `start`/`stop` from `headset_view::headset_buttons(state)`.
  - `health` = `view::health_view(s.health.as_ref())`.
  - `rx`/`tx` = `throughput_text(stats.rx_bytes_s / tx_bytes_s)`, `uptime` = `uptime_text(stats.uptime_s)`; no stats → all three `"—"`.
  - `tether`, `tunnel`, `auto_3d`, `adb_source` copied from the snapshot.
- **`rows [--filter <status>|all]`:** default `all`.
  - `games = catalog::catalog(p)?.games`;
  - `settings = switches::read_settings(&p.settings()).map_err(|e| ApiError::new("io", &e))?`;
  - `link = read_snapshot` → `None` gives `LinkState::Stopped`, else `.effective(now).state`;
  - `rows::game_rows(&games, filter, &settings, link)`.
- **`launch add|remove --options <string>`:**
  - The options are exactly one argv element, the one after `--options`. It may be `""` and may contain `--` or `%command%` inside it.
  - Nothing may follow the value. A missing `--options`, a missing value, or an extra element → usage.
  - `--json` anywhere in the head is removed by `parse_any` (WP-01) before the view parser sees argv, so its position does not matter.
  - Limitation: a value that is exactly `--` or exactly `--json` cannot be passed, because `parse_any` splits or strips it.
  - `add` → `LaunchText{options: launch_options::add_wrapper_at(&options, &p.wrapper_bin().to_string_lossy())}`.
  - `remove` → `LaunchText{options: launch_options::remove_wrapper(&options)}`.
- **`collection [--current <csv>]`:**
  - `--current ""` → `Some(vec![])`; no flag → `None`; a non-number item → usage.
  - `wanted = collection::three_d_ready_appids(&catalog::catalog(p)?.games)`.
  - `add`/`remove` from `collection_diff(cur, &wanted)`, both empty when `None`.
  - `name = collection::COLLECTION_NAME` (`"3D Ready"`).
- **`toasts <last_seq>`:** `last_seq` is a u64 (`-1` or missing → usage).
  - `events` = the raw snapshot's `events` (no stale check); no snapshot → `[]`.
  - `serde_json::to_value(toasts::toasts_since(&events, last_seq))`, then insert `"title": api::TOAST_TITLE`.
- **`logs [--limit n]`:** `n ≥ 1`, default `headset_view::DEFAULT_LOG_LINES` (500).
  - `lines = face::logs::read_logs(p, n)` (WP-06; missing files give `[]`);
  - `LogLines{text: headset_view::log_copy_text(&lines), lines}`.
  - No local `log_lines(text, limit)` and no `read_log_text`.
- **Output:** every command returns a JSON value. Bad args → code `usage`, exit 2. The view usage text is a private const in `cli/view.rs`; the locked `cli::USAGE` is unchanged.

**Tests:**
- **Locked, unchanged:** `tests/link_view.rs`, `tests/headset_view.rs`, `tests/cli_json.rs`.
- **New `cli_view.rs`:** a temp dir; `EnginePaths::from_env(&map, 1000)` with `HOME` and `XDG_RUNTIME_DIR` in the map pointing into it; `flat3d_inject::testing::FakeRunner` for `runner` and `relay`; `Ctx{paths, runner, relay, json: true}`.
  - **Parse table (`cli::parse_any` gives `View(..)`):**
    - `view`; `view --json` (json=true); `rows`; `rows --filter works-now`;
    - `launch add --options ""`; `launch add --json --options "%command% -dx11"`;
    - `launch add --options x --json` (accepted; json=true);
    - `launch add --options "FOO=1 %command% -- -dx11"` (keeps the inner `--`);
    - `launch add --options "-- %command%"`; `launch remove --options "flat3d-run %command%"`;
    - `collection --current 1,2`; `collection --current ""` → `Some([])`;
    - `toasts 7`; `logs --limit 3`.
  - **Bad lines → error code `usage`:**
    - `rows --filter`; `launch`; `launch add`; `launch add --options`;
    - `launch frob --options x`; `launch add --options a b`;
    - `collection --current a`; `toasts`; `toasts -1`; `logs --limit 0`.
  - **Model shapes (`face::link_model::link_view` direct):**
    - `None` → `state` Stopped, `adb_source` Missing, `auto_3d:false`, `rx:"—"`;
    - a snapshot with `updated = now-11` → stopped;
    - stats `rx_bytes_s:1536` → `"1.5 KB/s"`, `uptime_s:185` → `"3m 5s"`.
  - **Through `cli::view::run`:**
    - `launch add --options ""` → `options` is `<p.wrapper_bin()> %command%`;
    - `remove(add(x)) == x` for `""`, `"%command% -dx11"`, `"-- %command%"`;
    - `collection --current ""` with the empty HOME → `add==wanted==[]`, `remove==[]`; `--current 1,2` → `remove==[1,2]`;
    - `toasts 0` with no snapshot → `toasts==[]`, `last_seq==0`, `title=="Flat to 3D"`;
    - write a fresh snapshot (`watch::write_snapshot(&state::snapshot_path(&p.state_dir.to_string_lossy()), &s)`) with events seq 1..3 → `toasts 1` gives seq 2,3, oldest first;
    - `logs` with no files → `lines==[]`, `text==""`;
    - five lines in `p.link_log()`, `--limit 3` → the newest 3.

**Inline facts:**
- `LinkView{state,label,hint,start,stop,health,rx,tx,uptime,tether,tunnel,auto_3d,adb_source}` (code-map.md:110-149).
- kebab-case enums: `TetherMode{off,own,quest-tether}`, `AdbSource{running,bundled,missing}`.
- `WRAPPER="flat3d-run"`; `add_wrapper` inserts after leading env words (launch_options.rs:77-92).
- `api::ApiError` has no `From<String>`; map String errors with `ApiError::new("io", &e)`.
- `ToastBatch{toasts:Vec<LinkEvent>,last_seq}` (toasts.rs:6); `LinkEvent{seq,kind,text}`.

**Flags:**
- `Flag:` the scope said `launch <appid>`; #121, code-map:115 and decky.md:173 say `launch add|remove`, so this WP keeps `add|remove`. There is no `launch-sync` command (frozen `ViewCommand` has none). WP-21/WP-32 call `launch add|remove --options <cur>`.
- `Flag:` `ToastBatch` has no `title`; this WP adds it as an extra JSON key. No locked test builds a literal.

**Owner device:** none. No network.

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target cargo test --locked -p flat3d-inject --test cli_view --test link_view --test headset_view --test cli_json && tools/check.sh`
