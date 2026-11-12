# Roadmap: six Favorites in 3D

Where the project stands and what comes next. This is the plan. The SM2 track is [spiderman2/plan.md](spiderman2/plan.md), with technical maps for [Spidy](spiderman2/spidy-core-map.md) and [addresses](spiderman2/address-map.md). This page is the short version.


**1.0 scope (2026-10-26, widened 2026-10-31, reworked and approved 2026-11-01, games list changed and re-approved 2026-11-11, [SPEC.md](../SPEC.md)):** "Plug in and play". Steam is the launcher: Play a game as usual. With the headset on the cable the game plays in 3D; without it the same Play button plays flat. Six Favorites (the owner, 2026-11-11: "This is the 1.0 list.", ruling 43; Forza Horizon 6 out by ruling 45), in order by readiness (ruling 46): STAR WARS Zero Company (UEVR, our own hook as fallback), ACE COMBAT 7 (UEVR), Control (Geo-11 and the SBS viewer), 007 First Light (its dormant stereo mode through our proxy DLL, into the SBS viewer), Spider-Man 2 (Step 0 on Remastered, port Steps 1-8), Alan Wake 2 (our own alternate-eye hook, into the SBS viewer). Out of 1.0: Planet of Lana and Forza Horizon 6 (both moved out 2026-11-11), the big-bucket tests (except UEVR on Zero Company), the generic 3D-screen pieces (unless Zero Company needs the fixed-screen layer), other games and Steam Frame (#8, #9, #12, #13; #11 except its 007 First Light and Alan Wake 2 parts). Code map: [code-map.md](code-map.md).

## The 1.0 play round trip

1. **Plug in.** Z13 in Game Mode; Quest 2 on the USB-C cable, headset on, seated boundary. First time only: allow USB debugging. A user service (`flat3d-link`) sees the cable: it brings up WiVRn and the headset app, and a toast says 3D is ready (M10). The headset's internet over the cable is Quest Tether's job, a separate app (ruling 42, 2026-11-10).
2. **Press Play** on a game with the `3D Ready` collection badge. The launch wrapper `flat3d-run` sees the link is ready and the game's switch is on, applies the game's route and starts it with the OpenXR environment. With no headset, or the switch off, it starts the game unchanged (M9, M30).
3. **Play** on the giant fixed screen with depth, gamepad in hand (M19). Head movement looks around the room, not the game. Pause goes flat.
4. **Unplug** any time: the service keeps the link for 2 minutes in case of a replug, then stops it and says "restart the game for flat" (M18). A headset plugged in while a game is already running flat gets a toast to restart it in 3D.
5. **Watch** the headset from Quick Access: battery and temperature (toast at 38 C and 42 C) (M26); when Quest Tether is installed, its state as a read-only line (M29).

Today the same chain is done by hand in Desktop Mode ([STEP0-RUNSHEET.md](STEP0-RUNSHEET.md)).

**Order (ruling 46, 2026-11-11, "By readiness (Recommended)"):** Zero Company -> ACE COMBAT 7 (same UEVR route) -> Geo-11 and the SBS viewer -> Control -> 007 First Light -> Spider-Man 2 -> Alan Wake 2.

## The app: one engine, two faces (rulings 2026-10-31, 2026-11-01)

"Desktop app + Decky plugin (Recommended)", built in Rust ("Native Rust GUI", "iced (Recommended)"; "Rust backend, thin TS panel (Recommended)", 2026-11-01). The Decky panel (a thin TypeScript layer over the engine binary) runs in Game Mode (link, health, switches, 3D Ready games); the iced app runs in Desktop Mode (Setup, Headset, Games, Logs). Both read one `snapshot` from the Rust engine in `crates/engine/`, whose `flat3d-link` service owns the headset link; neither face has an Inject button. Shape and interlinks: [code-map.md](code-map.md).

## Where it stands

**Updated 2026-11-12** (rulings 49-52; `main` at `d257671`).

The 1.0 path is the six Favorites in SPEC.md, in order by readiness (rulings 43-47). Each game track now starts its hooking with a hook-point research step on the installed game (ruling 52, [below](#hook-point-research-ruling-52)). After 1.0 (kept, not sized): #4, #8, #9, #11, #12, #13, #14, and Planet of Lana's #45 and #48 (they still need the `after-1.0` label); #11 names 007 First Light and Alan Wake 2, now 1.0 games, so it needs re-cutting; Forza Horizon 6 is after 1.0 too (ruling 45) and has no issue.

**Roadmap v2 on main (`d257671`, 2026-11-12):** 35 of the 44 work packages are merged, and WP-36 has its first PR merged (#284, which removed the tether from the link service); the per-WP table with PR numbers is [`progress.md`](plans/roadmap-v2/progress.md). Built and tested on the host: the engine's commands (`list`, `status`, `inject`, `undo`, the link commands and the view commands), the link service, the UEVR and Geo-11 route files with undo, pinned fetches, the Windows UEVR injector, the SBS viewer's code, the app's four pages with their views and main loop, the Decky panel, the packaging scripts and the device session runner. Not merged: the rest of WP-36 and WP-37 (remove the tether, ruling 42; what #284 left, gnirehtet in `install.rs` and `cli/link.rs`, is listed for #226), WP-31 (the wrapper applies the route: today `flat3d-run` always plays flat, because its route step answers `not-built`), WP-09, WP-29 (`doctor` and `setup` answer `not-built`), WP-30, WP-38, WP-34, and WP-39 (after 1.0). The `port` route is not built (#27b). Every merged WP with a device half says `Refs`, so its issue stays open until its Q row in [the owner-device queue](plans/roadmap-v2/02-queues-and-decisions.md) passes. CI runs on GitHub-hosted `ubuntu-24.04` runners (ruling 48, #270). Spider-Man 2 Step 1 is done except for the full-hash manifest; see [the workbench](workbench.md). Device sessions are defined in `sessions/*.toml` and run with `tools/session/session.py`; `flat3d-inject doctor`, the local readiness report, comes with WP-29.

## Development stages

The game tracks below need the owner's devices; the engine, app and packaging stages can be built by satellites now and run alongside them. `needs-owner-device` issues wait for a session with the headset.

| Stage | What | Done | Open |
|---|---|---|---|
| E1 Engine core | Steam games, recipes, the shared JSON shape (`api.rs`), `--json` CLI, the catalog join and `list` / `status` | **Done:** #24, #58-#63; WP-00 fmt and clippy gate (#201, closes #183); WP-01a-d workspace reshape and frozen bones (#202, #204, #206, #207); WP-07 `list` and `status` (#245) | parents #25, #50 (closable) |
| E2 Link (WiVRn half) | WiVRn without its window, `adb reverse`, the app on the Quest; Stop 3D | **Done:** #64, #65, #66; WP-12 Stop 3D twice is safe, WiVRn server and Quest app detection (#246) | Q3 link smoke on the device. WP-35 moved the adb choice to `link/adb.rs` (#279) |
| E3 Injectors | the routes: `uevr.rs`, `geo11.rs`, `port.rs`; each returns the environment for the wrapper; dispatch and `inject <appid>` | #83, #51's fake-prefix half (#196); WP-13 pinned fetch (#215); WP-14 UEVR route files (#217); WP-15 `flat3d-uevr-inject.exe` and `tools/build-win.sh` (#236); WP-16 Geo-11 route, pure half (#250); WP-27 `inject`, `ensure_applied`, `undo` and the `setup fetch` function (#268) | #27b the `port` route (`port.rs` is empty); WP-31 makes the wrapper apply the route; device halves #51 (Q2, Q6) and #52 (Q9). Follow-ups from the merged verdicts: **WP-16:** `geo11::plan_files` searches the fetch cache up to 3 levels deep, so after a pin bump it can pick a stale version folder or a leftover `.tmp` folder; fix in `geo11.rs` before the `geo11` pin is filled and before Q9. **WP-14:** the `uevr-profile` pin install is merged (#278, closes #275) |
| E4 Undo and launch options | `route_of`, the `flat3d-run` word in launch options, the undo record and its steps, `undo <appid>` | **Done:** #79-#82, #84; WP-20 `add_wrapper_at` (#219); WP-27 `save_record`, `put_back` and the `undo` command (#268) | Follow-up from WP-27's verdict: undo leaves empty the folders the apply step made (the UEVR profile folder, Geo-11 subfolders), because the record does not list them; the record format is locked, so the fix needs a record-format change |
| E5 Link service | `flat3d-inject watch`: track-devices, plug/unplug/grace state machine, state file, event ring, run records, settings, link command parsing | #85-#94, #119 (`watch::run`); WP-08 link commands wired (#233, refs #120) | WP-09 watch service correctness, the 3D half (after WP-36); Q3 |
| E6 Tether | **To be removed (ruling 42, 2026-11-10).** Was: gnirehtet reverse tether ported from Quest Tether. Only the adb choice stays (WP-35, `link/adb.rs`). The closed issues are history | #95-#104, #124 (history); WP-10 tether plans and runner errors (#253, superseded by ruling 42); WP-35 (#279); WP-36's first PR (#284, closes #225) | the rest of WP-36, then WP-37 remove it; #125's tunnel, heal, throughput and soak parts are not planned |
| E7 Health and stats | battery and temperature zones, toast gate (kept); throughput and soak summary (to be removed with the tunnel, ruling 42) | **Done:** #105-#108; WP-11 health loop (#248) | #284 (WP-36) cut the loop back to the battery poll and toasts and deleted the stats module. Follow-up from WP-11's verdict: after a backward clock step the locked `Healer` holds heals off until the clock passes its last heal; the heal ladder leaves with WP-36, so this lapses there |
| E8 Wrapper | `flat3d-run`: plan, exec spec, run | **Done:** #109, #110, #126 | WP-31 the route step (today it answers `not-built`, so every launch plays flat) |
| E9 Face data | 3D Ready collection diff, new toasts, health/speed/uptime wording, the view commands | #113, #114, #115; WP-28 view commands (#267, closes #121, #75) | the tunnel and speed wording leave with WP-36 |
| E10 Face rules | status labels and colours, link wording, game rows, headset buttons and logs | **Done:** #74, #75, #77; WP-03 `rows` (#243, closes #76); WP-06 log lines (#247) | |
| I Install | user unit text, install/uninstall plans and their execution, Quest app check | #111, #112; WP-20 install execution (#219, refs #128) | WP-37 its own udev rule and no gnirehtet; WP-29 `doctor` and `setup`; #128's device half (Q3) |
| A App (iced) | crate and CI, the four pages' state, views, main wiring | #78 (PR #197); WP-04 Headset page (#249, closes #116); WP-05 Setup page (#244, closes #118); WP-23 Games page (#230, closes #117); WP-06 Logs page (#247); WP-33 views and main wiring (#272, refs #122, #123) | Q15 the window on the owner's device (#78, #122, #123); WP-37 removes the Tunnel switch and Setup step; WP-38 the Quest Tether line |
| D Decky (thin TS) | the engine binary as the backend, the panel, launch-option sync, 3D Ready collection, toasts | WP-21 scaffold (#221, refs #130); WP-32 panel, launch sync, collection, toasts (#251, refs #131-#133) | Q14 on the device (#130-#134); WP-37 removes the Tunnel toggle; WP-38 |
| P Packaging | pins, `build-tools.sh`, AppImage, Decky zip, all from one tagged commit; CI | WP-02 `tools/check.sh`, `packaging/pins` and `pin.sh` (#203); WP-22 packaging scripts (#222, refs #127, #129, #134); CI on GitHub-hosted runners (#270, ruling 48) | Q0 fills the `TBD-at-pin` pins (among them the `geo11` sha); Q1 tools build (#127); Q16 AppImage (#129); WP-37 drops gnirehtet |
| T Tooling and docs | docs check, docs scrub, device session runner, reverse-engineering tools | WP-25a docs check (#205); WP-25b docs scrub (#252); WP-24 session runner, stereo gates, `sessions/*.toml` (#231); docs-check fixes #254, #273 | WP-30 RE tooling and the address map (#19, #37; Q12); WP-34 the docs sync after the pass |
| G Games | Zero Company -> ACE COMBAT 7 -> Control -> 007 First Light -> Spider-Man 2 -> Alan Wake 2 (tracks below) | WP-26 recipe seeds (#237, test fixtures only; `recipes/` has no `.recipe` yet); the SBS viewer's code: WP-17 (#218), WP-18 (#220), WP-19 (#234); research notes #260 (Alan Wake 2), #261 (Forza Horizon 6, after 1.0), #262 (ACE COMBAT 7), #263 (Control), #264 (007 First Light), #265 (Spider-Man 2), #266 (Zero Company) | #31-#44, #46, #47, #49, #5, #7, #18-#23, #29; 007 First Light #288, #290, #291, #294-#298 and Alan Wake 2 #289, #292, #293, #299-#303 (2026-11-12; #11 stays the 007 fallback); hook-point research #285-#289 and #283 (ruling 52); #45 and #48 (Planet of Lana) are `after-1.0` work |
| X Design pass (last, R04) | the shared design system over the running app and the Decky panel; `design-pass` issues | | after the rest |

**Critical path to a first end-to-end try (Zero Company, by hand):** WP-31 (the wrapper applies the route) -> a Zero Company recipe in `recipes/` -> the owner's sessions: Q2 (build and install the injector exe), Q5 (Z1 offline), then Q6 (Z2, UEVR live, #51's device half). The link commands (#120, WP-08), `inject` (#27a, WP-27), the UEVR route files (WP-14) and the injector exe's code (WP-15) are merged. The faces (A, D) and packaging (P) are not needed for that try: `flat3d-inject watch` run by hand (or `flat3d-inject install`) plus `flat3d-run %command%` in the game's launch options is enough.

## ACE COMBAT 7 track (added 2026-10-31)

ACE COMBAT 7 (Steam 502500): Unreal 4, DX11, UEVR with the Nexus "UEVR Compatibility Mod" profile. Not installed today (its library drive isn't mounted). Spec M20.

| Step | What | Depends on |
|---|---|---|
| A1 (#40) | Install it, check it runs under Proton offline | the drive |
| AR (#286) | Hook-point research on the installed game (ruling 52): the fake stereo device's view-offset and projection functions and the `Nimbus` classes the UEVR plugin hooks, by pattern (no PDB), into `docs/hooks/502500-ace-combat-7.md` | the install (ruling 51) |
| A2 (#41) | UEVR on AC7 under Proton through the Zero Company route; disable decoupled pitch (known AC7 fix) | Z2, Z3, AR |
| A3 (#42) | Full sortie in the headset at 30+ fps on the gamepad | A2 |
| A4 (#47) | Recipe at works-now | A3, #25 |

## Geo-11 track: Control (added 2026-10-31, Planet of Lana moved out 2026-11-11)

Geo-11 (free DX11 stereo driver, says Proton works) writes the two views side by side; an SBS screen viewer puts them on the fixed screen. Spec M21, M22.

Control is the one Geo-11 game left in 1.0 (ruling 43), so it takes G1, the first Geo-11 test, which was planned on Planet of Lana. What that changes: G1 now needs Control's community fix and `Control_DX11.exe`, and G4 no longer has a simpler game before it. The issue titles, the session files (`sessions/g1-geo11.toml`, `g2-viewer.toml`, `g3-lana.toml`), the `lana-fix` pin and the Planet of Lana recipe seed still name Planet of Lana; they are changed in follow-up work, not here. The SBS viewer (G2) stays in 1.0: besides Control it is planned to show 007 First Light's and Alan Wake 2's side-by-side picture. Peers: Blackskydk/FlatToDepth, artumino/VRScreenCap (MIT).

| Step | What | Depends on |
|---|---|---|
| GR (#287) | Hook-point research on the installed game (ruling 52): the D3D11 entry Geo-11's proxy takes and which of the fix's shaders the current build still loads, into `docs/hooks/870780-control.md` | the install (ruling 51) |
| G1 (#43) | Geo-11 under Proton: Control (`Control_DX11.exe`) with the community 3D fix on the flat screen, check the SBS picture has depth | GR |
| G2 (#44) | SBS screen viewer: Geo-11's picture on the fixed screen in the headset (#9's viewer, in 1.0) | G1 |
| G4 (#46, recipe #49) | Control: `Control_DX11.exe` with the community 3D fix, in the headset at 30+ fps, recipe at works-now | G1, G2, #25 |

Moved out of 1.0 on 2026-11-11 (`after-1.0`): G3 (#45, recipe #48), Planet of Lana in the headset at 30+ fps with its recipe at works-now (was Spec M23). Its research ([games_group_a.md](notes/research/games_group_a.md#planet-of-lana)) and its recipe seed stay.

## New 1.0 games: 007 First Light and Alan Wake 2 (added 2026-11-11)

Ruling 43 added three games; ruling 45 moved Forza Horizon 6 to `after-1.0` the same day (ruling 7's "not safe" stands; its research note is kept). The two that stay have a planned route each from their research notes (one note per game under `docs/notes/research/games/`). Both routes are untested: nothing has run on the device. Their track issues were filed on 2026-11-12 from the notes' proposed issues (tables below).

| Game | Spec | Planned route (untested) | Fallback | Open |
|---|---|---|---|---|
| 007 First Light (Steam 3768760, Glacier, DX12), fourth in the order | M31 | a `dinput8.dll` proxy of our own switches the engine's dormant VR device to its `stereo3d` mode; the side-by-side picture goes to the SBS viewer (G2) | our own two-pass or alternate-eye DX12 stereo (#11) | whether the dormant mode still draws two real views |
| Alan Wake 2 (Epic only, through Heroic; no Steam appid; Northlight, DX12), sixth in the order | M32 | our own alternate-eye hook on the engine's camera class, loaded by a proxy DLL, into the SBS viewer (G2). Alternate-eye counts here because no workable two-real-views route is known (a two-pass render would double a GPU load the note already puts at its limit); the bar is 30+ fps per eye (ruling 44) | the NoMoreFlat OpenXR mod (also alternate-eye), only if its free build is published; it was not public on 2026-11-11 | recipes and the launch wrapper assume a Steam appid, so a non-Steam game needs a design of its own |

Both depend on G2 (#44), the SBS viewer, which Control needs first.

**007 First Light track** ([research note](notes/research/games/3768760-007-first-light.md)):

| Step | What | Depends on |
|---|---|---|
| F1 (#290) | Offline play, and the dummy VR device's log lines under Proton (note tests 1-3, no code) | - |
| FR (#288) | Hook-point research on the installed game (ruling 52): map `ZRenderVRDeviceDummy` (vtable, activate and set-mode methods, active flag, mode and IPD) and the loader hooks, into `docs/hooks/3768760-007-first-light.md` | the install (ruling 51) |
| F2 (#291) | Build ZKntSDK from source and load it under Proton | - |
| F3 (#294) | Probe mod: switch the device to `stereo3d`, report what is drawn | FR, F2 |
| F4 (#295) | Side-by-side output from the two eye images | F3 |
| F5 (#296) | Game logic flat and head fixed while the render device is in stereo | F3 |
| F6 (#297) | A mission in the headset at 30+ fps through the SBS viewer | F4, F5, G2 |
| F7 (#298) | A `glacier` route in the engine; recipe at works-now | F6 |

Fallback if F3 finds no two-eye picture: #11 (two-pass DX12 stereo).

**Alan Wake 2 track** ([research note](notes/research/games/epic-alan-wake-2.md)):

| Step | What | Depends on |
|---|---|---|
| W1 (#292) | Decision: how a non-Steam (Heroic) game gets a recipe key and the launch wrapper (Decisions for you, item 6) | the owner |
| W2 (#293) | The launch path on the device, and a do-nothing proxy `version.dll` load check | - |
| WR (#289) | Hook-point research on the installed game (ruling 52): `rend::View`'s setters and `renderWorldInternal` from Control's exports and the lambda RTTI names, the present hook below Streamline, one RenderDoc capture of the view constants, into `docs/hooks/epic-alan-wake-2.md` | the install (ruling 51) |
| W3 (#299) | One-eye camera shift through the `rend::View` hook | WR, W2 |
| W4 (#300) | Alternate-eye render with a per-eye previous view, side-by-side out | W3 |
| W5 (#301) | HUD and video flat at screen depth | W4 |
| W6 (#302) | Apply and undo: `renderer.ini` and the proxy DLL | W1, W2 |
| W7 (#303) | A chapter section in the headset at 30+ fps per eye; recipe at works-now | W4, W5, W6, G2 |

## Hook-point research (ruling 52)

The owner, 2026-11-12: "Hook points should be steps in the build where research is done on the installed game to figure out how to hook it." Asked when, he picked "Per game, before its port (Recommended)". So every game track has a research step on the installed game's binary that finds its hook points before any step that writes code against them. The games and the reverse-engineering tools are installed on the Windows desktop with the RTX 3090 Ti (ruling 51). SpidyVR's and peers' addresses (HEROVR, the UEVR fork's table, ZKntSDK, the research notes' own RVAs) are a checklist to verify, not a source (as R41 and #283 already asked).

**What each step outputs:** an address or pattern map in `docs/`. Its header gives the exe's file name, SHA-256, file version and store build (Steam build id, or the Epic version). One row per hook: what it is and which step uses it, its address (RVA), a unique byte pattern, how it was found (tool, method, date: PDB symbol, RTTI name, string reference, Ghidra Version Tracking, the xmatch matcher, a RenderDoc capture), and its status (`verified` or `lead`); a checklist column marks each peer or note address `agrees` or `differs`. No code uses a `lead`.

| Game | Step | Map | Before |
|---|---|---|---|
| STAR WARS Zero Company | ZR (#285) | `docs/hooks/2075800-star-wars-zero-company.md` | Z2, Z2b |
| ACE COMBAT 7 | AR (#286) | `docs/hooks/502500-ace-combat-7.md` | A2 |
| Control | GR (#287) | `docs/hooks/870780-control.md` (Geo-11's D3D11 entry and the fix's shaders) | G1 |
| 007 First Light | FR (#288) | `docs/hooks/3768760-007-first-light.md` | F3 |
| Spider-Man 2 | Step 2r (#283) | [spiderman2/address-map.md](spiderman2/address-map.md) | Step 3 ([plan](spiderman2/plan.md)) |
| Alan Wake 2 | WR (#289) | `docs/hooks/epic-alan-wake-2.md` | W3 |

## Zero Company track (first in 1.0, added 2026-10-31)

STAR WARS Zero Company (Steam 2075800): Unreal 5.6.1, DX12 only (vkd3d-proton), ProtonDB Platinum, no anti-cheat found, ships `SWZeroCompany.pdb`. A turn-based tactics game with an overhead camera, so stereo gives a tabletop look and 30 fps is plenty. Research: [games_group_a.md](notes/research/games_group_a.md#star-wars-zero-company). Spec Musts M13-M17.

| Step | What | Depends on | Must |
|---|---|---|---|
| Z1 (#31) | Offline check: link the EA account once, then play a mission with the network off | owner's Z13 | M13 |
| ZR (#285) | Hook-point research on the installed game (ruling 52): the fake stereo device's vtable and slots, `InitializeHMDDevice`, the HUD draw and the game build the UEVR fork checks, from the shipped PDB, into `docs/hooks/2075800-star-wars-zero-company.md` | the install (ruling 51) | M14 |
| Z2 (#32) | UEVR nightly under Proton: inject, Native Stereo then Synchronized Sequential; record what UE 5.6 does. Kaon (LorenDB/kaon) is the reference for installing UEVR into a Proton prefix | Z1, ZR | M14 |
| Z3 (#33) | 3D TV: test UEVR's 2D Screen Mode for per-eye depth; if it's flat, build the fixed-screen OpenXR layer (#9) | Z2 | M15 |
| Z2b (#34) | Fallback only if Z2 fails: our own UE 5.6 stereo hook from the PDB (lock the pose, fixed eye offset, two views), at the ZR map's addresses | Z2 failed, ZR | M14 |
| Z4 (#35) | Full mission in the headset at 30+ fps over WiVRn wired | Z3 | M16 |
| Z5 (#36) | Zero Company recipe (`works-now`, UEVR route and settings) and `flat3d-inject inject 2075800` | Z4, #25 | M17 |

Spider-Man 2 note (2026-10-31): HEROVR (lecutis7of9/HEROVR, [PEERS.md](../PEERS.md)) already shows Spider-Man 2 in stereo on Windows. Ruling: learn only, keep our port; its hook list is checked in [the address map](spiderman2/address-map.md#checked-against-herovrs-list) against our Ghidra results (#37), its clean-eye findings feed Step 5 (#21, [sm2-clean-eyes.md](notes/sm2-clean-eyes.md)), never as a source of addresses. SpidyVR 0.7.0's Proton + WiVRn path feeds Step 0 (#38).

Zero Company proves the headset chain, the link service and the `flat3d-run` wrapper on a real game while the Spider-Man 2 address mapping goes on.

## What's next

1. **Work packages, in dependency order** (the rules are in [`03-wp-index.md`](plans/roadmap-v2/03-wp-index.md); status per WP in [`progress.md`](plans/roadmap-v2/progress.md)). WP-35 is merged (#279) and WP-36 has its first PR merged (#284). Unblocked now: the rest of WP-36, then WP-37 (remove gnirehtet from the faces, Setup, install and packaging; add the app's own udev rule; two PRs); WP-31 (the wrapper applies the route; needs no tether WP); WP-30 (RE tooling and the address map). The wave order puts WP-36 and WP-37 (wave 2b) before WP-30 (wave 3) and WP-31 (wave 4); their dependencies alone would let them start now. After WP-36: WP-09 (watch service, the 3D half). After WP-37: WP-29 (`doctor` and `setup`) and WP-38 (the Quest Tether status line). Last: WP-34 (docs sync after the pass). WP-39 is after 1.0. WP-28 (#267) and WP-33 (#272) merged before WP-36 and WP-37, so the tunnel switch, the speed text and the tether snapshot fields they show go when those two land. The `port` route (#27b) is outside this pass.
2. **Device sessions** (Q rows in [the owner-device queue](plans/roadmap-v2/02-queues-and-decisions.md)): see Decisions for you, item 8.
3. **Hook-point research** (ruling 52): ZR #285, AR #286, GR #287, FR #288, Spider-Man 2 Step 2r #283 and WR #289, each on the installed game before its track's first hook ([above](#hook-point-research-ruling-52)).
4. Continue Spider-Man 2's port steps in [the plan](spiderman2/plan.md), with Step 2r's map before Step 3. Reverse-engineering setup and the command live in [the workbench](workbench.md).

## Decisions for you

1. **When to run the analysis.** Overnight is the obvious slot. Just say "run the Ghidra analysis tonight" and it starts with the memory watch on.
2. **REA.** Installed but not registered. Registering it (`rea setup`) changes Claude Code's config permanently, so it waits for your yes. Without it, I script Ghidra headless directly, which covers everything above.
3. **Spidy's author: closed.** Not contacted (ruling 49, 2026-11-12: "We're not going to hear back from Spidys author."). A licence is not a blocker for using Spidy's code here (ruling 50).
4. **Issue clean-up:** close #146 (done on main by #174) and the parents #25, #50, #26 and #39 (every child merged). #51 and #78 stay open for their device halves. The 8 merged PRs carry no `by:` / `model:` labels (the guard leaves labels to the coordinator; `by:strix-halo` and `model:claude-sonnet-5-5` don't exist yet).
5. **Drives.** ACE COMBAT 7 (A1, #40) and Spider-Man Remastered (Step 0, #5) need their library drive mounted or a fresh install.
6. **Alan Wake 2 is not a Steam game.** The 1.0 list, its order, alternate-eye rendering and the SPEC were ruled and approved on 2026-11-11 (rulings 43-47). Still open: how an Epic game run through Heroic gets a recipe and the `flat3d-run` wrapper (W1, #292).
7. **SPEC re-approval (ruling 42).** The Musts that ruling 42 changed after the 2026-11-11 approval wait for your re-approval: M25 removed; M10, M11, M18, M24, M26, M27, M28 and M29 revised (SPEC.md, the line under the approval).
8. **On-device checks.** Ready now (every WP they need is merged): Q0 (record the `TBD-at-pin` shas with `packaging/pin.sh --all-tbd`; the `geo11` pin has its version, 0.7.12, but its sha is still `TBD-at-pin`, and `control-fix`, `ac7-profile` and `lana-fix` are TBD throughout), Q2 (build and install `flat3d-uevr-inject.exe`; running `tools/build-win.sh --fetch-sdk` is you accepting the SDK licence), Q5 (Zero Company offline, Z1), Q13 (Spider-Man 2 offline, Step 0, Spidy 0.7.0) and Q16 (AppImage). Q1 (tools build) and Q14 (Decky) can run now but change again with WP-37, which drops gnirehtet. Waiting on code: Q3 link smoke (WP-09, WP-36), Q4 doctor (WP-29), Q6 UEVR live, Q9 Geo-11 and Q10 viewer (WP-31; Q9 also needs the `geo11` pin), Q12 RE overnight (WP-30), Q15 app window (WP-29); Q7, Q8 and Q11 follow Q6, Q9 and Q10.
