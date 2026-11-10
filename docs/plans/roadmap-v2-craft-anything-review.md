# Roadmap v2: Craft Anything agent-dev review (2026-11-09)

Status: **Proposal.** Nothing here changes roadmap-v2.md until the owner rules on section D. Source methodology: `storytold/craft-anything` at `2e7f276` (MIT OR Apache-2.0), dev workflow only. Method: 40 chunks of roadmap-v2.md reviewed one per agent, each suggestion checked by a second agent (144 kept, 76 refuted), then merged.

Scope: `docs/plans/roadmap-v2.md`, the 4,708-line plan. The input was 144 checked suggestions from all 40 chunks, merged here into 12 changes to the whole roadmap, per-WP lines, 2 new WPs and 6 rulings. Sizes are easy, medium or hard. "L" means a line in roadmap-v2.md.

In short: the plan already has stable IDs, 8 frozen error codes, a shared fake runner and an index that can be parsed. What it lacks is enforcement. Most of its rules are prose that only a reviewer checks. These changes turn them into checks that an agent can run on its own.

## A. Changes to the whole roadmap (ranked by value for agent work)

**A1. Fold the reviewer fixes into the WP text before wave 0.** Medium. L3, the 35 findings JSONs, the 13 `*-proposal.md` files.
- Today every worker merges three sources in its own context: the WP text, the findings JSON and the proposal. That is 196 unapplied fixes, and it is the load that makes sub-agents choke.
- One doc-only pass copies in the 13 proposals, applies the 129 fixes that target their own WP, and adds the 67 cross-WP fixes to the WPs they target. Each fold is re-checked against §2.
- After that, L3 becomes "read §0, §2 and your WP", and the findings step goes away.
- The pass also clears stale fixes. Example: WP-17.json still asks for the PixelFormat/Cpu shape, which §2 L392 dropped.

**A2. One gate: `tools/check.sh` is the only list of CI steps.** Medium. L53-57, L441-452, L1286-1301, L1405-1433, L481.
- The two step lists already differ in five places: py_compile `-W error`, the capture clippy flags, the `decky/tests` exclusion, `-p 'test_*.py'` and pnpm typecheck.
- check.yml does setup, then runs `tools/check.sh --ci --only rust|py|decky`. `--ci` turns every skip into a failure. The inline steps stay only behind `hashFiles('tools/check.sh')==''`, because WP-01 and WP-02 share wave 1.
- check.sh gets named steps, `FAIL <step>: fix: <hint>`, `--only`, `--list`, `--tests <cargo args>` and a closing pass/skip/fail summary. It sets CARGO_TARGET_DIR itself.
- Every done-check becomes `export CARGO_TARGET_DIR=$SCRATCH/target && <its tests> && tools/check.sh`. This fixes the done-checks of WP-04 (L1652), WP-05 (L1725), WP-11 (L2275), WP-15 (L2698), WP-18 (L2994, which keeps its craft-build clippy), WP-21 (L3327), and WP-23 (L3479) and WP-33 (L4583), both of which run inside craft-build. It also removes the duplicate chain at L1604.
- WP-01's 15-clause `&&` chain (L1362-1384) becomes labelled steps that print which clause failed.
- WP-00's fmt and clippy steps print their fix (L703-710), and WP-01's rewrite (L1368) keeps those messages.

**A3. A plan lint and a "what can start now" command.** Medium. L7, L506, L553, L612-650, L1408, L3705 vs L618.
- Add one read-only mode, `tools/pm/issues_report.py --roadmap` (WP-02). It lists the WPs that are ready (every dependency merged, read from merge-commit subjects), the blocked ones, the ones waiting on the owner device, the Q rows that are ready, and the ODs still on their default. It supports `--json`.
- The lint exits non-zero, naming the WP and the fix, when:
  - two WPs in one wave list the same file;
  - a WP depends on a WP in its own or a later wave;
  - an index `files:` list differs from the WP's "Files (exclusive)";
  - a WP touches a frozen path (L64-78) it is not named for;
  - the WP body and the index disagree on depends (WP-25a has this today).
- Add a scope step to check.sh: on a `v2/wp-NN-*` branch, every changed path must be in WP-NN's `files:`, plus the body-only allowlist at L799-803.
- The proposals put this lint in two places (check.py and issues_report.py). Keep only issues_report.py.

**A4. Settle the error contract.** Easy for the contradiction, medium for the rest. L173, L364-374, L436, L1969.
- L436 says no WP adds an error code. L1969 (WP-08) says a WP may add one. The locked `api_response.rs` test pins exactly 8. Strike L1969 and add a resolved line to §7 (L564-571).
- State the exit contract once in §2: usage and not-built exit 2, every other error exits 1, and JSON goes on stdout. The locked cli_json tests already enforce this.
- Optional: a frozen `api::error_help(code) -> (retryable, fix)`, plus a test that every code has an entry. The ApiError JSON stays `{code,message}`.
- Engine String errors that reach `inject` start with a locked code and ": " (WP-14 L2516-2526, WP-16 L2737-2753, contract at L3970).
- Every refusal that has one fix names the command: link-not-ready (L1958), pin mismatch (L2404), undo first (L4006), install errors (L3167).
- The faces keep the code instead of dropping it (WP-32 L4449, WP-33 L4510).
- Docs get one error-code table in code-map (WP-25b L3763).
- Tool exit codes stay per tool. A repo-wide table would conflict, because pin.sh, uevr-inject and packaging already give the same numbers different meanings (L173, L1451, L1516, L2633).

**A5. Face parity tests: every argv a face sends must parse.** Medium. L460, L496, L1676, L3237-3261, L4457, L4533-4542.
- Three reviewers caught the Decky drift by hand (`launch-sync`, the bare `launch` line) and gave three different fixes.
- Decky: one Rust test, `crates/engine/tests/decky_face.rs`, in WP-28 (add WP-21 to its depends, L643). It reads `decky/src/engine.ts` and runs each wrapper's sample argv, with the trailing `--json`, through `cli::parse_any`. It also checks the TS ErrorCode union and the keys of the types.ts mirrors. Drop the other two homes the proposals named (in WP-32).
- App: WP-33 moves the argv into a pure, exhaustive `engine::effect_args(&Effect)` and tests that every variant parses. This covers the WP-04 setting keys (L1648), WP-05 SetupKind (L1676) and the WP-23 Games actions (L3452).
- Reverse checks:
  - every row of the code-map `## Commands` table parses (WP-34, L4624);
  - link verbs match LINK_USAGE in both directions (WP-08, L1963-1967);
  - every `flat3d-inject` argv in the session TOMLs is in the Commands table (WP-24, L3535).

**A6. Layering lints that name their fix.** Easy to medium. L70, L77, L140, L771, L828, L1640, L3451, L4504, L4543.
- App: a check.sh step fails on `std::fs`, `std::process`, `std::net`, `std::thread` or `Command::new` in `crates/app/src` outside `engine.rs`, with the message "return an Action; engine.rs does the I/O". It passes today. Do not allow-list `flat3d_inject::` imports, because the pages import `setup::Packages`, `face::logs` and `rows::GameRow`.
- Engine: WP-01 adds `tests/architecture.rs`. `Command::new` may appear only in exec.rs, plus a tagged allowlist of today's sites (link.rs, watch.rs, tether/adb.rs, bin/flat3d-run.rs), each tagged with the WP that moves it. A strict repo-wide rule would fail on day one.
- WP-01 adds `mod_module_files = "deny"` (L828). L140 assumes this lint is on, but it is off by default.
- Small local backstops: WP-06 source-scan tests, the WP-10 tether grep, WP-33 test 18.

**A7. "Verified / Not verified" in every PR, and "Refs", not "Closes", for half WPs.** Easy. Needs your ruling (D1, D5).
- Add a §0 bullet after L57. The PR body lists Verified (each done-check command and its exit status) and Not verified (each host skip it hit, plus its device Q row).
- WPs that only fakes prove use the existing "Unverified on the device (Qn):" form (L2990).
- A WP whose device half is a Q row writes `Refs #N (code half)`, so the merge does not auto-close the issue: WP-04/05 (M24, L1610, L1658), WP-08 (#120), WP-14 (L2490), WP-22 (L3335), WP-31 (L4329). State the rule once in §2.
- WP-24 session results record `commit`, `dirty` and who checked each step, and the .md output has Verified and Not verified sections (L3548-3553).

**A8. Each deferred device claim maps to a session step.** Easy. L506-524, L3606-3622.
- link-smoke (L3610) gains these steps:
  - for WP-09: unplug/replug and sleep/wake heals;
  - for WP-10: tun0 has `inet`, the gnirehtet package is present, and Wi-Fi comes back after disconnect;
  - for WP-12: stop-3d twice, then status shows stopped, plus a Quest app version check;
  - for WP-07: `list --json`.
- decky (L3621) gains four steps for WP-32: launch-opts-persist, collection-save-persist, toast-shows, xdg-runtime-set.
- g2-viewer (L3617) gains two ask steps for WP-18: overlay-in-capture and portal-window-kde.
- q2 (L3607) is rewritten to its §5 meaning, the injector build (L510). It gates on `data.ok`, not the envelope's `ok`, and requires the quest-app detail to differ from "no headset".
- q12 compiles the Java scripts before the long analyze steps.
- Every TOML gets `q = "Qn"`, and `check` rejects unknown Q ids. q0, q2 and q12 follow §5 (L508, L510, L520).

**A9. Declared effects for each command.** Easy for the tests, medium for a registry. L236-245, L336-363, L460, L4624.
- Recommended cheap path: each command WP adds an "Effects:" line and a read-only test. The test checks that FakeRunner ran nothing (or only allowed probes) and that the temp tree is byte-identical. This applies to:
  - WP-07 list and status;
  - the WP-08 read verbs;
  - WP-11: tick never runs a heal;
  - the WP-12 detection probes;
  - WP-27: inject never writes localconfig.vdf;
  - the WP-28 view verbs;
  - the WP-29 doctor probes.
- The WP-34 Commands table gains an Effects column.
- Full path: a frozen `cli::COMMANDS: &[CommandSpec{id, group, output, effects}]` in WP-01, plus `AnyCommand::id()`. This reopens the frozen contract (D2). Don't reuse the name `Effect`, which is taken (L192-194, L4501).

**A10. Stable IDs and proof.** Easy. L82-136, L573-588, L974-977, L4644-4652.
- The audit table gets a Proof column. "fixed" splits into "planned" and "verified @sha", and WP-34 fills in the SHAs. IDs are never renumbered or reused.
- Stubs name their WP: `NotBuilt("<path> (WP-xx)")`. The architecture test checks that every stub names a real WP, and a `--stubs` mode lists the open ones by WP.
- In the WP-34 ROADMAP rewrite: unnumbered "not in this pass" items get NP-n IDs. Closed items are struck through, not deleted. The Q table is "copied from §5 with Driver rewritten", not "verbatim".

**A11. Shared named fixtures.** Easy. L1904-1916, L2340, L2541.
- One test-support directory, `crates/engine/tests/support/`. WP-07 adds `scenes.rs` (no_steam, two_libs_*, link_ready_fresh, link_stale…). WP-14 adds `uevr_home.rs` (uevr_ready, uevr_unfetched…, with `#![allow(dead_code)]`). WP-27 and WP-28 reuse them. The proposals named two different directories; use one.
- New tests use `testing::FakeRunner`, not a hand-rolled queue fake (WP-12, L2340).

**A12. Generated docs stay current.** Medium. L466, L3652, L4236, L4658-4664, L4698.
- check.py gets a `generated` rule. It covers BEGIN/END GENERATED blocks, and a stale block prints its regenerate command. WP-34 then drops its one-off diff (L4698).
- Fix the header clash between L3555 and L4663 by using WP-24's header.
- check.sh runs `fill_map.py --check` when the file exists (WP-30). This needs `--vt` to be optional in check mode (L4236).

## B. Per-WP revisions (WP order)

- **WP-00:** anchor the format-only proof to the merge-base and assert exactly 2 commits (L744-751). Easy. Replace "rehearsal passed" with a filled Verified block, because rehearsal counts are not branch results (L662, L753). Easy.
- **WP-01:** make `dispatch_routes_each_family` a table with one row per dispatch row, covering setup, doctor, inject and start-3d (L1161-1167, L1185-1189). Easy.
- **WP-01:** rows check routing only, so WP-07 filling in `list` does not break the test. This also fixes the stale flag at L1927. Easy.
- **WP-01:** hosts A6 (architecture.rs, mod_module_files) and A10 (stub tags). Medium.
- **WP-02:** pin.sh prints a stable stderr token on each exit, `pin: <token> retryable=<yes|no> fix: …`, and tests assert it (L1442-1466, L1516). Easy.
- **WP-02:** test_pins uses a stand-in `gh` on PATH to cover digest match, mismatch and absent. Define what happens when `gh api` fails (L1455-1459, L1480-1490). Easy.
- **WP-02:** add `tools/locked-test.sh <issue> <dest> <sha256>`. It replaces the awk/sha/CR/gh prose that WP-03/04/05/23 copy by hand (L1586-1591, L1644, L1712, L3299-3303). Easy.
- **WP-02:** hosts A2, A3, the app lint in A6, and the fill_map step in A12.
- **WP-03:** use locked-test.sh. The PR lists "M11/M24 on the device" as Not verified (L1544, L1601). Easy.
- **WP-04:** Closes becomes "SPEC M24, page logic only; device via #122/Q15" (L1610). Easy. Add an inline fact: `SetSetting.key` must parse as `setting set <key>`, tested in WP-33 (L1648). Easy.
- **WP-05:** `args()` drops `--json`, because run_cli appends it. Match the kind set to WP-33's three buttons, including doctor (L1675-1682 vs L4512, L4527, L4550). Add a `kind_args_parse` test through parse_any. Easy.
- **WP-05:** the hint follow-up (L1662) becomes an open item with an ID (D3). Fix the test reference `v[5]` → `v[0]`. Easy.
- **WP-06:** one "Command contract: logs" block covering input, engine call, output, effects and Msg Lines/Failed. WP-28 (L4043-4044, L4067-4070, L4104) and WP-33 (L4540, L4567) cite it instead of contradicting it. Medium.
- **WP-06:** source-scan tests for the no-I/O rules (L1773, L1784). Easy.
- **WP-07:** Effects line and test 8 (A9), the scenes file (A11), and a Not verified line for real vdf and compatdata (L1929). Easy.
- **WP-08:** test that the read verbs leave the runner empty and the tree unchanged. Bad args give `usage`, exit 2 and the exact usage text. Assert exit 1 for run errors (L1976-1986). Easy.
- **WP-08:** link-not-ready gets two messages, one for a missing snapshot and one for a stale one, each naming its fix (L1958). Add link usage parity and a Not verified list pointing to Q3 (L1963-1967, L1999). Easy.
- **WP-09:** add an "Unverified on the device (Q3)" line for the heal paths (L2066). Easy.
- **WP-09:** add the test `two_stop_requests_end_stopped`, and route StopLink through `link::stop_3d` (L2044-2052, L2334). Easy.
- **WP-10:** one gnirehtet APK path for connect and doctor, with a test that doctor checks the same path install uses (L2109, L2146 vs L4151-4188). Easy.
- **WP-10:** the done-check greps for no `Command::new` in tether, and the PR has a Not verified device list (L2155-2156). Easy.
- **WP-11:** lock the `Heal::word()` values and the soak line field order with tests, and state that they are never renamed (L2183, L2214, L2218, L2251). Easy.
- **WP-11:** add the test `tick_never_runs_a_heal` (L2223-2229). Easy.
- **WP-12:** use `testing::FakeRunner`, script every failure and assert `ran.len()` (L2340). Test that detection runs only read-only probes (L2353-2361). Name the link-smoke steps on the owner line (L2375). Easy.
- **WP-13:** test that each FetchError variant maps to its §2 code. Fix WP-27's mapping, which names variants that don't exist (L2450-2460, L3966-3967). Easy.
- **WP-13:** the mismatch errors carry the pin key, and their text names `pin.sh <key>` (L2402-2405). Easy.
- **WP-13:** with `FLAT3D_REQUIRE_HOST_TOOLS=1`, a live test that would skip fails instead. Delete the stale flag at L2475 (L2462). Easy.
- **WP-13:** add a pure `is_fetched()`. Doctor and WP-27 call it instead of rebuilding the cache path (L2428, L3948, L3963, L4150-4152). Medium.
- **WP-14:** the debug_cmd text is the only argv contract. WP-15 (L2694) and WP-27 (L4007) point to it, with an optional golden fixture. Drop the "mirrored word for word" claim (L2528-2535, L2556, L2572). Medium.
- **WP-14:** `quotes_survive_shlex` also runs the real python3 `shlex.split` and `isabs` (L2557, L2584). Easy.
- **WP-15:** move the wait/fallback logic into a pure `Waiter` in pick.rs and test it on Linux with scripted snapshots (L2599-2633, L2664-2671). Easy.
- **WP-15:** exit-code constants in args.rs, and stderr lines of the form `error[<token>] … fix:` (L2609-2643, L1257). Easy.
- **WP-15:** q2.toml runs the injector build, not doctor (L510 vs L3607, L3901). Easy.
- **WP-15:** Not verified: the `+crt-static` link, `--install` and live injection. K21 becomes "fixed in code, pending Q2" (L125, L2695). Easy.
- **WP-15:** keep one argv wording (L2694 vs L2529). Use WP-14's golden file, not a new Cargo dev-dependency, because Cargo.lock is frozen (L29, L757). Medium.
- **WP-16:** add code prefixes and a table-driven `error_cases` test, plus a flag that WP-27 passes bare apply errors through (L2737-2753, L3956-3970). Easy.
- **WP-16:** split Verified from Not verified, and pin "ShaderFixes is empty after undo" (L2776, L2789-2800). Easy.
- **WP-17:** replace the restated frame block with a pointer to §2 (L2810-2820). Easy. Add `eye_to_ppm` for `--dump-eyes`, or drop the clause that depends on that flag at L3617. Medium.
- **WP-18:** inline `cfg(test)` tests for the SPA map, params, bitmask, skip helper and token writer, run in craft-build. Fix L2973's claim about what CI runs (L443). Easy.
- **WP-18:** a list mapping each behaviour to the check or Q10 step that verifies it (L2989-2993). Easy.
- **WP-19:** move `split_until_pid` out of the gated bin and test it round-tripping with `to_args()` plus `--until-pid` (L3039, L3106, L3121). Easy.
- **WP-19:** pure `classify_system_err`, `on_state` and a loop supervisor, in an xr-gated lib file (L3048-3106). Medium.
- **WP-19:** an "Unverified on the device (Q10)" list (L3108-3118). Easy.
- **WP-20:** write the manifest even when an install fails partway, and test uninstall after a failed install (L3160-3171). Easy.
- **WP-20:** one `install_roots()` shared by install and the uninstall refusal (L3156-3173). Easy.
- **WP-20:** fixed fix clauses in error messages. Don't point at doctor, which arrives later (L3167, L3172). Easy.
- **WP-21:** fix or drop `launchSync` and the bare `launch(line)` (L3242-3243 vs L343, L4055). Every wrapper gets a row in decky_face.rs. Medium.
- **WP-21:** add `test_shim_env.py` with a stand-in engine (L3313-3315). Easy.
- **WP-21:** check.sh step 11 adds typecheck, and the CI pnpm version cites §2 (L1431, L3285). Easy.
- **WP-21:** a types.ts key check replaces "api.rs wins" (L3273). Medium.
- **WP-22:** `packaging/lib.sh` with one PAYLOAD table and a parity test between the two faces. This also adds flat3d-viewer (L262, L3337-3378). Medium.
- **WP-22:** test the real path of build-tools.sh with stand-in curl, cmake, ninja, ldd and patchelf (L3347-3396). Medium.
- **WP-22:** state the exit codes once (0-4), with an unknown flag exiting 2 (L3338-3350, L3390). Easy.
- **WP-23:** drop the Action→CLI restatement (L3452), which disagrees with L4536, and point to WP-33's map. Easy.
- **WP-24:** the `q` field and `session.py pending` (L3517-3523, L3642). Easy.
- **WP-24:** `expect_json` and `expect_code` replace regexes over the envelope (L3527, L3607, L3610). Easy.
- **WP-24:** a dry run of every session file against a stand-in PATH, and `check` rejects globs in run argv (L3574-3593). Medium.
- **WP-25a:** finding lines of the form `path:line: rule: msg (fix: …)`, frozen rule names and `--json` (L3653, L3694-3701). Easy.
- **WP-25a:** `--changed BASE` for the wave-2 done-checks (L3703). Easy.
- **WP-25a:** an `ids` rule: every cited M<n> and ruling <n> must exist (L3659-3676). Medium.
- **WP-25a:** make the body's depends match the index (L3705 vs L618). Easy.
- **WP-25b:** workbench.md becomes a table with a check command per tool, and records exits 75 and 137 as "retry later" (L3789-3800). Easy.
- **WP-26:** move to wave 3 and depend on WP-13. Use `fetch::parse_pins`/`pin_in`, drop the copied parser, and make works-now reject TBD pins (L7, L638, L3853-3884). Medium.
- **WP-26:** failure messages name the file, key and fix (L3873-3877). Easy.
- **WP-26:** seed headers name their real sessions (z5-recipe, a3-sortie, g3-lana, g4-control) (L3837). Easy.
- **WP-26:** resolve the stale "WP-01 must freeze" flag (L3892-3894). Easy.
- **WP-27:** a test that inject never touches localconfig.vdf and never launches anything (L3940, L3977-3985). Easy.
- **WP-27:** an Effects list (L3934-3971). Easy.
- **WP-27:** the undo-first refusal names `flat3d-inject undo <appid>` (L4006). Easy.
- **WP-27:** the owner line matches the index (Q6, Q9), with a Not verified block (L4017 vs L642). Easy.
- **WP-28:** delete the flag at L4115. A trailing `--json` is legal after parse_any strips it, so move that case from the bad lines to the good ones (L4055-4057, L4092). Easy.
- **WP-28:** a read-only test for all the view verbs, and whole-object `json!` literals for LinkView (L4096-4110). Easy.
- **WP-29:** a red doctor exits 1 and still prints the full report (D4) (L4146, L4176). Medium.
- **WP-29:** add `every_fix_is_runnable` and `doctor_runs_only_probes` (L4147-4176). Easy.
- **WP-29:** install-apps re-reads `pm list packages` after installing (L4164-4170). Easy.
- **WP-29:** a `SKIP_NO_HEADSET` constant, so a skipped check isn't read as verified (L4148). Easy.
- **WP-30:** a `FLAT3D_HEADLESS` stand-in seam to test the analyze.sh `all` chain and the 75/137 pass-through (L4229, L4251-4253). Easy.
- **WP-30:** "Unverified until Q12" and an optional javac compile test (L4257-4272, L4324). Easy.
- **WP-30:** each `found` row carries a `verified_by` receipt (L4276, L4284, L4309-4312). Medium.
- **WP-31:** a `reason\t<code>` line in flat run records, with a test that pins the set of codes (L4353-4367, L4381-4396). Easy.
- **WP-31:** bin-level tests of flat3d-run with a stand-in game and sidecar (L4369-4376, L4410). Medium.
- **WP-31:** Closes says "code half", plus a Not verified block (L4329, L4376, L4413). Easy.
- **WP-32:** call only the named helpers, never a raw `engine("word")`. Show `error.code` verbatim, and have `syncLaunchOptions` return ApiErrors (L4418, L4425, L4449). Easy.
- **WP-32:** replace the device prose at L4478 with the four session step IDs. Easy.
- **WP-33:** append `--json` only once, add `run_cli_at`, and test it with a stand-in CLI (L4512, L4546). Medium.
- **WP-33:** `parse_envelope` keeps the ApiError, and a `usage` code renders as "app/CLI mismatch" (L4510, L4570). Easy.
- **WP-33:** layering test 18 (L4504, L4543). Easy.
- **WP-33:** named view states that smoke-test each view (L4547-4571). Medium.
- **WP-33:** a "Not verified until Q15" list (L4593). Easy.
- **WP-34:** add the `generated` rule (A12), the NP-n IDs and the Q table from §5 (A10), and the Effects column (A9) (L4624-4664). Medium.

## C. Suggested new WPs

**C1. WP-F: fold the reviewer fixes.** Wave −1, before WP-00. Coordinator only, doc only. It edits roadmap-v2.md, archives the findings files, and runs the A3 lint by hand, because that tool comes in WP-02. Medium. It unblocks everything else, so do it first.

**C2. WP-35: cross-face gate close-out.** Optional. Wave 5, in parallel with WP-34, depending on every code WP. It owns:
- `crates/engine/tests/commands_table.rs`, the reverse parity test;
- the check of session argv against the Commands table;
- filling in the SHAs in the audit Proof column.

This keeps WP-34 docs-only (L4678 says "no new test files"). Medium.

Everything else goes into existing WPs: the A3 lint (WP-02), locked-test.sh (WP-02), decky_face.rs (WP-28) and the session steps (WP-24, through the fold).

## D. Needs your ruling

**D1. PR review rules (methodology pick 6, still pending).** These are the "Verified / Not verified" sections and "merge only the reviewed head SHA". A related question is merge strategy: either the two-commit landings stay as two commits (WP-00, L3483), or a squash is pinned to the reviewed SHA. A7 and about 25 B lines depend on this. Recommended: adopt both.

**D2. Reopen the frozen WP-01 contract?** §0-7 plus WP-01 are the corrected contract. The candidates to add are `api::error_help`, a dispatch hook that lets a command set its exit code (needed for D4), and the full `cli::COMMANDS` registry. Recommended: allow the first two, which are small and additive. Defer the registry and take A9's cheap path.

**D3. The WP-05 step-1 hint (L1662, L1687).** May it differ from the brief to match ruling 30? No WP owns `setup_page.rs` after wave 2, so whoever carries it out has to be named.

**D4. The red-doctor exit code.** Today a red report exits 0 inside an `ok:true` envelope (cli.rs:102-105). The options are exit 1 with the full report, or keep exit 0 and make every gate read `data.ok`. Callers rely on this, which is why it needs your call. Recommended: exit 1.

**D5. Issue closing for half WPs.** The repo convention is `Closes #N` (AGENTS.md:6-9). GitHub writes belong to you or the PM (L61). Recommended: `Refs #N` for any WP with a device Q row, closed when the session passes.

**D6. Confirm the 8 error codes are closed for this pass.** That means striking L1969. Recommended: yes, since the locked test already agrees.

No change needed: none. Every chunk kept at least one suggestion.

## Completeness check

> It was checked against the roadmap v2 WP index and a map of the current code, not against the report's items.

- **Idea 1, command registry with declared effects:** no WP in the WP index mentions a registry or descriptor. the code map shows command names copied by hand in four places (Command::name, parse, usage, decky). The best home is WP-07, WP-08, WP-27 or WP-28 (all cli/*.rs), or WP-01's frozen bones. Any report item on this needs a WP to anchor it.
- **Idea 2, face parity test:** this fits the project. WP-33 adds crates/app/src/engine.rs and WP-32 adds decky/src/engine.ts, but no WP tests that each app or Decky action has a CLI counterpart. It belongs in WP-33's tests or a small WP after it.
- **Idea 3, ErrorCode contract:** this fits. Codes are strings, exit codes come from string matching, and about 37 fns return `Result<_,String>` (the code map-27). Only WP-10 ("runner errors") touches errors. No WP owns api.rs ApiError, so the item needs an owner, most likely WP-01 since it freezes the contract.
- **Idea 4, named scenes and screenshots:** partly fits. The app is iced, not Tauri, so mockIPC/Playwright does not apply. The egui-style `demo.rs` scenes rendered to PNG do apply, to WP-33 app views and the WP-04/05/23 pages. Report items should not borrow the Vite/Playwright mechanics.
- **Idea 5, one CI gate with architecture lints:** WP-00 (fmt+clippy) and WP-02 (tools/check.sh) cover the gate. Architecture lints are not covered: "app does no I/O except through engine", "only exec.rs spawns", and "code-map/command list current" (WP-25a does docs only). These belong on WP-02.
- **Idea 6, PR review rules:** these are rules for the pipeline, not the repo. No coverage is correct here, but AGENTS.md (WP-25b) could carry the Verified / Not verified PR template.
- **Idea 7, budget/MCP:** the code map says there is no LLM spend and no MCP server, so no coverage is right. The one exception is **setup as transaction plus doctor**, which maps directly to WP-29 (doctor + setup) and WP-20 (install execute): plan, diff, backup, write, re-read, roll back.
- **In-scope WPs to check for an item:** WP-07/08/27/28 (CLI: registry, error codes), WP-33 (app views: parity, scenes), WP-00/02 (CI: lints), WP-25a/34 (docs check: generated command list), WP-29 (doctor/setup: transaction), WP-24 (tests: session runner could report the error codes).
- **Thin-evidence risk:** an item citing WP-01 as "the" registry home is inferred, because the WP index lists only file globs for WP-01. Any item claiming the app already calls the engine is wrong: the code map found zero `flat3d_inject::` uses in app/src.

