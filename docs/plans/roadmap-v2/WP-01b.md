<!-- split from WP-01.md (roadmap-v2.md lines 755-1385 at c0e6874), part b of four -->
### WP-01b: Leaf modules, exec split, architecture test

**Closes:** A12, K2, K7. It writes the full leaf modules (`exec`, `paths`, `fsx`, `sys`, `catalog`, `testing`, the `state`/`switches` readers and writers, the `recipes` route settings, the viewer `frame`/`source`/`spec`) with their tests, and lands the layering lint `tests/architecture.rs` (rulings 2026-11-09; Craft review A6, A10). `catalog::find_recipe` is the first half of the A5 fix; WP-01d wires it into the wrapper.

**One of four parts.** WP-01 is built as WP-01a → WP-01b → WP-01c → WP-01d, in that order inside wave 1, one agent and one branch per part. Each part leaves the tree compiling and passes its own done-check. "Depends on WP-01" in another file means all four parts merged. Every frozen form below is copied from bones §2 ("Frozen signatures" and "Bones WP-01 freezes"). If this file and bones §2 disagree, bones wins. In this file, "Craft review Ax" means that review's item, not the audit ID of the same name. The `questions.md` entries for all four parts sit under its `## WP-01` heading.

**Depends on:** WP-01a. **Wave:** 1 (second of the four WP-01 parts). **Owner device:** none (host). **Network:** none. `Cargo.lock` is frozen: build with `--locked`, and never run `cargo fetch`, `cargo update` or `cargo generate-lockfile`.

**Do not touch:** anything outside the Files list below. In particular `tools/**`, `packaging/**`, `decky/**`, `.gitignore`, `recipes/**`, every doc except `docs/code-map.md`, and the 52 moved test files that get no sanctioned edit (they move byte-identical). No GitHub writes. No `cargo update` and no `cargo generate-lockfile`.

**Order of work** (each step ends with a build that compiles; commit after each):
1. Do the exec split (`exec.rs`) and its re-export lines.
2. Write the full leaf modules and their tests: `paths`, `fsx`, `sys`, `catalog`, the `state`/`switches` readers and writers, the `recipes` route settings, `testing`, the viewer `frame`/`source`/`spec`.
3. Switch `watch` to `switches::read_settings` (A12).
4. Retag the two not-built literals, then write `tests/architecture.rs`.
5. Run the done-check.

**Files (exclusive):** `crates/engine/src/{lib,exec,testing,paths,fsx,sys,catalog,state,switches,recipes,link,cli,injectors}.rs`, `crates/engine/src/tether/adb.rs`, `crates/engine/src/watch/**`, `crates/engine/tests/{paths,fsx,catalog_find,switches_io,state_io,route_settings,architecture}.rs`, `crates/viewer/src/{frame,source,spec}.rs`, `crates/viewer/tests/spec.rs`. In detail:
- **New:**
  - Engine `crates/engine/src/`: `exec.rs`, `testing.rs`, `paths.rs`, `fsx.rs`, `sys.rs`, `catalog.rs`.
  - Engine `crates/engine/tests/`: `paths.rs`, `fsx.rs`, `catalog_find.rs`, `switches_io.rs`, `state_io.rs`, `route_settings.rs`, `architecture.rs`.
  - Viewer: `tests/spec.rs`.
- **Edited:**
  - Engine `src/`: `lib.rs` (six `pub mod` lines), `link.rs` (the runner types move out; one re-export line), `tether/adb.rs`, `state.rs`, `switches.rs`, `recipes.rs`, the `watch/` file that holds the private `read_settings` and its callers, `injectors.rs` (one retag), `cli.rs` (one interim retag and its unit-test string).
  - Viewer `src/`: `frame.rs`, `source.rs`, `spec.rs` (empty skeletons from WP-01a, written in full here).
- **Not edited:** `Cargo.*`, `check.yml`, any moved test file. **There is no `mod.rs` anywhere and no `x/x.rs`.**
- **Ownership after this part:** `exec.rs`: WP-01 owns it because it holds the trait split. WP-10 may change only the body of `ProcessRunner::run` and add private helpers. `paths.rs`, `testing.rs` and the viewer `frame`/`source`/`spec` are frozen (§0 "Frozen after wave 1", `00-run-the-pass.md`).

---

#### Module facades (explicit re-exports, never `*`)

Engine `lib.rs`: add
```rust
pub mod paths; pub mod exec; pub mod catalog; pub mod fsx; pub mod sys;
#[doc(hidden)] pub mod testing;
```
beside the lines WP-01a left. Nothing else in `lib.rs` changes.

`link.rs`, at the top: `pub use crate::exec::{CommandOutput, CommandRunner, SystemRunner};` (WP-01d adds the `wivrn` lines above and below it).

`tether/adb.rs`: `pub use crate::exec::{ProcessRunner, RelayRunner, RelayRunner as CommandRunner};`
- `AdbRunner<R: link::CommandRunner>` keeps its bound, constructor and `link::CommandRunner` impl (adb.rs:171-191).
- `TetherAdb<R: RelayRunner>` (adb.rs:134) is unchanged.

---

#### Exec split (`exec.rs`, full)

Move these verbatim:
- link shape: `CommandRunner { fn run(&mut self, argv: &[String]) -> CommandOutput }`, plus `CommandOutput` and `SystemRunner` (link.rs:100-140);
- relay shape: `RelayRunner` (renamed from `tether::adb::CommandRunner`), with `fn run(&mut self, &[String]) -> Result<String, String>`, `fn start_relay(&mut self, &[String], &str) -> Result<(), String>` and `fn stop_relay(&mut self) -> Result<(), String>`, verbatim from adb.rs:64-68, plus `ProcessRunner` (adb.rs:63-72).

There is no blanket impl.

#### Frozen signatures (this part writes all of these in full)

**`paths.rs`** (full; no owner afterwards):
```rust
pub struct EnginePaths { pub home: PathBuf, pub uid: u32, pub state_dir: PathBuf, pub config: PathBuf,
    pub data: PathBuf, pub cache_dir: PathBuf, pub log_dir: PathBuf, pub bin: PathBuf, pub recipe_dirs: Vec<PathBuf> }
impl EnginePaths { pub fn from_env(env: &BTreeMap<String, String>, uid: u32) -> Result<EnginePaths, ApiError> }
```
- A missing or empty `HOME` gives `ApiError` code `not-found`.
- Fields:

  | Field | Value |
  |---|---|
  | `state_dir` | `state::state_dir(env XDG_RUNTIME_DIR, uid)` |
  | `config` | `home/.config/flat3d` |
  | `data` | `undo::data_dir(env XDG_DATA_HOME, home)`, i.e. `$XDG_DATA_HOME/flat3d` else `home/.local/share/flat3d`. This is today's `engine_data_dir` |
  | `cache_dir` | `$XDG_CACHE_HOME/flat3d` (non-empty) else `home/.cache/flat3d` |
  | `log_dir` | `home/.local/state/flat3d` (watch.rs:323, flat3d-run.rs:34) |
  | `bin` | `home/.local/bin` |
  | `recipe_dirs` | `[$FLAT3D_SRC_DIR/recipes]` when set and non-empty, then `data/recipes` |

- Accessors:

  | Accessor | Returns |
  |---|---|
  | `settings()` | `home/.config/flat3d/settings` |
  | `link_log()` | `log_dir/link.log` |
  | `run_log()` | `log_dir/run.log` |
  | `cache()` | `cache_dir` |
  | `uevr_cache()` | `cache()/uevr` |
  | `env_file(appid: u32)` | `data/env/{appid}.env` |
  | `wrapper_bin()` | `bin/flat3d-run` (absolute) |
  | `viewer_bin()` | `bin/flat3d-viewer` (absolute) |
  | `pins()` | `data/pins` (kept, but unused in this pass: pins come from the compiled `fetch::PINS`, WP-13) |
  | `run_paths() -> RunPaths` | `{state: state_dir lossy, settings: settings(), recipes: data/recipes, log: run_log()}` |
  | `watch_paths() -> watch::Paths` | `{home, state_dir}` as lossy strings |

  Every accessor returns `PathBuf` except the last two.
- Production callers use `from_env(&std::env::vars().collect(), sys::current_uid())`. Locked `state::*_path(&str)` callers pass `p.state_dir.to_string_lossy()`.

**`fsx.rs` / `sys.rs`** (full):
- `fsx::write_atomic(&Path, &[u8]) -> io::Result<()>` writes a tmp file in the same dir, then renames it.
- `fsx::append_line(&Path, &str) -> io::Result<()>` creates the parent dirs.
- `fsx::read_opt(&Path) -> io::Result<Option<String>>` gives `None` when the file is NotFound.
- `sys::current_uid() -> u32` and `sys::unix_now() -> u64`.

**`state.rs` / `switches.rs` / `catalog.rs`** (full):
- `state::read_snapshot(p: &EnginePaths) -> Result<Option<LinkSnapshot>, String>`. A missing file gives `Ok(None)`.
- `state::write_request(p: &EnginePaths, r: Request) -> io::Result<()>`. `Request` is `Copy` (state.rs:117-118).
- `switches::read_settings(path: &Path) -> Result<Settings, String>`. NotFound gives `Ok(Settings::default())`.
- `switches::write_settings(path: &Path, s: &Settings) -> io::Result<()>`. It goes through `fsx::write_atomic`.
- `catalog::find_recipe(dirs: &[PathBuf], appid: u32) -> Result<Option<(PathBuf, String)>, String>`:
  - it tries `{id}.recipe`, then the sorted `{id}-*.recipe`;
  - the first dir with a match wins;
  - **a dir that does not exist is skipped**;
  - the result is `Ok(None)` when no dir has a match.

**`recipes.rs`** (full):
- `pub type RouteSettings = BTreeMap<String, String>;`
- `UEVR_KEYS: [&str; 6] = ["uevr-exe","uevr-rendering","uevr-2d-screen","uevr-native-stereo-fix","uevr-delay","uevr-profile"]`
- `GEO11_KEYS: [&str; 7] = ["geo11-fix","geo11-exe","geo11-layout","geo11-separation","geo11-convergence","geo11-d3dx","geo11-remove-first"]`
- `PORT_KEYS: [&str; 0]`
- `route_settings(text: &str) -> RouteSettings`:
  - each line splits at the first `:`, and both sides are trimmed;
  - one pair of surrounding quotes is stripped;
  - `#` lines and blank lines are skipped;
  - only `uevr-*`/`geo11-*`/`port-*` keys are kept, so it never returns the 9 recipe KEYS.
- `check_route_settings(route: &str, m: &RouteSettings) -> Result<(), String>`:
  - `route` is the raw `stereo-route` text (`Recipe.route`), mapped with `injectors::route_of`;
  - Port and no-route accept only an empty map;
  - a key outside the route's key list is refused;
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

**`testing.rs`** (`#[doc(hidden)] pub mod testing`, path `flat3d_inject::testing::FakeRunner`, full):
- `pub struct FakeRunner { pub ran: Vec<Vec<String>>, /* private reply queue */ }`. `FakeRunner::new()` treats unscripted argv as success with empty stdout and stderr.
- Builders: `reply(self, prefix: &[&str], out: CommandOutput) -> Self`, `ok(self, prefix: &[&str], stdout: &str) -> Self` and `fail(self, prefix: &[&str], stderr: &str) -> Self`.
- The replies form a queue: the first unconsumed entry whose prefix matches the argv is consumed.
- `impl link::CommandRunner` returns that `CommandOutput`.
- `impl RelayRunner`:
  - `run` gives `Ok(stdout)` on success, else `Err(stderr)`;
  - `start_relay` records `["relay", argv.., name]`, and `stop_relay` records `["relay", "stop"]`. Both return `Ok`.
- Every call pushes its argv to `ran`.

**Viewer** (full):
- `frame.rs` (Debug, no Clone, no `PixelFormat`, no `Cpu`; `fourcc` is a DRM fourcc, and NV12 uses two planes):
  ```rust
  pub struct Plane { pub fd: OwnedFd, pub offset: u32, pub stride: u32 }
  pub enum FrameData { Mem { stride: u32, bytes: Vec<u8> }, DmaBuf { planes: Vec<Plane>, modifier: u64 } }
  pub struct Frame { pub width: u32, pub height: u32, pub fourcc: u32, pub seq: u64, pub data: FrameData }
  ```
- `source.rs`:
  ```rust
  pub use crate::frame::Frame;
  pub trait FrameSource { fn next(&mut self, timeout: std::time::Duration) -> Result<Option<Frame>, String>; }
  pub mod test_pattern;
  #[cfg(feature = "capture")] pub mod pipewire;
  #[cfg(feature = "capture")] pub mod gamescope;
  #[cfg(feature = "capture")] pub mod portal;
  ```
- `spec.rs`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
  pub struct ViewerSpec { pub layout: Layout, pub source: Source, pub swap_eyes: bool, pub crop_aspect: Option<f32>, pub screen_width_m: f32, pub distance_m: f32 }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(rename_all = "snake_case")]
  pub enum Layout { #[default] Sbs, Tab, #[serde(rename = "sbs_reversed")] SbsRev, #[serde(rename = "tab_reversed")] TabRev }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(rename_all = "snake_case")]
  pub enum Source { #[default] Auto, Gamescope, Portal, TestPattern }
  impl ViewerSpec { pub fn from_args(args: &[String]) -> Result<ViewerSpec, String>; pub fn to_args(&self) -> Vec<String> }
  ```
  - `to_args` writes `--layout <l> --source <s>`, then `--swap-eyes` when true and `--crop-aspect <f>` when Some.
  - `--screen-width-m <f>` and `--distance-m <f>` are written only when > 0.0. 0.0 means the viewer default, which WP-19 resolves to 3.0 m / 2.5 m.
  - The strings are the serde names. `from_args` returns Err on an unknown flag.

---

#### Behaviour wired now

- **watch:**
  - The private `read_settings` (watch.rs:826) is removed. Callers use `switches::read_settings(Path::new(&p)).unwrap_or_default()` (A12).
- **Stub tags** (so the architecture test passes at the end of this part):
  - today's `NotBuilt("injectors::inject")` (injectors.rs:61) is retagged `injectors::inject (WP-27)`;
  - the old dispatch fallback (cli.rs:147, `NotBuilt("cli: the command bodies are wired by #27")`) becomes `NotBuilt("cli::engine::run (WP-07)")` for now, and the unit test `execute_human_not_built_goes_to_stderr` (cli.rs:198) expects `"error: not built yet: cli::engine::run (WP-07)\n"`. Both lines are interim: WP-01c replaces the dispatch and that unit test. Change nothing else in `cli.rs`.

---

#### Tests

The moved test files stay byte-identical to what WP-01a left (the done-check's `locked-14` step proves it).

**New engine tests** (`crates/engine/tests/`):
- `paths.rs`:
  - `from_env(&map, 1000)` with XDG set and with XDG unset;
  - a missing HOME gives code `not-found`;
  - `FLAT3D_SRC_DIR` puts `$FLAT3D_SRC_DIR/recipes` first;
  - one case per accessor: `settings`, `link_log`, `run_log`, `cache` (XDG set and unset), `uevr_cache`, `env_file(5)`, `wrapper_bin`, `viewer_bin`, `pins`, `run_paths` and `watch_paths`.
- `fsx.rs`: the atomic write leaves no tmp file; append creates the parent dirs; `read_opt` on a missing file gives `None`.
- `catalog_find.rs`: exact name; dashed name; the first dir wins; **a missing dir is skipped**; no match gives `Ok(None)`.
- `switches_io.rs`: write then read round-trips; a missing file gives Default.
- `state_io.rs`: a missing snapshot gives `Ok(None)`; `write_request` then `parse_request` round-trips.
- `route_settings.rs`:
  - each value rule, once ok and once bad;
  - an unknown `uevr-x` key is refused;
  - Port with a non-empty map is refused;
  - `route_settings` drops recipe KEYS and `#` lines, and strips one pair of quotes.
- `architecture.rs` (Craft review A6, A10). It reads the files under `crates/engine/src` from `CARGO_MANIFEST_DIR` and writes nothing.
  - **Only `exec.rs` spawns.** It fails on `Command::new` in any other file, except a tagged allowlist. Each entry names a file and the WP that moves its site into `exec.rs`. Today there are six `Command::new` lines in four files: link.rs:127, watch.rs:358 and :847, tether/adb.rs:78 and :89, and bin/flat3d-run.rs:53. WP-01 itself moves `SystemRunner` (link.rs) and `ProcessRunner` (tether/adb.rs) into `exec.rs`, so the list it lands with is:
    - the `watch/` file or files that hold today's watch.rs:358 (`adb track-devices`) and :847 (`ss -tlnp`) after the split, tagged `follow-up`. No WP in this pass moves them: WP-09 owns the callers but not `exec.rs`, and WP-10 may touch only `ProcessRunner::run`;
    - `bin/flat3d-run.rs` (the launcher that execs the game), tagged WP-31. It is a permanent entry: the exec replaces the process, so it is not a runner call;
    - `cli/link.rs`, tagged WP-08. It is permanent for the same reason: WP-08's `exec_launch` uses `Command::new` + `CommandExt::exec` to run the game in place.
  - The failure message names the file and line and says `move the spawn into exec.rs behind a runner trait`.
  - **Every stub tag names a real WP.** It finds each `NotBuilt("…")` and `"not-built: …"` literal in `crates/engine/src` and fails when one has no `(WP-xx)` tag, or when the tag is not a WP in the roadmap index (WP-00 to WP-24, WP-25a, WP-25b, WP-26 to WP-34; there is no plain WP-25).

**New viewer test:** `crates/viewer/tests/spec.rs`.
- `to_args`/`from_args` round-trip every `Layout` and `Source`, with `TabRev` ↔ `"tab_reversed"`, `SbsRev` ↔ `"sbs_reversed"` and `TestPattern` ↔ `"test_pattern"`.
- `ViewerSpec::default()` gives 0.0 for both f32 fields, and `to_args` then omits them.
- An unknown flag gives Err.

---

**Done-check** (run after committing on this part's branch, `v2/wp-01b-*`). Each step is labelled. A failing step prints `FAIL <step>: fix: <hint>` and stops (Craft review A2).
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
for t in paths fsx catalog_find switches_io state_io route_settings architecture; do
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
- `cli_any.rs` is not in `tests-new` yet: WP-01c adds it.
- Python, shellcheck and decky are not run here. Their CI steps are guarded until WP-02, WP-21 and WP-25b land.
- The `capture` clippy is not run here (no pipewire, spa or clang headers on the host); CI runs it.

**PR and merge** (rulings D1, D5):
- The PR body has `## Verified`: each done-check step and its exit status, quoted from the run. It never claims a run that was not done.
- It also has `## Not verified`: each skip the run hit. For WP-01b that is the `capture` clippy (CI only), the `py` and `decky` jobs, `check.sh` if WP-02 has not merged, and the CI run itself if it was not seen to finish.
- The merge takes only the head commit the review passed.
- WP-01b has no device Q row, so it keeps `Closes` for its issues (D5 applies only to WPs with a device half).
