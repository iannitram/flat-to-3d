<!-- split from roadmap-v2.md lines 1-137 at c0e6874 -->
# flat-to-3d roadmap v2 (2026-11-09)

Sections 0-7 and WP-01 are the corrected contract. The other WPs predate it: before coding a WP, read sections 0 and 2 (they win), then the WP, then `roadmap-v2-findings/WP-NN.json` and `WP-NN-proposal.md` if present (reviewer fixes checked against the code, not yet applied).

## 0. How to run the pass

- **Base:** flat-to-3d `main@2812f64`. One branch per WP, `v2/wp-NN-<slug>`. Merge order: wave 0 → wave 1 → … → wave 5. Inside a wave, WPs own disjoint files and merge in any order. A WP starts only after every WP in its `depends` is merged. A WP never depends on a WP in its own wave.
- **Wave 0 (WP-00, #183) lands alone.** It is format-only plus a clippy allow. Every later diff then stays readable.
- **Wave 1:** WP-01 (Rust reshape + bones, one agent), WP-02 (repo tooling, no Rust) and WP-25a (`tools/docs/*` only, no Rust) run in parallel. WP-01 is the only WP that may:
  - move files;
  - edit root `Cargo.toml`/`Cargo.lock`, any crate `Cargo.toml`, `crates/*/src/lib.rs`, `crates/engine/src/{api,cli,face,watch,exec,paths,testing}.rs`, the `crates/engine/src/cli/*.rs` entry signatures, the facade lines of `crates/engine/src/link.rs`, `crates/viewer/src/{frame,source,spec,gpu,xr}.rs`, `crates/app/src/lib.rs` (until WP-33) or `.github/workflows/check.yml`;
  - add a `mod` line.

  After wave 1, a WP that needs a new item adds it **inside its own file** behind a signature WP-01 already froze. The pub items other WPs call are listed in §2 "Frozen signatures" and "Wave-owned pub APIs".
- **SCRATCH:** `$SCRATCH` is the agent's own scratch dir. `<agent scratch>` means `$SCRATCH`. Every done-check starts with `export CARGO_TARGET_DIR=$SCRATCH/target`, a single `export`, so every cargo call in the chain uses it. A done-check written as `CARGO_TARGET_DIR=… cmd1 && cmd2` is read as that export.
- **Issue snapshot step:** this is a read-only GitHub read, not a write. The WP that needs it runs it; WP-00 does not.
  - Command: `mkdir -p $SCRATCH/issues; for n in 67 76 116 117 118; do gh issue view $n -R iannitram/flat-to-3d --json body -q .body > $SCRATCH/issues/$n.md; done`.
  - Where a WP says "WP-00 pre-step" for `$SCRATCH/issues/N.md`, it means this step, run by that WP itself. The WP keeps its sha256 check and its `gh` fallback.
- **Network:** none unless a WP declares it. No WP runs a network pre-step for another WP. Declared uses:
  - WP-01: one `cargo fetch` (crates.io source);
  - WP-21: its own single pnpm registry step (lockfile + store), then everything offline;
  - WP-02: release metadata reads for the pins table. It downloads no archives; unknown shas stay `TBD-at-pin`;
  - any WP: the issue snapshot step and other read-only `gh` reads.
- **Paths after WP-01:**
  - `inject/` and `app/` stop existing. Every path in WP-02..WP-34 and §3 is `crates/engine/{src,tests}/…`, `crates/app/{src,tests}/…`, `crates/viewer/…` or `crates/uevr-inject/…`.
  - "Existing test helpers" means `crates/engine/src/testing.rs` (`flat3d_inject::testing::FakeRunner`, §2), not `inject/tests`.
  - `tools/docs/check.py` (WP-25a) fails when an `inject/` or `app/src` path token appears in `docs/ROADMAP.md` or `docs/code-map.md`. The one exemption is the WP-01 git-mv block between `<!-- mv-map -->` markers.
- **Frozen signatures win:** a WP row whose call differs from a §2 "Frozen signatures" or "Wave-owned pub APIs" entry is wrong. The worker uses the §2 form and does not add a second one.
- **Cargo.lock is frozen after WP-01.**
  - WP-01 declares every dependency the pass needs, including the viewer's optional deps and uevr-inject's cfg(windows) deps.
  - It runs one online `cargo fetch`, because openxr and ashpd 0.13.13 are not cached. That is a crates.io source fetch, not a release binary.
  - CI and `tools/check.sh` build with `--locked`, and check.sh fails on `git diff --exit-code Cargo.lock`.
- **Toolchain:** every workflow pins `dtolnay/rust-toolchain@1.95.0` (host rustc/clippy 1.95.0). Nothing adds a `rust-toolchain.toml`. The host has only `stable` (= 1.95.0), so a toml naming 1.95.0 would make rustup download a toolchain.
- **Locked tests (R28):**
  - The locked set is the 49 `inject/tests/*.rs` files whose line 1 starts `//! Locked`, plus the test bodies in issues #67, #76, #116, #117 and #118.
  - The other 6 moved files carry no header and are ordinary tests: `event_ring_wrap`, `launch_options_quotes`, `runs_extra_lines`, `tether_runtime`, `tether_tunnel_review` and `wrapper_run`.
  - Locked bodies are committed verbatim, test first, then the code. The only sanctioned edits to locked tests:
    - WP-00 rustfmt (owner answer 1);
    - WP-01's 7 path edits (line numbers as at 2812f64; if WP-00's rustfmt shifted a line, find it by content). Together they are 14 `-`/`+` diff lines and leave all three files rustfmt-clean:

      | File:line | Edit |
      |---|---|
      | `api_response.rs:67`, `link_stop.rs:110` | `../docs/code-map.md` → `../../docs/code-map.md` |
      | `recipe_load.rs:138`, `recipe_load.rs:146` | `..` → `../..` |
      | `api_response.rs:69` | `inject/src/api.rs` → `crates/engine/src/api.rs` |
      | `link_stop.rs:112` | `inject/src/link.rs` → `crates/engine/src/link.rs` |
      | `recipe_load.rs:150` | `inject/src/recipes.rs` → `crates/engine/src/recipes.rs` |

      `wrapper_run.rs:206-207` is not edited: it is unheadered, and its `target/wrapper-tests` already sits under `.gitignore` `target/`.
    - One format-only `rustfmt --edition 2021 <file>` commit right after a WP's verbatim issue-body commit (owner answer 1). It touches only that WP's own locked file: WP-03 #76, WP-04 #116, WP-05 #118, WP-23 #117.

  Locked files never get `sys.path` or `use` edits beyond these.
- **Build and test:**
  - Use `CARGO_TARGET_DIR=$SCRATCH/target` and `tools/check.sh`, which exists from wave 1.
  - The viewer's `capture` feature builds only where libpipewire/libspa/libclang dev headers exist (CI apt, or craft-build). On the host, check only `-p flat3d-viewer` default + `--features xr`.
  - uevr-inject on the host: `cargo check -p flat3d-uevr-inject --target x86_64-pc-windows-msvc`, with the target installed via rustup. Linking happens only in `tools/build-win.sh`.
  - shellcheck is not on the host. Scripts run it when it is present (CI, or craft-build) and fall back to `bash -n` otherwise. CI's shellcheck step is the gate.
- **Never in the pass:**
  - downloading release archives (fetch and pin tests use `file://` fixtures);
  - `pin.sh` real runs;
  - GitHub writes (section 6 is for the owner/PM);
  - editing `docs/notes/DECISIONS.md`.
- **Device work** is written as `sessions/*.toml` and run by the owner with `tools/session/session.py run` (section 5).
- **Frozen after wave 1 (only the named WP may edit):**
  - nobody:
    - `crates/engine/src/{api,cli,lib,face,watch,paths,testing}.rs` (including the `watch.rs`/`face.rs` facades);
    - the `cli/*.rs` `run` signatures and the extra cli pub fns in §2 (the named WPs fill in the bodies only);
    - `crates/viewer/src/{frame,source,spec,gpu,xr}.rs`;
    - `Cargo.*` and `check.yml`.
  - `crates/engine/src/exec.rs`: WP-10 may change only the body of `ProcessRunner::run` and add private helpers. The traits and pub names stay as WP-01 wrote them.
  - `crates/engine/src/link.rs`: WP-12 owns the bodies. The WP-01 facade lines (`pub mod wivrn;` and the re-exports) stay.
  - `crates/app/src/lib.rs`: WP-33.
  - `docs/code-map.md`: WP-25b (w2), then WP-34 (w5).
  - `docs/ROADMAP.md`: WP-25b, WP-34.
  - `docs/spiderman2/address-map.md`: WP-25b (w2), then WP-30 (w3, the generated block only).
  - `tools/ghidra/headless.sh`: WP-02 (w1, quoting only), then WP-25b (w2).
  - `tools/check.sh`: WP-02 only.
  - `packaging/pins`: WP-02 writes it in wave 1. After that nobody in the pass edits it; the owner fills the TBDs via `pin.sh` (Q0).

## 1. Audit verdict (v1 roadmap A1-A24 + critique, with fixes now owned)

| # | Finding | Verdict | Owner |
|---|---|---|---|
| A1 | `cli_link::parse_link` (cli_link.rs:50) is never called | true | WP-01 pre-wires `parse_any`/dispatch + frozen `cli::<m>::run`; WP-08 bodies |
| A2/A3/A4 | stale doc comments main.rs:1, lib.rs:5-6, injectors.rs:1 | true | WP-01 |
| A5 | wrapper.rs:131 reads `{id}.recipe`, but recipes.rs:218-221 requires `<appid>-<slug>.recipe` (bug: every real recipe plays flat) | **bug** | WP-01 `catalog::find_recipe` + `run_with` |
| A6 | snapshot `tunnel = tether!=Off`, `health/stats: None` (watch.rs:761-781) | true | WP-01 pre-wires `HealthLoop::tick`; WP-11 body |
| A7 | `uevr::apply` env is empty (uevr.rs:74-102); `prepare` passes `&[]` | true | WP-14, WP-27, WP-31 |
| A8 | WiVRn start lacks `--early-active-runtime --no-publish-service`; Flatpak dir hard-coded (wrapper.rs:227-229) | true | WP-12 (`link/wivrn.rs`), WP-31 |
| A9/A10 | code-map lists `cli_view.rs` and `rows.rs`, but neither exists | true | WP-01 rows → `planned`; WP-03, WP-28 |
| A11 | code-map:21 mermaid has 2 statements per line | true | WP-25b + check.py (WP-25a) |
| A12 | `watch::read_settings` (watch.rs:826) is the only reader; there is no writer | true | WP-01 `switches::{read_settings,write_settings}` |
| A13 | locked `decky/tests/test_run_engine.py` (#67) is missing | true | WP-21 commits it verbatim |
| A14 | viewer "Windows binary in Proton" (code-map:85, SPEC:77) | superseded: Linux PipeWire→OpenXR quads | WP-17/18/19; SPEC via OD3 |
| A15 | bare `flat3d-run` fails (Game Mode PATH) | true | WP-20 `add_wrapper_at`; legacy stays (D4) |
| A16 | ROADMAP:204 and `tools/ghidra/headless.sh:6` point at `docs/sm2-address-map.md` | true | WP-25b |
| A17 | VT has no command written down | true | WP-30 |
| A18 | test counts in docs | drop | WP-25b |
| A19 | no fmt/clippy in CI; 48 files fail fmt; clippy fails at `switches_settings.rs:19` | true | WP-00 |
| A20 | stop_3d errors on the second Stop; no `allow-debugging`; undo never saved | true | WP-12, WP-09, WP-27 |
| A21 | #127 body drifted | true | section 6 |
| A22 | untracked `app/Cargo.lock`, `inject/Cargo.lock`; iced without `smol` | true | WP-01 |
| A23 | `dispatch` wires only Undo (cli.rs:141-149); unit test cli.rs:198 asserts `list` unbuilt (not locked) | true | WP-01 replaces the unit test |
| A24 | locked `cli_json.rs` runs `start-3d/stop-3d --json` for real | rule: no fresh snapshot ⇒ `link-not-ready`, spawn nothing; check.sh exports a temp `XDG_RUNTIME_DIR` | WP-08, WP-02 |
| K1 | v1 root lists a missing `viewer` member | fixed | WP-01 creates the `crates/viewer` stub |
| K2 | v1 merged two locked runner traits | fixed | §2 exec split (two distinct traits) |
| K3 | `module_inception` (`service/service.rs`); mod.rs/x.rs mix (E0761) | fixed | facade-file form everywhere (`x.rs` + `x/`), no `mod.rs`, no `x/x.rs` |
| K4 | v1 `wrapper::run(paths)` is wrong | fixed | real locked sig wrapper.rs:119 kept; `run_with` added |
| K5 | `run_with` loses NotFound→flat (wrapper.rs:129-131) | fixed | `switches::read_settings` NotFound→Default |
| K6 | route env `Vec<(String,String)>` can't carry unset/sidecar | fixed | `RouteEnv{env,unset,sidecar}`; `ExecSpec{argv,env,unset,sidecar}` |
| K7 | self dev-dep `features=["testing"]` duplicates types | fixed | `#[doc(hidden)] pub mod testing` always compiled |
| K8 | heal reconnect has no call site | fixed | WP-01 pre-wires `HealthTick.heal`; WP-09 executes the heal |
| K9 | `unittest discover -s tools` doesn't recurse without `__init__.py` | fixed | per-folder loop (§2 CI); decky run from `decky/` |
| K10 | shellcheck missing on host; old scripts never linted | fixed | CI apt; WP-02 lints the 2 scripts; check.sh skips with notice |
| K11 | check.py "commands == USAGE" contradicts locked USAGE | fixed | one-way USAGE ⊆ table |
| K12 | no node/pnpm in CI | fixed | setup-node 22.23.4 + pnpm 10.34.9, per-step `hashFiles` guards |
| K13 | doc contradictions C1-C19, C30 and private names (AGENTS.md:12) unowned | fixed | WP-25b (C1/C3/C4 stay open: DECISIONS untouched) |
| K14 | tether/mod.rs and the production callers of new link/tether fns unowned | fixed | WP-10 owns `tether.rs`; WP-09 switches the callers |
| K15 | watch.rs:915,922 tests fake "Tether command failed"/"Could not run tether command" | keep the prefixes | WP-10 appends `: exit N; <last 4 stderr lines>` |
| K16 | tun0 read from host-side `/sys` is wrong (tun0 is on the headset) | fixed | `adb shell ip addr show tun0` + `cat /proc/net/dev` |
| K17 | `flatpak info --show-location` prints the deploy dir | fixed | strip `/<arch>/<branch>/<commit>`; `--user` then `--system` |
| K18 | locked link_stop.rs: "Access denied" stays an error | honour | WP-12 |
| K19 | pins written by two WPs | fixed | `packaging/pins` owned by WP-02 only |
| K20 | windows-gnu can't link here | fixed | msvc + xwin + rust-lld in `tools/build-win.sh` |
| K21 | no WP built/installed the injector exe | fixed | WP-15 `--install` |
| K22 | geo11/wivrn-apk sha unknown | fixed | TOFU `pin.sh` (OD8); `fetch.rs` refuses `TBD-at-pin`; wivrn-apk sha recorded by the pins research |
| K23 | viewer xr/capture can't compile in CI without headers | fixed | apt in CI; split WP-17 (pure) / WP-18 (capture) / WP-19 (xr); shared `frame.rs` frozen by WP-01 |
| K24 | locked install_plan.rs:14-16,48 fixes `unit_text` | honour | `unit_text_with` new |
| K25 | SM2 appid is 2651280; "check with list" is circular | fixed | WP-26 seeds use 2651280 |
| K26 | `InjectResult.launched` undefined | fixed | `false` from CLI `inject` (it prepares, never launches) |
| K27 | #117 needs only `rows::GameRow` | fixed | games page in wave 2 (WP-23) |
| K28 | rows.rs fixes the `flat_reason` texts | honour | not unified (Not-in-pass) |
| K29 | `cli_undo.rs` gap; #37 HEROVR column | fixed | WP-27; WP-30 |
| K30 | Grace→Ready, `launch_options_before` | owner | OD4, OD5 |
| K31 | wave-parallel WPs disagreed on wivrn/HealthLoop/ensure_applied/EnginePaths/cli/viewer signatures | fixed | §2 "Frozen signatures" (WP-01) |
| K32 | bones, WP-01 and the implementing WPs disagreed on the wivrn path and types, HealthLoop, ensure_applied, viewer frame, from_env, Ctx, install and FakeRunner | fixed | §2 reconciled: the implementing WP's form for wivrn (WP-12), HealthLoop (WP-11), ensure_applied (WP-27), install (WP-20), wrapper (WP-31) and frame (WP-17/18/19); bones' form for from_env and Ctx |
