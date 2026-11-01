Stage: Dev Loop
Since: 2026-10-26
Kind: own
Spec: SPEC.md   Roadmap: docs/ROADMAP.md + docs/PORT-SPIDERMAN2.md (Steps 0-8)   Code map: docs/code-map.md
Peers reviewed: 2026-10-31 (PEERS.md)
Platforms: linux (SteamOS on the owner's Z13 + Quest 2 over USB; game DLLs cross-built for Windows and run under Proton)
Notes: Spec approved 2026-11-01 by the owner ("Plug in and play", Rust) (1.0 = five Favorites, in order Zero Company, ACE COMBAT 7, Control + Planet of Lana via Geo-11, Spider-Man 2; gamepad; Start 3D + Stop 3D). 1.0 issues: Zero Company #31-#36 (first; #34 only if #32 fails), ACE COMBAT 7 #40-#42, #47, Geo-11 #43-#46, #48, #49, Stop 3D #39, engine: link service #85-#94, #119, #120 and #26, #39 (parents), #64-#66; tether #95-#104, #124, #125; wrapper and install #105-#112, #126; routes #27, #51, #52, #79-#84; views #74-#77, #113-#115, #121; native Rust desktop app #78, #116-#118, #122, #123; thin Decky TS panel #130-#133 (needs-owner-device); packaging #127-#129, #134; engine base #50, #58-#63; Spider-Man 2 #5, #7, #18-#29; from-peer #37, #38 (#6 and #10 are parents; #15 and #16 are closed). Nearly every port check needs the owner's Z13, the games and the headset (needs-owner-device in each Done-when); satellites can take #24, #25 and #26 now. Easy issues #24 and #29 carry locked tests (R28). Engine is Rust and recipes live in recipes/ (the owner's 2026-10-26 rulings).

Stages: Proposal -> Spec -> Roadmap -> Dev Loop -> Prototype / Beta.
What each stage means and who moves a project: `PIPELINE.md` in the owner's private pipeline repo.
The Project Manager updates this file when the stage changes and adds a CHANGELOG line.
