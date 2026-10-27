# Roadmap: Spider-Man 2 in 3D

Where the project stands and what comes next. The full plan is [PORT-SPIDERMAN2.md](PORT-SPIDERMAN2.md) and the technical map is [spidy-stereo-core-map.md](spidy-stereo-core-map.md). This page is the short version.

Updated 2026-10-18.

**1.0 scope (2026-10-26, [SPEC.md](../SPEC.md)):** Step 0 on Remastered, then port Steps 1-8 on Spider-Man 2, started from Game Mode with the Inject app's Start 3D and Inject buttons. Out of 1.0: the big-bucket tests, the generic 3D-screen pieces, other games and Steam Frame (#8, #9, #11, #12, #13). Code map: [code-map.md](code-map.md).

## Where we are

| Step | What | State |
|---|---|---|
| 0 | Baseline: unmodified SpidyVR on Remastered, in the Quest 2 | **Waiting on a headset session.** Remastered (Steam 1817070) is finishing its install (66 GB downloaded). Run sheet: [STEP0-RUNSHEET.md](STEP0-RUNSHEET.md). |
| 1 | Workbench | **Done.** See below. |
| 2 | Get inside Spider-Man 2 | Not started. |
| 3 | Move the camera (find the camera functions in SM2) | **Started offline.** Tools ready; the Ghidra analysis has not run yet. On Hyperion the fast matcher (`tools/xmatch/`) is ready and waits on the downloads: [spiderman2-address-map.md](spiderman2-address-map.md). |
| 4–8 | Two eyes, clean images, headset, HUD, app | Not started. |

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

### Found on Hyperion (2026-10-18, from the games' file lists)

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
