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
| WP-07 | 2 | blocked | `v2/wp-07-list-and-status-commands` | [#212 (draft)](https://github.com/iannitram/flat-to-3d/pull/212) | Reconfirmed PR #212 head d85cb66. Exact Linux done-check passed all three focused suites (12+10+6), then tools/check.sh exited 1 (11 pass, 4 n/a, 1 fail). Frozen cli.rs:617:9 execute_human_not_built_goes_to_stderr asserts exit 2 for list and received 0. The published WP-01c follow-up changes the frozen CLI file and is outside WP-07 scope. See wp-07/resume-branch.log. Q3 pending. |
| WP-11 | 2 | blocked | `v2/wp-11-health-loop` | [#213 (draft)](https://github.com/iannitram/flat-to-3d/pull/213) | Reconfirmed PR #213 head 0f2960e. Exact Linux done-check passed all six focused suites, then tools/check.sh exited 1 (11 pass, 4 n/a, 1 fail). watch/service.rs:609:9 fake.borrow().ran.is_empty() fails on the required battery probe; :655:9 and :721:9 up.tunnel fail because the service fake does not report a live relay and tun0. Those tests are outside WP-11 scope. See wp-11/resume-branch.log. Q3 pending. |
| WP-12 | 2 | done | `v2/wp-12-link-stop-idempotence-wivrn` | [#214](https://github.com/iannitram/flat-to-3d/pull/214) | Windows done-check passed. Integration 10 passed, 1 skipped (capture), 5 n/a. Q3 deferred. |
| WP-13 | 2 | done | `v2/wp-13-pinned-fetch` | [#215](https://github.com/iannitram/flat-to-3d/pull/215) | Head c4a0679a66edfe95b536cc06b8516b75029b3c7e integrated at 619a8cae7d4595d25fbe67eb9be75c511c742a7d. Linux integration check.sh: 13 passed, 0 skipped, 3 n/a, 0 failed. Exact Linux branch gate passed (20 fetch tests, 12 gate passes, 0 skips, 4 n/a). Windows focused tests passed all 20 with FLAT3D_REQUIRE_HOST_TOOLS=1. Windows full gate separately failed tools/pm/tests/test_issues_report.py fixture git rev-parse HEAD^{tree}, exit 128. Q0 pending. |
| WP-14 | 2 | done | `v2/wp-14-uevr-route-file-half` | [#217](https://github.com/iannitram/flat-to-3d/pull/217) | Linux done-check passed (12 pass, 0 skip, 4 n/a). Isolated Linux integration 11 pass, 0 skip, 5 n/a. Windows fixture path/temp limitations and Q6 deferred in PR. |
| WP-16 | 2 | done | `v2/wp-16-geo-11-route-pure` | [#216](https://github.com/iannitram/flat-to-3d/pull/216) | Windows done-check passed after in-scope Clippy fix. Integration 10 passed, 1 skipped (capture), 5 n/a. PR remains draft. Q9 deferred. |
| WP-17 | 2 | done | `v2/wp-17-viewer-pure-modules` | [#218](https://github.com/iannitram/flat-to-3d/pull/218) | Windows done-check passed. Isolated Linux integration 11 passed, 0 skipped, 5 n/a. Q10 deferred. |
| WP-18 | 2 | done | `v2/wp-18-viewer-capture` | [#220](https://github.com/iannitram/flat-to-3d/pull/220) | Linux exact done-check passed including capture and xr,capture Clippy. Isolated Linux integration 11 passed, 0 skipped, 5 n/a. Q10 deferred. |
| WP-20 | 2 | done | `v2/wp-20-install-execute-launch-options` | [#219](https://github.com/iannitram/flat-to-3d/pull/219) | Linux done-check passed. Isolated Linux integration 11 passed, 0 skipped, 5 n/a. Q3 device checks deferred. |
| WP-21 | 2 | done | `v2/wp-21-decky-scaffold` | [#221](https://github.com/iannitram/flat-to-3d/pull/221) | Head b9fc03da1b5519ba5ed4cbb535cd7baece936166 integrated at f15b43a05c2a7051e8ba3cf0c0bd1ad85527f34b. Linux integration check.sh: 13 passed, 0 skipped, 3 n/a, 0 failed. Exact Linux branch check repeated with 14 passed, 0 skipped, 2 n/a. Offline pnpm 9.15.9. Q14 pending. |
| WP-22 | 2 | done | `v2/wp-22-packaging-scripts` | [#222](https://github.com/iannitram/flat-to-3d/pull/222) | Head a809d5f3d90b60e3f3faa6d21a06ec444d23965e integrated at 4f8467961bc6c274e007751a5b70a93e7e71e847. Linux integration check.sh: 13 passed, 0 skipped, 3 n/a, 0 failed. Exact Linux branch check passed (Ran 27 tests in 0.554s in packaging/tests). Q1, Q14 and Q16 pending. |
| WP-23 | 2 | done | `v2/wp-23-app-games-page` | [#230](https://github.com/iannitram/flat-to-3d/pull/230) | Head 6ba8bc617a15f8ac619a4dce902244679eb48120 integrated at 6138e8dd40c478ce92ed660f10955940230a3cd2. Linux integration check.sh: 13 passed, 0 skipped, 3 n/a, 0 failed. Exact Linux branch done-check passed (9 focused tests, 14 gate steps, 0 skipped, 2 n/a). Locked #117 test matched before format. |
| WP-24 | 2 | done | `v2/wp-24-session-runner-stereo-gates` | [#231](https://github.com/iannitram/flat-to-3d/pull/231) | Head cbac0b42ce3939a9f64ab9fefa72a148a5226b7c integrated at 921320c2224a9772148a3e67504bd39d364f03a7. Linux integration check.sh: 13 passed, 0 skipped, 3 n/a, 0 failed. Exact Linux branch check passed (14 gate steps, 0 skipped, 2 n/a). All 23 physical sessions pending. Issue #7 has no labeled Done when; its explicit offline verification sentence was used verbatim. |
| WP-25b | 2 | running | `v2/wp-25b-docs-scrub-contradictions` | - | Assigned to Luna from verified v2/integration at 6138e8d. |
| WP-08 | 3 | todo | - | - | |
| WP-10 | 3 | todo | - | - | Roadmap helper lists it ready before wave 2 settles. Held for wave 3. |
| WP-15 | 3 | todo | - | - | |
| WP-19 | 3 | todo | - | - | |
| WP-26 | 3 | todo | - | - | WP-13 cleared on isolated Linux. Eligible after wave 2 settles. |
| WP-27 | 3 | skipped | - | - | waits on WP-07 |
| WP-28 | 3 | skipped | - | - | waits on WP-07 |
| WP-30 | 3 | todo | - | - | |
| WP-09 | 4 | skipped | - | - | waits on WP-11 |
| WP-29 | 4 | skipped | - | - | waits on WP-27 |
| WP-31 | 4 | skipped | - | - | waits on WP-27 |
| WP-32 | 4 | skipped | - | - | waits on WP-28 |
| WP-33 | 4 | skipped | - | - | waits on WP-28 |
| WP-34 | 5 | skipped | - | - | waits on WP-07 |

## Resume notes

- Restored from published checkpoint 42d63c1 and verified local tip a6bf462. The coordinator progress matched the checkpoint. The first-parent merge history identifies the exact tested head for each completed WP. Published heads have since advanced for WP-01a, WP-01b, WP-01c, WP-01d, WP-02, WP-14, WP-16, WP-18, WP-20 and WP-25a. The verified integration keeps its original versions. WP-22 was still awaiting integration and its newer PR head was verified before merging.
- Roadmap helper lists blocked WP-07, WP-11 and WP-13 as ready and lists future-wave packages before wave 2 settles. Use the status table and wave order.

Run log (one line per stop: time, reason, tip of `v2/integration`):
- 2026-11-10 17:53:27 -07:00 | Owner asked to stop after current builders. WP-21 and WP-22 passed their branch done-checks and await integration gates. No new build started by coordinator. | a6bf4626a445b68438f308bf0a07ba1400adf407
