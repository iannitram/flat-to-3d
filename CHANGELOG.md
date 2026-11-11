# Changelog

Newest first. One line per change of stage, release or decision that the next session needs.

- 2026-11-11 Documentation scrub (WP-25b): consolidated the SPEC approval history here and aligned the README, roadmap, code map and workbench page with the current tree.
- 2026-11-11 Spec re-approved by the owner ("Approve (Recommended)", ruling 47) with a six-game 1.0: Zero Company, ACE COMBAT 7, Control, 007 First Light, Spider-Man 2, Alan Wake 2, in that order ("By readiness (Recommended)", ruling 46). Ruling 43 ("This is the 1.0 list.") brought in 007 First Light, Alan Wake 2 and Forza Horizon 6 and moved Planet of Lana to after-1.0; ruling 45 ("Move it to after-1.0") took Forza Horizon 6 out again; ruling 44 ("Yes, where nothing better exists (Recommended)") lets alternate-eye rendering count where no two-real-views route exists, at 30 fps per eye. SPEC M23 and M33 moved out, M31 and M32 added with planned, untested routes; Control takes the first Geo-11 test (G1, #43); the SBS viewer stays in 1.0 for Control, 007 First Light and Alan Wake 2 (#256).
- 2026-11-11 CI moved from the self-hosted runner to GitHub-hosted `ubuntu-24.04` for `check.yml` and `appimage.yml` (ruling 48); the packaging test and roadmap docs now say so.
- 2026-11-07 Wave 1 merged (PRs #190-#197, main `50c1e49`, 459 engine + 4 app tests): #66, #119, #75, #99, #84, #96, #51 file half, #78. Roadmap and code map synced; next is #183 alone, then wave 2 (#120, #125, #76, #116, #118).
- 2026-11-07 #78 app crate: `app/` (iced 0.14, linked to `inject/` by path) opens a window with the four tabs (Setup, Headset, Games, Logs) on placeholder text, tested in `lib.rs`; the `app` job in `check.yml` runs its `cargo test` and `cargo check`. The faces stage has started.
- 2026-11-07 Link stage E2 (WiVRn half) done: `link::stop_3d` closes the WiVRn app, the `adb reverse` and the server unit and reports Stopped, so Start 3D works again (#66, rebuilt after PR #188 was closed unmerged).
- 2026-11-07 Blocker check: the base is not built (logic tested, only `flat3d-run` wired); roadmap "What's next" now runs in waves from wave 1 (#66, #119, #75, #99, #84, #183, #96, #51, #78), stage table carries each open issue's blockers, and the recipes code-map cell is back to exact `present` for the locked test.
- 2026-11-07 Roadmap and code map audited against main (ff12fef): engine progress per module, issue numbers on every planned part, a "Where it stands" section and stage table with done and open issues, the critical path to a first Zero Company try; #66 found closed but not on main.
- 2026-11-02 Code map refreshed to match main: settings, setup, collection, toasts and the tests are present; the command-line rows say what is parsed and what is still stub.
- 2026-11-01 The engine's first dependencies: serde + serde_json for inject/src/api.rs (E1d).
- 2026-11-01 Spec re-approved by the owner after M12 and M17 were reworded to rulings 24-41.
- 2026-11-01 SPEC M12 and M17 reworded from the Inject button to the `flat3d-run` launch wrapper and per-game 3D switch (rulings 24-41); approval back to "not yet" until the owner re-approves.
- 2026-11-01 Spec approved by the owner (Plug in and play, Rust; rulings 24-41).
- 2026-11-01 Owner rulings 39-41 applied: Rust-only app (iced) and engine with a thin TS Decky panel, no Python backend, no ui-shared; code map, roadmap, SPEC and issues re-cut (new #85-#134, Tauri/ui-shared/Python issues #28, #53-#57, #67-#73 closed).
- 2026-11-01 Spec reworked by the "Plug in and play" grill (rulings 24-38): Steam is the launcher, the link starts on plug-in, Quest Tether's tether is ported into the Rust engine, a per-game 3D switch replaces Inject; M9-M11, M18, M24 rewritten, M25-M30 added. Approval back to "not yet".
- 2026-10-31 Engine Mediums #25, #26, #39, #50 split into Easy issues #58-#66: briefs and locked tests (recipes E1a-E1c, api.rs E1d-E1e, --json E1f, link E2a-E2c); the JSON contract is in docs/code-map.md "API shape".
- 2026-10-31 SPEC M10 reworded by the owner's ruling: Start 3D uses WiVRn's wired mode only; Quest Tether optional.
- 2026-10-31 One engine, two faces: a Tauri desktop app (M24) joins the Decky plugin, re-approved by the owner; code map rewritten (inject/ engine with api.rs and per-route injectors, app/, ui-shared/, decky/, viewer/, layer/, games/), roadmap development stages E1-D, issues #50-#57.
- 2026-10-31 Spec widened again and re-approved: 1.0 = five Favorites (Zero Company, ACE COMBAT 7, Control, Planet of Lana, Spider-Man 2), gamepad only, Stop 3D (M18-M23); #9's SBS viewer in 1.0; roadmap gains the play round trip and the AC7 and Geo-11 tracks.
- 2026-10-31 Spec widened and re-approved: STAR WARS Zero Company joins 1.0 and ships first (UEVR, our own hook as fallback; Musts M13-M17; roadmap track Z1-Z5). PEERS.md added with 8 approved peers (Prior art in SPEC.md); HEROVR already ports SpidyVR to Spider-Man 2 on Windows: learn only, our port stays.
- 2026-10-26 End-of-night sweep: roadmap and code map brought in line with main (nothing merged since 2026-10-19), PIPELINE notes corrected (#15 closed), #10 unblocked, size labels on #6 and #10.
- 2026-10-26 The owner's rulings: the Inject engine is Rust (crate `inject/`, binary `flat3d-inject`; the Python stubs are replaced; the Decky backend stays Python and calls the binary); recipes are a data file in this repo (#15). CI now runs cargo build + test on Linux.
- 2026-10-26 Spec approved (SPEC.md: Spider-Man 2 + Start 3D); code map docs/code-map.md; Inject engine skeleton (inject/, decky/, recipes/) with CI; issues #18-#29; stage Proposal -> Dev Loop.
- 2026-10-26 Normalized to pipeline v2 at Proposal.
- 2026-10-26 Project notes brought into the repo.
- 2026-10-18 Project started: README, GPL-3.0 licence.
- 2026-11-01 SPEC approval: Plug in and play, with M12 and M17 reworded to the launch-wrapper route and the Rust desktop app plus thin TypeScript Decky panel (rulings 24-41); the five-game list it approved was superseded by ruling 43 and the 2026-11-11 approval (ruling 47).
- 2026-10-31 SPEC approvals: the five Favorites and gamepad-only scope, including Start 3D/Stop 3D and the SBS viewer; Zero Company first with UEVR then our own hook if needed; WiVRn wired mode.
- 2026-10-26 Initial SPEC approval: Spider-Man 2 plus Start 3D.
