<!-- split from WP-01.md (roadmap-v2.md lines 755-1385 at c0e6874), part c of four -->
### WP-01c: CLI bones: parse_any, command registry, exit hook, error_help
> Note (2026-11-10, ruling 42): the tether parts of this WP are removed by WP-35..37; this file stays as history.

**Closes:** A1, A23. It writes `cli.rs` in its frozen form (`parse_any`, the shared dispatch, `Ctx`), the `cli/*.rs` entry points, and the owner's D2 additions (`cli::COMMANDS`, `cli::ok_exit`, `api::error_help`) with the D4 red-doctor exit code (through `ok_exit`) and the stub tags of the CLI stubs (rulings 2026-11-09; Craft review A10). It also adds the new `api.rs` DTOs.

**One of four parts.** WP-01 is built as WP-01a → WP-01b → WP-01c → WP-01d, in that order inside wave 1, one agent and one branch per part. Each part leaves the tree compiling and passes its own done-check. "Depends on WP-01" in another file means all four parts merged. Every frozen form below is copied from bones §2 ("Frozen signatures" and "Bones WP-01 freezes"). If this file and bones §2 disagree, bones wins. In this file, "Craft review Ax" means that review's item, not the audit ID of the same name. The `questions.md` entries for all four parts sit under its `## WP-01` heading.

**Depends on:** WP-01b. **Wave:** 1 (third of the four WP-01 parts). **Owner device:** none (host). **Network:** none. `Cargo.lock` is frozen: build with `--locked`, and never run `cargo fetch`, `cargo update` or `cargo generate-lockfile`.

**Do not touch:** anything outside the Files list below. In particular `tools/**`, `packaging/**`, `decky/**`, `.gitignore`, `recipes/**`, every doc except `docs/code-map.md`, and the 52 moved test files that get no sanctioned edit (they move byte-identical). No GitHub writes. No `cargo update` and no `cargo generate-lockfile`.

**Order of work** (each step ends with a build that compiles; commit after each):
1. Write the `api.rs` additions and the two `Serialize` derives.
2. Write the `cli/*.rs` files (the `run` stubs, the working `cli/undo.rs`, `cli::view::parse`, `request_3d`, `setup_fetch`).
3. Write `cli.rs` (parse_any, dispatch, registry, exit hook) and its unit test.
4. Write `tests/cli_any.rs`.
5. Run the done-check.

**Files (exclusive):** `crates/engine/src/cli.rs`, `crates/engine/src/cli/{engine,inject,undo,link,view,setup,setup_fetch,doctor}.rs`, `crates/engine/src/api.rs`, `crates/engine/src/face/{view,toasts}.rs`, `crates/engine/tests/cli_any.rs`. In detail:
- **New:**
  - Engine `crates/engine/src/`: `cli/{engine,inject,undo,view,setup,setup_fetch,doctor}.rs`. `cli/undo.rs` is new because today's undo dispatch body moves into it; it is not a git mv.
  - Engine `crates/engine/tests/`: `cli_any.rs`.
- **Edited:** engine `src/`: `cli.rs`, `cli/link.rs` (adds `run` and `request_3d`; the parse half WP-01a moved stays as it is), `api.rs`, `face/view.rs` and `face/toasts.rs` (Serialize derives).
- **Not edited:** `Cargo.*`, `check.yml`, `lib.rs`, any moved test file. **There is no `mod.rs` anywhere and no `x/x.rs`.**
- **Ownership after this part:** `cli.rs` and `api.rs` are frozen (§0 "Frozen after wave 1", `00-run-the-pass.md`), except the one `api.rs` line WP-01d adds. The named WPs fill the `cli/*.rs` bodies only.

---

#### Module facades (explicit re-exports, never `*`)

`cli.rs`:
- Adds `pub mod engine; pub mod inject; pub mod undo; pub mod link; pub mod view; pub mod setup; pub mod setup_fetch; pub mod doctor;`.
- **Removes `use crate::undo;` (cli.rs:6)**, which would clash with `pub mod undo` (E0255). `cli/undo.rs` calls `crate::undo::…` by its full path.
- `pub mod link;` is already there from WP-01a; the list above is the final one.

**Glob-import guard:** no new pub item in `api`, `install` or `tether::relay` may be named `json`, `c`, `p`, `H` or `full`, and none of those three modules gets a glob re-export. Locked tests glob-import them (install_plan.rs:3, api_snapshot.rs:3, tether_tunnel.rs:3, tether_relay_cmds.rs:3, tether_tunnel_review.rs:3).

---

#### Frozen signatures (stubs compile; this part writes the "full" ones in full)

Stub rules (the tag list covers WP-01d's stubs as well; this part writes the `cli::…` ones):
- A stub that returns `Result` gives `Err(NotBuilt("<path> (WP-xx)").into())` / `ApiError{code:"not-built"}` or `Err("not-built: <fn> (WP-xx)".into())`.
- **Every not-built stub names the WP that fills it** (Craft review A10). The tags are:
  - `cli::engine::run (WP-07)`, `cli::inject::run (WP-27)`, `cli::link::run (WP-08)`, `cli::link::request_3d (WP-08)`, `cli::view::run (WP-28)`, `cli::setup::run (WP-29)`, `cli::doctor::run (WP-29)`, `cli::setup_fetch::setup_fetch (WP-27)`;
  - `install::execute (WP-20)`, `install::execute_uninstall (WP-20)`, `injectors::ensure_applied (WP-27)`, `route (WP-31)`;
  - today's `NotBuilt("injectors::inject")` (injectors.rs:61) is retagged `injectors::inject (WP-27)` (WP-01b already did this);
  - any other stub that returns not-built takes the tag of the WP this file names as its body owner.
- `tests/architecture.rs` checks that every tag names a real WP (see Tests).
- Any other stub returns a neutral value, or delegates to the legacy fn as stated.
- Parameters take a `_` prefix. There is no `todo!()`, `unimplemented!()` or panic.
- WP-01b gave the old dispatch fallback the interim text `cli::engine::run (WP-07)`. That line and its unit test go away here, with the rewrite below.

**`cli.rs`** (frozen after this WP):
```rust
pub struct Ctx<'a> { pub paths: &'a EnginePaths, pub runner: &'a mut dyn CommandRunner, pub relay: &'a mut dyn RelayRunner, pub json: bool }
pub struct AnyCli { pub command: AnyCommand, pub json: bool }
pub enum AnyCommand { Engine(Command), Link(LinkCommand), View(ViewCommand), Setup(SetupCommand), Doctor(DoctorCommand) }
pub enum ViewCommand { View, Rows { filter: String }, Launch { add: bool, options: String },
    Collection { current: Option<Vec<u32>> }, Toasts { last_seq: u64 }, Logs { limit: usize } }
pub enum SetupCommand { Packages, InstallApps, LaunchEnv { appid: Option<u32> }, Fetch { key: String } }
pub enum DoctorCommand { Run }
pub fn parse_any(args: &[String]) -> Result<AnyCli, ApiError>
pub fn execute_with(args: &[String], ctx: &mut Ctx<'_>) -> Output
impl AnyCommand { pub fn id(&self) -> &'static str }
pub enum SideEffect { ReadOnly, Probes, WritesFiles, RunsCommands }
pub struct CommandSpec { pub id: &'static str, pub group: &'static str, pub args: &'static str,
    pub output: &'static str, pub effects: &'static [SideEffect] }
pub static COMMANDS: &[CommandSpec]
pub fn ok_exit(cmd: &AnyCommand, data: &serde_json::Value) -> i32
```
- All the enums derive `Debug, Clone, PartialEq, Eq`. There is no `InjectCommand`, and `now` is never a `Ctx` field.
- `parse_any`:
  1. Split at the first argv element exactly equal to `--`.
  2. Remove `--json` from the head only.
  3. Try, in order: `cli::link::parse_link` → `cli::view::parse` → setup (private) → doctor (private) → the locked `parse`. Each parser gets the head plus `--` plus the tail, unchanged.
  - The setup grammar: `setup packages`, `setup install-apps`, `setup launch-env [<appid>]` and `setup fetch <key>`. `launch-env abc` and `fetch` with no key give a usage error.
  - The doctor grammar is `doctor`.
  - A usage error has code `usage` and keeps the locked text.
  - `Logs.limit` defaults to `headset_view::DEFAULT_LOG_LINES` (500).
- `execute(args) -> Output` keeps its signature, and `main` still calls `cli::run`. It:
  1. parses first, so a usage error never needs HOME;
  2. builds `EnginePaths::from_env(&std::env::vars().collect(), sys::current_uid())`, plus a `SystemRunner` and a `ProcessRunner`;
  3. calls the shared dispatch.
- The render name is `Command::name()` for an engine command, and the first head token otherwise. A usage error renders with the empty name `""`, as today (locked `cli_json.rs`).
- **Exit hook (full, D2/D4):** the shared dispatch calls the locked `render` (cli.rs:102) unchanged. Then, on `Ok`, it sets `Output.code = ok_exit(&cmd, &data)`.
  - `ok_exit` returns 1 for `Doctor(DoctorCommand::Run)` when `data["ok"] == false` (a red report), and 0 otherwise. stdout still carries the full report.
  - Errors keep the locked `exit_code` (cli.rs:93).
- **Dispatch:**

  | Command | Goes to |
  |---|---|
  | `List`, `Status` | `cli::engine::run` |
  | `Inject` | `cli::inject::run` |
  | `Undo` | `cli::undo::run` |
  | `Start3d`/`Stop3d` | `cli::link::request_3d(Request::Start3d\|Stop3d, ctx.paths, sys::unix_now())`, serialised with `serde_json::to_value` |
  | Link, View, Setup, Doctor | their module's `run` |

- Per-module entry points (exactly one `run` each):

  | Module | Signature | WP-01 body |
  |---|---|---|
  | `cli/engine.rs`, `cli/inject.rs` | `pub fn run(cmd: &cli::Command, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError>` | `Err(NotBuilt("cli::<m>::run (WP-xx)").into())`: engine WP-07, inject WP-27 |
  | `cli/undo.rs` | same | **working**: today's dispatch body, `crate::undo::undo_game(&ctx.paths.data, appid)` |
  | `cli/link.rs` | `run(cmd: &LinkCommand, ctx)`, same return | not-built (WP-08) |
  | `cli/view.rs` | `run(cmd: &ViewCommand, ctx)` | not-built (WP-28) |
  | `cli/setup.rs` | `run(cmd: &SetupCommand, ctx)` | not-built (WP-29) |
  | `cli/doctor.rs` | `run(cmd: &DoctorCommand, ctx)` | not-built (WP-29) |

- The only other pub items under `cli/`:
  - `cli::link::request_3d(r: state::Request, p: &EnginePaths, now: u64) -> Result<api::LinkReport, ApiError>`. The stub returns not-built and spawns nothing (A24).
  - `cli::view::parse(args: &[String]) -> Option<Result<ViewCommand, String>>`. `None` means "not a view verb". The stub accepts `view` and `logs` (`logs` gives `Logs{limit: DEFAULT_LOG_LINES}`, as `cli_any.rs` expects), and WP-28 fills the rest of the grammar.
  - `cli::setup_fetch::setup_fetch(key: &str, p: &EnginePaths, r: &mut dyn CommandRunner) -> Result<api::SetupReport, ApiError>`. The stub returns not-built.
  - `parse_link`, `LinkCommand` and `SettingKey` stay pub because they are locked.
- **Command registry** (`cli::COMMANDS`, full, D2):
  - One row per command, stubbed ones included. It is the one owner of command names, args and effects.
  - `id` is the command words as typed (`list`, `setup fetch`). `group` is the `AnyCommand` family. `args` is the argument form after the id. `output` is the type name of `data` (an `api` DTO, or `ToastBatch` from `face::toasts`), the key set of an untyped `json!` object, or empty when the command prints no envelope.
  - `SideEffect`: `ReadOnly` means the runner ran nothing and the file tree is byte-identical. `Probes` means only read-only probe argv ran. The name is not `Effect`, which `watch` already uses.
  - `AnyCommand::id()` returns the row id. For `Engine(c)` it returns the locked `c.name()` (cli.rs:30). For the other families it returns the words as typed, so `SwitchGet` gives `switch get`. `id()` is not the render name.
  - The 26 rows. `effects` is WP-01's best reading of each command; the WP that fills a body keeps its row true and adds a test for it.

    | id | group | args | output | effects |
    |---|---|---|---|---|
    | `list` | Engine | | `Game` (array) | ReadOnly |
    | `inject` | Engine | `<appid>` | `InjectResult` | WritesFiles, RunsCommands |
    | `start-3d` | Engine | | `LinkReport` | WritesFiles |
    | `stop-3d` | Engine | | `LinkReport` | WritesFiles |
    | `status` | Engine | | `LinkReport` | ReadOnly |
    | `undo` | Engine | `<appid>` | `UndoResult` | WritesFiles |
    | `watch` | Link | | | WritesFiles, RunsCommands |
    | `snapshot` | Link | | `LinkSnapshot` | ReadOnly |
    | `install` | Link | | `InstallReport` | WritesFiles, RunsCommands |
    | `uninstall` | Link | | `InstallReport` | WritesFiles, RunsCommands |
    | `switch get` | Link | `<appid>` | `SwitchState` | ReadOnly |
    | `switch set` | Link | `<appid> on\|off` | `SwitchState` | WritesFiles |
    | `setting get` | Link | `<key>` | `SettingState` | ReadOnly |
    | `setting set` | Link | `<key> on\|off` | `SettingState` | WritesFiles |
    | `run` | Link | `<appid> -- <command...>` | | WritesFiles, RunsCommands |
    | `view` | View | | `LinkView` | ReadOnly |
    | `rows` | View | `[--filter <status>\|all]` | `GameRow` (array) | ReadOnly |
    | `launch` | View | `add\|remove --options <options>` | `LaunchText` | ReadOnly |
    | `collection` | View | `[--current <appids>]` | `CollectionPlan` | ReadOnly |
    | `toasts` | View | `<last_seq>` | `ToastBatch` | ReadOnly |
    | `logs` | View | `[--limit <n>]` | `LogLines` | ReadOnly |
    | `setup packages` | Setup | | `{wivrn,gnirehtet,missing}` | Probes |
    | `setup install-apps` | Setup | | `SetupReport` | WritesFiles, RunsCommands |
    | `setup launch-env` | Setup | `[<appid>]` | `LaunchText` | Probes |
    | `setup fetch` | Setup | `<key>` | `SetupReport` | WritesFiles, RunsCommands |
    | `doctor` | Doctor | | `DoctorReport` | Probes |

  - `tests/cli_any.rs` checks the registry (see Tests). The other faces are checked against it by their own WPs: the Decky wrappers in WP-28's `decky_face.rs`, the app's effect args in WP-33's `engine::effect_args` test, and the code-map `## Commands` table in WP-34.
- **Unit test (A23):** replace `execute_human_not_built_goes_to_stderr` (cli.rs:198), which asserts that `list` is unbuilt and is not locked, with `dispatch_routes_each_family`.
  - It is a table with one row per dispatch row: `list`, `status`, `inject 5`, `undo 5`, `start-3d`, `stop-3d`, one link verb (`snapshot`), `view`, `setup packages` and `doctor`.
  - Each row builds a `Ctx` from `EnginePaths::from_env` over a temp map (`HOME` and `XDG_RUNTIME_DIR` under a temp dir) and `FakeRunner`s. It never reads the real env.
  - **Rows check routing only**, so a later WP that fills a body does not break the test. A row passes when the call returns `Ok`, an error that is not `not-built`, or a `not-built` error whose message names that row's module (`cli::engine::run` for `list`, `cli::link::request_3d` for `start-3d`, and so on). It fails when the message names any other module.
  - `undo 5` is never `not-built`.
  - One more case checks the exit hook: `ok_exit(&Doctor(Run), &json!({"ok": false}))` is 1, and `json!({"ok": true})` gives 0, as does any other command.

**`api.rs`** DTOs:
- Every field is `pub`. Each type derives `Debug, Clone, PartialEq, Serialize, Deserialize`, except `LinkView`, which derives `Debug, Clone, PartialEq, Serialize` only.
  - `LinkView{state:LinkState, label:String, hint:String, start:bool, stop:bool, health:HealthView, rx:String, tx:String, uptime:String, tether:TetherMode, tunnel:bool, auto_3d:bool, adb_source:AdbSource}`
  - `LaunchText{options:String}`
  - `CollectionPlan{name:String, wanted:Vec<u32>, add:Vec<u32>, remove:Vec<u32>}`
  - `LogLines{lines:Vec<String>, text:String}`
  - `DoctorCheck{name:String, ok:bool, detail:String, fix:String}`, `DoctorReport{ok:bool, checks:Vec<DoctorCheck>}`
  - `SetupReport{steps:Vec<String>, summary:String}`
  - `Toast{title:String, body:String}`
- `pub const TOAST_TITLE: &str = "Flat to 3D";`. The named re-export `pub use crate::face::rows::GameRow;` is WP-01d's line, because `face/rows.rs` lands there.
- Add `Serialize` to `HealthView` (face/view.rs:15) and `ToastBatch` (face/toasts.rs:6).
- `InstallReport`, `LinkReport`, `Health`, `Stats`, `EventKind` and `ERROR_CODES` (the 8 locked codes) are unchanged. The code set is closed for this pass (D6): no WP adds a code.
- **`pub fn error_help(code: &str) -> (bool, &'static str)`** (full, D2) returns `(retryable, fix)`. Where one command fixes the error, `fix` names it. An unknown code gives `(false, "")`. The `ApiError` JSON stays `{code, message}`.

  | code | retryable | fix |
  |---|---|---|
  | `usage` | false | `check the command against the usage line in the message` |
  | `not-built` | false | `this part is not built yet; the message names the work package that builds it` |
  | `not-found` | false | `check the name or path in the message; flat3d-inject list shows the installed games` |
  | `bad-recipe` | false | `fix the recipe file named in the message` |
  | `refused` | false | `do what the message names first, then run the command again` |
  | `link-not-ready` | true | `systemctl --user restart flat3d-link.service` |
  | `command-failed` | true | `check the command named in the message, then run it again` |
  | `io` | true | `check the path in the message, then run the command again` |

  A unit test in `api.rs` checks that every `ERROR_CODES` entry has a non-empty fix (the D2 test; it has no second home in `cli_any.rs`).

---

#### Tests

The moved test files stay byte-identical to what WP-01a left (the done-check's `locked-14` step proves it). The unit tests are named above: `dispatch_routes_each_family` in `cli.rs` and the `error_help` test in `api.rs`.

**New engine test** (`crates/engine/tests/`):
- `cli_any.rs`:
  - `run 5 -- a --json` keeps `--json` in the tail and gives `json=false`;
  - `view --json` gives `View` with `json=true`;
  - `logs` gives `Logs{limit: DEFAULT_LOG_LINES}`;
  - `setup launch-env abc` gives code `usage`;
  - `fly` gives code `usage` with the locked USAGE text;
  - Sample values for the registry parse check: `<appid>` → `5`; `<key>` → `auto-3d` in a `setting` row and `uevr` in `setup fetch`; `on|off` → `on`; `add|remove` → `add`; `<options>` → `%command%`; `<command...>` → `true`; `<last_seq>` → `0`; a `[...]` part is left out.
  - Command words of a usage string: take the text after `usage: flat3d-inject `, drop a trailing `[--json]`, split on ` | `, and from each piece keep the leading tokens up to the first one that starts with `<` or `[`, contains `|`, or is `--`. So `switch set <appid> on|off` gives `switch set`, and `setting get auto-3d|tunnel` gives `setting get`. The private strings are read through the parsers: `parse(&["inject"])` and `parse(&["undo"])` return `INJECT_USAGE` and `UNDO_USAGE`; `parse_link` on `["switch"]`, `["setting"]` and `["run"]` returns the link forms. `watch`, `snapshot`, `install` and `uninstall` have no usage string.
  - **registry checks** (D2): every `COMMANDS` row's `id`, with a sample value for each `args` placeholder, parses through `parse_any` to an `AnyCommand` whose `id()` is that row's id; every `AnyCommand` variant's `id()` has exactly one row; and every command word in the locked usage strings has a row: `USAGE`, `INJECT_USAGE` and `UNDO_USAGE` (cli.rs:50-53), plus the `usage("<form>")` forms in `cli/link.rs`. The parse check loops over every row. It skips a `View`-group row other than `view` only while `parse_any` returns an error for it, so those rows join the check with no edit to this file once WP-28 fills the view grammar (WP-28 keeps its own parity test as well);

---

**Done-check** (run after committing on this part's branch, `v2/wp-01c-*`). Each step is labelled. A failing step prints `FAIL <step>: fix: <hint>` and stops (Craft review A2).
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
- It also has `## Not verified`: each skip the run hit. For WP-01c that is the `capture` clippy (CI only), the `py` and `decky` jobs, `check.sh` if WP-02 has not merged, and the CI run itself if it was not seen to finish.
- The merge takes only the head commit the review passed.
- WP-01c has no device Q row, so it keeps `Closes` for its issues (D5 applies only to WPs with a device half).
