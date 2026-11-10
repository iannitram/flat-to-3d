# WP-27 recheck: replacement text

Plan mode blocked edits, so WP-27.md is unchanged. Below is the drop-in replacement for
`scratchpad/split/wp/WP-27.md`. Copy it over that file once edits are allowed.

---

### WP-27: inject command, ensure_applied, setup fetch, undo tests (#27a)

**Closes:** #27 (27a: `inject <appid>` and the undo CLI test gap K29), A20 (`save_record`), and `setup fetch <key>`. The Port route stays `not-built` until #27b.

**Files (exclusive; the index must list exactly these):**
- `crates/engine/src/injectors.rs`: bodies only. WP-01 froze `RouteEnv`, `Applied` and `ensure_applied` here (bones §2). Keep those three exactly as written.
- `crates/engine/src/undo.rs`
- `crates/engine/src/cli/inject.rs`: body of the frozen `run` only.
- `crates/engine/src/cli/setup_fetch.rs`: body of the WP-01 stub `setup_fetch` only. WP-29's `cli/setup.rs` calls it.
- `crates/engine/tests/inject_fake_prefix.rs`
- `crates/engine/tests/ensure_applied.rs`
- `crates/engine/tests/cli_undo.rs`
- `crates/engine/tests/cli_setup_fetch.rs`

Not touched: `cli/undo.rs` (WP-01 already moved the working undo body there), `cli.rs`, `api.rs`, `paths.rs`, `steam.rs`, `catalog.rs`, `injectors/{uevr,geo11,fetch}.rs`.

**Signatures:**
```rust
// injectors.rs. Frozen by WP-01 (bones §2), body here:
pub fn ensure_applied(p:&EnginePaths, appid:u32, recipe:&Recipe, r:&mut dyn CommandRunner)->Result<Applied, ApiError>;
// Applied{route:Route, env:RouteEnv} and RouteEnv{env,unset,sidecar} are WP-01's, unchanged.
// New in injectors.rs (replaces the stub `inject(game,recipe)->Result<String,NotBuilt>`; nobody else calls it):
pub fn inject(p:&EnginePaths, appid:u32, r:&mut dyn CommandRunner)->Result<api::InjectResult, ApiError>;
pub fn env_file_text(route:Route, settings:&RouteSettings, env:&RouteEnv)->String;
pub fn parse_env_file(text:&str)->Result<(Route, RouteSettings, RouteEnv), String>;
#[doc(hidden)] pub fn fetch_api_error(e:&fetch::FetchError)->ApiError;
// undo.rs
pub fn save_record(data_dir:&Path, rec:&UndoRecord)->Result<PathBuf, ApiError>; // atomic; refuses if a record exists
pub fn put_back(rec:&UndoRecord)->Vec<String>; // best effort, never fails
// make the private escape/unescape (undo.rs:66,81) pub(crate) for the env file
// cli/setup_fetch.rs (WP-01 stub signature, body here)
pub fn setup_fetch(key:&str, p:&EnginePaths, r:&mut dyn CommandRunner)->Result<api::SetupReport, ApiError>;
// cli/inject.rs: the frozen `pub fn run(cmd, ctx:&mut Ctx<'_>)->Result<serde_json::Value,ApiError>`, body only
```
Keep unchanged: `Route`, `route_of`, `check_injectable` (wrapper.rs and locked tests use them); in undo.rs `UndoRecord`, `Replaced`, `UndoAction`, `undo_actions`, `record_path`, `data_dir`, `to_text`/`from_text` and `undo_game` (locked `undo_record.rs`, `undo_actions.rs`). In injectors.rs, refer to WP-14/WP-16's own `Applied{record,..}` as `uevr::Applied` / `geo11::Applied`; the bare name `Applied` is WP-01's.

**Behaviour:**
- **`inject(p, appid, r)`**, in this order (no writes before step 5):
  1. `catalog::find_recipe(&p.recipe_dirs, appid)`: `Ok(None)` → `not-found` "no recipe for {appid}"; `Err` → `bad-recipe`. Parse with `recipes::parse_recipe(appid, &text)`; Err → `bad-recipe`.
  2. `check_injectable(&recipe)`: Err(text) → `refused` with that text.
  3. `recipes::check_route_settings(&recipe.route, &recipes::route_settings(&text))`: Err → `bad-recipe`.
  4. `ensure_applied(p, appid, &recipe, r)`.
  5. Return `InjectResult{appid, route:<"uevr"|"geo11">, launched:false, summary}`. The summary is "{name}: already set up" when ensure_applied reused the existing setup (step 2 below), else "{name}: set up for {route}".
  - Never writes Steam launch options and never starts the game (#27, K26).
  - Locked `cli_json` runs `inject 5 --json`: appid 5 has no recipe, so it returns `not-found` with no writes and no runner calls.
- **`ensure_applied(p, appid, recipe, r)`:**
  1. `route_of(&recipe.route)`: Port → `not-built` "port route: #27b"; NoRoute → `refused`. Then re-read settings with `find_recipe` + `route_settings` (None → `not-found`).
  2. **Reuse:** the record (`undo::record_path(&p.data, appid)`) and `p.env_file(appid)` both exist, every path in the record's `created` exists, and `parse_env_file` gives the same route and settings → return `Applied{route, env}` from the file. No fetch, no writes, no runner calls.
  3. **Stale:** a record exists but step 2 failed → `undo::undo_game(&p.data, appid)?`, then continue.
  4. **Game:** `steam::installed_games(&p.home.join(steam::DEFAULT_STEAM_ROOT))`, find `appid`. Missing → `not-found` "{appid} is not installed". io Err → `io`.
  5. **Pins:** Uevr → `["uevr"]` plus `settings["uevr-profile"]` when set. Geo11 → `["geo11"]` plus `settings["geo11-fix"]` when set. Both values are pin keys (WP-26). For each key: `let pin = fetch::pin(key)`, then `fetch::fetch(&pin, p, r)`. `fetch` itself returns at once with no commands when `<cache_dir>/.flat3d-fetch` equals `sha256\n`, and unpacks archives itself; there is no separate unpack or sentinel step here. Errors go through `fetch_api_error`, except `UnknownKey` → `bad-recipe` naming the key (the key came from the recipe, not the user).
  6. **Apply:**
     - Uevr: `uevr::apply_with(&game, &settings, p)` → `(uevr::Applied, RouteEnv)`.
     - Geo11: `geo11::apply(&p.cache(), &game.install_dir, appid, &settings, &p.bin.join("flat3d-viewer"))` → `(geo11::Applied, RouteEnv)`. The sidecar argv comes from WP-16; WP-31 appends `--until-pid`.
     - An Err(String) from either → `refused` with the text verbatim (for example "…already set up; undo it first").
  7. `undo::save_record(&p.data, &applied.record)` (`launch_options_before` is `""`, OD5, already set by WP-14/16).
  8. `fsx::write_atomic(&p.env_file(appid), env_file_text(route, &settings, &env).as_bytes())`, creating the parent dir first.
  - If step 7 or 8 fails: `undo::put_back(&applied.record)`, remove the record file if step 7 wrote it, and return `io` with the cause.
- **Env file format** (`env_file_text` / `parse_env_file`, round-trips):
  - Line 1 `flat3d-env 1`; then `route\t<uevr|geo11>`; one `setting\tK\tV` per setting (BTreeMap order); `set\tK\tV` per env pair in order; `unset\tK`; `sidecar\targ\targ…` only when `Some`.
  - Every field goes through undo's `escape` (tab, newline, backslash). An unknown first line or tag → Err.
- **`save_record`:** create `<data>/undo/`; if `record_path` exists → `refused` "an undo record for {appid} already exists; undo it first"; else `fsx::write_atomic(path, rec.to_text())`; fs errors → `io`.
- **`put_back`:** runs the file steps of `undo_actions(rec)` like `undo_game` does (remove created, copy backups back and remove them), but never returns early: each step adds one line ("removed …", "restored …", or "failed … : <err>"). It does not touch the record file. `undo_game` stays as it is.
- **`fetch_api_error`** (contract-owner mapping, kept inside the 8 locked `ERROR_CODES`; the message is always `e.to_string()`, which names the key):
  - `UnknownKey` → `usage`;
  - `TbdAtPin`, `Manual` → `refused` (the owner's "manual" class: the user must run `packaging/pin.sh <key>` or fetch by hand from the page);
  - `SizeMismatch`, `ShaMismatch` → `command-failed` (the owner's "checksum" class);
  - everything else (`Command`, `BadPin`, `Scheme`, `Io`) → `io`.
  - This deliberately differs from WP-13's `From<FetchError> for ApiError`; WP-27 never uses that impl.
- **`setup_fetch(key, p, r)`:** `fetch::pin(key)`, then `cached = <fetch::cache_dir(&pin,p)>/.flat3d-fetch == sha256\n`, then `fetch::fetch(&pin, p, r)`. Errors → `fetch_api_error` (so an unknown key exits 2). Ok → `SetupReport{steps:[<"cached {dir}" | "fetched {dir}">], summary:"{key} {version} is ready in {dir}"}`.
- **`cli::inject::run`:** match the inject command WP-01's parser hands it (`Command::Inject{appid}` or the `InjectCommand` variant, whichever WP-01 froze), call `injectors::inject(ctx.paths, appid, ctx.runner)`, return `serde_json::to_value(result)` (a serde error → `io`). Any other variant → `usage`. Argv errors never reach here: the locked parser returns `INJECT_USAGE` (exit 2).
- **`undo <appid>`** stays WP-01's moved body in `cli/undo.rs`; WP-27 only adds its CLI test.

**Tests** (FakeRunner = `flat3d_inject::testing::FakeRunner`; read `crates/engine/src/testing.rs` for its constructor, scripted replies and recorded argv. New files carry no locked header.)
- **Shared fixture** (copy into each new test file; do not import from locked tests): a temp dir with `home/`, `data/`, `cache/`, `config/`, `state/`, `recipes/`. Build `EnginePaths` with WP-01's `from_env` from `HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`, `XDG_CONFIG_HOME`, `XDG_STATE_HOME` set to those dirs, then set `p.recipe_dirs = vec![tmp/recipes]`. A fake Steam root at `p.home/.local/share/Steam`: `steamapps/libraryfolders.vdf` and `steamapps/appmanifest_<id>.acf` written the way locked `steam_installed_games.rs:15-40` does, with `steamapps/common/<dir>/Foo.exe`. A recipe `recipes/<id>-test.recipe` with the stereo-* keys from locked `recipe_load.rs:10-12`, `stereo-status: works-now`, `stereo-safe: yes`, `stereo-route: uevr`, `uevr-exe: Foo`.
  - Seed UEVR: `let pin = fetch::pin("uevr").unwrap(); let d = fetch::cache_dir(&pin,&p);` write `d/UEVRBackend.dll`, `d/openxr_loader.dll`, `d/.flat3d-fetch` = `format!("{}\n", pin.sha256)`, plus `uevr::injector_path(&p)`.
- **Locked, unchanged, must pass:** `cli_json.rs` (`inject 5 --json` is one object, `ok==(code==0)`, code in `ERROR_CODES`; bad lines "inject", "inject abc", "inject -1 --json", "inject 1 2" exit 2 with `INJECT_USAGE`), `injector_route.rs`, `wrapper_exec.rs`, `wrapper_plan.rs`, `undo_record.rs`, `undo_actions.rs`, `uevr_profile.rs`.
- **`inject_fake_prefix.rs`:**
  - seeded cache → `inject` Ok, `launched==false`, the runner recorded no argv, the record exists with `launch_options_before==""`, and the env file starts `flat3d-env 1\nroute\tuevr\n`;
  - a second `inject` → Ok, summary ends "already set up", still no argv;
  - empty cache, FakeRunner scripted so `curl` exits 22 → Err `io`, no record, no env file, no `cache_dir` left;
  - `data/env` created as a plain file (so the env write fails) → Err `io`; the profile dir has no `.flat3d-applied` and no record file is left;
  - `stereo-safe: no` → `refused`; no recipe → `not-found`.
- **`ensure_applied.rs`:**
  - two calls → equal `env.env`, `env.unset`, `env.sidecar`; the second records no argv;
  - change `uevr-delay` to 20 in the recipe file → a fresh record (old one undone), the env file has `setting\tuevr-delay\t20`, and `env.env[0].1` differs from the first call;
  - a recipe parsed from text with `stereo-route: build` → Err code `not-built`;
  - `env_file_text` → `parse_env_file` round-trips a value holding a tab, a newline and a backslash;
  - `fetch_api_error`: `UnknownKey`→usage, `TbdAtPin("geo11")`→refused and the message contains `geo11`, `Manual{..}`→refused, `ShaMismatch`/`SizeMismatch`→command-failed, `Command{..}`/`Io`→io. (TBD is tested here, not through the real pins, because the owner fills them at Q0.)
- **`cli_undo.rs`:** `std::process::Command::new(env!("CARGO_BIN_EXE_flat3d-inject"))` with `.env("HOME",tmp).env("XDG_DATA_HOME",tmp)`; records live at `tmp/flat3d/undo/<appid>.txt`, written with `UndoRecord::to_text`.
  - no record → exit 1, `--json` code `not-found`;
  - a record whose `created` holds a temp file → exit 0, file gone, record gone;
  - the same again → exit 1, `not-found`;
  - `undo abc` → exit 2, stderr contains `usage: flat3d-inject undo <appid> [--json]` (the private `UNDO_USAGE`, cli.rs:53);
  - a record file with garbage text → exit 1, `--json` code `command-failed`.
- **`cli_setup_fetch.rs`** (calls `flat3d_inject::cli::setup_fetch::setup_fetch` directly):
  - seeded `uevr` marker → Ok, no argv, `steps[0]` starts "cached";
  - `no-such-key` → `usage`;
  - `ac7-prereqs` (a `fetch: manual` pin, WP-13 `real_pins`) → `refused`;
  - empty cache with `curl` exit 22 → `io`, no `cache_dir` left.
  - "`setup fetch` with no key exits 2" is a parser test and belongs to WP-29.

**Inline facts:**
- Undo record path `<data>/undo/<appid>.txt` (undo.rs:177); data dir `$XDG_DATA_HOME/flat3d`, else `~/.local/share/flat3d` (undo.rs:183).
- Undo summary "{n} change(s) put back for {appid}; the launch option is left to the Decky plugin".
- UEVR refuse text "the UEVR profile for {} is already set up; undo it first" (injectors/uevr.rs:87); marker `.flat3d-applied` (uevr.rs:56).
- UEVR env (WP-14) `PROTON_REMOTE_DEBUG_CMD`, `unset=["XR_RUNTIME_JSON"]`; Geo11 env `WINEDLLOVERRIDES="d3d11,nvapi64=n,b"` (WP-16).
- `steam::Game` has 3 fields (`appid`, `name`, `install_dir`); the Steam root is `p.home.join(steam::DEFAULT_STEAM_ROOT)` (WP-07).
- Fetch never runs a real download here: FakeRunner only records argv. No network, no device.

**Depends on:** WP-01, WP-02 (`packaging/pins`), WP-07, WP-13, WP-14, WP-16. All are wave 1-2, so wave 3 holds. Same-wave WPs (28, 29, 30) own none of these files.

**Owner device:** none for the code (fakes only). Real runs: Q6 (UEVR), Q9 (Geo-11).

**Done-check:** `CARGO_TARGET_DIR=$SCRATCH/target XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked -p flat3d-inject --test inject_fake_prefix --test ensure_applied --test cli_undo --test cli_setup_fetch --test cli_json --test injector_route --test wrapper_exec --test wrapper_plan --test undo_record --test undo_actions --test uevr_profile && tools/check.sh`

---
