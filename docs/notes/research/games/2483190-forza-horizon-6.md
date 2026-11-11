# Forza Horizon 6 (Steam 2483190): route research for 1.0

Research date: 2026-11-11. Issue: #258. This note adds to [games_group_a.md](../games_group_a.md) ("Forza Horizon 6") and does not repeat it. Facts from outside carry a link. Facts from the installed copy are marked **observed** (read-only static inspection on the handheld, the game was never started). Conclusions of mine are marked **inferred**.

## Short answer

- **Safety: injection is not safe for the owner's account, and nothing we build can make it safe.** There is no client anti-cheat program to block a DLL, so a mod loads. But the game needs an Xbox account sign-in, its code is encrypted on disk, it carries cheat-detection and telemetry code, and Forza's conduct rules list modifying the game as a bannable offence. No ban for a VR mod has been reported. The risk is small but real, it falls on the whole Xbox account, and only the owner can accept it. Ruling 7 in [DECISIONS.md](../../DECISIONS.md) still says "Forza Horizon 6 is not safe".
- **A free true-stereo mod now exists** (it did not on 2026-10-18): GrayHorizon, for Forza Horizon 3, 4, 5 and 6. It is a head-tracked VR mod, cockpit view only, binaries only, alternate-eye.
- **Recommended route if the owner accepts the risk:** our own `xinput1_3.dll` proxy that draws alternate eyes and hands a side-by-side picture to the SBS viewer. **Fallback:** GrayHorizon's binaries plus a fixed-screen OpenXR layer.
- **Biggest risk after the account:** the game updates often and its code is encrypted at rest, so every hook must be found at run time and can break on any patch.

## 1. Identity

| Item | Value | Source |
|---|---|---|
| Steam appid | 2483190, depot 2483191, build 26422872 | observed (`appmanifest_2483190.acf`) |
| Released | 2026-05-19 on Windows and Xbox Series (early access 2026-05-15) | [Wikipedia](https://en.wikipedia.org/wiki/Forza_Horizon_6) |
| Developer, engine | Playground Games with Turn 10; ForzaTech | [Wikipedia](https://en.wikipedia.org/wiki/Forza_Horizon_6) |
| Graphics API | Direct3D 12 only: the exe imports `d3d12.dll` and `dxgi.dll` and ships `D3D12/D3D12Core.dll` (Agility SDK). No DX11 or Vulkan path | observed |
| Process | `forzahorizon6.exe`, file version 6.483.970.0, linked 2026-11-03 | observed |
| Launcher | none. `forzaprotocolselector.exe` only handles multiplayer invite links | observed (`installscript.vdf`) |
| Account | an Xbox account sign-in is required, on Steam too | [GamingOnLinux, 2026-05](https://www.gamingonlinux.com/2026/05/forza-horizon-6-is-out-valve-update-proton-hotfix-for-linux-initial-thoughts/) |
| Offline play | "fully offline" is reported as not possible. The exe does hold offline states (`ContinueOffline`, `EnterFreeRoamOffline`, `BootOfflineWarning_Start`), so play after sign-in with the network down probably works | GamingOnLinux (same link); strings observed; the conclusion is inferred |
| Client anti-cheat | none found: no EasyAntiCheat, BattlEye or driver files in the folder, and no Forza title in AreWeAntiCheatYet (1,167 games, checked 2026-11-11) | observed; [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json) |
| DRM and tamper protection | not named by any source I could read. See section 2: most of the code is encrypted on disk | observed |
| ProtonDB | Gold, 1,092 reports, best reported tier Platinum (summary API, 2026-11-11). Steam Deck Verified. Silver at launch, fixed by a Proton hotfix that Valve made the default | [ProtonDB](https://www.protondb.com/app/2483190); [GamingOnLinux](https://www.gamingonlinux.com/2026/05/forza-horizon-6-is-out-valve-update-proton-hotfix-for-linux-initial-thoughts/); [PC Games Hardware](https://www.pcgameshardware.de/Forza-Horizon-6-Spiel-75404/News/Linux-Proton-Workarounds-VKD3D-1535291/) |
| Proton on the handheld | prefix made by Proton 11.0-100; a `vkd3d-proton.cache` sits in the game folder, so the game has run here | observed |

### Does anything block DLL injection?

Technically no. Three public mods load a DLL into the retail Steam build today (section 3), one of them as a plain `xinput1_3.dll` beside the exe.

What makes it unsafe is the service side:

- The exe carries `CheatDetection` and `CheatDetectionRouter`, a `ChecksumManagerCheckOffline` class with a `CheckIntegrity` method, `integrity_check`, and telemetry calls named `SendSecureTelemetry` and `SendAegisTelemetry`. It imports the module-listing calls `K32EnumProcessModules`, `CreateToolhelp32Snapshot` and `Module32FirstW`/`Module32NextW` (observed). A crash reporter uses those calls too, so they prove nothing alone. Together they mean the game is able to list the DLLs loaded into it and report to the server (inferred).
- Forza's conduct rules list "modifying game files" and running cheat tools beside the game under cheating and unallowed modding, per the search summary of [Forza Support: Code of Conduct](https://support.forza.net/hc/en-us/articles/360035563914-Forza-Code-of-Conduct). I could not read that page first-hand (HTTP 403), so the wording is second-hand.
- Bans in this game are real and long. In May 2026 players of a leaked build were banned until the year 9999 with "modding" as the stated reason, and Playground announced strict enforcement against anyone using that build — [The Escapist](https://www.escapistmagazine.com/news-forza-horizon-6-ban-mods-pirate/). Those players also ran a pirated copy, so it is not a clean modding case.
- The mod authors give no promise. GrayHorizon says online play is at the user's own risk, because the mod hooks camera and rendering — [GrayHorizon README](https://github.com/GrayHorizon-mod/Forza-Horizon-VR). The paid mod's shop page says there is no guarantee against a ban, per a search summary of [NALULUNA's Ko-fi page](https://ko-fi.com/s/03bdcc5fe9) (page not readable, HTTP 403).
- No ban for a VR or ReShade mod turned up in any search. One streamer reports over 30 hours with the paid VR mod, with no online racing, and no action — [post on X](https://x.com/macinvr/status/2064257777384509834). A fan guide found no bans tied to ReShade-style mods — [forzahorizon6.help](https://forzahorizon6.help/pc/mods/) (low-trust source).

**Verdict (inferred):** `stereo-safe` is `"no"` by the key's own definition (always-online sign-in). The practical risk for a camera-and-present hook used only in solo play looks low, and it is not zero. Playing with the network down after sign-in lowers it further, but queued telemetry may still upload at the next sign-in. Our own DLL has the same footprint as the public mods: a foreign module in the process. It is not safer than they are.

## 2. Installed copy (static inspection only)

Installed: yes, in `<Steam library>/steamapps/common/ForzaHorizon6/`, 159 GB. Everything below is observed unless marked.

- **Layout:** one flat folder. `forzahorizon6.exe` (184 MB), `forzaprotocolselector.exe`, the DLLs, and `media/` with the game data in zip archives (`Camera.zip`, `GameTunableSettings.zip`, `Cars/`, `Tracks/`).
- **Symbols:** no PDB ships. The exe names its PDB as `forzahorizon6.pdb` from a `release_final` build of a tree called `forte_hf` (the package name is `Microsoft.ForteBaseGame`). MSVC RTTI type names are present in plain text, about 396,000 strings in all, with camera classes among them: `OrbitCameraBehaviourComponent`, `ControlledCameraBehaviourComponent`, `CockpitCameraLocator`, `GameCameraLocator`, `CutsceneCamera`, `HomespaceCamera`, and a renderer class `GraphicsDevice11`.
- **Code is encrypted at rest.** `.text` is 100 MB. Measured in 4 MB blocks, 23 of 25 blocks have entropy 7.70 to 7.93 bits per byte; only the first and last blocks look like plain code (6.5 and 6.1). `.rdata` (4.8) and `.data` (5.2) are plain. So the functions the vtables point at cannot be read from the file, and a run-time dump is needed before any reverse-engineering (inferred). Which protection does this is not named by a source I could read.
- **Extra sections:** `.detourc` and `.detourd`, the marks of Microsoft Detours built into the exe, and `.xbld`.
- **Imports that matter for us:** `XINPUT1_3.dll` (not shipped in the folder, so a local file of that name is loaded first), `dxgi.dll`, `d3d12.dll`, `steam_api64.dll`, the four `PlayFab*.dll`, `libHttpClient.dll`, `GFSDK_Aftermath_Lib.x64.dll`, `libxess.dll`, `amd_fidelityfx_loader_dx12.dll`, `WinPixEventRuntime.dll`.
- **Upscalers and frame generation:** FSR (`amd_fidelityfx_upscaler_dx12.dll`), XeSS (`libxess.dll`), DLSS, DLSS ray reconstruction and DLSS frame generation (`nvngx_dlss.dll`, `nvngx_dlssd.dll`, `nvngx_dlssg.dll`) behind NVIDIA Streamline (`sl.interposer.dll` and eight `sl.*` plugins). Streamline wraps the swap chain, so frame generation must be off for any present hook (inferred).
- **Stereo or VR compiled in:** nothing. No OpenXR, OpenVR, Oculus or multiview strings. Every "stereo" string is audio; `LeftEye`/`RightEye` are character bones. Forza Horizon has no split screen, so there is no second scene view to reuse (inferred).
- **Config:** settings are not in the game folder. They live in the Proton prefix at `AppData/Local/ForzaHorizon6/LocalStorage_Shared/ForzaUserConfigSelections/UserConfigSelections`. Field-of-view keys exist per camera (`ChaseCamFOV`, `DriverCamFOV`, `HoodFOV`, `BumperCamFOV`, `DashboardCamFOV`). `media/Camera.zip` holds the camera rigs as XML (`CarRelativeCams.xml`, `CameraPhysics.xml`, `CameraLenses.xml`).
- **Benchmark mode** is built in (`BenchmarkModeEnabled`, `BenchmarkModeOverrideCamera`, `BenchmarkModeOutputFPS`): a repeatable scene for frame-rate tests.
- **Online parts:** `MicrosoftGame.Config` (title id `7bf69384`, store id `9NR1R1XWLCNB`, `MSAFullTrust`), `xgameruntime.dll`, `XCurl.dll`, `networkmanifest.xml`, PlayFab save sync, as group A already lists.
- **Not done:** the repo's `tools/xmatch/survey.py` needs numpy, which the host does not have, so the section table, imports and strings were read with `objdump`, `strings` and a short script instead. Exe SHA-256 starts `63791d0b686186c2`.

## 3. Existing ways to get true stereo

None of these is in the repo's earlier notes: all four mods appeared after the game's launch, three of them in October 2026.

| Tool | Version, date | Licence, price | Stereo method | Works under Proton with WiVRn? |
|---|---|---|---|---|
| [GrayHorizon](https://github.com/GrayHorizon-mod/Forza-Horizon-VR) | FH6 build 1.3.0, 2026-11-07 (release 1.3.1 on 2026-11-08 changes Forza Horizon 4 only) | free; PolyForm Noncommercial 1.0.0; source not public yet, binaries only | alternate-eye, two real camera positions, plus an optional depth warp of the older eye | unknown, no report found |
| [NALULUNA's VR mod](https://ko-fi.com/s/03bdcc5fe9) | v0.13.2 | closed; Ko-fi, pay what you want from 0 per [an installer hub's notes](https://github.com/Mr-Nlce/PCVR-Mods-Installer-Hub/blob/main/Core/ForzaHorizon6VR/README_ForzaHorizon6VR.md); group A found it as "paid" | AFR (alternate-eye) or DIBR. DIBR is a depth fake and does not count | unknown; own launcher `fh6vr.exe`; the author recommends Meta Link |
| [lufz VRMod](https://github.com/oofz/vrmod-releases) | v1.4.3, 2026-10-18 | closed, no licence file, releases only | head tracking through SteamVR; stereo method not stated | unknown; needs SteamVR and `vrmod-launcher.exe` |
| [CheekyRender](https://github.com/ClarkCheekyKent/cheeky-render-releases) | v0.2.2 ("Alpha 3.5"), 2026-10-16 | closed, no licence file, an early preview | mono, AFR at half rate, or AFR at full rate | unknown |

Routes that do not apply:

- **UEVR:** Unreal Engine only. ForzaTech is not Unreal.
- **Geo-11 / 3DMigoto / Helix Mod fixes:** DX11 only, with no DX12 plans ([games_group_b.md](../games_group_b.md), "Control"). Forza Horizon 4, 5 and 6 are DX12, and no Helix Mod fix for any Forza Horizon title was found (group A).
- **Engine stereo switch:** none (section 2).
- **Depth fakes (do not count, ruling 3):** ReShade with SuperDepth3D, vorpX (paid), and NALULUNA's DIBR mode — named in [a Japanese round-up](https://redfreshet.com/forza-horizon-6-vrmod/). ReShade is an injected DLL too, so it carries the same account question.

### What GrayHorizon shows is possible (FH3, FH4, FH5, FH6)

From its [README, TECHNICAL, SETTINGS and INSTALL pages](https://github.com/GrayHorizon-mod/Forza-Horizon-VR) and [release notes](https://github.com/GrayHorizon-mod/Forza-Horizon-VR/releases):

- One injector per game with one design. Tested builds: Forza Horizon 3 (Microsoft Store 1.0.125.x, own launcher, SteamVR only), Forza Horizon 4 (Steam 1.478.564.0), Forza Horizon 5 (Steam 1.688.109.0), Forza Horizon 6 (Steam 6.461.691.0 "and later"; the installed copy is 6.483.970.0). First public release 2026-10-17.
- **How it loads:** `xinput1_3.dll` beside the exe is a proxy that forwards every XInput call to the real Windows DLL.
- **How it hooks:** it gives the game's camera objects private copies of their virtual tables, hooks DXGI Present, and runs an OpenXR session on the game's own D3D12 device. It states that it never modifies the game's code or read-only data.
- **Stereo:** the two eyes are drawn on alternating game frames, so the game must run at twice the rate each eye is shown at. The left picture is one frame older; an optional warp moves it forward using scene depth.
- **Limits:** cockpit view only. Hood, bumper and chase views, menus and the map are shown flat. Screen-space and ray-traced reflections are turned off in Forza Horizon 6 because they reuse the previous frame, which is the other eye's. The mod asks for the game's TAA, DLSS, FSR, frame generation, HDR, V-Sync and motion blur to be off and supplies its own DLSS, FSR or TAA.
- **Updates:** a game update can break the hooks. After an update the mod can stay inactive until its author ships a new build.
- **For our fixed screen:** no side-by-side or 3D-screen output and no switch to turn head tracking off. `plain_views=1` gives ordinary (not canted) eye pictures and `hud_panels=3` keeps the game's flat HUD.

## 4. Recommended route and fallback

**Step zero for either route: the owner rules on the account risk (section 7, question 1).** Until then the recipe stays `not-safe` and nothing is injected.

### Recommended: our own alternate-eye hook, output to the SBS viewer

`build: xinput proxy aer + sbs viewer`. It follows the design GrayHorizon has proved on four ForzaTech games, drops OpenXR from the game process, and reuses the SBS viewer built for the Geo-11 games ([ROADMAP.md](../../../ROADMAP.md), G2). The screen is fixed by construction (ruling 4), and it is our own code (ruling 16).

Hook points:

1. **Load:** a `xinput1_3.dll` proxy beside `forzahorizon6.exe`, forwarding the XInput exports to the real DLL. Under Proton it needs `WINEDLLOVERRIDES="xinput1_3=n,b"` (inferred, the same rule as the `d3d11` override for Geo-11).
2. **Version gate:** read the exe's file version first and do nothing unless it matches a version the recipe lists. The game then runs flat.
3. **Camera:** find the live camera objects at run time through the RTTI type names (section 2), give them private vtable copies, and move the eye sideways by half the eye separation on alternate frames, with an off-axis projection so the screen plane sits at the convergence distance. No code bytes are patched, because the code is encrypted at rest and may be checked (inferred).
4. **Present:** hook `IDXGISwapChain::Present` through the swap chain's vtable. Keep the last left and right pictures, pack them side by side, and hand that to the SBS viewer the same way the Geo-11 route does.
5. **Game settings the recipe must set or check:** frame generation off, the game's TAA and upscaler off (or accept ghosting), screen-space and ray-traced reflections off, motion blur off, V-Sync off, HDR off.

Risks:

- **Account** (section 1). Same footprint as the public mods.
- **Size.** This is the hardest of the seven games: no source to port from, encrypted code, and the camera, frame-order and reflection problems GrayHorizon needed several releases to solve. It needs a memory dump from a running game, which this pass was not allowed to make.
- **Updates.** The installed exe was linked on 2026-11-03. A live-service game patches often, and each patch can move what we hook.
- **Frame rate.** Alternate-eye needs twice the per-eye rate: 60 fps in the game for ruling 6's 30 fps. A Steam Deck drops well below 30 fps on Low ([GamingOnLinux](https://www.gamingonlinux.com/2026/05/forza-horizon-6-is-out-valve-update-proton-hotfix-for-linux-initial-thoughts/)); the handheld is much faster, and its number is not yet measured.
- **Alternate-eye is not yet ruled as true stereo** (ruling 3, and the open questions at the end of DECISIONS.md). In a fast racing game the one-frame gap between the eyes is at its worst; a depth warp like GrayHorizon's would be our work too.

### Fallback: GrayHorizon's binaries plus a fixed-screen OpenXR layer

Files beside the exe: `xinput1_3.dll`, `openxr_loader.dll`, `gh6vr.ini`, `GrayHorizon Settings.exe`. Settings: `plain_views=1`, `hud_panels=3`, `ssr_reflections=0`, `rt_reflections=0`, `console=0`, stable camera on, cockpit view. The game's OpenXR calls go through Proton's `wineopenxr` to WiVRn ([linux_vr_stack.md](../linux_vr_stack.md), Q2). A fixed-screen OpenXR layer (out of 1.0 in [SPEC.md](../../../../SPEC.md) unless UEVR's 2D Screen Mode loses depth) would then pin the head pose and put the two eye pictures on the fixed screen.

Risks: it is a closed binary under a non-commercial licence, which ruling 16 does not allow as written; its settings program checks GitHub for updates and replaces the DLL; it is cockpit view only; nobody has reported it under Proton; it needs the fixed-screen layer that 1.0 does not have; and the account risk is the same.

## 5. Proposed seed recipe

As the rulings stand today (ruling 7 unchanged):

```text
# Seed: not safe until the owner rules on the account risk (docs/notes/research/games/2483190-forza-horizon-6.md).
# If ruling 7 is amended for this game: stereo-status becomes untested. stereo-safe stays "no" (sign-in required).
name: Forza Horizon 6
stereo-status: not-safe
stereo-route: build: xinput proxy aer + sbs viewer
stereo-engine: forzatech
stereo-api: dx12
stereo-mod: https://github.com/GrayHorizon-mod/Forza-Horizon-VR
stereo-free: yes
stereo-safe: no
stereo-checked: 2026-11-11
proxy-exe: forzahorizon6.exe
proxy-dll: xinput1_3.dll
proxy-overrides: xinput1_3=n,b
proxy-exe-version: 6.483.970.0
```

The four `proxy-*` lines are route settings, which the loader ignores today ([recipes/README.md](../../../../recipes/README.md)).

## 6. Tests still needed and proposed issues

On the device, cheapest first. Tests 3 to 8 load a foreign DLL and wait for the owner's ruling.

1. Flat baseline: the built-in benchmark at the settings from hook point 5, at 1280x800 and 1920x1200. Pass: 60 fps or more with headroom. No injection.
2. Offline check: sign in, take the network down, start the game again. Does it reach free roam, and does solo play work? No injection.
3. Proxy load: an empty `xinput1_3.dll` proxy that only forwards and writes a log line. Does the game start under Proton with the override, and does the gamepad still work?
4. Run-time dump: with the proxy in, save the decrypted `.text` once and map the camera classes with the repo's tools.
5. Camera hook: one fixed sideways offset in cockpit view. Does the picture shift, with no crash and no sign-in error?
6. Alternate-eye and side-by-side packing into the SBS viewer; check eye order after menus and rewinds.
7. Other views: chase, hood and bumper cameras, photo mode, cutscenes.
8. A 30-minute solo session, then a week of watching the account for any enforcement message.
9. Fallback only: GrayHorizon under Proton with WiVRn, `plain_views=1`.

Proposed issues (titles only):

- Ruling: may Forza Horizon 6 be injected, and under which play rules (amend ruling 7)
- FH6-1: flat baseline with the built-in benchmark at stereo-ready settings
- FH6-2: offline play check after sign-in
- FH6-3: `xinput1_3.dll` forwarding proxy and exe version gate
- FH6-4: run-time dump and camera class map for `forzahorizon6.exe`
- FH6-5: alternate-eye camera hook, cockpit view
- FH6-6: side-by-side packing into the SBS viewer
- FH6-7: fallback trial: GrayHorizon under Proton with WiVRn
- Recipes: read route keys for a proxy-DLL route

## 7. Open questions for the owner

1. **Account risk.** Ruling 7 and SPEC.md both mark this game "not safe", and this research confirms the reason still holds. Putting it in the 1.0 list overrides that. Do you accept a small, real risk of a ban on your Xbox account (it can cover every Forza game), and should the ruling be amended for this one game?
2. **Play rules, if yes.** Solo only with the network down while 3D is on, or online free roam too? The first is safer and means the wrapper should refuse to start 3D when the network is up.
3. **Does alternate-eye rendering count as true stereo** (ruling 3 leaves it open)? Every route for this game uses it.
4. **May a free, closed-source mod be used** (GrayHorizon, non-commercial licence)? Ruling 16 says open-source tools or our own code only. A yes makes the fallback the cheap first trial and turns the recommended route into the later replacement.
5. **Cockpit view only** in a first version: acceptable, or is the chase camera required?
6. **Order.** No order was given for the three new games. This one is the largest and the only one with an account risk, so I suggest it goes last of the seven.
7. **A run-time memory dump of the game** is needed for the recommended route. It reads the running game's memory on the handheld for the owner's own use, like the approved Spider-Man 2 mapping (ruling 22). Is that approved for this game?

## Gaps

- PCGamingWiki's page, the Forza Support conduct page, the Ko-fi shop page and one news article returned HTTP 403, so the DRM field, the exact rule text and the paid mod's own description are second-hand or missing.
- The name of the protection that encrypts the code is unknown.
- No source says whether any of the four mods runs under Proton.
- GrayHorizon's FH6 zip was not downloaded or inspected (no mod downloads in this pass).
