# Roadmap: five Favorites in 3D

Where the project stands and what comes next. The full plan is [PORT-SPIDERMAN2.md](PORT-SPIDERMAN2.md) and the technical map is [spidy-stereo-core-map.md](spidy-stereo-core-map.md). This page is the short version.

Updated 2026-10-26 (end-of-night sweep; the Step 0-3 notes below are from 2026-10-18).

**1.0 scope (2026-10-26, widened 2026-10-31, [SPEC.md](../SPEC.md)):** STAR WARS Zero Company first (UEVR, our own hook as fallback), then Step 0 on Remastered and port Steps 1-8 on Spider-Man 2, both started from Game Mode with the Inject app's Start 3D and Inject buttons. Out of 1.0: the big-bucket tests (except UEVR on Zero Company), the generic 3D-screen pieces (unless Zero Company needs the fixed-screen layer), other games and Steam Frame (#8, #9, #11, #12, #13). Code map: [code-map.md](code-map.md).

## The 1.0 play round trip

1. **Plug in.** Z13 in Game Mode; Quest 2 on the USB-C cable, headset on, seated boundary. First time only: allow USB debugging.
2. **Start 3D** (Quick Access → Flat to 3D): the engine starts the WiVRn server without its window, runs `adb reverse tcp:9757`, opens the WiVRn app on the Quest; the panel says Ready (M10).
3. **Inject** on a game: the engine applies the recipe (launch options, mod files) and launches the game through Steam. WiVRn must already be up: Proton looks for VR only at launch.
4. **Play** on the giant fixed screen with depth, gamepad in hand (M19). Head movement looks around the room, not the game. Pause goes flat.
5. **Quit** to Game Mode, then **Stop 3D** (M18, #39), unplug.

Today the same chain is done by hand in Desktop Mode ([STEP0-RUNSHEET.md](STEP0-RUNSHEET.md)). Making WiVRn run without its desktop window in Game Mode is part of #26.

**Order (ruling 2026-10-31):** Zero Company → ACE COMBAT 7 (same UEVR route) → Geo-11 and the SBS viewer → Control and Planet of Lana → Spider-Man 2.

## The app: one engine, two faces (ruling 2026-10-31)

"Desktop app + Decky plugin (Recommended)", built with "Tauri (Rust + web UI) (Recommended)". The Tauri app runs in Desktop Mode (library, setup, logs, per-game detail and undo); the Decky panel runs in Game Mode (Start 3D, Stop 3D, Inject). Both drive the Rust engine in `inject/`. Shape and interlinks: [code-map.md](code-map.md).

## Development stages

The game tracks below need the owner's devices; the app stages can be built by satellites now and run alongside them.

| Stage | What | Issues |
|---|---|---|
| E1 Engine core | Steam games, recipes, the shared JSON shape (`api.rs`), `--json` CLI | #24, #25, #50 |
| E2 Link | Start 3D (WiVRn without its window, adb), Stop 3D, status | #26, #39 |
| E3 Injectors | `uevr.rs`, `geo11.rs`, `port.rs`, each with undo; split of #27 | #27 (dispatch, port, undo), #51 UEVR, #52 Geo-11 |
| F1 Decky | backend over `--json`, QAM panel | #28 panel, #53 backend |
| F2 App skeleton | Tauri 2 + React + Vite under `app/`, `ui-shared/`, CI | #54 |
| F3 App pages | Library, Headset (with first-time setup), Game (recipe, undo), Logs | #55, #56 |
| G Games | Zero Company → ACE COMBAT 7 → Planet of Lana + Control → Spider-Man 2 (tracks below) | #31-#49, #5-#29 |
| P Packaging | AppImage and the Decky zip from one tagged commit | #57 |
| D Design pass (last, R04) | the shared design system over the running app and the Decky panel; `design-pass` issues | after the rest |

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
| Z5 (#36) | Zero Company recipe (`works-now`, UEVR route and settings) and `flat3d inject 2075800` | Z4, #25 | M17 |

Spider-Man 2 note (2026-10-31): HEROVR (lecutis7of9/HEROVR, [PEERS.md](../PEERS.md)) already shows Spider-Man 2 in stereo on Windows. Ruling: learn only, keep our port; its hook list is a map to check our Ghidra results against, never a source of addresses (#37). SpidyVR 0.7.0's Proton + WiVRn path feeds Step 0 (#38).

Zero Company proves the headset chain, Start 3D and Inject on a real game while the Spider-Man 2 address mapping goes on.

## Spider-Man 2 track: where we are

| Step | What | State |
|---|---|---|
| 0 | Baseline: unmodified SpidyVR on Remastered, in the Quest 2 (#5) | **Waiting on a headset session.** Remastered (Steam 1817070) is finishing its install (66 GB downloaded). Run sheet: [STEP0-RUNSHEET.md](STEP0-RUNSHEET.md). |
| 1 | Workbench | **Done.** See below. |
| 2 | Get inside Spider-Man 2 (#18, blocked by #7) | Not started. |
| 3 | Move the camera (find the camera functions in SM2) (#19, blocked by #18) | **Started offline.** Tools ready; the Ghidra analysis has not run yet. On the owner's PC the fast matcher (`tools/xmatch/`) is ready and waits on the downloads: [spiderman2-address-map.md](spiderman2-address-map.md). |
| 4–7 | Two eyes (#20), clean images (#21), headset (#22), HUD and memory (#23) | Not started. Each is blocked by the one before it. |
| 8 | The Inject app and Spider-Man 2's recipe | **Skeleton on main, nothing built.** `inject/` (Rust), `decky/` and `recipes/` hold stubs that return "not built yet"; no pull request has merged since 2026-10-19. Issues: #24 find games, #25 load recipes, #26 Start 3D link, #27 inject and commands, #28 Decky panel, #29 the recipe; parent #10. |

### Dev Loop issues (1.0)

| Issue | What | Size | State |
|---|---|---|---|
| #5 | Step 0 on Remastered | Hard | Open, needs the owner's headset |
| #7 | Does Spider-Man 2 play offline? | Question | Open, needs the owner's Z13; blocks #18 |
| #18-#23 | Port Steps 2-7 | Medium, then Hard | Open, chained #18 -> #19 -> #20 -> #21 -> #22 -> #23 |
| #24 | Inject: find Steam games | Easy | Open, locked tests in the issue; free to take |
| #25 | Inject: load and check recipes | Medium | Open; free to take (#15 is answered) |
| #26 | Inject: Start 3D link | Medium | Open; free to take |
| #27 | Inject: inject and commands | Medium | Open; blocked by #24, #25, #26, #18 |
| #28 | Decky plugin | Medium | Open; blocked by #27 |
| #29 | Spider-Man 2 recipe | Easy | Open, locked tests in the issue; blocked by #25, #23 |
| #6, #10 | Parents: the port, the Inject app | Hard | Open |
| #15, #16 | Recipes home; research notes | | Closed (done) |

After 1.0 (kept, not sized): #4, #8, #9, #11, #12, #13, #14.

### Done

- **Repo** `github.com/iannitram/flat-to-3d` (private, GPL-3.0), with SpidyVR v0.3.8 pinned as a submodule.
- **Spidy builds on the Z13.** Cross-compiled in the `craft-build` distrobox with clang-cl and the Windows SDK (via xwin). 337 of 337 tests pass under Proton's Wine.
- **Spidy's discovery tools work on any exe.** I copied four of them into `tools/research/` and removed the Remastered hash and path checks. Ilya Mezerowsky is credited in each file. They now read `Spider-Man2.exe`:
  - `inspect_game.py`: version, hash, sections. It now reads the version without Windows, and it names both known builds.
  - `research_game.py`: disassembly, cross-references, string search.
  - `discover_render_types.py`: C++ type names to vtables.
  - `find_method_owners.py`: which class a function belongs to.
- **Ghidra 12.1.4 and Java 21 (Temurin) installed** under `~/.local/share/flat-to-3d/`, user-space, checksums verified. Ghidra is set to use that Java.
- **REA 6.7.0 installed** under `~/.local/share/flat-to-3d/rea/`, with no install scripts run. **Not registered** in Claude Code, and not yet checked against Ghidra.
- **A copy of the SM2 exe** to analyse, at `~/.local/share/flat-to-3d/exe/sm2-2.810.0.0/`. SHA-256 is `c9b912ef…b738c9`, version 2.810.0.0.
- **`tools/ghidra/headless.sh`** runs Ghidra as a heavy job. It refuses to start while `qwen.lock` exists or another Ghidra run is going, and it stops Ghidra if free memory drops under 2 GiB.

### Good news found today

**Spider-Man 2's C++ type information is not stripped after all.** The plan said it was. The exe holds 27,183 type names, and all 124 camera and render types the tool looked for lead to their vtables. Among them are `Camera2::Camera`, `FollowCamera` and `CombatCamera`, plus `PlayerModelHudFollower`, `HeroLocal` and `TimeScaleSystem`. The strings Spidy's notes point to are also present: `ViewContextManager`, `Adapted Lum`, `TemporalAa` and the `kTaaTarget` family.

Spidy's own type-based tools therefore work on Spider-Man 2, which makes step 3 a good deal easier than planned. One thing is missing: no `ViewContextManager` type name turned up, so that object is found through its string instead.

### Found on the owner's PC (2026-10-18, from the games' file lists)

- **Spider-Man 2's UI is Coherent Gameface (HTML and JavaScript), not Scaleform.** Spidy's HUD and settings-menu hooks can't be ported. Drop the HUD gate and make GAME SCREEN the only mode.
- **It ships the PlayStation PC SDK.** Confirm it plays offline without PSN before injecting anything.
- **It ships NVIDIA's ray-tracing denoisers.** Plan on ray tracing off for the eyes.
- **It ships DirectStorage.** Low risk.
- Details: [spiderman2-address-map.md](spiderman2-address-map.md#what-the-file-lists-already-show).

## What's next

### 1. Ghidra analysis of Spider-Man 2 (heavy; best overnight)

- **How long:** probably several hours, roughly 3 to 8 for a 190 MB game exe. It's my estimate, not measured. Memory use goes up to about 24 GB, which is the cap the script sets. Nothing else heavy can run at the same time (no Qwen, no exports, no game).
- **One fix first:** today's start failed on a wrong argument. The folder inside the project goes in the project name (`spiderman/sm2`), not in `-folderPath`. Nothing ran.
- The command, once fixed:
  ```bash
  ~/dev/flat-to-3d/tools/ghidra/headless.sh ~/.local/share/flat-to-3d/ghidra-projects spiderman/sm2 -import ~/.local/share/flat-to-3d/exe/sm2-2.810.0.0/Spider-Man2.exe -max-cpu 16 -log ~/.local/share/flat-to-3d/logs/sm2-analyze.log
  ```
- It can be stopped and started over at any time without harm, but an interrupted analysis starts from zero.

### 2. Same for Remastered (needs the download)

- Remastered has to finish installing first. Its exe must be version 4.630.0.0, the one Spidy's addresses are for: `tools/research/inspect_game.py` checks that.
- Then Ghidra analyses `Spider-Man.exe` into the same project (`spiderman/remastered`). That's another several-hour run.

### 3. Match the two games (Version Tracking)

- Ghidra's Version Tracking pairs Remastered's functions with Spider-Man 2's. It runs headless (`AutoVersionTrackingScript`), which may take another hour or more.
- First targets, the addresses Spidy uses to move the camera and make the eye views:

  | Remastered address | What it is |
  |---|---|
  | `0x1899ab0` | view descriptor submit |
  | `0x187bb00` | lens builder |
  | `0x187ca10` | pose setter |
  | `0x7a34dd0` | ViewContextManager (global) |
  | `0x18a09c0` | create off-screen view |

- Each Spider-Man 2 candidate gets written into `docs/sm2-address-map.md` with its evidence (match score, shared strings, type names, code shape). Anything not checked by hand is marked unverified.
- Even without Remastered, the type names and strings give a head start: `research_game.py strings ViewContextManager` plus `refs` on that string already points at the code that uses it.

### 4. Then the plan resumes

- **Step 2:** load Spidy's `render_probe` into Spider-Man 2 and confirm it finds the game's GPU queue. Doesn't need Ghidra.
- **Step 3 test:** shift the game's own camera sideways on command, and watch the picture slide on the Z13's screen.
- **Step 0** whenever Remastered is in and you have time with the headset. It's yours to run, from the run sheet.

## Decisions for you

1. **When to run the analysis.** Overnight is the obvious slot. Just say "run the Ghidra analysis tonight" and it starts with the memory watch on.
2. **REA.** Installed but not registered. Registering it (`rea setup`) changes Claude Code's config permanently, so it waits for your yes. Without it, I script Ghidra headless directly, which covers everything above.
3. Still open from the plan: contacting Spidy's author.
