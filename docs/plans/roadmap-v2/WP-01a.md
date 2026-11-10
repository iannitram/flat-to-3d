<!-- split from WP-01.md (roadmap-v2.md lines 755-1385 at c0e6874), part a of four -->
### WP-01a: Workspace reshape: moves, manifests, CI

**Closes:** A2-A4, A9, A10, A22, C9, K1, K3. It does the workspace part of the reshape: every file move, every `Cargo.*` file, the two new crates as compiling skeletons, the path rewrite of `docs/code-map.md` and the final `check.yml`. It lands the `mod_module_files` layering lint (rulings 2026-11-09; Craft review A2, A6). No new behaviour: code only moves. WP-01b to WP-01d add the new modules, the CLI and the stubs.

**One of four parts.** WP-01 is built as WP-01a → WP-01b → WP-01c → WP-01d, in that order inside wave 1, one agent and one branch per part. Each part leaves the tree compiling and passes its own done-check. "Depends on WP-01" in another file means all four parts merged. Every frozen form below is copied from bones §2 ("Frozen signatures" and "Bones WP-01 freezes"). If this file and bones §2 disagree, bones wins. In this file, "Craft review Ax" means that review's item, not the audit ID of the same name. The `questions.md` entries for all four parts sit under its `## WP-01` heading.

**The four WP-01 parts are the only WPs that move files or add `mod` lines. WP-01a alone adds dependencies or features and edits `Cargo.*`/`check.yml`. `Cargo.lock` is frozen after WP-01a:** it declares every dependency the whole pass needs, so WP-01b to WP-01d never touch `Cargo.*`.

**Depends on:** WP-00 (rustfmt baseline). **Wave:** 1 (first of the four WP-01 parts; the chain runs in parallel with WP-02 and WP-25a). **Owner device:** none (host). **Network:** one online `cargo fetch` (crates.io source; openxr and ashpd 0.13.13 are not cached). Nothing else.

**Do not touch:** anything outside the Files list below. In particular `tools/**`, `packaging/**`, `decky/**`, `.gitignore`, `recipes/**`, every doc except `docs/code-map.md`, and the 52 moved test files that get no sanctioned edit (they move byte-identical). No GitHub writes. No `cargo update` and no `cargo generate-lockfile`.

**Order of work** (each step ends with a build that compiles; commit after each):
1. Seed the lock ("Fetching dependencies" step 1). `git mv` `inject/` and `app/` under `crates/`. Write the root `Cargo.toml` and the four crate manifests. Add the viewer and uevr-inject crates as stubs. Run the one `cargo fetch`. Make the 7 sanctioned test edits, and rewrite the `docs/code-map.md` paths in the same commit (three locked tests read them).
2. Do the moves inside the engine (`cli/link.rs`, `face/`, `tether.rs`) and the `lib.rs` re-exports.
3. Split `watch.rs` behind its facade.
4. Make the doc fixes and rewrite `.github/workflows/check.yml`.
5. Run the done-check.

**Files (exclusive):** `Cargo.toml`, `Cargo.lock`, `inject/**`, `app/**`, `crates/engine/**`, `crates/app/**`, `crates/viewer/**`, `crates/uevr-inject/**`, `.github/workflows/check.yml`, `docs/code-map.md`. In detail:
- **Moves** (`git mv`, so history follows):
  - `inject/**` → `crates/engine/**` and `app/**` → `crates/app/**`.
  - Inside the engine:
    - `src/cli_link.rs` → `src/cli/link.rs`;
    - `src/{collection,headset_view,link_view,toasts,view}.rs` → `src/face/`;
    - `src/tether/mod.rs` → `src/tether.rs`;
    - `src/watch.rs` → `src/watch/service.rs`. Then `frames`, `machine`, `files` and `run` are split out of `service.rs` into their own files, and a new `src/watch.rs` holds only the facade.
  - Packages, libs and bins keep their names: `flat3d-inject` / `flat3d_inject` / `[[bin]] flat3d-inject` and `flat3d-app` / `flat3d_app`.
  - Delete the untracked `inject/Cargo.lock`, `app/Cargo.lock` and the per-crate `target/` dirs where they exist (a clean checkout has none of them), but only after step 1 of "Fetching dependencies" has seeded the root lock.
  - **There is no `mod.rs` anywhere and no `x/x.rs`.** Every module with children uses the facade-file form (`x.rs` + `x/`).
- **New:**
  - Root: `Cargo.toml`, `Cargo.lock`.
  - Engine `crates/engine/src/`: `face.rs`; `watch/{frames,machine,files,run}.rs`, plus the facade `watch.rs`.
  - Viewer `crates/viewer/` (everything but `tests/spec.rs`, which WP-01b adds):
    - `Cargo.toml`;
    - `src/{lib,main,frame,spec,source,layout,status}.rs`;
    - `src/source/{test_pattern,pipewire,gamescope,portal}.rs`;
    - `src/gpu.rs`, `src/gpu/import.rs`;
    - `src/xr.rs`, `src/xr/{session,swapchain,frame}.rs`.
  - uevr-inject: `crates/uevr-inject/{Cargo.toml,src/{lib,main,args,pick,win}.rs}`.
- **Edited:**
  - Engine `src/`: `lib.rs`, `main.rs` (stale `//!` on line 1, A2), `cli.rs` (one `pub mod link;` line), `cli/link.rs` (the moved parse half), `injectors.rs` (A4 doc on line 1 only).
  - `crates/engine/Cargo.toml`, `crates/app/Cargo.toml`.
  - The 3 locked-path test files `api_response.rs`, `link_stop.rs` and `recipe_load.rs` get the 7 sanctioned edits (see Tests). `wrapper_run.rs` is **not** edited.
  - `docs/code-map.md`, `.github/workflows/check.yml`.
- **Left for later parts** (do not write them here): `exec.rs`, `paths.rs`, `fsx.rs`, `sys.rs`, `catalog.rs`, `testing.rs` and every new engine test (WP-01b); the `cli/*.rs` files other than `link.rs` and the `api.rs` additions (WP-01c); `face/{rows,link_model,logs}.rs`, `link/wivrn.rs`, `watch/health_loop.rs`, `injectors/{geo11,port,fetch}.rs` and the app page files (WP-01d).
- **Ownership after this part:** `Cargo.*` and `check.yml` are frozen (§0 "Frozen after wave 1", `00-run-the-pass.md`). The full ownership list is in WP-01d.

---

#### Cargo

Root `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/*"]
default-members = ["crates/engine", "crates/viewer", "crates/uevr-inject"]

[workspace.package]
edition = "2021"
license = "GPL-3.0-only"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
iced = { version = "0.14.0", features = ["smol"] }
flat3d-inject = { path = "crates/engine" }
flat3d-viewer = { path = "crates/viewer", default-features = false }
windows-sys = "=0.61.2"

[workspace.lints.clippy]
field_reassign_with_default = "allow"
mod_module_files = "deny"
```
`mod_module_files` is off by default. Turning it on as `deny` makes clippy enforce the facade-file form (no `mod.rs`) in every crate, including integration-test support modules (Craft review A6).

Every crate:
- keeps its own `version = "0.0.0"` and `publish = false`;
- sets `edition.workspace = true`, `license.workspace = true` and `[lints] workspace = true`.

`crates/engine/Cargo.toml`:
- Keeps `[package] name = "flat3d-inject"`, `[lib] name = "flat3d_inject"` and `[[bin]] name = "flat3d-inject" path = "src/main.rs"`. Locked `cli_json.rs` needs `env!("CARGO_BIN_EXE_flat3d-inject")`.
- `[[bin]] flat3d-run` stays as it is today.
- `description = "Flat to 3D engine: installed games, recipes, stereo routes, the link service and the launch wrapper"` (C9).
- Deps: `serde`, `serde_json` and `flat3d-viewer`, each `{ workspace = true }`.
  - Because the viewer has no default features, the engine never pulls in openxr or pipewire.
  - This edge is one-way: the viewer never depends on the engine.
- `sys::current_uid` keeps today's `extern "C" { fn getuid() -> u32; }` (flat3d-run.rs:8-13), so the engine needs no `libc`.

`crates/app/Cargo.toml`: `flat3d-inject`, `iced` (adds `smol`) and `serde_json`, each `{ workspace = true }`. Today it has `path = "../inject"`.

`crates/viewer/Cargo.toml` (package `flat3d-viewer`, lib `flat3d_viewer`):
```toml
[features]
default = []
xr = ["dep:openxr", "dep:ash"]
capture = ["dep:pipewire", "dep:ashpd", "dep:tokio"]

[dependencies]
serde = { workspace = true }
libc = "0.2"
openxr = { version = "=0.22.0", features = ["loaded"], optional = true }
ash = { version = "=0.38.0", optional = true }
pipewire = { version = "=0.10.1", optional = true }
ashpd = { version = "=0.13.13", default-features = false, features = ["screencast", "tokio"], optional = true }
tokio = { version = "1", default-features = false, features = ["rt", "time"], optional = true }

[[bin]]
name = "flat3d-viewer"
path = "src/main.rs"
required-features = ["xr", "capture"]
```
`libc` is not optional because WP-19 uses it for `sigaction`/`pidfd`.

`crates/uevr-inject/Cargo.toml` (package `flat3d-uevr-inject`, lib `flat3d_uevr_inject`, bin `flat3d-uevr-inject`):
```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { workspace = true, features = [
  "Win32_System_Diagnostics_ToolHelp", "Win32_System_Memory", "Win32_System_Threading",
  "Win32_System_LibraryLoader", "Win32_Foundation", "Win32_System_Diagnostics_Debug",
  "Win32_Security",
] }
```
`Win32_Security` is there because `CreateRemoteThread` is gated on it and takes `*const SECURITY_ATTRIBUTES`.

**Fetching dependencies** (the executor's job):
1. `cp app/Cargo.lock Cargo.lock` as the seed. Do this before the git mv deletes the untracked file, and keep a copy in `$SCRATCH`. The file is untracked, so a clean checkout does not have it: in that case skip the seed, and step 2 writes a fresh root `Cargo.lock`.
2. Run `cargo fetch` once, online. It is the single declared network use, and it adds the missing crates while keeping the seeded pins (or resolves the whole lock when there was no seed).
   - Do **not** run `cargo generate-lockfile` or `cargo update`, because they re-resolve everything.
3. Check: `cargo metadata --locked --offline --format-version 1 >/dev/null`.
4. Commit `Cargo.lock`. After WP-01 it never changes, because every gate uses `--locked`.
5. Cargo.lock lists optional dependencies even when their features are off, so this one lock covers `xr`, `capture` and the `cfg(windows)` deps.

---

#### Module facades (explicit re-exports, never `*`)

Engine `lib.rs`:
- Fix the stale doc (lines 1-6, A3).
- Keep a `pub mod` line for every top-level module. After the moves, this part's form is:
  ```rust
  pub mod face;
  pub use cli::link as cli_link;
  pub use face::{collection, headset_view, link_view, toasts, view};
  ```
  - The old top-level `pub mod collection; …; pub mod cli_link;` lines are removed. The re-exports keep every locked `flat3d_inject::<name>` path resolving.
  - WP-01b adds the `pub mod` lines of its new modules. WP-01d adds `rows` to the `face` re-export and the `pub use link::wivrn;` line.
- `NotBuilt` keeps its crate-root definition (lib.rs:39-47). Locked `api_response.rs:5,27` imports `flat3d_inject::NotBuilt`, and `From<NotBuilt> for ApiError` stays at api.rs:103.

`face.rs`: `pub mod collection; pub mod headset_view; pub mod link_view; pub mod toasts; pub mod view;` (WP-01d adds `rows`, `link_model` and `logs`).

`watch.rs` (facade only):
```rust
mod frames; mod machine; mod files; mod service; mod run;
pub use frames::{parse_track_frames, Device};
pub use machine::{step, link_commands, note_text, note_kind, note_event, Effect, Event, Inputs, Note, Phase, Watch, GRACE_SECS};
pub use files::{write_snapshot, Paths};
pub use service::{Feed, Service, TrackDevices};
pub use run::run;
```
- WP-01d adds `mod health_loop;` and its re-export line. Until then the list is exactly this one.
- The submodules stay private `mod`, as written. Moved code reaches a sibling with `super::<file>::<item>`.
- Which item goes to which file follows the re-export list: `frames.rs` gets `Device`, `parse_track_frames` and their private helpers; `machine.rs` gets `GRACE_SECS`, `Phase`, `Watch`, `Event`, `Inputs`, `Note`, `Effect`, `step`, `link_commands`, `note_text`, `note_kind` and `note_event`; `files.rs` gets `Paths`, `write_snapshot` and the private file helpers (`append_log`, `take_request`, `game_modes`); `run.rs` gets `run`; everything else (`Feed`, `TrackDevices`, `Service`, `Shared`, `running_adb_exe`) stays in `service.rs`. Each `#[cfg(test)]` module moves with the code it tests.
- `note_event` stays a `pub fn` in `machine.rs` (watch.rs:452 today).
- The submodule is named `files`, not `paths`, so it never clashes with `crate::paths`.
- Locked tests need 13 of these names (watch_manual.rs:4-6, watch_frames.rs:3, watch_grace.rs:3).

`cli.rs`: add `pub mod link;` (the moved `cli_link.rs`). Nothing else in `cli.rs` changes in this part; WP-01c adds the other `pub mod` lines and rewrites the dispatch.

Viewer `lib.rs`:
```rust
pub mod frame; pub mod spec; pub mod layout; pub mod status; pub mod source;
#[cfg(feature = "xr")] pub mod xr;
#[cfg(feature = "xr")] pub mod gpu;
```
- `gpu.rs` holds `pub mod import;`.
- `xr.rs` holds `pub mod session; pub mod swapchain; pub mod frame;`.

uevr-inject `src/lib.rs`: `pub mod args; pub mod pick; #[cfg(windows)] pub mod win;`

#### New crate skeletons

**Viewer** (all stubs in this part):
- Stubs that compile empty: `layout.rs`, `status.rs`, `source/*.rs`, `gpu/import.rs` and `xr/*.rs`.
- `main.rs` is `fn main() {}` plus a one-line `eprintln!("flat3d-viewer: not built (WP-19)")` and `std::process::exit(2)`. It builds only with `--features xr,capture`.
- `frame.rs` and `spec.rs` are empty here, and `source.rs` holds only its four `mod` lines:
  ```rust
  pub mod test_pattern;
  #[cfg(feature = "capture")] pub mod pipewire;
  #[cfg(feature = "capture")] pub mod gamescope;
  #[cfg(feature = "capture")] pub mod portal;
  ```
  WP-01b writes all three in full.

**uevr-inject stubs:**
- `args.rs` and `pick.rs` are empty, and `win.rs` is empty.
- `main.rs`:
  - on non-Windows it prints `flat3d-uevr-inject: windows only` and exits 3;
  - on Windows it prints `flat3d-uevr-inject: not built (WP-15)` and exits 5.
- WP-15 overwrites all four bodies. The final exit codes are WP-15's: 2 bad args, 3 not Windows, 4 no game found, 5 inject failure.

---

#### Behaviour wired now

Nothing changes behaviour in this part. It makes these text edits:
- **Doc fixes:** `lib.rs:1-6` (A3), `main.rs:1` (A2) and `injectors.rs:1` (A4).
- **`docs/code-map.md`:**
  - Rewrite every `inject/` → `crates/engine/` and `app/src` → `crates/app/src` path token, using the new module paths (`cli/link.rs`, `face/*.rs`, `tether.rs`, `watch.rs` + `watch/`).
  - Add the `<!-- mv-map -->` … `<!-- /mv-map -->` block holding the bones §2 git-mv map. It is the one exemption in `check.py`.
  - The `cli_view.rs` and `rows.rs` parts become their own rows (`crates/engine/src/cli/view.rs`, `crates/engine/src/face/rows.rs`) marked `planned` (A9/A10).
  - Rows 56, 59 and 70 (as at 2812f64) are written exactly as ``| `crates/engine/src/recipes.rs` | present |``, ``| `crates/engine/src/link.rs` | present |`` and ``| `crates/engine/src/api.rs` | present |``. The locked tests read these cells.
  - WP-01d adds the rows for the new stub files.
- **`.github/workflows/check.yml`:** exactly bones §2 "CI final shape".
  - Every job uses `runs-on: [self-hosted, linux]` and `dtolnay/rust-toolchain@1.95.0`.
  - **One gate (Craft review A2):** each job does its setup, then runs `tools/check.sh --ci --only <job>` (`rust`, `py` or `decky`). That step carries `if: hashFiles('tools/check.sh')!=''`. `--ci` turns every skip for a missing tool or target into a failure. A step whose input file has not landed yet still skips.
  - **Setup, never skipped:** the apt line, the toolchain pin and `rustup target add x86_64-pc-windows-msvc` (rust); setup-node 22.23.4 and pnpm 10.34.9 (decky). The setup steps carry no `if:`.
  - Actions: every job starts with `actions/checkout@v4`. The `py` job keeps today's `actions/setup-python@v5` with `python-version: "3.12"`. The `decky` job uses `actions/setup-node@v4` with `node-version: 22.23.4` and `pnpm/action-setup@v4` with `version: 10.34.9`. The two old jobs (`engine`, `app`) are replaced by `rust`, `py` and `decky`.
  - **Wave-1 fallback:** the inline steps below run only while check.sh is missing. Each carries `if: hashFiles('tools/check.sh')==''`, with its own file guard added to that one. WP-02's check.sh runs the same steps with the same flags and file guards.
  - Job `rust`:
    - apt via the existing `SUDO=$(command -v sudo || true)` pattern: `build-essential curl ca-certificates pkg-config libxkbcommon-dev libwayland-dev libfontconfig1-dev libpipewire-0.3-dev libspa-0.2-dev libclang-dev clang shellcheck libarchive-tools` (setup);
    - `cargo fmt --all --check`, printing `::error::FAIL fmt: fix: cargo fmt --all` on failure (kept from WP-00);
    - `cargo clippy --locked --workspace --all-targets -- -D warnings`, printing `::error::FAIL clippy: fix: run cargo clippy --workspace --all-targets -- -D warnings and fix each lint it names` on failure (WP-00's message, with the workspace path);
    - `cargo clippy --locked -p flat3d-viewer --all-targets --features xr,capture -- -D warnings`;
    - `XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked --workspace`;
    - `cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings`.
  - Job `py`:
    - `python3 -W error -m py_compile $(git ls-files '*.py')`;
    - `for d in $(git ls-files '*test_*.py'|xargs -n1 dirname|sort -u|grep -vx 'decky/tests'); do python3 -m unittest discover -s "$d" -p 'test_*.py'||exit 1; done`;
    - step `if: hashFiles('decky/tests/test_*.py')!=''` → `cd decky && python3 -m unittest discover -s tests -p 'test_*.py'`;
    - step `if: hashFiles('docs/workbench.md')!=''` → `python3 tools/docs/check.py`;
    - `shellcheck $(git ls-files '*.sh')` runs only inside check.sh. It has no inline fallback, because its old guard was check.sh's own existence.
  - Job `decky`: every step after the setup steps is guarded with `if: hashFiles('decky/package.json')!=''`, never at job level. Steps: `pnpm i --frozen-lockfile && pnpm run typecheck && pnpm run build`, run with `working-directory: decky`.

---

#### Tests

No new test file. Every test that passes at the base passes here, from its new path.

**Locked tests:**
- All 55 moved files stay verbatim (`git mv`) except **7 sanctioned path edits in 3 files**. Together they are 14 `-`/`+` diff lines, and all three files stay rustfmt-clean. Line numbers are as at 2812f64; if WP-00's rustfmt shifted a line, find it by content.

  | File:line | Edit |
  |---|---|
  | `api_response.rs:67` | `.join("../docs/code-map.md")` → `.join("../../docs/code-map.md")` |
  | `api_response.rs:69` | `` `inject/src/api.rs` `` → `` `crates/engine/src/api.rs` `` |
  | `link_stop.rs:110` | `.join("../docs/code-map.md")` → `.join("../../docs/code-map.md")` |
  | `link_stop.rs:112` | `` `inject/src/link.rs` `` → `` `crates/engine/src/link.rs` `` |
  | `recipe_load.rs:138` | `.join("..")` → `.join("../..")` |
  | `recipe_load.rs:146` | `.join("..")` → `.join("../..")` |
  | `recipe_load.rs:150` | `` `inject/src/recipes.rs` `` → `` `crates/engine/src/recipes.rs` `` |

- `wrapper_run.rs:206-207` is **not** edited. It is unheadered, and its `CARGO_MANIFEST_DIR/target/wrapper-tests` now resolves under `crates/engine/target/`, which `.gitignore`'s `target/` already covers.
- The 6 unheadered files (`event_ring_wrap`, `launch_options_quotes`, `runs_extra_lines`, `tether_runtime`, `tether_tunnel_review`, `wrapper_run`) are ordinary tests, but WP-01 still leaves them byte-identical.

**Inline facts:**
- iced 0.14.0 needs the `smol` feature.
- openxr 0.22.0 with `loaded` dlopens the loader, so it needs no link library.
- ashpd 0.13.13 has MSRV 1.87.
- A `dep:` feature needs `optional = true` on its dependency.
- In windows-sys 0.61.2, `CreateRemoteThread` needs `Win32_Security` + `Win32_System_Threading`.
- The `x86_64-pc-windows-msvc` target is already installed on the host, so `clippy --target` needs no download. Linking happens only in `tools/build-win.sh` (WP-15).
- The `capture` feature is not built on the host, which lacks the pipewire, spa and clang headers. CI checks it.
- `decky/main.py:3` still names `inject/src/cli.rs`. That is left for WP-21, which owns the file.

---

**Done-check** (run after committing on this part's branch, `v2/wp-01a-*`). Each step is labelled. A failing step prints `FAIL <step>: fix: <hint>` and stops (Craft review A2).
```sh
export CARGO_TARGET_DIR=$SCRATCH/target
B=$(git merge-base main HEAD)
fail() { echo "FAIL $1: fix: $2"; exit 1; }
{ test ! -e inject && test ! -e app; } || fail moved "git mv inject/ and app/ under crates/"
test -z "$(git ls-files '*mod.rs')" || fail no-mod-rs "use the facade-file form (x.rs + x/)"
cargo metadata --locked --offline --format-version 1 >/dev/null || fail lock "re-seed Cargo.lock as in Fetching dependencies"
cargo fmt --all --check || fail fmt "cargo fmt --all"
cargo clippy --locked --workspace --all-targets -- -D warnings \
  || fail clippy "run cargo clippy --workspace --all-targets -- -D warnings and fix each lint it names"
cargo clippy --locked -p flat3d-viewer --all-targets --features xr -- -D warnings || fail clippy-xr "fix the viewer lints it names"
XDG_RUNTIME_DIR=$(mktemp -d) cargo test --locked --workspace || fail test "fix the failing test it names"
cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings || fail clippy-win "fix the uevr-inject lints it names"
git diff --exit-code Cargo.lock || fail lock-frozen "restore the committed Cargo.lock; never run cargo update"
test "$(git ls-tree --name-only "$B" inject/tests/ | grep -c '\.rs$')" = 55 || fail tests-55 "the merge-base must hold 55 inject/tests files"
for f in $(git ls-tree --name-only "$B" inject/tests/); do
  test -f "crates/engine/tests/${f#inject/tests/}" || fail tests-moved "git mv $f"
done
test "$(for f in $(git ls-tree --name-only "$B" inject/tests/); do git show "$B:$f" | diff -u - "crates/engine/tests/${f#inject/tests/}"; done | grep -c '^[-+][^-+]')" = 14 \
  || fail locked-14 "undo every edit to a moved test file except the 7 sanctioned ones"
if [ -x tools/check.sh ]; then tools/check.sh || fail check.sh "fix the step check.sh names"
else echo "SKIP check.sh: WP-02 not merged yet"; fi
```
- `$B` is the merge-base, so the check stays valid after `main` moves.
- The `locked-14` step proves that exactly the 7 sanctioned edits (14 diff lines) touched the 55 moved test files.
- There is no `runners` or `tests-new` step yet: the exec split and the new test files are WP-01b's.
- Python, shellcheck and decky are not run here. Their CI steps are guarded until WP-02, WP-21 and WP-25b land.
- The `capture` clippy is not run here (no pipewire, spa or clang headers on the host); CI runs it.

**PR and merge** (rulings D1, D5):
- The PR body has `## Verified`: each done-check step and its exit status, quoted from the run. It never claims a run that was not done.
- It also has `## Not verified`: each skip the run hit. For WP-01a that is the `capture` clippy (CI only), the `py` and `decky` jobs, `check.sh` if WP-02 has not merged, and the CI run itself if it was not seen to finish.
- The merge takes only the head commit the review passed.
- WP-01a has no device Q row, so it keeps `Closes` for its issues (D5 applies only to WPs with a device half).
