# Flat to 3D

Play installed PC games in **true stereoscopic 3D**, where the game itself draws two camera views (no faked depth). The picture appears on a giant fixed virtual 3D TV in a Quest 2, with a controller in hand. Everything runs on the ROG Flow Z13 under SteamOS and Proton, with the headset on a USB cable.

## Where we're at

**2026-10-31:** 1.0 now covers **STAR WARS Zero Company** (first, through UEVR) and Spider-Man 2. See [SPEC.md](SPEC.md), [docs/ROADMAP.md](docs/ROADMAP.md) and the peer projects in [PEERS.md](PEERS.md). The notes below are from 2026-10-18.

### 2026-10-18

**Phase: first trial being set up. No game has run in 3D on the Z13 yet.**

The first target is **Marvel's Spider-Man 2**, ported from [SpidyVR](https://github.com/IlyaMez/SpidyVR). Spidy is an open-source VR mod for Spider-Man Remastered, and its **GAME SCREEN 3D** mode is already the 3D TV we want. Step 0 runs Spidy unchanged on Remastered to prove the whole chain on this machine. Everything for it is installed, waiting on a headset session: [docs/STEP0-RUNSHEET.md](docs/STEP0-RUNSHEET.md).

In parallel, the Spider-Man 2 address mapping has started on Hyperion ([docs/spiderman2-address-map.md](docs/spiderman2-address-map.md)). Its matcher (`tools/xmatch/`) is written and lists all 85 items on Spidy's stereo path. Nothing has been mapped yet: both games were still downloading on Hyperion when the first leg ended. Steam's file lists already change the plan in four places:
- Spider-Man 2 draws its UI with **Coherent Gameface, not Scaleform**, so Spidy's HUD hooks and settings tab can't be ported.
- It ships the **PlayStation PC SDK**, so check that it plays offline before injecting anything.
- It ships **NVIDIA's ray-tracing denoisers**, which makes ray tracing off for the eyes likely.
- It uses **DirectStorage**, which is low risk.

## Done so far

- **Requirements settled** ([docs/notes/DECISIONS.md](docs/notes/DECISIONS.md)): Z13 and SteamOS only, true stereo only, free and open-source only (nothing paywalled, even if already owned), offline games only, a giant fixed 3D TV with no head-steered camera.
- **Research report** on every relevant tool, the Linux VR streaming stack and each Favorite game ([docs/notes/research-report.md](docs/notes/research-report.md)).
- **Library classified:** all 99 installed games have a `## 3D (stereo)` section and `stereo-*` recipe keys in my per-game notes (the key schema is in [docs/notes/DECISIONS.md](docs/notes/DECISIONS.md); the per-game table is in [docs/notes/research/library_engine_scan.md](docs/notes/research/library_engine_scan.md)). Counts: 5 works-now, 36 likely, 6 build, 2 not safe, 49 won't work, 1 untested.
- **Spider-Man 2 route found and planned:** `Spider-Man2.exe` shares Spidy's engine internals but not its addresses. [docs/PORT-SPIDERMAN2.md](docs/PORT-SPIDERMAN2.md) is the 8-step plan, and [docs/spidy-stereo-core-map.md](docs/spidy-stereo-core-map.md) maps Spidy's stereo-screen code.
- **Workbench:** Spidy cross-compiles on the Z13 (clang-cl + xwin in the `craft-build` distrobox), and its 337 unit tests pass under Proton 11's Wine.
- **Step-0 kit installed:** WiVRn 26.9 server (Flatpak), the matching headset APK and the Spidy 0.3.8 release. Spider-Man Remastered is finishing its install.

## Plan going forward

1. **Step 0, Spidy on Remastered:** the game on a 3D screen in the Quest 2 over USB at 30+ fps. Proves Proton + WiVRn + Quest 2 + the technique. Afterwards, file a compatibility report upstream to SpidyVR.
2. **Spider-Man 2 port, steps 1–8** of [the plan](docs/PORT-SPIDERMAN2.md):
   - Map addresses by diffing the two exes. The offline matcher is ready (`tools/xmatch/run_match.py`, pattern scans included) and runs as soon as both installs finish on Hyperion. Ghidra is the cross-check.
   - Before step 2, confirm Spider-Man 2 runs with no PSN sign-in (offline rule).
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
5. **Plug in and play:**
   - Steam launches the games. Plugging in the headset will start the `flat3d-link` service, which brings up the link.
   - The `flat3d-run` wrapper will check the link and each game's 3D switch, then apply that game's 3D route. If the link is not ready or the switch is off, the game runs flat.
   - The engine will include Quest Tether's tether, ported to Rust.
6. **Our own stereo for the remaining DX12 games:**
   - 007 First Light's dormant built-in stereo mode first.
   - Then Alan Wake 2.
   - Then a Vulkan-level layer for the rest.
7. **Uncharted 3 through my own Linux port** (`uncharted_pc` on shadPS4):
   - Most of the PS3 stereo 3D mode survives in the PS4 remaster's code: the two-eye camera, the eye-offset maths and side-by-side viewports, all behind one render-mode value.
   - Tested on the Legion Go (2026-10-18): the game's own two-eye camera works as alternate-frame stereo (each eye 30 Hz at 60 FPS); the side-by-side path does not draw a second view. Whether alternate eyes count as true stereo is pending my ruling. See [docs/uncharted3-stereo.md](docs/uncharted3-stereo.md).

**Future:** Steam Frame support. Keep the 3D-screen output on plain OpenXR so it can also run through SteamVR.

## Repository layout

| Path | What |
|---|---|
| `docs/PORT-SPIDERMAN2.md` | Spider-Man 2 port plan and its decisions |
| `docs/spidy-stereo-core-map.md` | How Spidy's GAME SCREEN 3D works, with every game-specific address on that path |
| `docs/spiderman2-address-map.md` | Those addresses, Remastered → Spider-Man 2, with pattern scans and confidence |
| `tools/xmatch/` | Offline Remastered → SM2 matcher, step-0 exe check, single-exe survey |
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

Decisions, research and notes are in this repo under [docs/notes/](docs/notes/):

- [docs/notes/DECISIONS.md](docs/notes/DECISIONS.md): every ruling and the recipe-key schema.
- [docs/notes/research-report.md](docs/notes/research-report.md): the research report.
- [docs/notes/research/](docs/notes/research/): the raw research notes behind the report (tool landscape, Linux VR stack, per-game notes, library engine scan).
- [docs/notes/library-scan.tsv](docs/notes/library-scan.tsv): the installed-games list.
- Status and the plan: [docs/ROADMAP.md](docs/ROADMAP.md) and this repo's issues.

## Credits and license

GPL-3.0. Spider-Man support builds on [SpidyVR](https://github.com/IlyaMez/SpidyVR) by Ilya Mezerowsky (GPL-3.0). The app design follows [Rai Pal](https://github.com/Raicuparta/rai-pal) (GPL-3.0).
