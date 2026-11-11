# Roadmap v2 progress

Written only by the coordinator. Rules: `00-run-the-pass.md` §0a "Progress file".
Status: `todo`, `running`, `done`, `blocked`, `skipped`.

| WP | Wave | Status | Branch | PR | Notes |
|---|---|---|---|---|---|
| WP-00 | 0 | done | `v2/wp-00-cargo-fmt-clippy-gate` | [#201](https://github.com/iannitram/flat-to-3d/pull/201) | Windows, Git Bash, Rust 1.95.0. All done-checks passed, 463 tests. |
| WP-01a | 1 | done | `v2/wp-01a-workspace-reshape-moves-manifests` | [#202](https://github.com/iannitram/flat-to-3d/pull/202) | Windows, Git Bash, Rust 1.95.0. Done-check passed. check.sh skipped as specified before WP-02. |
| WP-01b | 1 | done | `v2/wp-01b-leaf-modules-exec-split` | [#204](https://github.com/iannitram/flat-to-3d/pull/204) | Done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. |
| WP-01c | 1 | done | `v2/wp-01c-cli-bones-parse-any` | [#206](https://github.com/iannitram/flat-to-3d/pull/206) | Done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. Roadmap helper listed it ready while running. |
| WP-01d | 1 | done | `v2/wp-01d-frozen-stubs-later-wps` | [#207](https://github.com/iannitram/flat-to-3d/pull/207) | Done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. Function-scoped Clippy allowances preserve the frozen signature. |
| WP-02 | 1 | done | `v2/wp-02-repo-tooling-check-sh` | [#203](https://github.com/iannitram/flat-to-3d/pull/203) | Done-check and integration gate passed. Integration 10 passed, 1 skipped (capture), 5 n/a. AC7 source page TBD-at-pin. |
| WP-25a | 1 | done | `v2/wp-25a-docs-check` | [#205](https://github.com/iannitram/flat-to-3d/pull/205) | Done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. |
| WP-03 | 2 | done | `v2/wp-03-gamerow-rows-view` | [#208](https://github.com/iannitram/flat-to-3d/pull/208) | Done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. Device checks Q14/Q15 unverified. |
| WP-04 | 2 | done | `v2/wp-04-app-headset-page` | [#210](https://github.com/iannitram/flat-to-3d/pull/210) | Windows done-check passed, 9 locked tests. Integration 10 passed, 1 skipped (capture), 5 n/a. Docker fixture commit mode changes corrected in 076c61d before verification. Q15 deferred. |
| WP-05 | 2 | done | `v2/wp-05-app-setup-page` | [#209](https://github.com/iannitram/flat-to-3d/pull/209) | Windows done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. A delayed Docker fixture appended local mode-only commit dde6f59 after the verified head 43949cb. PR and integration use 43949cb. |
| WP-06 | 2 | done | `v2/wp-06-logs-view-model-app` | [#211](https://github.com/iannitram/flat-to-3d/pull/211) | Windows done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. |
| WP-07 | 2 | blocked | `v2/wp-07-list-and-status-commands` | [#212 (draft)](https://github.com/iannitram/flat-to-3d/pull/212) | `tools/check.sh` exited 1. `crates/engine/src/cli.rs:617` frozen test `execute_human_not_built_goes_to_stderr`: assertion failed, left: 0, right: 2. WP-07 implements list and cannot edit cli.rs. Focused tests 8+12+6 passed. Roadmap helper still lists blocked WP-07 as ready. |
| WP-11 | 2 | blocked | `v2/wp-11-health-loop` | [#213 (draft)](https://github.com/iannitram/flat-to-3d/pull/213) | tools/check.sh exited 1. Frozen watch/service.rs assertions: :655:9 and :721:9 assertion failed: up.tunnel. :609:9 assertion failed: fake.borrow().ran.is_empty(). All six focused suites passed. Roadmap helper still lists blocked WP-11 as ready. |
| WP-12 | 2 | done | `v2/wp-12-link-stop-idempotence-wivrn` | [#214](https://github.com/iannitram/flat-to-3d/pull/214) | Windows done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. Q3 deferred. |
| WP-13 | 2 | blocked | `v2/wp-13-pinned-fetch` | [#215](https://github.com/iannitram/flat-to-3d/pull/215) | Linux branch done-check passed. Windows post-merge tools/check.sh exited 1 and merge rolled back. Four fetch tests panicked at fetch.rs:327:5: assertion left == right failed: sha output starts with a backslash, left: 65, right: 64. See wp-13-integration.log. Roadmap helper still lists blocked WP-13 ready. |
| WP-14 | 2 | done | `v2/wp-14-uevr-route-file-half` | [#217](https://github.com/iannitram/flat-to-3d/pull/217) | Linux done-check passed (12 pass, 0 skip, 4 n/a). Isolated Linux integration 11 pass, 0 skip, 5 n/a. Windows fixture path/temp limitations and Q6 deferred in PR. |
| WP-16 | 2 | done | `v2/wp-16-geo-11-route-pure` | [#216](https://github.com/iannitram/flat-to-3d/pull/216) | Windows done-check passed after in-scope Clippy fix. Integration 10 passed, 1 skipped (capture), 5 n/a. PR remains draft. Q9 deferred. |
| WP-17 | 2 | done | `v2/wp-17-viewer-pure-modules` | [#218](https://github.com/iannitram/flat-to-3d/pull/218) | Windows done-check passed. Isolated Linux integration 11 passed, 0 skipped, 5 n/a. Q10 deferred. |
| WP-18 | 2 | done | `v2/wp-18-viewer-capture` | [#220](https://github.com/iannitram/flat-to-3d/pull/220) | Linux exact done-check passed including capture and xr,capture Clippy. Isolated Linux integration 11 passed, 0 skipped, 5 n/a. Q10 deferred. |
| WP-20 | 2 | done | `v2/wp-20-install-execute-launch-options` | [#219](https://github.com/iannitram/flat-to-3d/pull/219) | Linux done-check passed. Isolated Linux integration 11 passed, 0 skipped, 5 n/a. Q3 device checks deferred. |
| WP-21 | 2 | running | `v2/wp-21-decky-scaffold` | [#221](https://github.com/iannitram/flat-to-3d/pull/221) | Builder done-check passed at b9fc03da1b5519ba5ed4cbb535cd7baece936166 (Linux 14 passed, 0 skipped, 2 n/a). Awaiting integration merge and gate per owner stop. Offline pnpm 9.15.9 store available in wp21-work. |
| WP-22 | 2 | running | `v2/wp-22-packaging-scripts` | [#222](https://github.com/iannitram/flat-to-3d/pull/222) | Builder done-check passed at af342e2eb41d30bc5ffec090336acc90fef663fb (Linux 12 passed, 0 skipped, 4 n/a). Packaging fixture suite ran 26 with 1 skip (unzip absent). Awaiting integration merge and gate per owner stop. |
| WP-23 | 2 | todo | - | - | |
| WP-24 | 2 | todo | - | - | |
| WP-25b | 2 | todo | - | - | |
| WP-08 | 3 | todo | - | - | |
| WP-10 | 3 | todo | - | - | Roadmap helper lists it ready before wave 2 settles. Held for wave 3. |
| WP-15 | 3 | todo | - | - | |
| WP-19 | 3 | todo | - | - | |
| WP-26 | 3 | skipped | - | - | waits on WP-13 |
| WP-27 | 3 | skipped | - | - | waits on WP-07 |
| WP-28 | 3 | skipped | - | - | waits on WP-07 |
| WP-30 | 3 | todo | - | - | |
| WP-09 | 4 | skipped | - | - | waits on WP-11 |
| WP-29 | 4 | skipped | - | - | waits on WP-27 |
| WP-31 | 4 | skipped | - | - | waits on WP-27 |
| WP-32 | 4 | skipped | - | - | waits on WP-28 |
| WP-33 | 4 | skipped | - | - | waits on WP-28 |
| WP-34 | 5 | skipped | - | - | waits on WP-07 |

Run log (one line per stop: time, reason, tip of `v2/integration`):
- 2026-11-10 17:53:27 -07:00 | Owner asked to stop after current builders. WP-21 and WP-22 passed their branch done-checks and await integration gates. No new build started by coordinator. | a6bf4626a445b68438f308bf0a07ba1400adf407
