Stage: Dev Loop
Since: 2026-10-26
Kind: own
Spec: SPEC.md   Roadmap: docs/ROADMAP.md + docs/PORT-SPIDERMAN2.md (Steps 0-8)   Code map: docs/code-map.md
Peers reviewed: 2026-10-31 (PEERS.md)
Platforms: linux (SteamOS on the owner's Z13 + Quest 2 over USB; game DLLs cross-built for Windows and run under Proton)
Notes: Spec re-approved 2026-10-31 by the owner (1.0 = Zero Company first, then Spider-Man 2, both from Start 3D + Inject; first approval 2026-10-26). 1.0 issues: Zero Company #31-#36 (first; #34 only if #32 fails), Spider-Man 2 #5, #7, #18-#29; from-peer #37, #38 (#6 and #10 are parents; #15 and #16 are closed). Nearly every port check needs the owner's Z13, the games and the headset (needs-owner-device in each Done-when); satellites can take #24, #25 and #26 now. Easy issues #24 and #29 carry locked tests (R28). Engine is Rust and recipes live in recipes/ (the owner's 2026-10-26 rulings).

Stages: Proposal -> Spec -> Roadmap -> Dev Loop -> Prototype / Beta.
What each stage means and who moves a project: `PIPELINE.md` in the owner's private pipeline repo.
The Project Manager updates this file when the stage changes and adds a CHANGELOG line.
