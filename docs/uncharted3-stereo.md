# Uncharted 3: the PS3 stereo 3D mode in the PS4 remaster

Checked 2026-10-18 against Uncharted 3 from The Nathan Drake Collection (PS4, version 1.02), reading the
executable statically. Nothing was run. The question was whether the PS3 game's stereoscopic 3D mode is
still in the remaster's code and could feed Flat to 3D through Ian's own Uncharted PC port
(`uncharted_pc`, which runs the PS4 build natively on Linux through shadPS4's Vulkan backend, with no
Proton, on the Legion Go and the Z13 under SteamOS).

Labels: **measured** means read in the code. **Inferred** means a reading of what the code is for.
**Guess** means not yet supported by evidence.

## Update 2026-10-18 afternoon: tested on the Legion Go — the game's own stereo camera works

Three private patch versions were tried through the port's new per-executable patch loader (Legion Go, SteamOS, 800p, 60 Hz):

- **v1** (force stereo mode from boot): crashed at once. In the remaster, ~15 draw sites pass a null window context in multi-view modes (the text renderer asserts).
- **v2** (v1 + a guard in the text renderer): ran at 60 FPS with no crash, but the remaster's renderer only ever draws window 0 (a fatal assert guards it), so the scene stayed single; only the UI doubled (top/bottom).
- **v3, alternate-frame stereo** (normal mode; the game's own two-eye camera is built every frame and view 0 flips between the left and right eye each frame): **works.** Ian, watching the screen with the separation exaggerated ×5 for the test: "it looks like I'm crosseyed. Rapidly opening and closing my left and right eye." That is the expected look of frame-sequential stereo viewed flat: the game itself draws each eye from its own offset camera (true stereo, not depth reprojection), at 30 Hz per eye when the game runs at 60.

**So the route changes:** not side-by-side halves from the game, but **frame-sequential eyes** at 60 Hz. Flat to 3D's presenter takes even/odd frames as left/right and shows them on the fixed 3D screen. Open: eye-order detection (a known parity, or a marker the port can read), TAA/motion-blur ghosting between eyes, the normal separation multiplier, and whether "alternate eyes from the game's own camera" counts as true stereo under the project's rules (recommended: yes; Ian to confirm).

## Verdict (static analysis, before the test)

**Most of the stereo path survives (verdict b, strong form). The output-to-3D-TV side is gone.**

- Confidence is **high** that the stereo mode setting, the two-eye camera build, the eye-offset maths,
  the side-by-side viewport layout and the per-eye HUD code are still compiled in. All of them are
  switched by one render-mode value (measured).
- Confidence is **medium** that setting that value would draw a usable side-by-side frame (inferred).
- Confidence is **low** that it would be clean without fixes. Bluepoint's PS4 post-processing and the
  untested two-view GPU path are the unknowns (guess).

Uncharted 2 (same collection) has only the shared-engine leftovers: the same mode tests in shared
library code and a stereo screenshot-capture path. It has none of the game-side pieces, so it is not a
candidate. Uncharted 1 was not checked.

## What survives in Uncharted 3

1. **A render-mode setting** in the global render-options block (measured). From how the code uses it
   (inferred): 1 is normal, 2 and 3 are split-screen (side-by-side and top/bottom), 4 and 5 are stereo
   3D, and 6 is a stereo capture mode for the developers' frame dumper.
   - An options-menu item type with exactly two values, 1 and 4, writes the requested mode (measured).
     This is most likely the PS3 "3D off / on" option (inferred).
   - Developer command-line switches also set it: `3d` gives 5, and `capture3d` / `capture2d` give the
     capture modes (measured).
2. **A script query, `is-3d?`**, which returns "mode is 4" (measured). It is not a system query and
   not a constant: the game asks the PS4 nothing about 3D displays, and there are no 3D, VR or HMD
   imports (measured).
3. **The two-eye camera build** (measured). In stereo modes the scene-window code copies the one final
   camera into two views, marks one as the left eye and one as the right, and stores them in the
   engine's two view slots. Those are the same slots split-screen uses.
4. **Off-axis (parallel-axis plus asymmetric-frustum) stereo maths** in the view builder (measured
   arithmetic, inferred meaning). Each eye moves along the camera's right axis, and the projection
   shifts the opposite way. The inputs are:
   - a separation and a convergence value, computed each frame from camera data with defaults in the
     render options;
   - a 65 mm eye constant;
   - a screen-size field that defaults to a 30-inch TV. Nothing in the PS4 build fills that field, so
     the default applies.
5. **Side-by-side viewports** (measured). In stereo modes the left eye draws into the left half of the
   frame and the right eye into the right half: half-width viewports in one render target.
6. **Per-eye HUD placement** (measured). Screen-space markers are projected for both eyes and placed
   between them.
7. **A 3D render-settings preset** (inferred). Stereo modes pick a different data-driven render
   settings entry, probably the PS3's lighter-quality 3D preset.
8. **Developer extras** (measured). A "camera horizontal 3D offset" debug tweak shifts the single
   camera sideways, and the frame dumper has a `stereo3d` output folder.

## What is gone or unverified

- **Gone** (measured, string and import scan): any frame-packed or HDMI 3D output, any 3D display
  capability check, and any named stereo or convergence settings.
- **Unverified:**
  - Whether the GPU-side render loop really draws both views in this build. Split-screen multiplayer
    was cut from the Collection, so the remaster never exercises this loop (inferred).
  - How Bluepoint's added full-screen effects behave with half-width viewports.
  - Whether the 3D render-settings preset exists in the PS4 data.
  - Exactly where the requested mode becomes the live mode.

## Proposed route through the Uncharted PC port

Whether this fits Flat to 3D's "true stereo only" rule is pending Ian's ruling (the game draws each eye from its own camera, but in alternate frames rather than both per frame). Testing on the Legion Go was a works/doesn't-work check; 3D runs belong on the Z13 per the project's "Z13 and SteamOS only" rule.

**A. Native side-by-side.** Try this first.
1. Force the render mode to 4 at boot from the port side. Use 4 rather than the `3d` switch's 5,
   because scripts treat 5 as not-3D.
2. The game draws left and right eyes into the two halves of its normal 1920x1080 frame.
3. The port takes the flipped frame and passes it on as side-by-side-half. Either the planned
   side-by-side viewer or the fixed 3D-screen presenter can show it in the Quest 2.
4. Eye separation and convergence come from the game's own camera data. The port can tune them
   through the screen-size field and the separation and convergence defaults.

**B. Fallback: alternate frames.** Keep normal mode and drive the debug camera offset left and right
on alternate frames, then pair the frames. This works even if route A's GPU path is broken. The costs
are half the frame rate per eye and no convergence shift, which the presenter can approximate with a
horizontal image shift.

**C. Later: full resolution per eye.** Widen the stereo viewports and send the second eye to its own
render target. This needs a second target and a hook between the two views in the port. Not scoped.

**First test (cheap; done 2026-10-18, see the update at the top):** boot the port with the mode forced to 4 and take a screenshot. Two slightly
shifted half-width images mean route A is live.

## Risks

- **An untested path.** Two-view rendering has not run in the remaster, so expect crashes or
  corruption from shadPS4 or from the game.
- **Seams.** Full-screen post effects may bleed across the middle of the frame.
- **Missing data.** If the 3D render-settings preset is absent from the PS4 data, the game may fall
  back or fail.
- **Mode changes.** Switching mode mid-game may need render targets rebuilt, so set it at boot.
- **Frame time.** The scene is drawn twice, so the 60 fps target on the Legion Go or Z13 is at risk.
  Half-width eyes lower the effective resolution.
- **Behaviour changes.** Script code that asks `is-3d?` may change cinematics or gameplay as it did
  on PS3.
- **Flat video.** Pre-rendered cutscene videos stay flat.

## Reference (Uncharted 3, 1.02, patch addresses = PS4 image offset + 0x400000)

| Item | Address |
|---|---|
| Render-options block (mode at +0x0, requested mode at +0x8) | 0x01CC6280 |
| `is-3d?` native function | 0x015D7800 |
| Two-eye mode switch in scene-window view setup | 0x00971DC9 |
| Stereo view builder (eye offset maths around +0x82B) | 0x015C0D30 |

Detailed working notes stay private with Ian's local copies, not in this repo.
