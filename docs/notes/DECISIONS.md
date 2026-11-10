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
| 5 | Headset link | *(Link layer superseded 2026-11-01 by ruling 30: the tether is now ported into the engine.)* **Quest Tether** (SteamOS port + Decky plugin) stays the USB link layer, and a Decky "Start 3D" button comes in the build phase. Research picked **WiVRn** as the streamer: its wired mode (`adb reverse tcp:9757`) runs alongside Quest Tether's internet sharing. Steam Link VR is the fallback. |
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
| 24 | Plug-in | **Plug in and the 3D link starts by itself** (2026-11-01, question tool: "Start right away (Recommended)"), with a toast when ready; unplug runs Stop 3D. |
| 25 | Unplug mid-game | **The game keeps running and the link waits about 2 minutes for a replug** (2026-11-01: "Keep game, wait for replug (Recommended)"); after that Stop 3D and a "restart for flat" toast. |
| 26 | Controls | **A per-game 3D switch, on by default; no Inject button** (2026-11-01: "Per-game 3D switch (Recommended)"). Headset ready + switch on = 3D, otherwise flat. |
| 27 | Launch option | **The Decky plugin adds `flat3d-run %command%` itself** when a recipe reaches works-now, keeping existing options (2026-11-01: "Plugin adds it (Recommended)"). |
| 28 | Late plug-in | **Plugging in while a 3D-ready game runs flat brings the link up and toasts "restart for 3D"**; the game is left alone (2026-11-01: "Toast, leave game (Recommended)"). |
| 29 | Desktop app | **The desktop app shrinks to Setup, Headset, the per-game switches with recipes, and Logs**; Steam launches the games (2026-11-01: "Shrink it (Recommended)"). |
| 30 | Quest Tether in the app | **Quest Tether's tether is built into Flat to 3D, ported to Rust** (2026-11-01: "I think we should build the Quest tether INTO this app given the necessity of plugging in."; ruling "Port it into Rust"). Internet over the cable is **on by default** ("On by default (Recommended)"). Replaces ruling 5's separate Quest Tether link layer. |
| 31 | Coexistence | **Quest Tether keeps its own Linux app** ("Keep both on Linux"); when it is already tethering, **Flat to 3D uses its tunnel and adb and adds only WiVRn + `adb reverse`** ("Use its tunnel (Recommended)"). |
| 32 | Headset health | **Battery and the green/yellow/red temperature in the panel, with a toast at 38 C and at 42 C during play** (2026-11-01: "Yes, with a warning (Recommended)"). |
| 33 | Port scope | **All of Quest Tether's link for 1.0:** detect, heal, tunnel with an honest tun0 state, Wi-Fi-first disconnect, NetworkManager Wi-Fi sync, health, throughput/uptime/reconnects and the soak check (2026-11-01, multi-select: "Detect, heal, tunnel (Recommended)", "Headset health", "Soak + stats", "Wi-Fi sync"). |
| 34 | adb | **One adb, never in git:** reuse a running server (Quest Tether's included), else the bundled adb **built from Android's open source (Apache-2.0)**, so no SDK licence (2026-11-01: "Find it, don't ship it (Recommended)", then "Quest Tether's adb (Recommended)", then "Open-source adb (Recommended)"). |
| 35 | Manual control | **Start 3D / Stop 3D stay as manual buttons, plus an "Auto 3D on plug-in" switch, on by default** (2026-11-01: "Keep both + auto switch (Recommended)"). Auto off = plugging in only tethers. |
| 36 | QAM layout | **Top to bottom: link state + Start/Stop 3D, headset health, tunnel and Auto 3D switches, then the 3D Ready games with their switches** (2026-11-01: "That order (Recommended)"). |
| 37 | Steam library | **An automatic "3D Ready" Steam collection; no badges, cover art untouched** (2026-11-01: "Auto '3D Ready' (Recommended)", "Collection is enough (Recommended)"). |
| 38 | Which games | **Only works-now games get the wrapper and switch in 1.0** (2026-11-01: "works-now only (Recommended)"). |
| 39 | Desktop app toolkit | **A native Rust GUI, not Tauri and not a web UI** (2026-11-01, question tool: "Native Rust GUI", toolkit "iced (Recommended)"). The app is a Rust crate `app/` (iced, MIT) that depends on `inject/` by path; its Elm-style state and `update` functions are plain Rust and unit-tested in CI. Packaging is an AppImage of that binary (no WebKitGTK). Supersedes "Tauri (Rust + web UI) (Recommended)" of 2026-10-31. |
| 40 | Decky plugin | **A Rust backend and a thin TypeScript panel** (2026-11-01, question tool: "Rust backend, thin TS panel (Recommended)"). No Python backend: the backend is the engine binary (the PowerTools way). The panel has no logic: it draws the engine's `--json` and calls Steam's client API (launch options, `collectionStore`, toasts) only because only JavaScript can. The logic that was planned for `ui-shared/` (status labels and colours, sort, link wording, launch-option compose and remove, collection diff, toast selection) lives in Rust in `inject/` and reaches the panel through `--json`. The `ui-shared/` package is dropped. |
| 41 | Other languages | **The Spider-Man 2 port stays C++ on SpidyVR and the research tools stay Python** (2026-11-01, question tool: "Leave them as is (Recommended)"). |

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
