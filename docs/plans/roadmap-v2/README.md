# Roadmap v2 (split)

Roadmap v2 used to be one 393 KB file, too big for one agent context. It is now one file per section and one per work package. Read `00-run-the-pass.md`, the parts of `01-new-bones.md` your WP names, your own `WP-NN.md` (WP-01 is four files, `WP-01a.md` to `WP-01d.md`, one per builder), and the `questions.md` entries under §0, §2 and your WP.

**Unattended run:** start at `00-run-the-pass.md` §0a. It is the one entry point: what to read, wave order, branch names, `progress.md` and when to stop.

History only (a builder does not need this): old line numbers (`L1969`, `roadmap-v2.md:1969`) in the findings, the Craft review and older notes map to files through this table. Each file's first line also gives its old range.

| File | Old lines | Contents |
|---|---|---|
| [00-run-the-pass.md](00-run-the-pass.md) | 1-137 | §0 How to run the pass, §1 Audit verdict |
| [01-new-bones.md](01-new-bones.md) | 138-476 | §2 New bones (the frozen contract) |
| [02-queues-and-decisions.md](02-queues-and-decisions.md) | 477-609 | §3 conversions, §5 owner-device queue, §6 issue housekeeping, §7 owner decisions, Not in this pass |
| [03-wp-index.md](03-wp-index.md) | 610-651 | §4 Work packages: index |
| [WP-00.md](WP-00.md) | 652-754 | WP-00: cargo fmt + clippy gate (#183) |
| [WP-01.md](WP-01.md) | 755-1385 | WP-01: Workspace reshape + frozen bones. Pointer only: the WP is split into the four parts below, built in order. "Depends on WP-01" means all four merged |
| [WP-01a.md](WP-01a.md) | 755-1385 (part) | WP-01a: Workspace reshape: moves, manifests, CI |
| [WP-01b.md](WP-01b.md) | 755-1385 (part) | WP-01b: Leaf modules, exec split, architecture test |
| [WP-01c.md](WP-01c.md) | 755-1385 (part) | WP-01c: CLI bones: parse_any, command registry, exit hook, error_help |
| [WP-01d.md](WP-01d.md) | 755-1385 (part) | WP-01d: Frozen stubs later WPs fill, and the wiring to them |
| [WP-02.md](WP-02.md) | 1386-1542 | WP-02: Repo tooling: check.sh, pins, pin.sh, issues report |
| [WP-03.md](WP-03.md) | 1543-1608 | WP-03: GameRow rows view (#76) |
| [WP-04.md](WP-04.md) | 1609-1656 | WP-04: App Headset page (#116) |
| [WP-05.md](WP-05.md) | 1657-1729 | WP-05: App Setup page (#118) |
| [WP-06.md](WP-06.md) | 1730-1840 | WP-06: Logs view model + app Logs page |
| [WP-07.md](WP-07.md) | 1841-1931 | WP-07: list and status commands, catalog join (#27a part) |
| [WP-08.md](WP-08.md) | 1932-2001 | WP-08: Link commands wiring (#120) |
| [WP-09.md](WP-09.md) | 2002-2068 | WP-09: Watch service correctness |
| [WP-10.md](WP-10.md) | 2069-2171 | WP-10: Tether plans, runner errors, Wi-Fi sync |
| [WP-11.md](WP-11.md) | 2172-2282 | WP-11: Health loop (#125) |
| [WP-12.md](WP-12.md) | 2283-2385 | WP-12: Link stop idempotence + WiVRn detection |
| [WP-13.md](WP-13.md) | 2386-2487 | WP-13: Pinned fetch (`fetch.rs`) |
| [WP-14.md](WP-14.md) | 2488-2585 | WP-14: UEVR route file half |
| [WP-15.md](WP-15.md) | 2586-2701 | WP-15: flat3d-uevr-inject.exe + build-win.sh |
| [WP-16.md](WP-16.md) | 2702-2806 | WP-16: Geo-11 route pure half |
| [WP-17.md](WP-17.md) | 2807-2902 | WP-17: Viewer pure modules (#44a) |
| [WP-18.md](WP-18.md) | 2903-3003 | WP-18: Viewer capture (#44b) |
| [WP-19.md](WP-19.md) | 3004-3130 | WP-19: Viewer OpenXR + main (#44c) |
| [WP-20.md](WP-20.md) | 3131-3219 | WP-20: Install execute + launch options (#128) |
| [WP-21.md](WP-21.md) | 3220-3333 | WP-21: Decky scaffold (#130) |
| [WP-22.md](WP-22.md) | 3334-3419 | WP-22: Packaging scripts (#127/#129/#134) |
| [WP-23.md](WP-23.md) | 3420-3489 | WP-23: App Games page (#117) |
| [WP-24.md](WP-24.md) | 3490-3645 | WP-24: Session runner, stereo gates, device sessions |
| [WP-25a.md](WP-25a.md) | 3646-3712 | WP-25a: Docs check (wave 1) |
| [WP-25b.md](WP-25b.md) | 3713-3820 | WP-25b: Docs scrub + contradictions (wave 2) |
| [WP-26.md](WP-26.md) | 3821-3902 | WP-26: Recipe seeds + repo recipe lint |
| [WP-27.md](WP-27.md) | 3903-4022 | WP-27: inject, undo and setup fetch commands + ensure_applied (#27a) |
| [WP-28.md](WP-28.md) | 4023-4124 | WP-28: View commands (#121, #75) |
| [WP-29.md](WP-29.md) | 4125-4214 | WP-29: doctor + setup commands |
| [WP-30.md](WP-30.md) | 4215-4327 | WP-30: RE tooling + address map (#19 tools, #37 column) |
| [WP-31.md](WP-31.md) | 4328-4415 | WP-31: Wrapper route end-to-end |
| [WP-32.md](WP-32.md) | 4416-4485 | WP-32: Decky panel, launch sync, collection, toasts (#131-#133) |
| [WP-33.md](WP-33.md) | 4486-4594 | WP-33: App views + main wiring (#122/#123) |
| [WP-34.md](WP-34.md) | 4595-4709 | WP-34: Docs sync after the pass |
| [progress.md](progress.md) | — | Coordinator's status table for the unattended run (§0a) |
| [rulings-2026-11-09.md](rulings-2026-11-09.md) | — | Owner rulings D1-D6 (already folded into §0, §2, §7 and the WPs) |
| [questions.md](questions.md) | — | Open owner questions with recommended answers, and unresolved verify problems |
| [fold-log/](fold-log/) | — | One record per WP and section of each fold decision (history, not contract) |
