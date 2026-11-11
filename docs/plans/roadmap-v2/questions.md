<!-- written by the fold consistency pass, 2026-11-10 -->
# Owner questions (roadmap v2)

These are the open owner questions the fold agents raised. Duplicates are merged and the questions are grouped by file or WP. Each one has a recommended answer (**Rec**). Until the owner answers, the pass runs on the Rec, or on the §7 default where §7 has one.

**Unattended builders** (`00-run-the-pass.md` §0a): build what your WP file says; use a Rec only where the WP is silent. Do not carry out a Rec yourself when it needs any of the following; list it in the PR body as "Owner follow-up (question N)":
- a GitHub write or owner setting: 14, 19, 30, 38, 54, 64, 66 (follow-up issues or WPs), 16;
- an edit to §0, §2 or another WP's file that is not marked "Folded", "Settled" or "Withdrawn": 48, 55. If your own WP file already carries the change, it is done; if it does not, §2 and your WP file stand as written;
- network: 56 is a read-only fetch at a pinned revision that WP-30 declares itself; no other question allows a download.

The second part lists verify problems: claims the fold could not check, and gaps no file settles yet. The third part lists questions this pass settled, so nobody asks them again. The last part lists what still needs the owner but does not block the run.

## §0 (00-run-the-pass.md)

1. **Wave 2+ done-checks need WP-02.** From wave 2 on, every done-check ends in `tools/check.sh` (WP-02). But WP-03, WP-04, WP-05, WP-23 and others list only WP-01 in depends. Should §0 make wave 1 a gate, or should each WP add WP-02?
   **Rec:** add one §0 line: "a WP in wave 2 or later branches from `main` only after all of wave 1 has merged". It costs nothing, since wave 1 merges first anyway. Leave the depends lists alone.
   **Folded** into §0 "Base" (2026-11-10 final check).
   **Also (cross-file pass):** `03-wp-index.md` now states the rule: `depends` lists WP-02 only for a WP that uses another WP-02 file. WP-07, WP-15 and WP-28 dropped WP-02 on that rule.
2. **Refs or Closes for a WP that does only part of an issue.** Ruling D5 names only WPs with a device Q row. WP-06 writes `Refs #121, #122` and WP-07 writes `Refs #27`, though neither has its own Q row. WP-04, WP-05 and WP-23 write `Closes` (page logic only; the device check is Q15). Craft A7 listed WP-04 and WP-05 as Refs WPs.
   **Rec:** one rule in §2 "PR and issue rules": write `Refs` when a device Q row or another WP still owes work on the issue, and `Closes` otherwise. That keeps WP-06/07 on Refs and WP-04/05/23 on Closes, as the files say now.
   **Folded** into §2 "PR and issue rules".

## §2 (01-new-bones.md)

3. **Registry shape.** `CommandSpec{id, group, args, output, effects:&[SideEffect]}` with `SideEffect{ReadOnly, Probes, WritesFiles, RunsCommands}`. D2 named id/args/effects; A9 added group/output. **Rec:** confirm as written.
4. **`cli::ok_exit` and the doctor.** WP-01 writes it in full: a red doctor report exits 1, and the JSON envelope stays `ok:true` with `data.ok:false`, because the locked render does not change. **Rec:** confirm.
5. **CI py_compile.** The CI fallback uses check.sh's `python3 -W error -m py_compile` for both file lists. **Rec:** confirm `-W error`.
6. **What `--ci` turns into a failure.** `--ci` fails a missing tool or target. A step whose input file has not landed yet (`decky/package.json`, `docs/workbench.md`) reports `n/a`, and `n/a` stays `n/a`. WP-02 already writes it this way. **Rec:** confirm. File guards never turn into failures.
7. **The `api::error_help` table.** Should §2 freeze it, and are the values right? WP-01 sets `retryable` true only for `link-not-ready`, `command-failed` and `io`, and gives `link-not-ready` the fix `flat3d-inject start-3d`. WP-08's two link-not-ready messages name `systemctl --user start|restart flat3d-link.service` instead (plus `flat3d-inject install` the first time).
   **Rec:** keep the table in WP-01 only (it sits in WP-01c.md), with WP-01 as its single writer, and do not copy it into §2. Change the `link-not-ready` fix to `systemctl --user restart flat3d-link.service` so it matches WP-08. That is a one-line WP-01 edit.
   **Folded** into the `error_help` table in WP-01c.md.
8. **`setup packages` output.** Its COMMANDS output cell is an untyped key set, not a DTO name. **Rec:** accept it for this pass. A typed report is a later WP-29/api change.
9. **D-number clash.** Rulings D1-D6 (2026-11-09) share numbers with the older plan decisions D3-D6 in the §7 OD table and in "Resolved without the owner". 02 has a note and cites the new ones as "ruling Dn". **Rec:** keep the note for this pass. Rename the old ones to PD-n in the WP-34 doc pass, in one edit.
   **Settled:** the note in 02 stays and the PD-n rename is dropped. No WP owns it (WP-34 cannot edit 02).

## WP-01

WP-01 is built as four parts (`WP-01a.md` to `WP-01d.md`), and all four read this heading. Questions 10 and 11 are about WP-01c (`cli::COMMANDS`, `AnyCommand::id()`); 12, 13 and 14 are about WP-01b (the `architecture.rs` allowlist); 15 is about WP-01c and WP-01d (the glob-import guard). In §2 above, questions 4 and 7 (`ok_exit`, the `error_help` table) are WP-01c's.

10. **COMMANDS rows.** Confirm the 26 rows' effects and output columns: `start-3d`/`stop-3d` WritesFiles only, `setup packages` and `doctor` Probes, `launch`/`collection` ReadOnly, `watch`/`run` with empty output. **Rec:** confirm. The WP-29 reviewer checks WP-29's final Effects against these rows.
11. **`AnyCommand::id()`** returns the typed words (`switch get`), not the render name (`switch`). **Rec:** confirm the split. The registry is keyed by what the user types.
12. **`bin/flat3d-run.rs` allowlist entry.** The launcher execs the game in place of its own process, so it is not a runner call. **Rec:** keep it as a permanent, tagged allowlist entry (WP-31).
13. **`cli/link.rs` exec_launch (WP-08).** It uses `Command::new` + `CommandExt::exec`, and the architecture allowlist has no entry for `cli/link.rs`. **Rec:** WP-01 adds a tagged `cli/link.rs (WP-08)` entry, for the same reason as question 12. Do not move it into the frozen `exec.rs`.
   **Folded** into the allowlist in WP-01b.md, §2 "Only `exec.rs` spawns", §0 and WP-08.
14. **Who moves the `watch/` spawns into `exec.rs`?** The allowlist tags the `adb track-devices` and `ss -tlnp` sites with WP-09. But WP-09's Files do not include `exec.rs`, and only WP-10 may touch it (body only). **Rec:** retag both entries "follow-up" instead of WP-09, and file a follow-up issue. No WP in this pass owns the move.
   **Folded** into the allowlist in WP-01b.md, §2, §0 and WP-09. The follow-up issue is still for the owner or PM to file.
15. **Glob-import guard scope.** *(void, 2026-11-10 ruling 42: `tether::relay` is removed from the guard.)* It covers `api`, `install` and `tether::relay`. Should `tether::wifi` or `cli` join it? **Rec:** no. Keep the three.

## WP-02

16. **"ODs still on their default".** The §7 OD table has no marker that tells a default from a confirmed answer. **Rec:** add a `confirmed` column to §7. `issues_report.py --roadmap` then lists the rows without it.
17. **Pins for non-GitHub keys.** `lana-fix`, `control-fix`, `ac7-profile` and `ac7-prereqs` may state no `license`, and the `fetch: auto` rows for the first three give no `version`, which has no `TBD-at-pin` allowance. **Rec:** write `license = "unstated"`, allow `version = "TBD-at-pin"` for page-sourced keys, and fill both at Q0.
   **Folded** into WP-02 ("Defaults for a value that neither this table nor a metadata read gives").
18. **`pin.sh` when `gh api` fails.** It exits 3 `gh-failed` (retryable) instead of skipping the digest check. **Rec:** confirm. This is the safer trust-on-first-use choice.
19. **No squash or rebase merges.** `--roadmap` reads merged WPs from merge-commit subjects with `v2/wp-NN-`. **Rec:** confirm. The owner sets the repo to allow merge commits only (a GitHub setting, owner-run).

## WP-05

20. **Step-1 hint (ruling D3).** The fold wrote `"Run flat3d-inject install."`, because install (WP-20) puts the bundled adb in place. `appimage.yml` ships `--no-tools` (WP-22). **Rec:** keep the string. The AppImage path finds adb through the doctor and the "not bundled" note.
21. **Doctor button.** WP-05 has `SetupKind::Doctor` (Craft B) and WP-33 has three buttons, including Run doctor. An older WP-33 proposal dropped it. **Rec:** keep the doctor button.
22. **Show the install-apps/doctor summary.** Should `Done` carry `Result<String,String>` and `SetupState` gain a `note`? **Rec:** not in this pass. Only errors show, and that is a follow-up.

## WP-09 and WP-10

23. **`Tether::set_wivrn` signature.** *(void, 2026-11-10 ruling 42: WP-10 is dropped.)* §2 freezes `set_wivrn(app:String, flags:Vec<String>)`. A WP-10 proposal wanted `&WivrnStart`. **Rec:** keep the §2 form.
24. **`PlanCtx` adb field.** *(void, 2026-11-10 ruling 42: WP-10 is dropped.)* §2 freezes `PlanCtx{pub relay}`, so `SpawnRelay` calls `start_relay(&ctx.relay, "")`. A caller that runs `run_plan_in` on a bare `ProcessRunner` would start gnirehtet with `ADB=""`. **Rec:** add `pub adb:String` to §2 and WP-01 before wave 1 starts, while it is still cheap. If that is too late, add a doc line saying only `Tether` may call `run_plan_in`.
   **Settled:** §2 keeps `PlanCtx{pub relay}` with no `adb` field. WP-10 adds the doc comment that only `Tether` may call `run_plan_in`.
25. **"No adb is available" refusal.** Should it name a fix (Craft A4)? **Rec:** yes: `fix: flat3d-inject install`, the command that copies the bundled adb into tools_dir.
   **Folded** into WP-10 (`; fix: flat3d-inject install`).

## WP-11

26. **Raw runner in `publish()`.** *(void, 2026-11-10 ruling 42: the tether adapter is removed.)* From the WP-11 merge (wave 2) until WP-09 adds its `TetherAdb` adapter (wave 4), the tick's adb probes run bare `adb`. **Rec:** accept the gap and list it under WP-11's `## Not verified`. Nothing ships between those waves.
   **Folded** into WP-11's `## Not verified` list and a known-interim note in §2.
27. **`tunnel` meaning.** *(void, 2026-11-10 ruling 42: `tunnel` is removed.)* The filled tick reports `tunnel: false` in QuestTether mode, and in Own mode until tun0 is probed Connected. Today it is `tether != Off`. **Rec:** intended. `tunnel` now means "our tun0 is up".
   **Folded** into WP-11 (tick step 1).

## WP-13

28. **Hash-mismatch hint.** It says `packaging/pin.sh <key> --check`, because plain `pin.sh <key>` exits 5 when the key already has a sha. **Rec:** confirm `--check`.
29. **Full uevr sha in `real_pins`.** The test uses the full 64-hex value, which WP-02's pins table also carries. **Rec:** confirm. Both must stay equal.

## WP-14 and WP-15

30. **UEVR profile copy.** `apply_with` carries only the `uevr-profile` pin key. Nothing copies a fetched profile (such as `ac7-profile`) into the prefix, so AC7's profile has no effect. **Rec:** file a follow-up issue. It is not in this pass.
31. **`to_win_path`.** WP-15 keeps the conversion and adds an absolute-path check after it, so `--dll x` is an Err. **Rec:** keep that. Do not go strict Windows-only.
   **Settled the other way: no conversion.** WP-15 has no `to_win_path`; `--dll` is strict Windows-only, and WP-14's `debug_cmd` already sends `Z:\` paths. The Rec above is withdrawn.

## WP-16

32. **Error codes.** `bad-recipe` for bad route values and for an ini with no line for a key; `not-found` for a missing exe, cache or viewer; `refused` for "already set up"; `io` for a file-system failure after rollback. **Rec:** confirm, and keep a missing fix or geo11 cache as `not-found` with its `setup fetch <key>` hint.

## WP-17, WP-18 and WP-19 (viewer)

33. **`--dump-eyes`.** Add `eye_to_ppm` (WP-17) and the flag (WP-19), or drop the g2-viewer clause that needs it? **Rec:** drop the clause from WP-24's g2-viewer row. Its measure is already optional, and adding it is new scope.
   **Folded:** no viewer WP ships an eye-dump flag, and WP-24's g2-viewer measure is optional and never blocks PASS.
34. **Portal restore-token reuse** (a second start with no picker) has no Q10 step. **Rec:** add an ask step `portal-restore-token` to WP-24's g2-viewer session.
   **Folded** into WP-24's g2-viewer row and §5.
35. **Named Q10 steps for WP-19's unverified items.** **Rec:** yes. Add ask steps to g2-viewer in WP-24, the way WP-18 has `overlay-in-capture` and `portal-window-kde`.
   **Folded** into WP-24's g2-viewer row and §5 (six WP-19 ask steps).
36. **`split_until_pid` and the supervisor** sit in `xr/session.rs`. A separate `xr/run.rs` would change WP-01's `xr.rs` and the §2 tree. **Rec:** keep them in `xr/session.rs`.

## WP-20

37. **Missing `flat3d-viewer` in the release folder.** Install skips it and adds it to the "not bundled" note, as it does for tools. **Rec:** keep the skip. The doctor reports a missing viewer, and WP-16's `viewer_check` names the fix.
38. **Full backup and rollback for install** (Craft completeness idea 7). **Rec:** not in this pass. The partial-failure manifest and the atomic unit write stay. File a follow-up.

## WP-22

39. **`build-appimage.sh --bin-dir`** versus re-exec into craft-build. **Rec:** keep `--bin-dir`. It matches `build-decky.sh` and lets a viewer built in craft-build be packaged.

## WP-23

40. **#117 locked-test sha256.** No source pins it, so WP-23 cannot call `tools/locked-test.sh 117 …`. **Rec:** WP-23 uses the awk extract and records the sha under `## Verified`. The owner then adds it to the locked-test list, as for #76/#116/#118.

## WP-24

41. **q12 Java compile argv.** **Rec:** WP-30 names it (javac path and Ghidra classpath under `{re_home}`), because it owns the Java files. WP-24 copies it in.
   **Folded:** WP-24 gives the argv under "q12 compile step", and WP-30 quotes the same six elements and tests them from `sessions/q12.toml`.
42. **One `q` per TOML.** link-smoke drives both Q3 and Q4. **Rec:** keep a single `q = "Q3"`, with the doctor step labelled as Q4's driver.
43. **q12 device.** **Rec:** the owner names the RE host's device id. Until then the row says "RE host".
   **Folded:** WP-24's q12 row uses `device = ["build-box"]`.

## WP-25a and WP-25b

44. **`check.py` runs `session.py render`.** **Rec:** OK. It is a pure, read-only render.
45. **Address-map staleness.** Should check.py's `generated` rule also run `fill_map.py --check`? **Rec:** no. Keep it in check.sh (A12), and reword the bones line to "a stale block fails `tools/check.sh`".
   **Folded:** the §2 line now says a stale block fails `tools/check.sh`, and check.py does not run `fill_map.py`.
46. **Generated markers in `recipes/README.md`.** WP-34 step 6 expects `<!-- BEGIN GENERATED: route-keys -->`. **Rec:** WP-25b places that named pair. check.py checks only that the markers are there and in order; WP-34 writes the block.
   **Withdrawn:** WP-25b defines one unsuffixed pair (`<!-- BEGIN GENERATED -->` / `<!-- END GENERATED -->`), and WP-34 places that pair in `recipes/README.md` and writes the block.

## WP-26

47. **SM2 (2651280) seed header** says "no session in this pass (port route, #27b)". **Rec:** confirm.

## WP-27

48. **`cli/undo.rs` in WP-27's Files.** WP-01 already writes its working body, and WP-27 only adds `cli_undo.rs`. **Rec:** drop it from WP-27's Files and the index row, in the same edit.
   **Not applied (cross-file pass):** `cli/undo.rs` stays in WP-27's Files. If a `cli_undo.rs` case fails, that file is the builder's only place to fix it, and keeping it costs nothing.
49. **WP-20 in WP-27's depends.** Nothing in WP-27 uses install or launch options. **Rec:** drop it from both the body and the index. WP-27 stays in wave 3 because of WP-13.
   **Folded** into WP-27 and the index.

## WP-28

50. **`cli_any.rs` and the view rows.** §2 says the `view` sub-rows "join the parse check when WP-28 fills the view grammar", but `cli_any.rs` is a WP-01 file. **Rec:** WP-01 writes the loop over every `cli::COMMANDS` row and skips the view sub-rows only while `parse_any` returns the stub's `not-built` result, so they join with no edit. WP-28 keeps its own parity test as well.
   **Folded** into the `cli_any.rs` registry checks in WP-01c.md (it skips a view sub-row while `parse_any` errors on it).
51. **Stale snapshot and toasts.** **Rec:** keep reading raw events. Toasts are one-shot and carry their own `seq`.

## WP-29

52. **The doctor's `pins` check source.** **Rec:** no `p.pins()` seam. Read the compiled table that `fetch::pin` uses, and test through a `pins_in(&BTreeMap)` form, as `fetch::pin_in` does.
   **Folded** into WP-29 ("Pins source for the `pins` check").
53. **One gnirehtet APK path.** *(void, 2026-11-10 ruling 42: gnirehtet leaves the app.)* The doctor checks `relay::apk_path`. **Rec:** install-apps uses `apk_path` too, so there is one path.
   **Folded** into WP-29 (check 4 and `setup install-apps`).
54. **Quest-app version mismatch fix.** No command fixes it. **Rec:** the fix text says to update the headset app to the server's version, and a follow-up adds a command. The detail keeps both versions.
   **Folded** into WP-29 check 2.
55. **Q4 prerequisites.** A green doctor needs Q0 (all pins) and Q2 (the injector exe). **Rec:** yes, list Q0 and Q2 as Q4's prerequisites in §5.

## WP-30

56. **HEROVR names.** One read-only fetch of `BUILD-SM2.md` at `herovr_rev`. **Rec:** approve it and add it to §0's declared network uses.
   **Folded** into §0's Network list and WP-30.
57. **`verified_by` format** `<csv basename> sha256:<first 12 hex>`. **Rec:** confirm.

## WP-31

58. **Which issues the PR refs.** **Rec:** `Refs #32, #43, #44, #51, #52 (code half)`.
59. **The 13 `FLAT_REASONS` codes** (appid, snapshot, stale, settings, recipe, no-recipe, switch-off, link-not-ready, not-injectable, home, route, wivrn, sidecar). **Rec:** confirm.
60. **Sidecar spawn test.** **Rec:** the pure `sidecar_argv` test plus a `## Not verified (Q10)` line is enough. No test seam.
61. **`WINEDLLOVERRIDES` from Steam launch options.** **Rec:** merge. Append the route's entries to the inherited value, and the route wins for the same dll.
   **Folded** into WP-31 (merge rule and test 21).
62. **`viewer_check` from the launch layer** (WP-16.md says WP-31 calls it). **Rec:** strike WP-16's line. A sidecar failure already falls back to flat with reason `sidecar`.
   **Folded:** WP-16 says no other caller is required, and WP-31 says it does not call `viewer_check`.

## WP-32

63. **Collection errors in the panel.** `syncCollection` returns null on an `!ok` envelope. **Rec:** return `CollectionPlan | ApiError` and show the error, as launch errors are shown.
   **Folded** into WP-32 (`CollectionPlan | ApiError`).

## WP-33

64. **Headless PNG renders of the app views** (Craft completeness idea 4). **Rec:** later, as a follow-up issue. Test 20's named view states cover this pass.
65. **"Check again" runs `setup packages` twice.** **Rec:** accept the cost. It keeps WP-05's frozen busy rule.

## WP-34

66. **A durable commands-table test** (Craft C2, an optional WP-35). **Rec:** yes, as a follow-up WP. In this pass, WP-34 keeps the regex done-check and fills the §1 Proof cells itself.

---

## Unresolved verify problems

- **No pre-fold text to diff against.** `roadmap-v2.md` is now a stub and git was off-limits, so these could not be traced:
  - §6 never having had a WP-05 hint follow-up;
  - NP-1..NP-14 kept word for word;
  - WP-03's "rustfmt changes 68 diff lines" (#76);
  - the 8 rustfmt hunks for #117 (WP-23);
  - WP-23's Closes wording;
  - WP-30's HEROVR seed check values and the `evidence != herovr` lint.
- **Citations to files outside the repo**, not checked:
  - `uevr-openxr.md`, `critique.md` and buildenv (WP-15);
  - `geo11-viewer.md` (WP-16, WP-18 lines 9 and 102, WP-19:133);
  - the pipewire-0.10.1 and openxr 0.22 registry line refs (WP-18, WP-19);
  - the `@decky/ui` typings and `decky.md` refs (WP-32).
  Also unconfirmed: whether `CCollection.apps` is a Set or a Map (`.keys()` works either way).
- **Pins with no source page.** `geo11` and `wivrn-apk` have no `page` in WP-02's table. WP-02 writes `page: TBD-at-pin` for them; the owner fills it before Q0.
- **"NOT READY" text for WP-25b.** A review asked for that cause text to be updated, but no plan file holds it. Nothing was changed.
- **Lint note.** The `files-drift` rule in WP-02 is now written as the hand lint here ran it: the Files block starts at a `**Files` line, paths are backticked tokens outside parentheses, brace forms expand, and globs match. Every WP now puts its Files heading on its own line. The hand lint finds 0 problems on the current files.

## Settled by this pass (no owner action)

- WP-34 owns the §1 Verdict and Proof cells. Its Files and the index row list `00-run-the-pass.md (§1 Verdict and Proof cells only)`.
- WP-02 defines `tools/locked-test.sh`, and its index row lists it with its test.
- WP-14's `support/uevr_home.rs` and `fixtures/uevr-debug-cmd.txt` are in its Files and its index row, and WP-15 reads the fixture (WP-15 moved to wave 3).
- WP-22's stray review note about WP-19 and WP-21 moved to 02's review notes.
- WP-22's index row lists Q1, Q14 and Q16.
- WP-24, WP-25a and WP-34 all use the long STEP0-RUNSHEET header (`<!-- generated by tools/session/session.py render … -->`).
- WP-01 writes every COMMANDS row, `list` and `status` included. WP-07 does not edit `cli.rs`.
- WP-29's Effects match WP-01's COMMANDS rows (`doctor`, `setup packages` and `setup launch-env` Probes; `setup install-apps` WritesFiles, RunsCommands). Checked in the final check, 2026-11-10.
- *(void, 2026-11-10 ruling 42: WP-10 is dropped and `sync_wifi` is removed.)* WP-09 calls `Tether::sync_wifi` once after an Own connect, and WP-10's connect does not. Both files agree.
- WP-07 writes `Refs #27`. WP-04, WP-05 and WP-23 write `Closes`. The index matches every WP's Refs/Closes line (question 2 asks whether that rule should be written down).
- WP-16 cache paths: `cache_dir` is always the cache root `p.cache()` (WP-27 passes it), and WP-16's depth ≤ 3 searches allow for the `<version>` folder.
- check.py on the whole repo: WP-25a always skips `docs/plans/`, and WP-25b's notes say so.
- WP-06 outside citations: WP-06 no longer cites a file outside the repo.
- WP-02 check.sh decky steps: the `decky` step runs `pnpm install --frozen-lockfile --offline` locally and drops `--offline` under `--ci`; it matches WP-21's done-check.
- `ensure_applied` error text: §2, WP-27 (`"<code>: <message>"`) and WP-01's stub (`Err("not-built: <fn> (WP-xx)")`) agree.
- WP-26's two review notes about WP-24's sessions moved to 02's review notes.
- Depends cleanups: WP-27 no longer depends on WP-20, WP-29 no longer on WP-22, and WP-30 now depends on WP-24.

## Needs the owner (not blocking the run)

Each line gives the default the builders use until the owner says otherwise.

- **WP-12:** which issue the PR references. Default: audit IDs only, no issue number.
- **WP-10:** no issue reference. Default: none.
- **WP-16:** no `geo11-fix` key means overlay only; a missing `geo11-exe` gives `bad-recipe`; the `.flat3d-fetch` marker is left out of copies. Default: as written.
- **WP-15:** `--dll` is strict Windows-only, and `--dry-run` may exit 3 when the SDK is missing. Default: as written.
- **WP-21:** `jsx: react-jsx` with the pinned rollup is unverified, and so are the node and pnpm versions. Default: if `pnpm run build` fails for that reason, record `blocked`.
- **WP-26:** `geo11-exe: Planet of Lana.exe` is unconfirmed. Default: write it as given; session `g1-geo11` checks it.
- **WP-29:** *(void, 2026-11-10 ruling 42: gnirehtet leaves the app.)* the AppImage ships `--no-tools`, so `setup install-apps` cannot install gnirehtet there. The doctor stays red on pins until the Q0 pins are filled, which is correct under ruling D4. Default: as written.
- **WP-30:** the HEROVR items H1-H11 are not listed in the plan; the `conf` and `how` texts are defaults; the SM2 seed header is question 47. Default: as written.
- **WP-31:** the Geo-11 `d3d11.dll` on a flat fallback. Default: leave as is; decide after Q9.
- **WP-02 pins:** `geo11` and `wivrn-apk` have no source page. Default: `page: TBD-at-pin`, filled before Q0.
- **Locked tests:** the sha256 values and the "rustfmt changes N lines" counts for #76, #116 and #118 are unverified (#117 and #67 were checked live). Default: a mismatch is a `blocked` result, not an edit to the locked test.
- **`third_party/SpidyVR`** is not checked out, so WP-24's citations into it are unverified. Default: WP-24 builds from its own text.
- **WP-18 and WP-19** are the hardest for a small model (PipeWire and ashpd; unsafe OpenXR and Vulkan; both need the build box). Default: give them to a stronger model, or accept a `blocked` result.
- **WP-24 is oversized** (27 KB, 23 session TOMLs). A split into "runner and stereo tools" and "session TOMLs" is proposed, not done. Default: one WP.
