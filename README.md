# Flat to 3D

Play installed PC games in **true stereoscopic 3D**, where the game itself draws two camera views (no faked depth). The picture appears on a giant fixed virtual 3D TV in a Quest 2, with a controller in hand. Everything runs on the ROG Flow Z13 under SteamOS and Proton, with the headset on a USB cable.

## Where we're at (2026-10-18)

**Phase: first trial being set up. No game has run in 3D on the Z13 yet.**

The first target is **Marvel's Spider-Man 2**, ported from [SpidyVR](https://github.com/IlyaMez/SpidyVR). Spidy is an open-source VR mod for Spider-Man Remastered, and its **GAME SCREEN 3D** mode is already the 3D TV we want. Step 0 runs Spidy unchanged on Remastered to prove the whole chain on this machine. Everything for it is installed, waiting on a headset session: [docs/STEP0-RUNSHEET.md](docs/STEP0-RUNSHEET.md).

In parallel, a separate session is setting up Ghidra (and REA, if Ian approves its config change) to start mapping `Spider-Man2.exe` against Remastered.

## Done so far

- **Requirements settled** with Ian (vault `DECISIONS.md`): Z13 and SteamOS only, true stereo only, free and open-source only (nothing paywalled, even if already owned), offline games only, a giant fixed 3D TV with no head-steered camera.
- **Research report** on every relevant tool, the Linux VR streaming stack and each Favorite game (vault `reports/True stereo 3D on SteamOS.md`).
- **Library classified:** all 99 installed games have a `## 3D (stereo)` section and `stereo-*` recipe keys on their wiki page. Counts on the wiki: 5 works-now, 36 likely, 6 build, 2 not safe, 49 won't work, 1 untested.
- **Spider-Man 2 route found and planned:** `Spider-Man2.exe` shares Spidy's engine internals but not its addresses. [docs/PORT-SPIDERMAN2.md](docs/PORT-SPIDERMAN2.md) is the 8-step plan, and [docs/spidy-stereo-core-map.md](docs/spidy-stereo-core-map.md) maps Spidy's stereo-screen code.
- **Workbench:** Spidy cross-compiles on the Z13 (clang-cl + xwin in the `craft-build` distrobox), and its 337 unit tests pass under Proton 11's Wine.
- **Step-0 kit installed:** WiVRn 26.9 server (Flatpak), the matching headset APK and the Spidy 0.3.8 release. Spider-Man Remastered is finishing its install.

## Plan going forward

1. **Step 0, Spidy on Remastered:** the game on a 3D screen in the Quest 2 over USB at 30+ fps. Proves Proton + WiVRn + Quest 2 + the technique. Afterwards, file a compatibility report upstream to SpidyVR.
2. **Spider-Man 2 port, steps 1–8** of [the plan](docs/PORT-SPIDERMAN2.md):
   - Map addresses by diffing the two exes in Ghidra.
   - Get inside the game, then move the camera.
   - Render two eyes, pair them in the same frame, and clean up the images.
   - Put them in the headset as a 3D screen.
   - Add the HUD, then pattern scans so game patches don't break it.
3. **Prove the other big buckets** (from the research report):
   - UEVR on an installed Unreal 4 game (Sifu or Little Nightmares).
   - Geo-11 on Control and Planet of Lana. Geo-11 alone covers about 33 library games.
   - The 5-minute check of UEVR's 2D Screen Mode.
4. **Build the generic 3D-screen pieces** that the other mods need:
   - An OpenXR layer that pins any VR mod's two eyes to a fixed screen.
   - A viewer for side-by-side output.
5. **The Inject app:**
   - A command-line engine plus a Decky plugin. It finds installed games, reads each game's wiki recipe and injects the right tool.
   - "Start 3D" brings up Quest Tether and WiVRn.
   - Rai Pal is the model.
6. **Our own stereo for the remaining DX12 games:**
   - 007 First Light's dormant built-in stereo mode first.
   - Then Alan Wake 2.
   - Then a Vulkan-level layer for the rest.
7. **Uncharted 3 through Ian's own Linux port** (`uncharted_pc` on shadPS4):
   - Most of the PS3 stereo 3D mode survives in the PS4 remaster's code: the two-eye camera, the eye-offset maths and side-by-side viewports, all behind one render-mode value.
   - First test: force that mode at boot and look for a side-by-side frame. See [docs/uncharted3-stereo.md](docs/uncharted3-stereo.md).

**Future:** Steam Frame support. Keep the 3D-screen output on plain OpenXR so it can also run through SteamVR.

## Repository layout

| Path | What |
|---|---|
| `docs/PORT-SPIDERMAN2.md` | Spider-Man 2 port plan and its decisions |
| `docs/spidy-stereo-core-map.md` | How Spidy's GAME SCREEN 3D works, with every game-specific address on that path |
| `docs/STEP0-RUNSHEET.md` | Step-by-step for the first headset trial |
| `docs/uncharted3-stereo.md` | What survives of Uncharted 3's PS3 stereo mode in the PS4 build, and the route through the PC port |
| `third_party/SpidyVR` | SpidyVR v0.3.8, pinned submodule |
| `cmake/clang-cl-xwin.cmake` | Cross-compile toolchain (Windows x64 DLLs on Linux) |
| `tools/build-spidy.sh` | Fetches Spidy's pinned dependencies and builds it |

## Building (on the Z13)

Windows DLLs are cross-compiled inside the `craft-build` distrobox with clang-cl and lld-link, against Microsoft's C runtime and Windows SDK. `xwin` unpacked those into `~/.local/share/flat-to-3d/xwin` (Microsoft license accepted).

```bash
git submodule update --init
distrobox enter craft-build -- tools/build-spidy.sh   # output: ~/.cache/flat-to-3d/build-spidy
```

Heavy jobs on the Z13 (builds, Ghidra, the game) run one at a time under a 2 GiB free-memory watch.

## Ground rules

- True stereo only. Depth-buffer reprojection doesn't count.
- Free and open-source tools only. Nothing paywalled.
- Offline and single-player games only. Never inject into a game with anti-cheat or always-online play.
- Flat 2D-art games are out of scope.
- Verified findings that help an upstream project get reported upstream.

## Where the planning lives

Decisions, research and per-game recipes live in Ian's Nextcloud vault:

- `Workspaces/Flat to 3D/README.md`: project overview and status.
- `Workspaces/Flat to 3D/DECISIONS.md`: every ruling and the recipe-key schema.
- `Workspaces/Flat to 3D/reports/True stereo 3D on SteamOS.md`: the research report.
- `Wiki/Technology/Gaming/<Game>.md`: each game's `stereo-*` frontmatter keys are the recipe this app reads.

## Credits and license

GPL-3.0. Spider-Man support builds on [SpidyVR](https://github.com/IlyaMez/SpidyVR) by Ilya Mezerowsky (GPL-3.0). The app design follows [Rai Pal](https://github.com/Raicuparta/rai-pal) (GPL-3.0).
