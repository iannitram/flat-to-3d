<!-- split from roadmap-v2.md lines 1-137 at c0e6874 -->
# flat-to-3d roadmap v2 (2026-11-09)

Sections 0-7 and the WP files are the contract. The reviewer fixes, the Craft review and the owner's rulings of 2026-11-09 are folded into them.

- **Unattended run:** start at §0a below. It names what to read, the order of work, the branch, the progress file and when to stop.
- **What a WP pass reads:** this file (§0 and §1), the parts of `01-new-bones.md` (§2) that the WP names, and the WP's own `WP-NN.md`. §0 and §2 win over a WP file. `03-wp-index.md` is the one-line view of every WP; it matches the WP files.
- **No findings step.** The fold already applied the findings, so a WP pass does not read or apply them again.
- **Fold log:** `fold-log/` records each fold decision, one file per WP (`WP-NN.md`) and per section (`S0.md`, `S2.md`, `S3.md`). Read it to see why a line says what it says. It is history, not contract.
- **Owner questions:** the open owner questions, each with a recommended answer, are in `questions.md`, grouped by WP. Until the owner answers, the pass runs on the recommended answer or the §7 default. A WP pass also reads the `questions.md` entries under §0, §2 and its own WP. An entry marked "Folded" is already in the files and needs no action.

## 0. How to run the pass

- **Base:** the flat-to-3d `main` commit this plan merges into (the tip of `main` when the pass starts). Only docs commits lie between it and `2812f64`, the commit the plan's line citations were taken at, so every "as at 2812f64" citation still holds. One branch per WP, `v2/wp-NN-<slug>`. Merge order: wave 0 → wave 1 → … → wave 5. Inside a wave, WPs own disjoint files and merge in any order. A WP starts only after every WP in its `depends` is merged. A WP never depends on a WP in its own wave. The one exception to these three rules is WP-01, which is split into four parts: WP-01a → WP-01b → WP-01c → WP-01d run and merge in that order inside wave 1, each depends on the one before, and they share files (each part's file says which lines are its own). "WP-01" in a `depends` list, or in a phrase such as "after WP-01", means all four parts merged; `WP-01.md` itself is only a pointer. A WP in wave 2 or later branches from `main` only after all of wave 1 has merged, because its done-check needs `tools/check.sh` and `tools/locked-test.sh` (WP-02). `tools/pm/issues_report.py --roadmap` (WP-02) checks these rules and lists the WPs that can start now.
- **Git references in done-checks.** No done-check names `main` literally, because in §0a `main` does not move. Every done-check uses these two references, which mean the same thing in both run modes:
  - **The integration line**, written `"${FLAT3D_BASE:-origin/main}"`: the branch this WP will merge into.
    - In §0 the variable is unset, so the line is `origin/main`; it must hold the commit the WP branch was cut from. A runner who cuts branches from a local `main` that is ahead of `origin/main` exports `FLAT3D_BASE=main`.
    - In §0a the coordinator exports `FLAT3D_BASE=v2/integration` for every builder.
    - "What did this branch change" is a diff against `"$(git merge-base "${FLAT3D_BASE:-origin/main}" HEAD)"`, the commit the branch was cut from. A done-check is therefore run on the WP branch before that branch is merged; after the merge the merge-base is `HEAD` itself.
  - **`v2-wp00`**, a local lightweight tag on the integration-line commit where WP-00 landed: the tip of `main` (§0) or of `v2/integration` (§0a) straight after WP-00's merge. It is the fixed picture of `inject/tests/` after WP-00's rustfmt and before WP-01a's move.
    - Who places it: the runner, straight after WP-00's merge is settled and before WP-01a is handed out. In §0a that is the coordinator: `git tag v2-wp00 v2/integration`. In §0 it is whoever merges WP-00: `git tag v2-wp00 <the main commit where WP-00 landed>`.
    - Fallback: if the tag is absent when WP-01a's done-check runs, that done-check places it on `git merge-base "${FLAT3D_BASE:-origin/main}" HEAD`. That commit holds the same `inject/` tree, because only WP-00 and WP-01a may change `inject/` (WP-02 and WP-25a, which may land between them, own no file there). WP-01b, WP-01c and WP-01d have no fallback: without the tag they stop as `blocked`.
    - It is local: never pushed (the builder's `git push -u origin v2/wp-NN-<slug>` does not send it; never use `--tags`), and never moved once a WP-01 part has passed against it. All worktrees of one clone share it. A builder in a separate clone does not have it; the runner places it there on the same commit.
    - The WP-01 parts check the tag before they use it: it must be an ancestor of `HEAD`, hold WP-00 (`inject/Cargo.toml` has the `field_reassign_with_default` lint line) and still hold `inject/tests/`.
    - Use: "the 55 moved test files are what WP-00 left, apart from WP-01a's 7 edits" is checked in all four WP-01 parts by reading the old path from the tag and the new path from the working tree: `git show "v2-wp00:inject/tests/<f>" | diff -u - "crates/engine/tests/<f>"`.
- **Wave 0 (WP-00, #183) lands alone.** It is format-only plus a clippy allow. Every later diff then stays readable.
- **Wave 1:** WP-01 (Rust reshape + bones; four parts WP-01a to WP-01d, built one after the other, one agent each), WP-02 (repo tooling, no Rust) and WP-25a (`tools/docs/*` only, no Rust) run in parallel: the WP-01 chain runs beside the other two, which need none of its parts. WP-01 (its four parts; `Cargo.*` and `check.yml` only in WP-01a) is the only WP that may:
  - move files;
  - edit root `Cargo.toml`/`Cargo.lock`, any crate `Cargo.toml`, `crates/*/src/lib.rs`, `crates/engine/src/{api,cli,face,watch,exec,paths,testing}.rs`, the `crates/engine/src/cli/*.rs` entry signatures, the facade lines of `crates/engine/src/link.rs`, `crates/viewer/src/{frame,source,spec,gpu,xr}.rs`, `crates/app/src/lib.rs` (until WP-33) or `.github/workflows/check.yml`;
  - add a `mod` line.

  After wave 1, a WP that needs a new item adds it **inside its own file** behind a signature WP-01 already froze. The pub items other WPs call are listed in §2 "Frozen signatures" and "Wave-owned pub APIs".
- **SCRATCH:** `$SCRATCH` is the agent's own scratch dir. `<agent scratch>` means `$SCRATCH`. From wave 2 on, every done-check has the form `export CARGO_TARGET_DIR=$SCRATCH/target && <its own tests> && tools/check.sh`. The single `export` means every cargo call in the chain uses it, and the closing `tools/check.sh` is the gate. Waves 0 and 1 run before check.sh has merged, so their done-checks follow their own WP text: WP-00 and the four WP-01 parts run their steps directly, WP-02 runs the check.sh it adds, and WP-25a runs check.sh only if it exists. A done-check that runs inside craft-build ends the same way there. A done-check written as `CARGO_TARGET_DIR=… cmd1 && cmd2` is read as that export.
- **Issue snapshot step:** this is a read-only GitHub read, not a write. The WP that needs it runs it; WP-00 does not.
  - Command, one issue at a time, run from inside the clone (so `gh` finds the repo itself and no `-R` is needed): `mkdir -p "$SCRATCH/issues"; [ -s "$SCRATCH/issues/N.md" ] || gh issue view N --json body -q .body > "$SCRATCH/issues/N.md"`. `N` is 67, 76, 116, 117 or 118. Every WP and `tools/locked-test.sh` use this one form.
  - Where a WP says "WP-00 pre-step" for `$SCRATCH/issues/N.md`, it means this step, run by that WP itself. The WP keeps its sha256 check and its `gh` fallback.
- **Network:** none unless a WP declares it. No WP runs a network pre-step for another WP. Declared uses:
  - WP-00: none when the untracked per-crate `Cargo.lock` files are present; on a clean checkout its first build reads the crates.io index (see "Cargo.lock before WP-01" below);
  - WP-01a: one `cargo fetch` (crates.io source); WP-01b, WP-01c and WP-01d: none;
  - WP-21: its own single pnpm registry step (lockfile + store), then everything offline;
  - WP-02: release metadata reads for the pins table. It downloads no archives; unknown shas stay `TBD-at-pin`;
  - WP-30: one read-only `gh api` read of HEROVR's `BUILD-SM2.md` at the pinned `herovr_rev` (the command is in WP-30.md);
  - the issue snapshot step, a read-only `gh` read, by the WPs that need it: WP-03 (#76), WP-04 (#116), WP-05 (#118), WP-21 (#67), WP-23 (#117) and WP-24 (one read per issue, for its `done_when` texts);
  - any WP: other read-only `gh` reads that its own file names.
- **Line numbers are hints.** A `file.rs:N` citation in a WP is the line as at the base commit, before WP-00's rustfmt and WP-01's moves. Find the code by its content, not by the number. A citation to a file outside the repo (a research note, a registry source line) is provenance only: a builder does not need to read it and does not stop when it is missing.
- **Paths after WP-01:**
  - `inject/` and `app/` stop existing. Every path in WP-02..WP-34 and §3 is `crates/engine/{src,tests}/…`, `crates/app/{src,tests}/…`, `crates/viewer/…` or `crates/uevr-inject/…`.
  - "Existing test helpers" means `crates/engine/src/testing.rs` (`flat3d_inject::testing::FakeRunner`, §2), not `inject/tests`.
  - `tools/docs/check.py` (WP-25a) fails when an `inject/` or `app/src` path token appears in `docs/ROADMAP.md` or `docs/code-map.md`. The one exemption is the WP-01 git-mv block between `<!-- mv-map -->` markers.
- **Frozen signatures win:** a WP row whose call differs from a §2 "Frozen signatures" or "Wave-owned pub APIs" entry is wrong. The worker uses the §2 form and does not add a second one.
- **Cargo.lock before WP-01.** No `Cargo.lock` is tracked at the base. `inject/Cargo.lock` and `app/Cargo.lock` are untracked, so a clean checkout or a fresh worktree has neither.
  - WP-00 builds without `--locked`. If the files are missing, cargo writes them on the first build (a crates.io index read). WP-00 leaves them untracked and never commits them.
  - WP-01a seeds the root `Cargo.lock` from `app/Cargo.lock` when it is there; otherwise its one `cargo fetch` writes a fresh root lock. Then it deletes the per-crate files if they exist.
  - A missing per-crate `Cargo.lock` is never a reason to stop.
- **Cargo.lock is frozen after WP-01.**
  - WP-01a (the first of the four WP-01 parts) writes the lock, and the lock is frozen from then on: WP-01b to WP-01d do not touch `Cargo.*`. It declares every dependency the pass needs, including the viewer's optional deps and uevr-inject's cfg(windows) deps.
  - It runs one online `cargo fetch`, because openxr and ashpd 0.13.13 are not cached. That is a crates.io source fetch, not a release binary.
  - CI and `tools/check.sh` build with `--locked`, and check.sh fails on `git diff --exit-code Cargo.lock`.
- **Toolchain:** every workflow pins `dtolnay/rust-toolchain@1.95.0` (host rustc/clippy 1.95.0). Nothing adds a `rust-toolchain.toml`. The host has only `stable` (= 1.95.0), so a toml naming 1.95.0 would make rustup download a toolchain.
- **Locked tests (R28):**
  - The locked set is the 49 `inject/tests/*.rs` files whose line 1 starts `//! Locked`, plus the test bodies in issues #67, #76, #116, #117 and #118.
  - The other 6 moved files carry no header and are ordinary tests: `event_ring_wrap`, `launch_options_quotes`, `runs_extra_lines`, `tether_runtime`, `tether_tunnel_review` and `wrapper_run`.
  - Locked bodies are committed verbatim, test first, then the code. The only sanctioned edits to locked tests:
    - WP-00 rustfmt (owner answer 1);
    - WP-01a's 7 path edits (line numbers as at 2812f64; if WP-00's rustfmt shifted a line, find it by content). Together they are 14 `-`/`+` diff lines and leave all three files rustfmt-clean:

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
  - Use `CARGO_TARGET_DIR=$SCRATCH/target` and `tools/check.sh`, which exists from wave 1 (WP-02).
  - `tools/check.sh` is the only list of CI steps. `check.yml` does setup, then runs `tools/check.sh --ci --only rust|py|decky`. `--ci` turns every skip (a tool or target missing on this machine) into a failure. A step whose files have not landed yet prints `n/a`, which `--ci` leaves alone. Until check.sh exists, check.yml keeps its inline steps behind `hashFiles('tools/check.sh') == ''`, because WP-01 and WP-02 share wave 1.
  - check.sh names each step. A failing step prints `FAIL <step>: fix: <hint>`, and the run ends with a pass/skip/n/a/fail summary. It also takes `--only`, `--list` and `--tests <cargo args>`.
  - The viewer's `capture` feature builds only where libpipewire/libspa/libclang dev headers exist (CI apt, or craft-build). On the host, check only `-p flat3d-viewer` default + `--features xr`.
  - uevr-inject on the host: `cargo check -p flat3d-uevr-inject --target x86_64-pc-windows-msvc`, with the target installed via rustup. Linking happens only in `tools/build-win.sh`.
  - shellcheck is not on the host. Scripts run it when it is present (CI, or craft-build) and fall back to `bash -n` otherwise. On the host check.sh skips it with a notice. Under `--ci` that skip fails, so CI is the gate.
  - **Host or build box (stated once, for every WP).** check.sh's `clippy` and `test` steps cover the whole workspace, so they build the iced app (`flat3d-app`). The host has no `pkg-config` and may lack a system library that build needs. check.sh has no flag that leaves the app out, and no WP adds one. So: run the done-check on the host first. If a step fails there only because a system library or `pkg-config` is missing, rerun the same chain, unchanged, inside the build box: `distrobox enter craft-build -- sh -c 'export CARGO_TARGET_DIR=$SCRATCH/target && <the same chain>'` (export `SCRATCH` first). Say which form ran under `## Verified`. If it fails in the box for the same kind of reason, or the box is not there, the WP is `blocked`: report it, and the coordinator records it in `progress.md`. A WP's own copy of this rule says the same thing; where one differs, this line wins.
- **PR body (ruling D1):**
  - Every PR body has a `## Verified` section (each done-check command and its exit status) and a `## Not verified` section (each host skip it hit, with its device Q row from §5).
  - It never claims a run that was not done. A WP that only fakes prove also keeps its "Unverified on the device (Qn):" line.
  - A merge takes only the head commit the review passed. The two-commit landings (WP-00) stay as two commits.
- **Issue closing (ruling D5):** the `Refs #N (code half)` rule for a WP with a device Q row is stated once, in §2 "PR and issue rules". Closing the issue when its device session passes is a GitHub write, so the owner or PM does it.
- **Never in the pass:**
  - downloading release archives (fetch and pin tests use `file://` fixtures);
  - `pin.sh` real runs;
  - GitHub writes (section 6 is for the owner/PM). One exception **[default, owner to confirm]**: a builder may push its own `v2/wp-NN-*` branch and open one PR for it, as §0a "Builder rules" says. That is why every WP has a PR body. Nothing else: no merge, no issue edit or close, no label, no comment;
  - editing `docs/notes/DECISIONS.md`.
- **Device work** is written as `sessions/*.toml` and run by the owner with `tools/session/session.py run` (section 5).
- **Frozen after wave 1 (only the named WP may edit):**
  - nobody:
    - `crates/engine/src/{api,cli,lib,face,watch,paths,testing}.rs` (including the `watch.rs`/`face.rs` facades);
    - the `cli/*.rs` `run` signatures and the extra cli pub fns in §2 (the named WPs fill in the bodies only);
    - `crates/viewer/src/{frame,source,spec,gpu,xr}.rs`;
    - `Cargo.*` and `check.yml`.
  - `crates/engine/src/exec.rs`: WP-10 may change only the body of `ProcessRunner::run` and add private helpers. The traits and pub names stay as WP-01 wrote them. `crates/engine/tests/architecture.rs` (WP-01b) allows `Command::new` only in exec.rs, plus a tagged allowlist of the sites still outside it. Today's sites are link.rs, watch.rs, tether/adb.rs and bin/flat3d-run.rs. WP-01b moves the link.rs and tether/adb.rs runners into exec.rs, so it lands the list with the watch sites (tagged `follow-up`: no WP in this pass moves them), bin/flat3d-run.rs (WP-31) and cli/link.rs (WP-08). The last two exec the game in place, so they are permanent entries.
  - `crates/engine/src/link.rs`: WP-12 owns the bodies. The WP-01 facade lines (`pub mod wivrn;` and the re-exports) stay.
  - `crates/app/src/lib.rs`: WP-33.
  - `docs/code-map.md`: WP-25b (w2), then WP-34 (w5).
  - `docs/ROADMAP.md`: WP-25b, WP-34.
  - `docs/spiderman2/address-map.md`: WP-25b (w2), then WP-30 (w3, the generated block only).
  - `tools/ghidra/headless.sh`: WP-02 (w1, quoting only), then WP-25b (w2).
  - `tools/check.sh`: WP-02 only. A step for a later WP's file is written by WP-02 in wave 1 and runs only when that file exists (for example `fill_map.py --check`, WP-30).
  - `packaging/pins`: WP-02 writes it in wave 1. After that nobody in the pass edits it; the owner fills the TBDs via `pin.sh` (Q0).
- **Enforcement:** `issues_report.py --roadmap` exits non-zero, naming the WP and the fix, when a WP lists a frozen path it is not named for above. On a `v2/wp-NN-*` branch, check.sh's scope step fails any changed path that is not in WP-NN's `files:` list or on a "Frozen after wave 1" line above that names WP-NN. Those lines are the body-only allowlist; WP-01d "Ownership after this WP" says the same.

## 0a. Unattended run (coordinator loop)

This section is the entry point for a run with nobody watching. It adds to §0 and changes two §0 rules for the unattended run only; both are marked "owner to confirm" below. Everything else in §0 holds.

**Who reads what**
- Coordinator: this file (§0 and §0a), `03-wp-index.md`, `progress.md`. It does not read WP files or §2.
- Builder (one WP per agent), in this order:
  1. this file, §0 and §0a (skip §1);
  2. its own `WP-NN.md` (a WP-01 builder reads its own part, `WP-01a.md` to `WP-01d.md`, not the pointer `WP-01.md` or the other parts);
  3. from `01-new-bones.md` (§2): every part its WP names, plus the default parts every builder reads: under "Frozen signatures" the bullet for each file in its Files list, then "Wave-owned pub APIs", "ERROR_CODES", "Exit contract" and "PR and issue rules". Each part of §2 has a `###` heading with the part's name; list them with `grep -n '^### ' 01-new-bones.md` and read from the heading to the next one. A WP cites a part by its heading name. A quoted name that is not a heading (for example "Only `exec.rs` spawns") is a bold label inside a part; find it with `grep -n`. A line number into `01-new-bones.md` is a hint only; the heading wins;
  4. the `questions.md` entries under §0, §2 and its WP (for a WP-01 part, the `## WP-01` heading).
  It does not read `fold-log/`, `02-queues-and-decisions.md` (unless its WP names a row), other WP files, or anything outside this folder.

**Base and branches**
- `v2/integration` is a local branch the coordinator owns. It starts at the §0 base commit plus this plan folder. If `docs/plans/roadmap-v2/03-wp-index.md` is not on it, stop the run and record that in `progress.md`.
- Each WP gets its own worktree and one branch `v2/wp-NN-<slug>` (slug: the WP title in lower-case words joined by `-`, at most four words), cut from the tip of `v2/integration` at the moment the WP is handed out. Each builder gets its own `$SCRATCH`.
- Nothing merges to `main` overnight. Wave N+1 builds on wave N like this: after a WP's done-check passes, the coordinator runs `git merge --no-ff v2/wp-NN-<slug>` on `v2/integration` (the default subject keeps the branch name, which `issues_report.py --roadmap` needs). Then, if `tools/check.sh` exists, it runs `tools/check.sh` on `v2/integration`. If that fails, it puts `v2/integration` back to the commit before this merge and marks the WP `blocked`. `v2/integration` is never pushed.
- Straight after WP-00's merge into `v2/integration` is settled, and before WP-01a is handed out, the coordinator runs `git tag v2-wp00 v2/integration` (§0 "Git references in done-checks"). On a resume with WP-00 `done` and no tag, it places the tag on WP-00's merge commit: `git tag v2-wp00 "$(git log --merges --first-parent --format=%H --grep='v2/wp-00-' v2/integration | tail -n1)"`. The tag is never pushed.
- So for this run, §0 "merged" (in "A WP starts only after every WP in its `depends` is merged" and "branches from `main` only after all of wave 1 has merged") means "merged into `v2/integration`", and "branches from `main`" means "branches from `v2/integration`". **Owner to confirm.**
- Integration line: the coordinator exports `FLAT3D_BASE=v2/integration` for every builder, from WP-00 on. Every done-check reads it as `"${FLAT3D_BASE:-origin/main}"` (§0 "Git references in done-checks"), so no done-check looks at `main`, which lacks every WP merged this run.
- Scope step: `tools/check.sh` passes `--base "${FLAT3D_BASE:-origin/main}"` to `issues_report.py --scope` (WP-02 writes it so). With `FLAT3D_BASE=v2/integration` exported, the scope step sees only the WP's own changes. The coordinator calls `issues_report.py --roadmap --base v2/integration`.

**Picking the next WP**
- Wave order is the `Waves:` line of `03-wp-index.md`. Wave 0 alone, then wave 1, and so on.
- A WP can start when its `progress.md` status is `todo`, every WP in its index `depends` is `done`, and (wave 2 or later) WP-00, WP-01a, WP-01b, WP-01c, WP-01d, WP-02 and WP-25a are all `done`. A `depends` entry `WP-01` is met only when all four WP-01 parts are `done`.
- WPs of one wave may run in parallel; the coordinator merges them into `v2/integration` one at a time. The four WP-01 parts are the exception: their `depends` chain makes them run one after the other (WP-01b is cut from `v2/integration` only after WP-01a is merged there, and so on), while WP-02 and WP-25a run beside them.
- Once WP-02 is `done`, `tools/pm/issues_report.py --roadmap --base v2/integration` prints the same "ready" list. If it disagrees with `progress.md`, `progress.md` wins and the difference goes in Notes.

**Builder rules**
- Change only the paths in the WP's Files list (and a "Frozen after wave 1" line that names the WP). Never edit a locked test, a frozen path, another WP's file or the done-check itself.
- Run the WP's done-check exactly as written. Done means every command in it exited 0.
- Owner questions: build what the WP file says. Where the WP is silent, use the **Rec** in `questions.md`, or the §7 default. A Rec that needs a GitHub write, an owner setting, a device, or an edit outside the WP's Files list is not done by the builder: list it under `## Not verified` as "Owner follow-up (question N)".
- Blocked: if the WP cannot be built as written (a missing input, two texts that disagree and §0/§2 do not settle it, a done-check that needs a device, a download, or a file no earlier wave made, or the done-check still failing after two fix attempts), stop. Do not guess and do not weaken a test. Commit nothing further, and report `blocked` with the file, the line and the reason.
- GitHub: the builder's only writes are `git push -u origin v2/wp-NN-<slug>` and one `gh pr create --base main` for that branch, with the §0 "PR body" sections and a first line `Stacked on v2/integration@<sha>; contains WP-…` listing the earlier WPs in the branch. No merge, no issue edit or close, no label, no comment, no push to any other branch, no force push. A blocked WP opens no PR. This is the one exception that §0 "Never in the pass: GitHub writes" names. **[default, owner to confirm]**; if the owner says no, builders stop after the local done-check and the PR column stays `-`.

**Progress file (resume point)**
- `docs/plans/roadmap-v2/progress.md`, in the coordinator's own checkout of the plan. Only the coordinator writes it. It is never committed on a `v2/wp-*` branch and never copied into a builder worktree as an untracked file (the scope step would fail).
- One row per WP: `| WP | Wave | Status | Branch | PR | Notes |`. Wave is the WP's wave number from `03-wp-index.md`; it is set once and never changes. Status is one of `todo`, `running`, `done`, `blocked`, `skipped`.
- The coordinator writes `running` before it hands a WP out, and `done` (with branch and PR) or `blocked` (with the reason) straight after the builder returns and the integration merge is settled.
- A WP whose `depends` holds a `blocked` or `skipped` WP becomes `skipped` with the note `waits on WP-NN`.
- Resume after a crash: read `progress.md`. A `running` row is treated as not started: if its branch exists and its done-check passes there, finish it (merge, PR, `done`); otherwise mark it `blocked` with the note `interrupted` and leave the branch as it is. Never delete or reset a `v2/wp-*` branch.

**When to stop**
- every row is `done`, `blocked` or `skipped`; or
- WP-00 or any wave 1 WP is `blocked` (everything after needs them); or
- `v2/integration` cannot be put back after a failed check; or
- any step would need a merge to `main`, a GitHub write other than the two above, a real download, a device, or an edit to a frozen path.

On stop, the coordinator writes one last `progress.md` line under the table: the time, the reason and the tip of `v2/integration`.

## 1. Audit verdict (v1 roadmap A1-A24 + critique, with fixes now owned)

IDs are stable: they are never renumbered or reused. "planned" means a WP owns the fix but it has not merged. When the fix merges, "planned" becomes `verified @<sha>`; WP-34 fills in the SHAs. Proof names the merge SHA and the test or check that shows the fix. It stays `—` until then.

| # | Finding | Verdict | Owner | Proof |
|---|---|---|---|---|
| A1 | `cli_link::parse_link` (cli_link.rs:50) is never called | true | WP-01 pre-wires `parse_any`/dispatch + frozen `cli::<m>::run`; WP-08 bodies | — |
| A2/A3/A4 | stale doc comments main.rs:1, lib.rs:5-6, injectors.rs:1 | true | WP-01 | — |
| A5 | wrapper.rs:131 reads `{id}.recipe`, but recipes.rs:218-221 requires `<appid>-<slug>.recipe` (bug: every real recipe plays flat) | **bug** | WP-01 `catalog::find_recipe` + `run_with` | — |
| A6 | snapshot `tunnel = tether!=Off`, `health/stats: None` (watch.rs:761-781) | true | WP-01 pre-wires `HealthLoop::tick`; WP-11 body | — |
| A7 | `uevr::apply` env is empty (uevr.rs:74-102); `prepare` passes `&[]` | true | WP-14, WP-27, WP-31 | — |
| A8 | WiVRn start lacks `--early-active-runtime --no-publish-service`; Flatpak dir hard-coded (wrapper.rs:227-229) | true | WP-12 (`link/wivrn.rs`), WP-31 | — |
| A9/A10 | code-map lists `cli_view.rs` and `rows.rs`, but neither exists | true | WP-01 rows → `planned`; WP-03, WP-28 | — |
| A11 | code-map:21 mermaid has 2 statements per line | true | WP-25b + check.py (WP-25a) | — |
| A12 | `watch::read_settings` (watch.rs:826) is the only reader; there is no writer | true | WP-01 `switches::{read_settings,write_settings}` | — |
| A13 | locked `decky/tests/test_run_engine.py` (#67) is missing | true | WP-21 commits it verbatim | — |
| A14 | viewer "Windows binary in Proton" (code-map:85, SPEC:77) | superseded: Linux PipeWire→OpenXR quads | WP-17/18/19; SPEC via OD3 | — |
| A15 | bare `flat3d-run` fails (Game Mode PATH) | true | WP-20 `add_wrapper_at`; legacy stays (D4) | — |
| A16 | ROADMAP:204 and `tools/ghidra/headless.sh:6` point at `docs/sm2-address-map.md` | true | WP-25b | — |
| A17 | VT has no command written down | true | WP-30 | — |
| A18 | test counts in docs | drop | WP-25b | — |
| A19 | no fmt/clippy in CI; 48 files fail fmt; clippy fails at `switches_settings.rs:19` | true | WP-00 | — |
| A20 | stop_3d errors on the second Stop; no `allow-debugging`; undo never saved | true | WP-12, WP-09, WP-27 | — |
| A21 | #127 body drifted | true | section 6 | — |
| A22 | untracked `app/Cargo.lock`, `inject/Cargo.lock`; iced without `smol` | true | WP-01 | — |
| A23 | `dispatch` wires only Undo (cli.rs:141-149); unit test cli.rs:198 asserts `list` unbuilt (not locked) | true | WP-01 replaces the unit test | — |
| A24 | locked `cli_json.rs` runs `start-3d/stop-3d --json` for real | rule: no fresh snapshot ⇒ `link-not-ready`, spawn nothing; check.sh exports a temp `XDG_RUNTIME_DIR` | WP-08, WP-02 | — |
| K1 | v1 root lists a missing `viewer` member | planned | WP-01 creates the `crates/viewer` stub | — |
| K2 | v1 merged two locked runner traits | planned | §2 exec split (two distinct traits) | — |
| K3 | `module_inception` (`service/service.rs`); mod.rs/x.rs mix (E0761) | planned | facade-file form everywhere (`x.rs` + `x/`), no `mod.rs`, no `x/x.rs` | — |
| K4 | v1 `wrapper::run(paths)` is wrong | planned | real locked sig wrapper.rs:119 kept; `run_with` added | — |
| K5 | `run_with` loses NotFound→flat (wrapper.rs:129-131) | planned | `switches::read_settings` NotFound→Default | — |
| K6 | route env `Vec<(String,String)>` can't carry unset/sidecar | planned | `RouteEnv{env,unset,sidecar}`; `ExecSpec{argv,env,unset,sidecar}` | — |
| K7 | self dev-dep `features=["testing"]` duplicates types | planned | `#[doc(hidden)] pub mod testing` always compiled | — |
| K8 | heal reconnect has no call site | planned | WP-01 pre-wires `HealthTick.heal`; WP-09 executes the heal | — |
| K9 | `unittest discover -s tools` doesn't recurse without `__init__.py` | planned | per-folder loop (§2 CI); decky run from `decky/` | — |
| K10 | shellcheck missing on host; old scripts never linted | planned | CI apt; WP-02 lints the 2 scripts; check.sh skips with notice | — |
| K11 | check.py "commands == USAGE" contradicts locked USAGE | planned | one-way USAGE ⊆ table | — |
| K12 | no node/pnpm in CI | planned | setup-node 22.23.4 + pnpm 10.34.9, per-step `hashFiles` guards | — |
| K13 | doc contradictions C1-C19, C30 and private names (AGENTS.md:12) unowned | planned | WP-25b (C1/C3/C4 stay open: DECISIONS untouched) | — |
| K14 | tether/mod.rs and the production callers of new link/tether fns unowned | planned | WP-10 owns `tether.rs`; WP-09 switches the callers | — |
| K15 | watch.rs:915,922 tests fake "Tether command failed"/"Could not run tether command" | keep the prefixes | WP-10 appends `: exit N; <last 4 stderr lines>` | — |
| K16 | tun0 read from host-side `/sys` is wrong (tun0 is on the headset) | planned | `adb shell ip addr show tun0` + `cat /proc/net/dev` | — |
| K17 | `flatpak info --show-location` prints the deploy dir | planned | strip `/<arch>/<branch>/<commit>`; `--user` then `--system` | — |
| K18 | locked link_stop.rs: "Access denied" stays an error | honour | WP-12 | — |
| K19 | pins written by two WPs | planned | `packaging/pins` owned by WP-02 only | — |
| K20 | windows-gnu can't link here | planned | msvc + xwin + rust-lld in `tools/build-win.sh` | — |
| K21 | no WP built/installed the injector exe | planned; device pending Q2 | WP-15 `--install` | — |
| K22 | geo11/wivrn-apk sha unknown | planned | TOFU `pin.sh` (OD8); `fetch.rs` refuses `TBD-at-pin`; wivrn-apk sha recorded by the pins research | — |
| K23 | viewer xr/capture can't compile in CI without headers | planned | apt in CI; split WP-17 (pure) / WP-18 (capture) / WP-19 (xr); shared `frame.rs` frozen by WP-01 | — |
| K24 | locked install_plan.rs:14-16,48 fixes `unit_text` | honour | `unit_text_with` new | — |
| K25 | SM2 appid is 2651280; "check with list" is circular | planned | WP-26 seeds use 2651280 | — |
| K26 | `InjectResult.launched` undefined | planned | `false` from CLI `inject` (it prepares, never launches) | — |
| K27 | #117 needs only `rows::GameRow` | planned | games page in wave 2 (WP-23) | — |
| K28 | rows.rs fixes the `flat_reason` texts | honour | not unified (Not-in-pass) | — |
| K29 | `cli_undo.rs` gap; #37 HEROVR column | planned | WP-27; WP-30 | — |
| K30 | Grace→Ready, `launch_options_before` | owner | OD4, OD5 | — |
| K31 | wave-parallel WPs disagreed on wivrn/HealthLoop/ensure_applied/EnginePaths/cli/viewer signatures | planned | §2 "Frozen signatures" (WP-01) | — |
| K32 | bones, WP-01 and the implementing WPs disagreed on the wivrn path and types, HealthLoop, ensure_applied, viewer frame, from_env, Ctx, install and FakeRunner | planned | §2 reconciled: the implementing WP's form for wivrn (WP-12), HealthLoop (WP-11), ensure_applied (WP-27), install (WP-20), wrapper (WP-31) and frame (WP-17/18/19); bones' form for from_env and Ctx | — |
