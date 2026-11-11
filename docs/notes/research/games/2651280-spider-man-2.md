# Marvel's Spider-Man 2 (2651280): research for the 1.0 port route

Research date: 2026-11-11. Render machine: the handheld (AMD integrated graphics, SteamOS, Proton). Viewer: Quest 2 over WiVRn.

This note adds to the Spider-Man 2 section of [games_group_b.md](../games_group_b.md) and to the port plan in [PORT-SPIDERMAN2.md](../../../PORT-SPIDERMAN2.md). It does not repeat them. What those two files already settle: the engine is Insomniac's own, the game is DirectX 12 only, Geo-11 cannot work (it is DirectX 11 only), no free 3D Vision fix exists, and R.E.A.L. VR is ruled out by [DECISIONS.md](../../DECISIONS.md) (rulings 8 and 16).

Each fact below is marked **observed** (read from the installed copy or from a file in this repo, read-only, on 2026-11-11), **source** (an outside page, linked), or **inferred** (my reasoning from the other two). Nothing here was run: no game was started, no mod was downloaded, and nothing was written into the game folder or the Proton prefix.

## Short answer

- The installed copy is the exact build the port needs: version 2.810.0.0, the same SHA-256 that [sessions/sm2-offline.toml](../../../../sessions/sm2-offline.toml) expects and that the one public Spider-Man 2 VR port (HEROVR) requires.
- The game has no anti-cheat, no packer and no Steam DRM wrapper on the exe. Nothing found blocks loading a DLL into it.
- No public report shows SpidyVR 0.7.0 or later, or HEROVR, running Spider-Man 2 under Proton with WiVRn. SpidyVR's Linux path is proven only for Spider-Man Remastered, on a Quest 3 with a desktop graphics card. HEROVR is Windows only.
- A shorter route than the full port does not exist inside the current rulings. Two shortcuts are worth one cheap check each (section 4), but neither replaces the port.
- Keep the route `build: port spidy stereo screen`. The largest risk is frame rate: three views of this game on integrated graphics.

## 1. Identity

| Item | Value | Kind |
|---|---|---|
| Steam appid | 2651280 | source: [Steam store data](https://store.steampowered.com/api/appdetails?appids=2651280) |
| Released on PC | 2025-01-30 | source: same |
| Developers | Insomniac Games, Nixxes Software (the PC port) | source: same |
| Engine | Insomniac Engine, with the Nixxes PC platform layer | source: [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Marvel%27s_Spider-Man_2) |
| Graphics API | Direct3D 12 only. No Vulkan, no DirectX 11 | source: same |
| Process name | `Spider-Man2.exe` (one exe, no separate launcher process when started with `-nolauncher`) | observed |
| Build on disk | version 2.810.0.0, Steam build id 24651585 | observed |
| Launcher | A settings launcher window opens first; the game's own switch `-nolauncher` skips it | observed (switch is in the exe and in the owner's launch options) |
| Account | PlayStation Network sign-in is optional | source: PCGamingWiki; observed in the game log (below) |
| DRM | Steam only. The store data lists no third-party DRM and no outside account | source: Steam store data |
| Anti-cheat | None | source: PCGamingWiki (field empty); observed: no anti-cheat file in the install |
| Blocks DLL loading? | No sign of it | inferred, see section 2 |
| Offline play | Single-player game; network-off start is still untested | see issue #7 |
| ProtonDB | Gold, 111 reports, best reported Platinum | source: [ProtonDB summary](https://www.protondb.com/api/v1/reports/summaries/2651280.json) |
| Steam Deck rating | Playable ("default configuration is not performant") | source: Steam store data |

Middleware named by PCGamingWiki and matched by the files on disk (observed): Havok, Wwise, Coherent Gameface (the HUD and menus), Bink Video.

## 2. The installed copy

The game is installed: `<Steam library>/steamapps/common/Marvel's Spider-Man 2/`, about 109 GB. All of this section is observed unless marked.

### The exe, and what port Step 2 (issue #18) needs from it

| Item | Value |
|---|---|
| File | `Spider-Man2.exe`, 194,262,992 bytes |
| SHA-256 | `c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9` |
| File version | 2.810.0.0 |
| Built | 2026-08-10 (PE time stamp) |
| Image base | `0x140000000` |
| Size of image in memory | 218,533,888 bytes (`0xD069000`) |
| Sections | 11: `.text`, `BINK`, `.rdata`, `.data`, `.pdata`, `BINKDATA`, `BINKCONS`, `UserSyst`, `_RDATA`, `.rsrc`, `.reloc` |
| Signature | Authenticode certificate present |
| Symbols | Not shipped. The exe names `Spider-Man2.pdb` from a `Final_Steam` build folder, but no PDB is in the install |
| Type names | C++ run-time type names are present (thousands of `.?AV...` strings), for example `TimeScaleSystem`, `UISystemMenu...`, `PlayerModelHudFollower...` and photo mode types |

Issue #18 asks for five things. Where each stands:

1. **Record the hash and version.** Done above. They match the `expect` value in [sessions/sm2-offline.toml](../../../../sessions/sm2-offline.toml).
2. **Keep a copy of the exe.** A copy with the same hash already exists in the owner's analysis folder outside the repo.
3. **Turn off auto-updates.** Not done. The Steam app manifest still has `AutoUpdateBehavior 0` (update automatically). The last update landed on 2026-10-05. One update changes the exe and every address in the map.
4. **Adapt the attach script.** SpidyVR's script and probe hard-code Spider-Man Remastered in four places (read from the `third_party/SpidyVR` copy at v0.3.8):
   - `tools/spidy-linux.sh`: the Steam bus name `com.steampowered.App1817070` and the process match `Spider-Man\.exe`. For this game: `App2651280` and `Spider-Man2\.exe`.
   - `src/render_probe.cpp`: the module name `Spider-Man.exe`, and the image size limit `140496896` used to bound the code scan. For this game: `Spider-Man2.exe` and `218533888`.
   - `src/stereo_probe.cpp`: the same module name once more.
   - `tools/inspect_game.py`: a hash check for the Remastered exe.
5. **Load only `render_probe` and log the Direct3D 12 queue.** Needs the device. No session file exists for this step yet (section 6).

### Protection

- No packer: `.text` has normal code entropy (6.65), and the section list is the plain compiler set plus Bink.
- No Steam DRM wrapper: there is no `.bind` section. The exe imports `steam_api64.dll` in the normal way.
- The only anti-debug string is the standard `IsDebuggerPresent` import.
- HEROVR loads its DLL into this same build on Windows and requires this same SHA-256 (source: [HEROVR README](https://github.com/lecutis7of9/HEROVR)).

Inferred: loading a DLL from outside is not blocked. This agrees with the group B note.

### Graphics parts shipped beside the exe

| Part | File version |
|---|---|
| Direct3D 12 Agility SDK (`D3D12Core.dll`) | 1.613.1.0 |
| AMD FSR (`amd_fidelityfx_dx12.dll`), upscaling and frame generation | 1.0.1.41314 |
| Intel XeSS (`libxess.dll`) | 2.0.1.34 |
| NVIDIA DLSS, ray reconstruction and frame generation (`nvngx_dlss*.dll`) | 310 series |
| NVIDIA Streamline (`sl.*.dll`) | 2.7.2.0 |
| NVIDIA ray-tracing denoiser (`NRD.dll`, `NRI.dll`) | 4.9.1.0, 1.126.0.0 |
| DirectStorage (`dstorage.dll`) | 1.2.2504.401 |
| Coherent Gameface (`cohtml.WindowsDesktop.dll`) | 1.38.5.6 |
| Bink 2 | 2.7.13.0 |
| Steam API | 8.63.11.84 |
| PlayStation PC SDK runtime (installer in the game folder) | 3.50.00.11 |

The upscaler and frame-generation DLLs are delay-loaded, so they load only when chosen. The game has its own switch `-noStreamline`.

### Settings and switches

- Settings are not in a file. They live in the Windows registry of the Proton prefix, under `HKCU\Software\Insomniac Games\Marvel's Spider-Man 2\Graphics`, with values such as `UpscaleMethod`, `FrameGen`, `RTReflections`, `AA`, `ExclusiveFullscreen`.
- **No stereo or VR value exists** in that key, and none is in the list of setting names inside the exe.
- Switches found in the exe: `-nolauncher`, `-noStreamline`, `-fixed30hz`, `-benchmark`, `-resetSettings`, `-d3ddebug`, `-windowx`, `-windowy`, `-pixelAspectRatio`, `-tool_cam`.
- The owner's Steam launch options today: two FSR 4 upgrade variables, then `%command% -nolauncher`. No VR or OpenXR variable is set.

### What the game's own log shows under Proton

The last session log (2026-11-07, GE-Proton 11, the handheld's Radeon graphics on the open Mesa driver) shows:

- `Build: v2.810.0.0`, distribution platform Steam.
- Streamline was switched off by the game itself. FSR upscaling and FSR frame generation were in use.
- The PlayStation PC SDK failed to start (`scePsPcInitialize failed: 8a800111`), and the session still ran for about 18 minutes. So a failing PlayStation layer does not stop the game.
- The network was on in that session, so this does not answer issue #7 (network-off start).

### Stereo and VR leftovers in the exe (new)

No OpenXR, OpenVR or headset code strings are present.

The exe does carry text from an old **NVIDIA 3D Vision and side-by-side stereo** path in the Nixxes platform layer:

- "Changed stereo separation.", "Changed stereo popout.", "Changed stereo near separation multiplier."
- "Disabling side-by-side stereoscopic 3d mode, we're running in windowed mode."
- "Stereoscopic 3D has been disabled because exclusive fullscreen was enabled (DX12)."
- `NvAPI_Stereo_...` error texts, and the names `LinkStereoPairs` and `Wants3DModeVisible`.
- The game's own log prints `stereo = 0` for every display mode it lists.

Inferred: this is display code carried over from older Nixxes ports, with no setting that reaches it and its own message saying it is off under DirectX 12. It is very unlikely to be a working stereo renderer for this engine. It costs one short disassembly check to be sure (test 2 in section 6).

## 3. Existing ways to get true stereo

| Way | Version, date | Licence, price | Works on this game? | Proton + WiVRn | Counts as true stereo? |
|---|---|---|---|---|---|
| SpidyVR | v0.8.6, 2026-11-11 (repo pin: v0.3.8; issue #155 plans 0.7.0) | GPL-3.0, free | No. Remastered only, one Steam build (4.630.0.0) | Tested by its users on Remastered only | Yes |
| HEROVR | v0.0.79, 2026-11-09 | GPL-3.0, free | Yes, this exact exe. Experimental, free roam only | No report. Windows only | Yes |
| SM2VR | 0.0.27, closed | Closed binaries, no public source found | Yes (HEROVR's addresses come from it) | No report | Not checked |
| R.E.A.L. VR | see group B note | Barred by rulings 8 and 16 | Yes | Not relevant | Alternate-eye |
| Geo-11 / 3DMigoto | see group B note | Free | No. DirectX 11 only | Not relevant | Yes |
| UEVR | any | Free | No. Not an Unreal game | Not relevant | Yes |
| The game's own stereo leftovers | built into 2.810.0.0 | Part of the game | Unknown, most likely dead | Unknown | If alive, yes |
| ReShade depth filters (SuperDepth3D and the like) | any | Free | Would load | Likely | **No.** One camera plus a depth guess |

### How far SpidyVR goes under Proton with WiVRn

Sources: the [SpidyVR repo](https://github.com/IlyaMez/SpidyVR), its `docs/LINUX.md` at v0.8.6, and its issues 7, 8, 17, 32, 33 and 50.

- The Linux guide calls itself experimental. It records two test runs, both on **Remastered**: one with SpidyVR 0.3.2 on SteamVR with Steam Link (Quest 3, desktop Radeon), and one with 0.3.6 on **WiVRn 26.9 from Flatpak** (Quest 3, Proton Experimental). Its own verdict: Steam Link "stayed the more reliable of the two".
- The guide has not recorded a new Linux test since 0.3.6. So no public report covers 0.7.0 or later under Proton, on either runtime. Issue #38 in this repo (the 0.7.0 path into Step 0) would be the first.
- Launch options the guide gives: `STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%`, plus for the WiVRn Flatpak `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1` and a `PRESSURE_VESSEL_FILESYSTEMS_RW` path to the WiVRn install. The runtime must be started before the game.
- Upstream issue 50 (open): a WiVRn user needed the Visual C++ runtimes installed into the prefix and the per-user Flatpak path. The handheld's WiVRn is also a per-user Flatpak, version 26.9 (observed), so that path applies.
- Limits the guide lists under Proton: the mod attaches at the menu, so the render memory cannot be enlarged; menus do not read VR controllers; quitting to the main menu may close the game (the Steam ticket guard); frame generation, vsync and ray-traced reflections must be set off by hand.
- The GAME SCREEN 3D mode (the fixed stereo screen this project ports) is still in SpidyVR at v0.8.6.
- No report anywhere covers a Quest 2, integrated graphics, or SteamOS.
- Observed: Spider-Man Remastered (1817070) is not in any Steam library mounted on the handheld today, so Step 0 and issue #38 cannot run until it is.

### How far HEROVR goes

Sources: the [HEROVR repo](https://github.com/lecutis7of9/HEROVR), its `BUILD-SM2.md` and `CREDITS.md`, and its issue 9. PEERS.md and issue #37 hold this project's standing rule on it: learn from it, copy nothing.

- It is a full head-tracked VR port of SpidyVR to Spider-Man 2. It refuses any exe other than 2.810.0.0 with the SHA-256 above.
- Its own words: experimental, works "decent" in free roam, story mode has faults (progression, objectives, chases). Other listed faults: flickering building windows, late texture sharpening, aliasing.
- Speed on its testers' desktop cards with a Quest 3: about 52 to 59 frames a second with each eye at 70% size; its issue 9 reports about 40 at full Quest 3 size because the two eye renders fight for the graphics card.
- What its build notes prove about this exe (useful as a checklist for our own map, with no address copied): the render probe works once it accepts the new exe name; the 512 MB render memory ring must be replaced within about 3 seconds of process start; ray tracing is switched off for the session; the game's flat view is kept on; the Steam ticket guard had to be ported; two engine calls take one more argument than in Remastered.
- It has no Linux, Proton or WiVRn support and no report of any. Its launcher is Windows only.
- Whether SpidyVR's GAME SCREEN 3D mode still works inside HEROVR is not established. Its in-game settings tab is not ported.
- Its addresses come from the closed SM2VR mod's binaries, by its own credit file. That is the reason for this project's "learn only" rule.

## 4. Route for 1.0, and one fallback

### Recommended: keep the port (`build: port spidy stereo screen`)

This is the route of ruling 17 and of the port plan. This research changes three details, not the route:

1. **The exe is pinned and proven injectable.** HEROVR shows the SpidyVR technique works on exactly this build. Step 2 can start as soon as issue #7 is answered.
2. **Timing differs from Remastered.** The render memory ring has to be enlarged within seconds of process start. SpidyVR's Linux script attaches at the menu, which is too late. Our loader has to be in the process from the start (the plan's own loader, issue #22, already aims at this; it should say so).
3. **Hook points to find in our own map**, in the order the port plan uses them: the Direct3D 12 command queue and swap chain (Step 2), the camera and projection update (Step 3, issue #19), the eye job pair and scene submit with its extra argument (Steps 4 and 5), the Steam ticket guard, and the HUD draw. The address map in [spiderman2-address-map.md](../../../spiderman2-address-map.md) stays the only source of addresses.

Settings for every test: ray tracing off, frame generation off, vsync off, FSR upscaling on, `-nolauncher`, and `-noStreamline`.

Risks:

- **Frame rate.** Three views (flat, left, right) on integrated graphics. HEROVR reaches about 40 to 59 frames a second on desktop cards. The floor here is 30.
- **Updates.** Auto-update is still on. One patch voids the map.
- **No step has run on the device.** Step 0 is blocked because Remastered is not installed. No session file exists for Steps 2 to 8.
- **The Steam ticket guard** closes the game on return to the main menu under Proton in SpidyVR's own notes.

### Fallback: render the second eye on alternate frames

If two extra views cannot hold 30 frames a second, draw one eye per game frame and reuse the other eye's last image. The port plan lists this under its risks. It is true two-camera stereo with a known cost (ghosting on fast sideways motion). It needs an owner ruling first: ruling 3 demands true stereo and does not say whether alternate-eye rendering passes.

### Is there a shorter route?

| Shortcut | Verdict |
|---|---|
| Run HEROVR as it is, through SpidyVR's Linux script changed for this game | Not open. It is head-tracked VR and not a fixed screen, its addresses come from a closed mod, and its binaries may not be downloaded under this project's rules. Building it from source for a one-time proof would answer the frame-rate question weeks early, but it cuts against "learn only, keep our port". Owner question 3 |
| Wake the stereo leftovers in the exe | Very unlikely to work. One short check settles it (test 2) |
| Wait for SpidyVR to support Spider-Man 2 itself | Not planned upstream: "Spidy supports one Steam build" |

So: no shorter route today. The fastest honest gain is to run Step 2 now, because the exe facts it needs are already recorded here.

## 5. Proposed seed recipe

The seed in `crates/engine/tests/fixtures/recipe-seeds/` already has the right route, engine, API and mod. Only the note and the date would change:

```
# Seed: untested. Exe 2.810.0.0 (sha256 c9b912ef...) is the pinned build; no port step has run on device.
name: Marvel's Spider-Man 2
stereo-status: untested
stereo-route: build: port spidy stereo screen
stereo-engine: custom
stereo-api: dx12
stereo-mod: https://github.com/IlyaMez/SpidyVR
stereo-free: yes
# pending #7
stereo-safe: yes
stereo-checked: 2026-11-11
```

## 6. Tests still needed, cheapest first

1. Turn off auto-updates for the game in Steam and confirm build id 24651585 stays. (Two minutes, no headset.)
2. Disassembly check of the stereo leftovers: find what refers to "Changed stereo separation." and `LinkStereoPairs`, and whether any setting or switch reaches that code. (Desk work on the pinned copy, no game start.)
3. Network-off start (issue #7): `sessions/sm2-offline.toml` as written.
4. Step 2 smoke: load only the render probe into `Spider-Man2.exe` under Proton and log the Direct3D 12 queue. Needs a new session file.
5. Loader timing: confirm our loader is in the process before the render memory ring is created (within about 3 seconds of start).
6. Flat-view frame rate at the test settings (ray tracing off, frame generation off), as the base number for the three-view budget.
7. Install Remastered, then Step 0 and issue #38 with SpidyVR under Proton and WiVRn on the Quest 2. This is the first proof of the whole chain on this hardware.
8. Camera shift on Spider-Man 2 (issue #19), then the eye pair, then the stereo screen in the headset, measuring frames a second at each step.
9. Return to the main menu and quit, to see whether the Steam ticket guard closes the game.

Proposed issues (titles only):

- "Spider-Man 2: turn off auto-update and record build 24651585 as the pinned build"
- "Spider-Man 2: session file for port Step 2 (render probe smoke under Proton)"
- "Spider-Man 2: check whether the 3D Vision stereo leftovers in the exe are reachable"
- "Spider-Man 2: loader must be in the process before the render memory ring is created"
- "Spider-Man 2: measure flat-view frame rate at the stereo test settings on the handheld"
- "Port plan: install Spider-Man Remastered so Step 0 can run"
- "Bump the SpidyVR pin to 0.8.6 instead of 0.7.0 (update #155)"

## 7. Open questions for the owner

1. Does alternate-eye rendering pass ruling 3? The fallback depends on it.
2. May auto-update be turned off for this game now? Every address depends on build 24651585.
3. May HEROVR be built from its public source, once, as a throw-away frame-rate proof on the handheld, with nothing from it kept? Today's rule ("learn only") reads as no.
4. Is Spider-Man Remastered going back on the handheld? Step 0 and issue #38 need it.
5. Issue #155 names SpidyVR 0.7.0. Upstream is at 0.8.6 with the stereo screen still present. Pin 0.8.6 instead?
6. This game is the least proven of the 1.0 list and has the longest route. Should it keep its place in the order, or move behind the three new games?

## Could not be done

- Nothing was run on the device; every Proton and WiVRn statement about this game is from other people's reports on other games or hardware, or is inferred.
- No public source for SM2VR was found.
- The PCGamingWiki page was read through its API, not the page view.
- The repo's own exe survey tool was not run here (its numeric library is not on the handheld's host system); the exe facts come from a small read-only header reader and a string scan.
