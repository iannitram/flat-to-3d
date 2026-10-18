# Plan: Spider-Man 2 in true 3D, ported from SpidyVR

**Goal.** Play Marvel's Spider-Man 2 on a giant fixed 3D screen in the Quest 2, with a controller, on the Z13 under Proton. The game itself draws both eyes in the same frame: real stereo, not alternating eyes and not faked depth.

**Starting point.** [SpidyVR](https://github.com/IlyaMez/SpidyVR) (GPL-3.0, pinned at v0.3.8, commit `aed5c0c`), a full VR mod for Spider-Man Remastered. Its **GAME SCREEN 3D** mode already does exactly what we want, but only for Remastered. We port only that mode. Swinging, body, punching, gadgets and menus stay behind.

The detailed code map behind this plan is [spidy-stereo-core-map.md](spidy-stereo-core-map.md). This page is the plain-language version plus the steps.

Status: **plan only.** Nothing is installed, built or run.

---

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
- Its C++ type information is **stripped** (two traces, against the many Spidy's tools read in Remastered). Spidy's type-based discovery tools won't work directly.
- Spider-Man 2 leans on ray tracing and newer upscalers. Spidy's anti-aliasing and brightness tricks may need rework.

## How we find the addresses

The fastest route is **binary diffing**. Spidy already names hundreds of Remastered functions. Load the Remastered exe and the Spider-Man 2 exe into **Ghidra** (free, from the NSA) and run its **Version Tracking**, which matches functions across two related builds. Each Spidy address in Remastered then points to its Spider-Man 2 counterpart, which we verify by hand. Strings (`Adapted Lum`, `kTaaTarget0`, `RenderAlloc`, `ViewContextManager`, `TemporalAa`, `ScaleformRTTStream`) anchor the matches.

This needs the **Remastered exe** (version 4.630.0.0), so Spider-Man Remastered must be installed at some point. Ian owns it.

**Survive patches.** Spidy breaks on every game update. Our port should find each address by **byte-pattern search** at startup, with the fixed address as a fallback. Then a small Spider-Man 2 patch doesn't break everything.

## Steps

Each step ends in a test that can be checked without guessing.

### Step 0: Baseline (needs Remastered installed)
Run unmodified Spidy on Spider-Man Remastered on the Z13 in GAME SCREEN 3D, through WiVRn over Quest Tether's USB link.
- **Proves:** Proton + WiVRn + Quest 2 + the stereo-screen technique all work on this machine, before we spend weeks porting.
- **Test:** the Remastered city shows on a 3D screen in the Quest 2 with visible depth, at 30+ fps.
- Can run in parallel with steps 1–2.

### Step 1: Set up the workbench
- Fork SpidyVR into our repo as a pinned `third_party/SpidyVR` (git submodule). Our port lives in `games/spiderman2/`.
- Build Spidy's Windows DLLs on the Z13 inside the existing `craft-build` distrobox (clang-cl + Windows SDK via `xwin`, or MinGW). Alternative: build on Hyperion with Visual Studio. **Decision needed** (below).
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
Write Spider-Man 2's recipe into its wiki page (`stereo-status: works-now`) and wire it to the Inject button once the app exists.

## Risks

| Risk | What happens | Fallback |
|---|---|---|
| Spider-Man 2's renderer changed how off-screen views work | Step 4 stalls | Render the second eye by re-submitting the main view, at a cost in frame rate |
| No GPU timing markers in Spider-Man 2's render jobs | Can't pair eyes | Insert our own fences or markers |
| HUD drawn differently | The 3D screen stays flat under Spidy's gate | Drop that gate (planned in step 6) |
| Game updates | Fixed addresses break | Pattern scans (step 7); pin the build |
| Ray tracing tied to the main view | Eyes lack reflections or crash | RT off for the eyes |
| Z13 frame rate | Three views (game + 2 eyes) cost a lot | Lower eye resolution, dynamic resolution, skip the main view's full render if possible |
| Late attach (Linux script attaches at the menu) | Can't enlarge render memory | Launch wrapper that loads early |

**Effort, honestly:** steps 3–5 are reverse-engineering work, measured in weeks of sessions, not days. The binary diff from Remastered is what keeps it from being months. Spidy's author built the original with heavy AI help, and we'd work the same way.

## Machine rules that apply

- **Z13 memory:** Ghidra on a 190 MB exe and the game itself are both heavy. Run one heavy job at a time under the 2 GiB watch, and check `qwen.lock` first.
- **On Hyperion:** builds go through the dev gate (`Workspaces/CLAUDE.md` part 1).
- **Commits:** iannitram no-reply identity, never force-push.
- **Licence:** GPL-3.0 throughout. Our port is a derivative of Spidy, so credit Ilya Mezerowsky in the README and file headers.

## Decisions for Ian

1. **Install Spider-Man Remastered?** It's needed for the baseline (step 0) and for the binary diff in steps 3–5. Roughly 75 GB; it fits on internal storage (237 GB free) or the card (137 GB free).
2. **Where to build the Windows DLLs:** on the Z13 in the `craft-build` distrobox (self-contained, Z13-only), or on Hyperion with Visual Studio (Spidy's native toolchain, less setup)?
3. **Contact Spidy's author?** He's asking for ideas for "the next full VR conversion", and a Spider-Man 2 port overlaps with that. Working with him, or at least telling him, could save weeks.
