# Flat to 3D: decisions

Every ruling I have made on the project. Rulings 1-15 came from a grilling Q&A on 2026-10-18, before any research; the rest followed the same day. Current status and the plan are in the repo [README](../../README.md) and [ROADMAP](../ROADMAP.md).

## The goal

Play installed games in **true stereoscopic 3D**: the game draws two separate camera views. The picture appears on a giant virtual 3D TV in the Quest 2, seated, controller in hand. Head movement does not steer the game camera.

## Rulings

| # | Topic | Decision |
|---|---|---|
| 1 | Render machine | **Z13 only.** |
| 2 | Operating system | **SteamOS only**, games through Proton. No Windows dual-boot. |
| 3 | What counts as depth | **True stereo only** (two cameras). Depth-buffer reprojection (SuperDepth3D-style) does not count. Whether alternate-eye rendering (AER) counts is **not yet ruled**. Since ruling 16 it matters only as a fallback for our own builds. |
| 4 | Experience | **Giant 3D TV.** A fixed screen, no head-steered camera. |
| 5 | Headset link | **Quest Tether** (SteamOS port + Decky plugin) stays the USB link layer, and a Decky "Start 3D" button comes in the build phase. Research picked **WiVRn** as the streamer: its wired mode (`adb reverse tcp:9757`) runs alongside Quest Tether's internet sharing. Steam Link VR is the fallback. |
| 6 | Frame rate | **A steady 30+ fps** is fine. Lower resolution and settings are acceptable. |
| 7 | Online / anti-cheat | **Offline and single-player only.** Any game with anti-cheat or always-online play is marked "not safe" and skipped. Forza Horizon 6 is not safe. |
| 8 | Money | **Free only.** Existing open-source VR mods for specific games are the baseline: keep their injection method, switch the output to a 3D screen. |
| 9 | Build vs adopt | Adopt existing tools and build the glue. Where no path exists for a game with a **real 3D world**, we **build our own** stereo. Flat 2D-art games (Hades, Hades II) are marked "won't work", and that is fine. |
| 10 | End product | A **Rai Pal-style app**: a command-line engine plus a **Decky plugin** in Game Mode. It detects installed games and shows an **Inject** button. |
| 11 | Recipe source | Each game carries machine-readable **header keys** (schema below). The app reads those as its recipes. |
| 12 | Per-game notes format | Header keys plus a `## 3D (stereo)` section after the shared quirks. Z13 test results go in the Z13 section. |
| 13 | Library scan | **A record for every installed game**, including "won't work" ones. Deep notes for the 10 Favorites. |
| 15 | Phasing | Research first, then build. Research is done. The build phase opened with the Spider-Man trial (ruling 17). |
| 16 | Paywalled tools | **No paywall-guarded tools, even ones I already own** (my old R.E.A.L. VR copies included). Open-source tools or our own code only. |
| 17 | First trial | **Spider-Man.** Spider-Man 2 is built from **SpidyVR** (GPL-3.0, a Spider-Man Remastered VR mod). Its "VR VIEW: GAME SCREEN" + "GAME SCREEN 3D" mode is already the 3D TV. Step 0 is Spidy itself on Remastered; then its stereo-screen core is ported to Spider-Man 2. |
| 18 | Code repo | **github.com/iannitram/flat-to-3d**, private, GPL-3.0 (it reuses GPL code from SpidyVR and Rai Pal). Commits use the iannitram no-reply identity. |
| 19 | Future feature | **Steam Frame compatibility.** Keep the 3D-screen output on standard OpenXR, not WiVRn-only, so it can later run on Valve's Steam Frame through SteamVR. |
| 20 | Spider-Man Remastered | **Installed on the Z13's internal drive** for step 0 and for diffing its exe against Spider-Man 2's. |
| 21 | Build toolchain | **Windows DLLs build on the Z13** in the `craft-build` distrobox with clang-cl + lld-link, using Microsoft's C runtime and Windows SDK fetched by `xwin`. I accepted Microsoft's license for that download. |
| 22 | Reverse-engineering tools | **REA** (github.com/morluto/rea, MIT, drives Ghidra) is approved in principle for mapping Spider-Man 2 and for performance work, since the mod is for my own use only. Registering its MCP server in Claude Code's config needs a separate yes. |
| 23 | Upstream reports | **File verified, useful reports upstream** (SpidyVR, WiVRn, UEVR...) from the iannitram account without asking each time, and tell me the link. Pull requests still get a heads-up first. |

Ruling 14 (where decisions, research and status are kept) is retired: they now live in this repo, under `docs/notes/`.

## Recipe keys (header keys on every game record)

| Key | Allowed values |
|---|---|
| `stereo-status` | `works-now` · `likely` · `build` · `not-safe` · `wont-work` · `untested` |
| `stereo-route` | short tool name(s), e.g. `uevr + vrto3d`, `geo-11`, `build: port spidy stereo screen`, `none` |
| `stereo-engine` | e.g. `unreal 4.26`, `unity 2020.3 il2cpp`, `northlight`, `custom` |
| `stereo-api` | `dx9` · `dx11` · `dx12` · `vulkan` · `opengl` · `unknown` |
| `stereo-mod` | URL of the baseline mod or tool, or `none` |
| `stereo-free` | `"yes"` · `"no"` · `unclear` (quoted so YAML keeps them as text; a build-our-own route counts as `"yes"`) |
| `stereo-safe` | `"yes"` · `"no"` (`"no"` = anti-cheat or always-online) |
| `stereo-checked` | date of the last research pass, `YYYY-MM-DD` |

Each game's notes also carry a `## 3D (stereo)` section. The per-game records themselves (one page per game, 99 games) were kept outside this repo; [library-scan.tsv](library-scan.tsv) lists the games they cover, and [research/library_engine_scan.md](research/library_engine_scan.md) holds the per-game classification table. Moving the records into the repo is part of building the Inject app.

## Facts the rulings rest on

- **Library:** all 99 installed games were classified on 2026-10-18, 5 of them with newly written notes.
- **Installed on the Z13** (2026-10-18): 98 Steam games across internal storage and the `FF8S5` card ([library-scan.tsv](library-scan.tsv)), plus Alan Wake 2 and Baldur's Gate II through Heroic. ACE COMBAT 7 is on the `sandisk_4tb` library, which wasn't mounted.
- **Quest Tether** on SteamOS reverse-tethers the Quest over USB with gnirehtet (measured ~28 Mbit/s with Steam Link). WiVRn's wired mode needs only `adb reverse`, so Quest Tether isn't required for a WiVRn session.
- **Spider-Man 2** (`Spider-Man2.exe` v2.810.0.0) contains the same engine system names SpidyVR hooks in Remastered, with no packer. Its C++ type information is stripped.

## Favorites and verdicts

| Game | Verdict | Route |
|---|---|---|
| Marvel's Spider-Man 2 | build (first trial) | port SpidyVR's stereo screen |
| ACE COMBAT 7 | likely, best candidate | UEVR (not mounted right now) |
| Control | likely | Geo-11 + the community Control fix, DX11 mode |
| Planet of Lana | likely | Geo-11 fix |
| STAR WARS Zero Company | likely, uncertain | UEVR nightly (UE 5.6) |
| 007 First Light | build, strong lead | its own dormant stereo mode |
| Alan Wake 2 | build, later | our own DX12 hook |
| Forza Horizon 6 | not safe | none |
| Hades, Hades II | won't work | none |

Open questions: does AER count as true stereo (fallback for our own builds)? Should I contact SpidyVR's author about the Spider-Man 2 port? Go-ahead to register REA in Claude Code's config.
