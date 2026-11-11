# Plan: Spider-Man 2 in true 3D, ported from SpidyVR

**Goal.** Play Marvel's Spider-Man 2 on a giant fixed 3D screen in the Quest 2, with a controller, on the Z13 under Proton. The game itself draws both eyes in the same frame: real stereo, not alternating eyes and not faked depth.

**Starting point.** [SpidyVR](https://github.com/IlyaMez/SpidyVR) (GPL-3.0, pinned at v0.3.8, commit `aed5c0c`), a full VR mod for Spider-Man Remastered. Its **GAME SCREEN 3D** mode already does exactly what we want, but only for Remastered. We port only that mode. Swinging, body, punching, gadgets and menus stay behind.

The detailed code map behind this plan is [spidy-core-map.md](spidy-core-map.md). This page is the plain-language version plus the steps.


## How Spidy's 3D screen works

1. **Getting in.** A script loads Spidy's code into the running game. On Linux the script runs inside the game's Proton session.
2. **Two extra cameras.** Spidy asks the engine for two extra off-screen views, the same kind the game uses internally. Every frame, just before the game hands its camera to the renderer, Spidy copies the game's camera into both views. It then shifts one a little left and one a little right, and angles their lenses so both converge on a screen 2.5 m away.
3. **Same-frame pairs.** Each eye's GPU work is tagged with the frame number. A pair is shown only when both eyes finished the same frame, so the eyes never drift apart in time.
4. **Image quality.** Each eye keeps its own anti-aliasing history (the engine's own TAA) and copies the game's brightness so both eyes match. The game's DLSS or FSR upscaler is bypassed for the eyes.
5. **The screen.** The two images go to the headset through OpenXR as two flat panels at the same spot: one visible only to the left eye, one only to the right. That's the 3D TV. The game's HUD is moved onto the screen plane so text has no depth.

## What carries over and what doesn't

**Reusable as-is (engine-independent):** the OpenXR screen output, the D3D12 frame copying and pairing, the Proton fixes, gamepad reading, the stereo math, and the injection scripts with names changed.

**Must be found again in Spider-Man 2.** Spidy finds everything in the game by fixed addresses for one exact Remastered build, with no searching. For Spider-Man 2 every address is different. For the 3D-screen path alone that means:

| What | Count |
|---|---:|
| Game functions Spidy hooks | ~23 |
| Game functions Spidy calls | ~11 |
| Global variables | ~16 |
| Object layouts (field offsets) | ~85 |
| Byte checks, return addresses, type tables | ~20 |

Another ~40 addresses serve only the swinging and menu features and can be dropped.

**Good news from the Spider-Man 2 exe (checked on the Z13, 2026-10-18):**
- Version **2.810.0.0**, Steam build 24651585.
- It contains the same engine system names Spidy relies on: `PlayerModelHudFollower`, `TimeScaleSystem`, `HeroOfferProcessor`, the `uisystemmenu` config. So the internal design matches.
- It has no packer or protection sections. The code is plain, so injection and hooks should work as they do for Remastered.

**Bad news:**
- ~~Its C++ type information is stripped.~~ **Corrected 2026-10-18:** it is not. The exe holds 27,183 type names, and Spidy's `discover_render_types.py` finds vtables for all 124 camera and render types it looks for, so the type-based tools work on Spider-Man 2.
- Spider-Man 2 leans on ray tracing and newer upscalers. Spidy's anti-aliasing and brightness tricks may need rework.

**From the games' shipped file lists (a second machine (2026-10-18 scan leg), 2026-10-18).** Measured by file name. What each file is comes from its maker's documentation, and no exe was read. Details: [address-map.md#what-the-file-lists-already-show](address-map.md#what-the-file-lists-already-show).
- **The UI is Coherent Gameface (HTML and JavaScript), not Scaleform.** Spidy's four HUD hooks and its eleven settings-menu hooks have no counterpart. What changes:
  - Step 6 drops the HUD gate from the start.
  - GAME SCREEN becomes the only mode, so there's no in-game settings tab.
  - Step 7 has to find how Gameface's HUD reaches the screen. `PlayerModelHudFollower` still exists, so the HUD may still be a 3D panel.
- **The PlayStation PC SDK (PSN sign-in runtime) ships with it.** Remastered has none. Before step 2, confirm the game plays offline without PSN (offline-only rule).
- **NVIDIA's ray-tracing denoisers (NRD) and DLSS ray reconstruction ship with it.** Plan on ray tracing off for the eyes from the start (step 5).
- **DirectStorage ships with it.** It adds GPU copy and compute queues. Spidy's probe picks the direct queue, so this is low risk for step 2.
- **The same as Remastered:** DX12 with the Agility SDK, Streamline (DLSS frame generation), and no anti-cheat files.

## How we find the addresses

The fastest route is **binary diffing**. Spidy already names hundreds of Remastered functions. Load the Remastered exe and the Spider-Man 2 exe into **Ghidra** (free, from the NSA) and run its **Version Tracking**, which matches functions across two related builds. Each Spidy address in Remastered then points to its Spider-Man 2 counterpart, which we verify by hand. Strings (`Adapted Lum`, `kTaaTarget0`, `RenderAlloc`, `ViewContextManager`, `TemporalAa`, `ScaleformRTTStream`) anchor the matches.

This needs the **Remastered exe** (version 4.630.0.0). Spider-Man Remastered is being installed on the Z13 for this and for step 0, and on a second machine (2026-10-18 scan leg) for the diff. The second machine's Steam copy is 121,901,008 bytes, the exact size of 4.630.0.0. The hash isn't checked yet.

**Faster first pass (a second machine (2026-10-18 scan leg)):** `tools/xmatch/run_match.py` matches each Spidy function by masked byte windows plus instruction-sequence similarity. It follows calls and global references out of every confident match, matches vtables by RTTI name, and writes a unique pattern scan for each hit. It takes minutes and about 2 GB of memory. Ghidra Version Tracking stays as the cross-check for low-confidence items.

**REA** ([morluto/rea](https://github.com/morluto/rea), MIT) lets the agent drive Ghidra directly: decompile, follow cross-references, search strings, and keep evidence for each conclusion. the owner approved it in principle for steps 3–5 and for later performance work, such as skipping work the game's own main view does while both eyes render. Registering it in the Script Code's config needs owner approval first.

**Survive patches.** Spidy breaks on every game update. Our port should find each address by **byte-pattern search** at startup, with the fixed address as a fallback. Then a small Spider-Man 2 patch doesn't break everything.

## What the file lists already show

Spider-Man 2's shipped files identify Coherent Gameface, PlayStation PC SDK, NVIDIA ray-tracing denoisers, DLSS ray reconstruction and DirectStorage. The HUD/settings hooks need a different plan, offline play must be confirmed before injection, ray tracing may need to be off for the eyes, and DirectStorage is low risk. These findings came from filenames in a second machine (2026-10-18 scan leg), not from reading an executable.

The executable also contains 27,183 C++ type names. The type survey found vtables for the 124 camera and render types it checked, including `Camera2::Camera`, `FollowCamera`, `CombatCamera`, `PlayerModelHudFollower`, `HeroLocal` and `TimeScaleSystem`. The `ViewContextManager` object is found through its string because its type name did not appear. Spidy's type-based tools therefore work on Spider-Man 2 and make the camera mapping easier.

## Steps

Each step ends in a test that can be checked without guessing.

### Step 0: Baseline (needs Remastered installed)
Run unmodified Spidy (official 0.3.8 release) on Spider-Man Remastered on the Z13 in GAME SCREEN 3D, through WiVRn's wired mode (`adb reverse`) on the USB cable. Quest Tether's internet sharing isn't needed for this.
- **Proves:** Proton + WiVRn + Quest 2 + the stereo-screen technique all work on this machine, before we spend weeks porting.
- **Test:** the Remastered city shows on a 3D screen in the Quest 2 with visible depth, at 30+ fps.
- Can run in parallel with steps 1–2.

### Step 1: Set up the workbench
- Pin SpidyVR in our repo as `third_party/SpidyVR` (git submodule, v0.3.8). **Done.** Our port will live in `games/spiderman2/`.
- Build Spidy's Windows DLLs inside the distrobox build container (`$FLAT3D_BUILD_BOX`, see ../workbench.md) with clang-cl and lld-link, using the Microsoft C runtime and Windows SDK fetched by `xwin`. The target is Linux; MinGW is not an option because Spidy uses Microsoft-only `__try`/`__except`. The build path is `cmake/clang-cl-xwin.cmake` and `tools/build-spidy.sh`.
- Strip Spidy's discovery tools (`inspect_game.py`, `research_game.py`, `find_method_owners.py`) of their Remastered hash checks so they run on any exe.
- Record Spider-Man 2's exe hash and version. Turn off its automatic updates in Steam where possible, and keep a copy of the exe for analysis.
- **Test:** a minimal DLL builds; Spidy's tools read `Spider-Man2.exe` without refusing.

### Step 2: Get inside the game
Adapt `spidy-linux.sh` and the injection scripts to Spider-Man 2: Steam app 2651280, `Spider-Man2.exe`, its size constant. Load only `render_probe`, which finds the game's GPU queue.
- **Test:** the game keeps running and the log names its D3D12 queue.

### Step 3: Move the camera
Find the camera submit, lens builder and pose setter in Spider-Man 2 (Ghidra diff from Remastered). Repeat Spidy's earliest experiment (`game_view.cpp`): shift the game's own camera sideways.
- **Test:** the picture on the Z13's screen visibly slides left and right on command.

### Step 4: Two eyes
Find the view manager, the off-screen view creator and the per-frame placement point. Create two eye views and place them every frame.
- **Test (no headset):** a Spider-Man 2 version of Spidy's `probe_game_screen.py --stereo` captures both eyes. Distant buildings differ by tens of pixels between the eyes; HUD text differs by about 0.

### Step 5: Same-frame pairs and clean images
Port the render-job tagging and GPU pairing. Get each eye its own anti-aliasing history, matching brightness and correct tone mapping. Ray-traced reflections off for the eyes, upscaler bypassed.
- **Test:** every shown pair has matching frame numbers; both eyes are equally bright; no shimmering edges.

### Step 6: In the headset
Write a ~300-line worker to replace Spidy's big VR controller. It starts the eyes, sends the 3D-screen command and hands pairs to OpenXR as the two eye-only panels.
- Spidy shows the screen only while the game clock runs (game time is exactly 0 when paused) and while its HUD hook works. Start with the clock gate only, and make the HUD a later step.
- Keep the output on plain OpenXR so it also works through SteamVR. That leaves room for the **Steam Frame** later (DECISIONS ruling 19).
- **Test:** in the Quest 2, Spider-Man 2 plays on a big 3D screen with the controller, depth visible, 30+ fps, and pausing drops to a flat screen.

### Step 7: HUD, memory and polish
- Move the HUD onto the screen plane, or composite it ourselves if Spider-Man 2 draws its HUD differently.
- Enlarge the render memory ring if three views overflow it. That has to happen within ~3 s of game start, so the launch script must load us early.
- Swap fixed addresses for pattern scans.
- **Test:** a full play session (swinging, combat, a cutscene, a pause) without dropped eyes or crashes.

### Step 8: Hook into the app
Write Spider-Man 2's recipe keys (`stereo-status: works-now`) and wire it to the Inject button once the app exists.

## Risks

| Risk | What happens | Fallback |
|---|---|---|
| Spider-Man 2's renderer changed how off-screen views work | Step 4 stalls | Render the second eye by re-submitting the main view, at a cost in frame rate |
| No GPU timing markers in Spider-Man 2's render jobs | Can't pair eyes | Insert our own fences or markers |
| HUD drawn differently (**now likely**: SM2 ships Coherent Gameface, not Scaleform) | The 3D screen stays flat under Spidy's gate | Drop that gate (planned in step 6) |
| PSN sign-in required (SM2 ships the PlayStation PC SDK) | Breaks the offline-only rule | Check before step 2. If it's required, stop and ask the owner |
| Game updates | Fixed addresses break | Pattern scans (step 7); pin the build |
| Ray tracing tied to the main view | Eyes lack reflections or crash | RT off for the eyes |
| Z13 frame rate | Three views (game + 2 eyes) cost a lot | Lower eye resolution, dynamic resolution, skip the main view's full render if possible |
| Late attach (Linux script attaches at the menu) | Can't enlarge render memory | Launch wrapper that loads early |

**Effort, honestly:** steps 3–5 are reverse-engineering work, measured in weeks of sessions, not days. The binary diff from Remastered is what keeps it from being months. Spidy's author built the original with heavy AI help, and we'd work the same way.

## Machine rules that apply

- **Z13 memory:** Ghidra on a 190 MB exe and the game itself are both heavy. Run one heavy job at a time under the 2 GiB watch, and check `$FLAT3D_HEAVY_LOCK` first.
- **Commits:** use the project no-reply identity and never force-push.
- **Licence:** GPL-3.0 throughout. Our port is a derivative of Spidy, so credit Ilya Mezerowsky in the README and file headers.

## Decisions

Settled:
1. Spider-Man Remastered is the Step 0 baseline.
2. DLLs build in the distrobox build container (`$FLAT3D_BUILD_BOX`, see ../workbench.md) with clang-cl and xwin; `cmake/clang-cl-xwin.cmake` and `tools/build-spidy.sh` provide the build path.
3. REA is approved in principle for reverse-engineering; registering it in the agent configuration needs owner approval.
4. **No paywalled tools**, so this port is built from SpidyVR, not from R.E.A.L. VR.

Open:
- **Contact Spidy's author?** He's asking for ideas for "the next full VR conversion", and a Spider-Man 2 port overlaps with that. Working with him, or at least telling him, could save weeks. A compatibility report after step 0 is a natural first contact.
- **AER as a fallback?** If the Z13 can't afford both eyes every frame in Spider-Man 2, alternating eyes is the fallback. the owner has not ruled on whether that still counts as true stereo.
