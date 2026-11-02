# Changelog

Newest first. One line per change of stage, release or decision that the next session needs.

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
