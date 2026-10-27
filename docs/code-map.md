# Code map

One row per part of the repo: what it owns, who uses it and which Must it serves (SPEC.md). Written 2026-10-26 from the tree at main. The deep map of Spidy's stereo core is [spidy-stereo-core-map.md](spidy-stereo-core-map.md), and the Spider-Man 2 address map is [spiderman2-address-map.md](spiderman2-address-map.md).

Platforms (R04): the host is Linux (SteamOS on the owner's Z13) only (rulings 1, 2). The game DLLs are Windows binaries, cross-built on the Z13 with clang-cl + xwin and run under Proton. Mac and Windows hosts don't apply. CI runs the engine's tests on `ubuntu-latest`; the DLL build and every game test need the Z13.

## How it fits together

```mermaid
flowchart TD
    QAM[decky/ QAM panel: Start 3D, Inject] --> Back[decky/main.py backend]
    Back --> CLI[inject/cli.py: list, inject, start-3d]
    CLI --> Steam[inject/steam.py: installed games]
    CLI --> Rec[inject/recipes.py: stereo keys]
    CLI --> Inj[inject/injectors.py: route per recipe]
    CLI --> Link[inject/link.py: Quest Tether + WiVRn wired mode]
    Rec --> RecFiles[(recipes/)]
    Inj --> Port[games/spiderman2/: the port's DLLs + launch script]
    Port --> Spidy[third_party/SpidyVR: the baseline, v0.3.8]
    Build[tools/build-spidy.sh + cmake/clang-cl-xwin.cmake] --> Port
    Match[tools/xmatch + tools/ghidra + tools/research] -->|addresses| Port
```

## Parts

| Path | Status | Owns | Musts |
|---|---|---|---|
| `third_party/SpidyVR` | present (submodule, v0.3.8) | the baseline mod (GPL-3.0): injection, stereo screen, probes | M1, M3-M8 |
| `cmake/clang-cl-xwin.cmake`, `tools/build-spidy.sh` | present | cross-building Windows DLLs on Linux in the `craft-build` distrobox (ruling 21) | M3-M8 |
| `tools/research/` | present | Spidy's discovery tools, with the Remastered hash gates removed so they read any PE32+ | M3-M5 |
| `tools/xmatch/` | present | the offline matcher from Remastered's addresses to Spider-Man 2's (`run_match.py A.exe B.exe out.json`) | M3-M6, M8 (pattern scans) |
| `tools/ghidra/headless.sh` | present | headless Ghidra as one heavy job, with a memory guard | M3-M6 |
| `games/spiderman2/` | planned | the port: render probe, camera shift, eye views, pairing, the OpenXR worker, HUD and memory, launch script | M3-M8 |
| `inject/` | skeleton | the engine: `steam.installed_games`, `recipes.load_recipes`, `injectors.inject`, `link.start_3d` / `start_3d_commands`, `cli` (`python3 -m inject list / inject <appid> / start-3d`) | M9, M10, M12 |
| `recipes/` | skeleton | one file per game with the eight stereo keys; format and canonical home are #15's question | M12 |
| `decky/` | skeleton (`plugin.json`, `main.py`) | the Game Mode plugin: backend calls the engine; the QAM panel (`decky/src/`) is planned | M11 |
| `tests/` | present | engine tests, stdlib `unittest`, no games or hardware | M9, M10, M12 |
| `docs/` | present | rulings (`notes/DECISIONS.md`), research, the port plan, Step 0 runsheet, maps | all |

## Open-source pieces

- [SpidyVR](https://github.com/IlyaMez/SpidyVR) (GPL-3.0): the baseline and the code the port comes from (ruling 17).
- [WiVRn](https://github.com/WiVRn/WiVRn): the streamer, used in its wired mode (ruling 5).
- Quest Tether (our own repo): the USB link layer that Start 3D brings up.
- [Rai Pal](https://github.com/Raicuparta/rai-pal) (GPL-3.0): the reference for game detection and the Inject flow (ruling 10). Its logic is ported to Python, not linked.
- [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader): the Game Mode host.
- Ghidra and REA (ruling 22): reverse engineering for the address map.
