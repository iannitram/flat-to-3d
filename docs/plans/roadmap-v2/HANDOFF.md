# Flat to 3D roadmap v2 handoff

## Latest local checkpoint

Verified integration tip **921320c2224a9772148a3e67504bd39d364f03a7**. WP-13 and WP-21 through WP-24 have passed their integration gates. For the current statuses and PRs, see [progress.md](progress.md). The historical handoff below documents the original published checkpoint. If restoring from this local checkpoint, use its verified integration tip in the restore command.

The 2026-11-10 run stopped after the owner asked the two active builders to finish. Of 39 runnable work packages, 17 passed integration, two passed their branch checks and await integration, three are blocked, nine are skipped on dependencies, and eight are queued. [Progress and PRs](progress.md)

## Restore the coordinator checkout

Read the repository’s `AGENTS.md`, [roadmap README](README.md), and [runbook §0a](00-run-the-pass.md) before resuming. The owner requested two Luna builders at a time in wave order, one branch and worktree per package, and a draft PR for useful partial work on a blocked package. Keep issue access read-only and integrate locally. The owner’s stop remains in effect until the next round is authorized.

`origin/v2/roadmap-handoff` carries the verified integration history plus the coordinator’s notes. The tested integration tip is `a6bf4626a445b68438f308bf0a07ba1400adf407`. In the existing checkout, use the local `v2/integration` and its working copy of `progress.md`. In a fresh clone, create the local integration branch at that exact tip and restore the progress file from the handoff branch.

```bash
git fetch origin refs/heads/v2/roadmap-handoff:refs/remotes/origin/v2/roadmap-handoff
git switch -c v2/integration a6bf4626a445b68438f308bf0a07ba1400adf407
git restore --source origin/v2/roadmap-handoff --worktree -- docs/plans/roadmap-v2/progress.md
git tag v2-wp00 8429e1fcc943be0cc02f3ae78502e1b8a85d7512
```

The tag command is for a fresh clone where `v2-wp00` is absent. Keep it local. Use the tested integration tip as the base for work package branches. The two handoff commits belong to the coordinator’s checkpoint.

## Resume wave 2

1. Reconfirm the existing PR heads and apply the [§0a resume procedure](00-run-the-pass.md) to the two `running` rows. Both builders already passed their exact done-checks. Integrate and gate one package at a time, with `FLAT3D_BASE=v2/integration`. Use the existing PRs.

   | Package | Verified head | Builder result |
   | --- | --- | --- |
   | WP-21 | `b9fc03da1b5519ba5ed4cbb535cd7baece936166` | Linux gate 14 passed, 0 skipped, 2 n/a. Locked test hash matched. Q14 remains pending. [PR #221](https://github.com/iannitram/flat-to-3d/pull/221) |
   | WP-22 | `af342e2eb41d30bc5ffec090336acc90fef663fb` | Linux gate 12 passed, 0 skipped, 4 n/a. Packaging suite ran 26 tests with one declared skip because `unzip` was absent. [PR #222](https://github.com/iannitram/flat-to-3d/pull/222) |

2. If a post-merge `tools/check.sh` fails, restore the prior local integration tip and record the exact failure in `progress.md`. If it passes, mark the row `done`.
3. The remaining queued wave 2 packages are WP-23, WP-24, and WP-25b. After wave 2 settles, the currently queued wave 3 packages are WP-08, WP-10, WP-15, WP-19, and WP-30. WP-30 requires WP-24 and WP-25b. Use the [work package index](03-wp-index.md) and the progress table to select eligible packages. The roadmap helper has listed blocked packages and future-wave packages as ready. Follow the progress table and wave order when they differ.

## Recover blocked packages

- **WP-07** implements `list`, while the frozen `execute_human_not_built_goes_to_stderr` test in `crates/engine/src/cli.rs:617` expects exit code 2. It received 0. Review which command the frozen test should exercise under the roadmap and get the owning scope resolved before changing it. Preserve the test’s coverage of the unimplemented-command path. [Draft PR #212](https://github.com/iannitram/flat-to-3d/pull/212)
- **WP-11** passed its focused suites. The full gate failed three frozen assertions in `crates/engine/src/watch/service.rs` at lines 609, 655, and 721. Reconcile the health-loop changes and the service’s command and tunnel expectations with the owning work package before retrying. [Draft PR #213](https://github.com/iannitram/flat-to-3d/pull/213)
- **WP-13** passed its branch gate on Linux. The Windows integration gate failed four tests at `crates/engine/tests/fetch.rs:327` because `sha256sum` emitted an escaped digest token of length 65. The merge was rolled back. Recheck the published head against current integration in isolated Linux. If Windows support is required for these fixtures, handle the escaped output within WP-13 while retaining the exact digest validation. [PR #215](https://github.com/iannitram/flat-to-3d/pull/215)

Recompute skipped rows when a blocker clears. WP-07 directly holds WP-27 and WP-28, WP-11 holds WP-09, and WP-13 holds WP-26. WP-29, WP-31, WP-32, and WP-33 are downstream. Finish WP-34 after its dependencies pass.

## Reproduce the checks

- The full integration gate ran in a separate Linux clone with Rust 1.95.0, Python 3.12, and PipeWire/SPA and Clang development packages. On the original machine, the Docker image is `flat3d-wp13:ubuntu24-rust195`, and the independent integration clone and build cache are in `flat3d-integration-linux`. Use each package’s exact done-check, `FLAT3D_BASE=v2/integration`, and its own `SCRATCH` and `CARGO_TARGET_DIR`. Refresh the independent clone to the exact commit under test.
- WP-21 used Node 22.23.4 and pnpm 9.15.9 for local verification. Its preseeded offline pnpm store is `/work/.pnpm-store/v3` in `wp21-work`, and the Node installation was placed at `/opt/node` in `wp21-builder`. Reuse that toolchain and store for the offline Decky portion of `tools/check.sh`. CI uses the version prescribed by the runbook.
- Use a normal independent Git clone inside Docker. The previous shared-worktree setup leaked Git environment variables into test fixtures, causing mode changes and shared config edits. Use a read-only source mount for cloning or fetching, and keep Git metadata and test scratch inside the isolated environment. The WP-04 PR includes the corrective commit. The local WP-05 fixture commit `dde6f59` is mode-only, and the published PR and integration contain the verified head `43949cb`.
- Device checks remain as recorded in the package PRs and the roadmap. Keep the documented skips and unverified checks visible when updating the progress table.
