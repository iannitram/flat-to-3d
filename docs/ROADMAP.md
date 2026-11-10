# Roadmap: five Favorites in 3D

Where the project stands and what comes next. The full plan is [PORT-SPIDERMAN2.md](PORT-SPIDERMAN2.md) and the technical map is [spidy-stereo-core-map.md](spidy-stereo-core-map.md). This page is the short version.

Updated 2026-11-07 (after wave 1 merged, main `50c1e49`: engine progress, issue states, what's next; the Spider-Man 2 Step 0-3 notes below are from 2026-10-18).

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

"Desktop app + Decky plugin (Recommended)", built in Rust ("Native Rust GUI", "iced (Recommended)"; "Rust backend, thin TS panel (Recommended)", 2026-11-01). The Decky panel (a thin TypeScript layer over the engine binary) runs in Game Mode (link, health, switches, 3D Ready games); the iced app runs in Desktop Mode (Setup, Headset, Games, Logs). Both read one `snapshot` from the Rust engine in `inject/`, whose `flat3d-link` service owns the headset link; neither face has an Inject button. Shape and interlinks: [code-map.md](code-map.md).

## Where it stands (2026-11-07)

- **Wave 1 is merged (2026-11-07, PRs #190-#197):** `stop_3d` (#66), the watch loop `watch::run` (#119), `link_view` (#75), the tether plan (#99), `undo <appid>` (#84), the reshaped `AdbRunner` (#96), the UEVR route's fake-prefix half (#51) and the iced app crate with its CI job (#78). 459 engine tests and 4 app tests pass on main. Each PR was reviewed on two lenses with every finding checked by two skeptics; the confirmed bugs were fixed before the merge (in #119: the tether stuck on after an unplug, WiVRn started with Auto 3D off, a failed start reading Ready; in #51: a partial backup blocking later applies, an invented env variable).
- **Engine: the parts exist, the commands mostly don't.** The link service's loop exists but nothing starts it: `watch`, `snapshot`, `switch`, `setting`, `install` are parsed but not wired into `main.rs` (#120). `cli::dispatch` answers "not built yet" for everything but `undo` (`list`, `inject`, `start-3d`, `stop-3d`, `status`, #27), and `injectors::inject` is still a stub (#27). The routes: UEVR's file half is built and tested on a fake prefix (#51, device half waits on Z2 #32), Geo-11 (#52) and port (#27) are not started.
- **Is the base built? Nearly, not yet.** `flat3d-run` and `undo` are the only commands that do real work; `flat3d-run` still always plays flat, because no service writes `link.json` until `watch` is a command (#120). #120 is the one issue between here and a running link service.
- **Faces and packaging: started.** `app/` is a skeleton (four placeholder tabs, #78). No `decky/src/` or `packaging/` yet.
- **Games: no headset session yet.** None of the Zero Company, ACE COMBAT 7 or Geo-11 steps has run, and the Spider-Man 2 Ghidra analysis never ran (its project folder is empty; the only log is the failed start of 2026-10-18).
- **Installed today:** Zero Company, Spider-Man 2 and Control on the internal drive, Planet of Lana on a second library drive. ACE COMBAT 7 and Spider-Man Remastered are not on any drive mounted today (three library drives are unmounted).
- **Known gaps from the wave-1 review (not bugs against their issues; for later issues):** `stop_3d` reports an error when the WiVRn unit is already gone (a second Stop 3D); no code emits the `allow-debugging` event yet; the snapshot's `tunnel` is "tether is up", not "tun0 carries traffic", until the health loop (#125); the UEVR route builds an undo record but nothing saves it yet (#27 does); `undo`'s command line has no test of its own.
- **Housekeeping:** main fails `cargo fmt --check` and `clippy -D warnings`, which CI doesn't run (#183; now the only wave-1 issue left, best done before wave 2 starts so its reformat doesn't collide). #146 is done on main and can close. Parents #25 and #50 (every child merged) and #26 and #39 (E2: #64-#66 merged) can close. The merged `build/*` branches and their worktrees can go.

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
| Z5 (#36) | Zero Company recipe (`works-now`, UEVR route and settings) and `flat3d inject 2075800` | Z4, #25 | M17 |

Spider-Man 2 note (2026-10-31): HEROVR (lecutis7of9/HEROVR, [PEERS.md](../PEERS.md)) already shows Spider-Man 2 in stereo on Windows. Ruling: learn only, keep our port; its hook list is a map to check our Ghidra results against, never a source of addresses (#37). SpidyVR 0.7.0's Proton + WiVRn path feeds Step 0 (#38).

Zero Company proves the headset chain, the link service and the `flat3d-run` wrapper on a real game while the Spider-Man 2 address mapping goes on.

## Spider-Man 2 track: where we are

| Step | What | State |
|---|---|---|
| 0 | Baseline: unmodified SpidyVR on Remastered, in the Quest 2 (#5) | **Waiting on a headset session.** Remastered (Steam 1817070) is not on any drive mounted on 2026-11-07 (it was 66 GB into its install on 2026-10-18; it may be on an unmounted library drive). Run sheet: [STEP0-RUNSHEET.md](STEP0-RUNSHEET.md). After it passes, move SpidyVR to 0.7.0 (#155). |
| 1 | Workbench | **Done.** See below. |
| 2 | Get inside Spider-Man 2 (#18, blocked by #7) | Not started. |
| 3 | Move the camera (find the camera functions in SM2) (#19, blocked by #18) | **Started offline.** Tools ready; the Ghidra analysis has still not run (checked 2026-11-07). On the owner's PC the fast matcher (`tools/xmatch/`) is ready and waits on the downloads: [spiderman2-address-map.md](spiderman2-address-map.md). |
| 4–7 | Two eyes (#20), clean images (#21), headset (#22), HUD and memory (#23) | Not started. Each is blocked by the one before it. |
| 8 | The plug-in-and-play engine and Spider-Man 2's recipe | **Engine pure parts built; wiring, faces and the recipe not.** See "Where it stands" and the stage table above (parent #10). Spider-Man 2's own pieces: the `port` route (#27) and its recipe (#29, blocked by #23). |

### Dev Loop issues (1.0)

| Issue | What | Size | State |
|---|---|---|---|
| #5 | Step 0 on Remastered | Hard | Open, needs the owner's headset |
| #7 | Does Spider-Man 2 play offline? | Question | Open, needs the owner's Z13; blocks #18 |
| #18-#23 | Port Steps 2-7 | Medium, then Hard | Open, chained #18 -> #19 -> #20 -> #21 -> #22 -> #23 |
| #24 | Inject: find Steam games | Easy | Closed (on main, `b2bfdc3`) |
| #25 | Inject: load and check recipes (parent of #58-#60) | Medium | Open; every child merged, can close |
| #26, #39 | Link: WiVRn half of Start 3D and Stop 3D (parents of #64-#66) | Medium | Open; every child merged, can close |
| #27 | Route dispatch, `port` route, `list` and `inject` command bodies | Medium | Open; blocked by #120 (#79, #80 are merged) |
| #28 | Decky plugin (Python era) | Medium | Closed, superseded by #130-#133 |
| #29 | Spider-Man 2 recipe | Easy | Open, locked tests in the issue; blocked by #23 |
| #6, #10 | Parents: the port, the Inject app | Hard | Open |
| #15, #16 | Recipes home; research notes | | Closed (done) |
| #155 | SpidyVR 0.3.8 -> 0.7.0 | Medium | Open; after Step 0 passes |
| #183 | `cargo fmt` and `clippy` red on main; add both to CI | | Open; next, before wave 2 |

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

In order of what unblocks the most.

### 1. Engine wiring (satellites, now)

Wave 1 merged on 2026-11-07 (PRs #190-#197). Blockers re-checked against main `50c1e49`.

- **First: #183** fmt and clippy, and both in CI. Its `cargo fmt` rewrites about 45 files, locked tests among them (format only), so it goes in alone before wave 2 branches open, and not as a Qwen job (R28).
- **Wave 2 (ready now):** #120 link commands (Medium; the biggest unblocker: it makes `watch` a real command and unblocks #27, #121, #128), #125 tether health loop (Medium), #76 rows (Easy), #116 Headset page and #118 Setup page (Easy).
- **Wave 3 (after #120, #76):** #27 command bodies, dispatch and the port route, #121 view commands, #117 Games page, #128 install wiring. With #27 in, Zero Company can be tried by hand.
- **Wave 4:** #130 Decky scaffold (after #121), #122 app views, then #123, #131-#133, and packaging (#129, #134). #127 `build-tools.sh` is ready any time but needs the owner's device.

### 2. Zero Company with the headset (owner session)

- Zero Company is installed. Z1 (#31, offline after the EA link) needs only the Z13. Z2 (#32, UEVR under Proton) can run by hand before the engine wiring is done; Kaon is the reference.

### 3. Ghidra analysis of Spider-Man 2 (heavy; best overnight)

- **How long:** probably several hours, roughly 3 to 8 for a 190 MB game exe. It's my estimate, not measured. Memory use goes up to about 24 GB, which is the cap the script sets. Nothing else heavy can run at the same time (no Qwen, no exports, no game).
- **One fix first:** today's start failed on a wrong argument. The folder inside the project goes in the project name (`spiderman/sm2`), not in `-folderPath`. Nothing ran.
- The command, once fixed:
  ```bash
  ~/dev/flat-to-3d/tools/ghidra/headless.sh ~/.local/share/flat-to-3d/ghidra-projects spiderman/sm2 -import ~/.local/share/flat-to-3d/exe/sm2-2.810.0.0/Spider-Man2.exe -max-cpu 16 -log ~/.local/share/flat-to-3d/logs/sm2-analyze.log
  ```
- It can be stopped and started over at any time without harm, but an interrupted analysis starts from zero.

### 4. Same for Remastered (needs the download)

- Remastered has to finish installing first. Its exe must be version 4.630.0.0, the one Spidy's addresses are for: `tools/research/inspect_game.py` checks that.
- Then Ghidra analyses `Spider-Man.exe` into the same project (`spiderman/remastered`). That's another several-hour run.

### 5. Match the two games (Version Tracking)

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

### 6. Then the Spider-Man 2 plan resumes

- **Step 2:** load Spidy's `render_probe` into Spider-Man 2 and confirm it finds the game's GPU queue. Doesn't need Ghidra.
- **Step 3 test:** shift the game's own camera sideways on command, and watch the picture slide on the Z13's screen.
- **Step 0** whenever Remastered is in and you have time with the headset. It's yours to run, from the run sheet.

## Decisions for you

1. **When to run the analysis.** Overnight is the obvious slot. Just say "run the Ghidra analysis tonight" and it starts with the memory watch on.
2. **REA.** Installed but not registered. Registering it (`rea setup`) changes Claude Code's config permanently, so it waits for your yes. Without it, I script Ghidra headless directly, which covers everything above.
3. Still open from the plan: contacting Spidy's author.
4. **Issue clean-up:** close #146 (done on main by #174) and the parents #25, #50, #26 and #39 (every child merged). #51 and #78 stay open for their device halves. The 8 merged PRs carry no `by:` / `model:` labels (the guard leaves labels to the coordinator; `by:strix-halo` and `model:claude-sonnet-5-5` don't exist yet).
5. **Drives.** ACE COMBAT 7 (A1, #40) and Spider-Man Remastered (Step 0, #5) need their library drive mounted or a fresh install.
