# Roadmap: five Favorites in 3D

Where the project stands and what comes next. The full plan is [PORT-SPIDERMAN2.md](PORT-SPIDERMAN2.md) and the technical map is [spidy-stereo-core-map.md](spidy-stereo-core-map.md). This page is the short version.

Updated 2026-11-11 from the verified [Roadmap v2 handoff](plans/roadmap-v2/HANDOFF.md). The Spider-Man 2 Step 0–3 notes below record the 2026-10-18 investigation.

**1.0 scope (2026-10-26, widened 2026-10-31, reworked and approved 2026-11-01, [SPEC.md](../SPEC.md)):** "Plug in and play". Steam is the launcher: Play a game as usual. With the headset on the cable the game plays in 3D; without it the same Play button plays flat. Five Favorites, in order: STAR WARS Zero Company (UEVR, our own hook as fallback), ACE COMBAT 7 (UEVR), Control and Planet of Lana (Geo-11 and the SBS viewer), then Spider-Man 2 (Step 0 on Remastered, port Steps 1-8). Out of 1.0: the big-bucket tests (except UEVR on Zero Company), the generic 3D-screen pieces (unless Zero Company needs the fixed-screen layer), other games and Steam Frame (#8, #9, #11, #12, #13). Code map: [code-map.md](code-map.md).

## The 1.0 play round trip

1. **Plug in.** Z13 in Game Mode; Quest 2 on the USB-C cable, headset on, seated boundary. First time only: allow USB debugging. A user service (`flat3d-link`) sees the cable: it brings up the tunnel (internet from the PC over USB, the Quest Tether link ported to Rust), WiVRn and the headset app, and a toast says 3D is ready (M10, M25).
2. **Press Play** on a game with the `3D Ready` collection badge. The launch wrapper `flat3d-run` sees the link is ready and the game's switch is on, applies the game's route and starts it with the OpenXR environment. With no headset, or the switch off, it starts the game unchanged (M9, M30).
3. **Play** on the giant fixed screen with depth, gamepad in hand (M19). Head movement looks around the room, not the game. Pause goes flat.
4. **Unplug** any time: the service keeps the link for 2 minutes in case of a replug, then stops it and says "restart the game for flat" (M18). A headset plugged in while a game is already running flat gets a toast to restart it in 3D.
5. **Watch** the headset from Quick Access: battery, temperature (toast at 38 C and 42 C), throughput, uptime, reconnects (M26, M27).

The manual Desktop Mode session is documented in [Step 0 run sheet](STEP0-RUNSHEET.md).

**Order (ruling 2026-10-31):** Zero Company -> ACE COMBAT 7 (same UEVR route) -> Geo-11 and the SBS viewer -> Control and Planet of Lana -> Spider-Man 2.

## The app: one engine, two faces (rulings 2026-10-31, 2026-11-01)

"Desktop app + Decky plugin (Recommended)", built in Rust ("Native Rust GUI", "iced (Recommended)"; "Rust backend, thin TS panel (Recommended)", 2026-11-01). The Decky panel (a thin TypeScript layer over the engine binary) runs in Game Mode (link, health, switches, 3D Ready games); the iced app runs in Desktop Mode (Setup, Headset, Games, Logs). Both read one `snapshot` from the Rust engine in `crates/engine/`, whose `flat3d-link` service owns the headset link; neither face has an Inject button. Shape and interlinks: [code-map.md](code-map.md).

## Where it stands (2026-11-11)

The unattended build reached local `v2/integration` at `c9c46b7`. The isolated Linux gate passed 13 checks, with no skips or failures and 3 checks not applicable. The [progress log](plans/roadmap-v2/progress.md) records the verified heads, PRs and device work. GitHub's `main` and the handoff have different histories. The results below describe the handoff's integration tree.

| Wave | Verified on integration | Blocked or skipped |
|---|---|---|
| 0 | WP-00 added formatting and Clippy gates | |
| 1 | WP-01a–d reshaped the workspace and CLI; WP-02 and WP-25a added repository and docs tooling | |
| 2 | WP-03–07, WP-11–14, WP-16–18 and WP-20–24 delivered the page models, list/status, health loop, route work, Decky scaffold, packaging and session tooling | WP-25b (docs scrub) |
| 3 | WP-08, WP-15, WP-19, WP-26 and WP-28 delivered link commands, the Windows injector build, Linux viewer, recipe fixtures and view commands | WP-10 (tether), WP-27 (inject, undo and fetch), WP-30 (RE tooling) |
| 4 | WP-32 and WP-33 delivered the Decky panel and desktop app wiring | WP-09 (watch correctness), WP-29 (doctor and setup), WP-31 (wrapper route) |
| 5 | | WP-34 (final docs sync) waits on earlier packages |

**31 done, 3 blocked, 5 skipped.** The code for the viewer, desktop app and Decky panel passed host checks. Headset, Steam Game Mode, AppImage and game sessions remain pending. The device queue below tracks the remaining checks.

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
| #27 | Route dispatch, `port` route, `list` and `inject` command bodies | Medium | `list` passed in WP-07. Inject and route orchestration await WP-27 and WP-31. |
| #28 | Decky plugin (Python era) | Medium | Closed, superseded by #130-#133 |
| #29 | Spider-Man 2 recipe | Easy | Open, locked tests in the issue; blocked by #23 |
| #6, #10 | Parents: the port, the Inject app | Hard | Open |
| #15, #16 | Recipes home; research notes | | Closed (done) |
| #155 | SpidyVR 0.3.8 -> 0.7.0 | Medium | Open; after Step 0 passes |
| #183 | `cargo fmt` and `clippy` red on main; add both to CI | | Open; next, before wave 2 |

After 1.0 (kept, not sized): #4, #8, #9, #11, #12, #13, #14.

### Done

- **Repo** `github.com/iannitram/flat-to-3d` (private, GPL-3.0), with SpidyVR v0.3.8 pinned as a submodule.
- **Spidy builds on the Z13.** Cross-compiled in the Linux build container with clang-cl and the Windows SDK (via xwin). 337 of 337 tests pass under Proton's Wine.
- **Spidy's discovery tools work on any exe.** I copied four of them into `tools/research/` and removed the Remastered hash and path checks. Ilya Mezerowsky is credited in each file. They now read `Spider-Man2.exe`:
  - `inspect_game.py`: version, hash, sections. It now reads the version without Windows, and it names both known builds.
  - `research_game.py`: disassembly, cross-references, string search.
  - `discover_render_types.py`: C++ type names to vtables.
  - `find_method_owners.py`: which class a function belongs to.
- **Ghidra 12.1.4 and Java 21 (Temurin) installed** under `~/.local/share/flat-to-3d/`, user-space, checksums verified. Ghidra is set to use that Java.
- **REA 6.7.0 installed** under `~/.local/share/flat-to-3d/rea/`, with no install scripts run. **Not registered** in Claude Code, and not yet checked against Ghidra.
- **A copy of the SM2 exe** to analyse, at `~/.local/share/flat-to-3d/exe/sm2-2.810.0.0/`. SHA-256 is `c9b912ef…b738c9`, version 2.810.0.0.
- **`tools/ghidra/headless.sh`** runs Ghidra as a heavy job. It checks for competing jobs before starting and stops Ghidra if free memory drops under 2 GiB.

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

1. **WP-10** needs the service fixture to match the configured gnirehtet path before its tether gate can pass. WP-09 waits on it. [Draft #235](https://github.com/iannitram/flat-to-3d/pull/235)
2. **WP-25b** needs the docs checker findings in the build and check scripts resolved, along with the code-map conflict on its draft branch. WP-30 waits on it. [Draft #232](https://github.com/iannitram/flat-to-3d/pull/232)
3. **WP-27** needs the injector's Clippy and stale test-call failures fixed, plus a complete reuse-record check. WP-29 and WP-31 wait on it. [Draft #238](https://github.com/iannitram/flat-to-3d/pull/238)

WP-34 follows those dependency chains. See the [work package index](plans/roadmap-v2/03-wp-index.md) for the wave and dependency order.

## Owner-device queue

Run the listed TOML files with `python3 tools/session/session.py run sessions/<name>.toml` from the repository root. The [full queue](plans/roadmap-v2/02-queues-and-decisions.md) records the acceptance criteria and issue references.

| Check | Session files under `sessions/` | Outstanding prerequisites |
|---|---|---|
| Q0 Pin download hashes | `q0.toml` | Owner download |
| Q1 Build bundled tools | `tools-build.toml` | |
| Q2 Build and install injector executable | `q2.toml` | SDK download and license acceptance |
| Q3 Install, link smoke and soak | `link-smoke.toml` | WP-09, WP-10 |
| Q4 Doctor baseline | `link-smoke.toml` (doctor step) | WP-29 |
| Q5 Zero Company offline | `z1-offline.toml` | |
| Q6 Live UEVR | `z2-uevr.toml` | WP-27, WP-31 |
| Q7 Zero Company screen, mission and recipe | `z3-screen.toml`, `z4-mission.toml`, `z5-recipe.toml` | Q6 |
| Q8 ACE COMBAT 7 | `a1-install.toml`, `a2-uevr.toml`, `a3-sortie.toml` | Q6 and game library |
| Q9 Geo-11 | `g1-geo11.toml` | WP-27, WP-31 |
| Q10 Linux viewer | `g2-viewer.toml` | WP-31 |
| Q11 Planet of Lana and Control | `g3-lana.toml`, `g4-control.toml` | Q9, Q10 |
| Q12 Reverse-engineering analysis | `q12.toml` | WP-30 |
| Q13 Spider-Man offline, Step 0 and SpidyVR | `sm2-offline.toml`, `step0.toml`, `spidy-070.toml` | Game library |
| Q14 Decky in Game Mode | `decky.toml` | |
| Q15 Desktop app window | `app-window.toml` | WP-27, WP-29 for the full flow |
| Q16 AppImage | `appimage.toml` | |

## Decisions for you

- The [roadmap rulings and defaults](plans/roadmap-v2/02-queues-and-decisions.md) record OD1–OD8 and the scope deferred from this pass.
- REA registration and AER follow the existing [rulings](notes/DECISIONS.md).
- Contacting the SpidyVR author remains open.
- ACE COMBAT 7 and Spider-Man Remastered need their library drives mounted or a fresh install.
