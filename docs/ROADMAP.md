# Roadmap: five Favorites in 3D

Where the project stands and what comes next. This is the plan. The SM2 track is [spiderman2/plan.md](spiderman2/plan.md), with technical maps for [Spidy](spiderman2/spidy-core-map.md) and [addresses](spiderman2/address-map.md). This page is the short version.


**1.0 scope (2026-10-26, widened 2026-10-31, reworked and approved 2026-11-01, [SPEC.md](../SPEC.md)):** "Plug in and play". Steam is the launcher: Play a game as usual. With the headset on the cable the game plays in 3D; without it the same Play button plays flat. Five Favorites, in order: STAR WARS Zero Company (UEVR, our own hook as fallback), ACE COMBAT 7 (UEVR), Control and Planet of Lana (Geo-11 and the SBS viewer), then Spider-Man 2 (Step 0 on Remastered, port Steps 1-8). Out of 1.0: the big-bucket tests (except UEVR on Zero Company), the generic 3D-screen pieces (unless Zero Company needs the fixed-screen layer), other games and Steam Frame (#8, #9, #11, #12, #13). Code map: [code-map.md](code-map.md).

## The 1.0 play round trip

1. **Plug in.** Z13 in Game Mode; Quest 2 on the USB-C cable, headset on, seated boundary. First time only: allow USB debugging. A user service (`flat3d-link`) sees the cable: it brings up the tunnel (internet from the PC over USB, the Quest Tether link ported to Rust), WiVRn and the headset app, and a toast says 3D is ready (M10, M25).
2. **Press Play** on a game with the `3D Ready` collection badge. The launch wrapper `flat3d-run` sees the link is ready and the game's switch is on, applies the game's route and starts it with the OpenXR environment. With no headset, or the switch off, it starts the game unchanged (M9, M30).
3. **Play** on the giant fixed screen with depth, gamepad in hand (M19). Head movement looks around the room, not the game. Pause goes flat.
4. **Unplug** any time: the service keeps the link for 2 minutes in case of a replug, then stops it and says "restart the game for flat" (M18). A headset plugged in while a game is already running flat gets a toast to restart it in 3D.
5. **Watch** the headset from Quick Access: battery, temperature (toast at 38 C and 42 C), throughput, uptime, reconnects (M26, M27).

Today the same chain is done by hand in Desktop Mode ([STEP0-RUNSHEET.md](STEP0-RUNSHEET.md)).

**Order (ruling 2026-10-31):** Zero Company -> ACE COMBAT 7 (same UEVR route) -> Geo-11 and the SBS viewer -> Control and Planet of Lana -> Spider-Man 2.

## The app: one engine, two faces (rulings 2026-10-31, 2026-11-01)

"Desktop app + Decky plugin (Recommended)", built in Rust ("Native Rust GUI", "iced (Recommended)"; "Rust backend, thin TS panel (Recommended)", 2026-11-01). The Decky panel (a thin TypeScript layer over the engine binary) runs in Game Mode (link, health, switches, 3D Ready games); the iced app runs in Desktop Mode (Setup, Headset, Games, Logs). Both read one `snapshot` from the Rust engine in `crates/engine/`, whose `flat3d-link` service owns the headset link; neither face has an Inject button. Shape and interlinks: [code-map.md](code-map.md).

## Where it stands

The 1.0 path is the five Favorites in SPEC.md. The Rust engine, shared API, link state machine, route planning, app shell and core CI are in place; implementation proceeds through the roadmap packages and device sessions. Spider-Man 2 Step 1 is done except for the full-hash manifest; see [the workbench](workbench.md). Device results belong in `sessions/*.toml`. Run `flat3d-inject doctor` for a local readiness report.

## Development stages

The game tracks below need the owner's devices; the engine, app and packaging stages can be built by satellites now and run alongside them. `needs-owner-device` issues wait for a session with the headset.

| Stage | What | Done | Open |
|---|---|---|---|
| E1 Engine core | Steam games, recipes, the shared JSON shape (`api.rs`), `--json` CLI | **Done:** #24, #58-#63 | parents #25, #50 (closable) |
| E2 Link (WiVRn half) | WiVRn without its window, `adb reverse`, the app on the Quest; Stop 3D | **Done:** #64, #65, #66 | parents #26, #39 (closable) |
| E3 Injectors | the routes: `uevr.rs`, `geo11.rs`, `port.rs`; each returns the environment for the wrapper; dispatch and `inject <appid>` | #83 (UEVR profile text), #51's fake-prefix half (`uevr::apply` with undo, PR #196) | #27 dispatch + port (blocked by #120), #51 device half (fetch UEVR and .NET, the real env; waits on Z2 #32), #52 Geo-11 (blocked by #43 and #44, both owner-device) |
| E4 Undo and launch options | `route_of`, the `flat3d-run` word in launch options, the undo record and its steps, `undo <appid>` | **Done:** #79-#82, #84 | |
| E5 Link service | `flat3d-inject watch`: track-devices, plug/unplug/grace state machine, state file, event ring, run records, settings, link command parsing | #85-#94, #119 (`watch::run`) | #120 link commands (ready: makes `watch` a command) |
| E6 Tether | gnirehtet reverse tether ported from Quest Tether: one adb, tun0 honesty, Wi-Fi-first disconnect, NetworkManager sync, heal ladder, step-aside | #95-#104, #124 | #125 health loop (ready) |
| E7 Health and stats | battery and temperature zones, toast gate, throughput, soak summary | **Done:** #105-#108 | (run by #119, #125) |
| E8 Wrapper | `flat3d-run`: plan, exec spec, run | **Done:** #109, #110, #126 | |
| E9 Face data | 3D Ready collection diff, new toasts, health/speed/uptime wording, the view commands | #113, #114, #115 | #121 view commands (blocked by #76 and #120) |
| E10 Face rules | status labels and colours, link wording, game rows, headset buttons and logs | #74, #75, #77 | #76 `rows` (ready) |
| I Install | user unit text, install/uninstall plans, Quest app check | #111, #112 | wiring is #128 (blocked by #120) |
| A App (iced) | crate and CI, the four pages' state, views, main wiring | #78's crate, tabs and CI job (PR #197) | #78 stays open for opening the window on the owner's device and the CI job's first run on the runner; #116 and #118 (ready), #117 (by #76), #122 (by #116-#118, #121, #76), #123 (by #122) |
| D Decky (thin TS) | the engine binary as the backend, the panel, launch-option sync, 3D Ready collection, toasts | | #130 (blocked by #121; owner device for the no-Python launcher question), #131 and #133 (by #130), #132 (by #130 and #121's `launch`) |
| P Packaging | `build-tools.sh`, install wiring, AppImage, Decky zip, all from one tagged commit | | #127 (ready, owner device; its body has drifted from the code map's ruling 34 on where adb comes from and where the tools go), #128 (by #120), #129 (by #123, #127, #128), #134 (by #131) |
| G Games | Zero Company -> ACE COMBAT 7 -> Planet of Lana + Control -> Spider-Man 2 (tracks below) | | #31-#49, #5, #7, #18-#23, #29 |
| X Design pass (last, R04) | the shared design system over the running app and the Decky panel; `design-pass` issues | | after the rest |

**Critical path to a first end-to-end try (Zero Company, by hand):** #120 -> #27 -> a Zero Company recipe -> the owner's headset session (Z1, Z2, then #51's device half). #119, #66 and #51's file half are merged. The faces (A, D) and packaging (P) are not needed for that try: `flat3d-inject watch` run by hand (or the user unit, #128) plus `flat3d-run %command%` in the game's launch options is enough.

## ACE COMBAT 7 track (added 2026-10-31)

ACE COMBAT 7 (Steam 502500): Unreal 4, DX11, UEVR with the Nexus "UEVR Compatibility Mod" profile. Not installed today (its library drive isn't mounted). Spec M20.

| Step | What | Depends on |
|---|---|---|
| A1 (#40) | Install it, check it runs under Proton offline | the drive |
| A2 (#41) | UEVR on AC7 under Proton through the Zero Company route; disable decoupled pitch (known AC7 fix) | Z2, Z3 |
| A3 (#42) | Full sortie in the headset at 30+ fps on the gamepad | A2 |
| A4 (#47) | Recipe at works-now | A3, #25 |

## Geo-11 track: Control and Planet of Lana (added 2026-10-31)

Geo-11 (free DX11 stereo driver, says Proton works) writes the two views side by side; an SBS screen viewer puts them on the fixed screen. Spec M21-M23. Peers: Blackskydk/FlatToDepth, artumino/VRScreenCap (MIT).

| Step | What | Depends on |
|---|---|---|
| G1 (#43) | Geo-11 under Proton: Planet of Lana's fix on the flat screen, check the SBS picture has depth | - |
| G2 (#44) | SBS screen viewer: Geo-11's picture on the fixed screen in the headset (#9's viewer, in 1.0) | G1 |
| G3 (#45, recipe #48) | Planet of Lana in the headset at 30+ fps, recipe at works-now | G2, #25 |
| G4 (#46, recipe #49) | Control: `Control_DX11.exe` with the community 3D fix, in the headset at 30+ fps, recipe at works-now | G2, #25 |

## Zero Company track (first in 1.0, added 2026-10-31)

STAR WARS Zero Company (Steam 2075800): Unreal 5.6.1, DX12 only (vkd3d-proton), ProtonDB Platinum, no anti-cheat found, ships `SWZeroCompany.pdb`. A turn-based tactics game with an overhead camera, so stereo gives a tabletop look and 30 fps is plenty. Research: [games_group_a.md](notes/research/games_group_a.md#star-wars-zero-company). Spec Musts M13-M17.

| Step | What | Depends on | Must |
|---|---|---|---|
| Z1 (#31) | Offline check: link the EA account once, then play a mission with the network off | owner's Z13 | M13 |
| Z2 (#32) | UEVR nightly under Proton: inject, Native Stereo then Synchronized Sequential; record what UE 5.6 does. Kaon (LorenDB/kaon) is the reference for installing UEVR into a Proton prefix | Z1 | M14 |
| Z3 (#33) | 3D TV: test UEVR's 2D Screen Mode for per-eye depth; if it's flat, build the fixed-screen OpenXR layer (#9) | Z2 | M15 |
| Z2b (#34) | Fallback only if Z2 fails: our own UE 5.6 stereo hook from the PDB (lock the pose, fixed eye offset, two views) | Z2 failed | M14 |
| Z4 (#35) | Full mission in the headset at 30+ fps over WiVRn wired | Z3 | M16 |
| Z5 (#36) | Zero Company recipe (`works-now`, UEVR route and settings) and `flat3d-inject inject 2075800` | Z4, #25 | M17 |

Spider-Man 2 note (2026-10-31): HEROVR (lecutis7of9/HEROVR, [PEERS.md](../PEERS.md)) already shows Spider-Man 2 in stereo on Windows. Ruling: learn only, keep our port; its hook list is a map to check our Ghidra results against, never a source of addresses (#37). SpidyVR 0.7.0's Proton + WiVRn path feeds Step 0 (#38).

Zero Company proves the headset chain, the link service and the `flat3d-run` wrapper on a real game while the Spider-Man 2 address mapping goes on.

## What's next

1. Continue the engine, app and docs work in the order recorded in [`docs/plans/roadmap-v2/03-wp-index.md`](plans/roadmap-v2/03-wp-index.md), using [`progress.md`](plans/roadmap-v2/progress.md) for the integration run.
2. Run the Zero Company, ACE COMBAT 7 and Geo-11 tracks in the headset; record device results in `sessions/*.toml`.
3. Continue Spider-Man 2's address mapping and port steps in [the plan](spiderman2/plan.md). Reverse-engineering setup and the command live in [the workbench](workbench.md).

## Decisions for you

1. **When to run the analysis.** Overnight is the obvious slot. Just say "run the Ghidra analysis tonight" and it starts with the memory watch on.
2. **REA.** Installed but not registered. Registering it (`rea setup`) changes Claude Code's config permanently, so it waits for your yes. Without it, I script Ghidra headless directly, which covers everything above.
3. Still open from the plan: contacting Spidy's author.
4. **Issue clean-up:** close #146 (done on main by #174) and the parents #25, #50, #26 and #39 (every child merged). #51 and #78 stay open for their device halves. The 8 merged PRs carry no `by:` / `model:` labels (the guard leaves labels to the coordinator; `by:strix-halo` and `model:claude-sonnet-5-5` don't exist yet).
5. **Drives.** ACE COMBAT 7 (A1, #40) and Spider-Man Remastered (Step 0, #5) need their library drive mounted or a fresh install.
