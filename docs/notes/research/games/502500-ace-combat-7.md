# ACE COMBAT 7: SKIES UNKNOWN (502500): research for 1.0

Researched 2026-11-11 on the handheld (SteamOS). Web sources plus a read-only look at the installed copy: the game was not launched, and nothing was written to the game folder or the Proton prefix. No mod or game binary was downloaded; the UEVR profile and UEVR itself were read as source on GitHub.

This note goes deeper than [games group A](../games_group_a.md) (the ACE COMBAT 7 section) and does not repeat it. Group A already covers: Unreal 4 and DX11, Denuvo removed on 2021-10-19, no anti-cheat listing, the PCGamingWiki "UEVR with VRto3D" row, the player reports, the PSVR history and the generic UEVR-under-Proton steps. The tool survey is in [stereo tools landscape](../stereo_tools_landscape.md) and the headset stack in [linux VR stack](../linux_vr_stack.md).

Each finding is marked **observed** (seen in the installed files or in a cited source) or **inferred** (my reading of them, not yet tested).

## Summary

- The game is installed now, on a removable Steam library, and has been played under Proton 11 on the handheld.
- It is **Unreal 4.18**, internal project name `Nimbus`, one 62 MB exe at the install root, no PDB, DX11 by default.
- UEVR's **2D Screen Mode keeps two real views, but only on OpenXR**. I read this in UEVR's source, which settles the conflict recorded in the tools survey. On OpenVR the screen gets the left view only.
- The community profile (keton) is a config plus one plugin DLL. The plugin acts **only in the cockpit view**, and the cockpit view also needs four pak mods and a finished campaign. The chase view and the HUD-only view need none of that (inferred).
- Recommended for 1.0: UEVR nightly, OpenXR, native stereo, 2D Screen Mode, chase or HUD-only view, the profile's config values, no pak mods. Fallback: the engine's own two-view path (`-emulatestereo`) shown through the SBS viewer.
- Biggest risk: at true scale a flight game has almost no visible depth beyond the player's own aircraft, so the picture may look flat until the eye separation is raised.

## 1. Identity

| Item | Value | Mark |
|---|---|---|
| Steam appid | 502500, depot 502501, build 9855922 | observed (`appmanifest_502500.acf`) |
| Engine | Unreal Engine 4.18 (see section 2), 64-bit | observed |
| Graphics APIs | DX11 by default. D3D12, Vulkan and OpenGL back ends are compiled into the exe. | observed (imports and strings) |
| Process | `Ace7Game.exe` at the install root; no launcher, no child process | observed |
| Account | Steam only; no publisher account | observed (no account strings beyond a network EULA dialog) |
| DRM | Steam DRM wrapper: the exe has a `.bind` section. Denuvo was removed in 2021 (group A). | observed section, inferred wrapper |
| Anti-cheat | None: no Easy Anti-Cheat, BattlEye or similar file or string | observed |
| Blocks DLL injection | No sign of it; Windows players inject UEVR into this build | inferred |
| Offline play | Campaign and single missions are offline (group A); not yet checked on the device (#40) | inferred |
| ProtonDB | Platinum, 213 reports, trending Platinum | observed, [ProtonDB summary API](https://www.protondb.com/api/v1/reports/summaries/502500.json), 2026-11-11 |
| Steam Deck | Verified; SteamOS "compatible". Both carry the note that external controllers are not supported for the first player. | observed, [Steam compatibility report](https://store.steampowered.com/saleaction/ajaxgetdeckappcompatibilityreport?nAppID=502500&l=english) |
| Store facts | Released 2019-01-31; single-player plus online PvP; full controller support; Steam Cloud | observed, [Steam store API](https://store.steampowered.com/api/appdetails?appids=502500&l=english) |

The store's system requirements list DirectX 11 and add a line about DirectX 12 on Windows 10. Together with the compiled-in D3D12 back end this suggests `-dx12` starts the game (inferred, untested). UEVR and the seed recipe assume DX11, so leave the default.

### Known Proton faults

All from the game's thread on the Proton tracker, [ValveSoftware/Proton #2307](https://github.com/ValveSoftware/Proton/issues/2307), unless another source is given.

| Fault | When | Bearing on 1.0 |
|---|---|---|
| Rare crash: `CreateTexture2D` fails for a 4096x4096 texture (about 5 in 40 hours) | 2019 | Two eye views raise texture memory use; watch for it in a long sortie |
| Freeze of about 30 seconds at the main menu | 2019 | The injector waits a fixed time (`uevr-delay`); see risk 5 |
| Audio stutter, fixed with `PULSE_LATENCY_MSEC=30` | 2019 | Only if heard |
| Gamepad not picked up on some starts; a Switch Pro controller hung loading | 2021-2022 | The 1.0 session is gamepad-only; test with the owner's pad |
| No window, black screen or a Steam client crash after a Steam client update | 2023-2024 | Each was cured by a later Steam client, not by Proton |
| Online multiplayer stutter | 2024, 2026 | Out of scope (campaign only) |
| Missing `x3daudio1_7.dll` after a broken Proton update | 2021 | Cured by reinstalling Proton |

No report on the tracker or in the ProtonDB summary names UEVR, WiVRn or any stereo tool with this game. That gap from group A is still open.

## 2. Installed copy (static inspection)

Installed at `<Steam library>/steamapps/common/ACE COMBAT 7/` on a removable library. 133 files, 62 GB. All **observed** unless marked.

### Layout

- `Ace7Game.exe` sits at the install root: one monolithic 62 MB exe (65,331,992 bytes, SHA-256 `c7da97f5f8a807d4f1264adbb074146fcffe9bdc2ffa98791b822cd28e558f4f`). There is no `Binaries/Win64/` folder and no bootstrap exe, so the UEVR process name is `Ace7Game` and its profile folder is `Ace7Game`.
- PE header: linked 2022-11-02, image base `0x140000000`, sections `.text .rdata .data .pdata .tls .gfids _RDATA .rsrc .reloc .bind`.
- **No PDB ships.** The exe names `Nimbus-Win64-Shipping.pdb`, which is absent. There are no class names from C++ type information for the stereo or headset classes either. So no symbol addresses can be listed the way the Zero Company recon on #34 does; any hook of our own needs byte-pattern scans.
- `Game/Content/Paks/`: 12 pak files (`pakchunk0` to `pakchunk5`, each with a `_0_P` patch pak), 42 GB. No `.sig` files and no `~mods` folder: **no mod is installed**. The pak contents were not opened.
- `Game/Content/CriWare/`: the video and audio streams (CRI middleware).
- `Engine/Plugins/`: only `TrueSkyPlugin` (the sky and clouds renderer), with `Engine/Binaries/ThirdParty/Simul/Win64/TrueSkyPluginRender_MT.dll`.
- Other DLLs: PhysX 3.4 and APEX, Ogg and Vorbis, `steam_api64.dll` (Steamworks 1.39), `GFSDK_Aftermath_Lib.dll`, `WinPixEventRuntime.dll`.
- **No upscaler or frame-generation DLL** (no DLSS, FSR or XeSS file or string). Nothing to switch off in the game itself.

### Engine version and build

- Build paths in the exe read `++UE4+Release-4.18` (the PhysX build tree) and `bns_ws\update12\Nimbus\Main\UE4\...` (the game's own tree). So: Unreal 4.18, project `Nimbus`, branch `update12`.
- Game classes are prefixed `Nimbus` (`ANimbusPlayerController`, `ANimbusPlayerCameraManager`, `ANimbusHUD`). The community plugin hooks these by name (section 3).
- Engine modules compiled in: `D3D11RHI`, `D3D12RHI`, `VulkanRHI`, `OpenGLDrv`, `OnlineSubsystemSteam`, `CriWare`, `TrueSkyPlugin`, Wwise audio, and `HeadMountedDisplay`.

### Stereo and VR code compiled in

- Engine stereo switches: `emulatestereo` (command line), `r.EnableStereoEmulation`, `vr.bEnableStereo`, `vr.bEnableHMD`, `vr.InstancedStereo`, `vr.MirrorMode`, `vr.SpectatorScreenMode`, `vr.WorldToMetersScale`, `vr.MonoscopicFarField`.
- The PS4 VR mode is still in the PC exe: source path `Nimbus\Source\Nimbus\VR\NimbusBlueprintLibraryVR.cpp`, settings `Neo.VR.Mission.RenderSetting`, `Neo.VR.Hangar.RenderSetting`, `Neo.VR.AirShow.RenderSetting` and `ace.vr.r.PS4VROverlayAdditiveBlend`, `bEnableMorpheus`, the shader `PostProcessHMDMorpheus.usf`, and dozens of `Psvr...` menu text keys.
- **No PC headset plugin**: no SteamVR, OpenVR, Oculus or OpenXR module, DLL or string. The engine's "HMD module not found" message is present. So the VR mode has no device to run on without a tool such as UEVR (inferred).
- Because `emulatestereo` is present, the engine's built-in fake stereo device is very likely compiled in, as it is in Zero Company (inferred; without a PDB the class cannot be named).

### Proton prefix and Steam state

- The prefix was made by **Proton 11.0-100**. Steam records about 7 hours of play on this device, so the game starts and runs under Proton here (the offline check of #40 is still to do).
- The prefix holds only the save (`AppData/Local/BANDAI NAMCO Entertainment/ACE COMBAT 7/SaveGames/<Steam id>/ACE7Save.sav`, synced by Steam Cloud). There is **no `Config` folder**, so there is no ini file to edit; graphics settings appear to live in the save (inferred).
- No UEVR profile folder (`AppData/Roaming/UnrealVRMod/`) and no .NET runtime are in the prefix: UEVR has never run here.
- The game's Steam launch options currently wrap it in a frame-generation layer (lsfg-vk) with a 90 fps cap. See risk 4.

## 3. Ways to get two real views

### UEVR

- Version: nightly 01143, 2026-08-30, built from commit `4ee5c6b`; last stable 1.05, 2024-11-16 — [UEVR-nightly releases](https://github.com/praydog/UEVR-nightly/releases); [UEVR releases](https://github.com/praydog/UEVR/releases). Observed.
- Price and licence: free to download. The source is public but the `LICENSE` file reserves all rights, so it is not open source and must be fetched from upstream, not copied into this repo — [UEVR LICENSE](https://github.com/praydog/UEVR/blob/master/LICENSE). Observed. This closes the "licence text not read" gap in the tools survey.
- Engine range 4.8 to 5.4 (group A), so 4.18 is inside it.
- Under Proton: the procedure in group A. The wiki page was updated on 2026-10-21 and still names the .NET 6.0 desktop runtime (6.0.25) and the OpenXR choice for WiVRn — [VR on Linux wiki: UEVR Injector](https://vronlinux.org/docs/games/uevrinjector/). No report for this game.

#### What 2D Screen Mode does (read from the source at nightly 01143)

All inferred from reading the code; none of it has been run here.

1. **Two real views are kept.** In the view-offset hook, 2D Screen Mode skips the head position and the head rotation but still moves each eye sideways by the headset's eye offset times the world scale. The two cameras are parallel and a fixed distance apart — [FFakeStereoRenderingHook.cpp, lines 4923-4965](https://github.com/praydog/UEVR/blob/4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/src/mods/vr/FFakeStereoRenderingHook.cpp#L4923-L4965).
2. **The projection is fixed at 90 degrees wide**, symmetric, sized to the game's own render target and not to the headset. A code comment marks the fixed angle as unfinished work — [same file, line 5134 onward](https://github.com/praydog/UEVR/blob/4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/src/mods/vr/FFakeStereoRenderingHook.cpp#L5134).
3. **On OpenXR each eye gets its own screen.** The left half of the frame is drawn to one texture and the right half to another, the game's UI is drawn on both, and they are submitted as two layers with left-only and right-only eye visibility — [D3D11Component.cpp, lines 392-497 and 692](https://github.com/praydog/UEVR/blob/4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/src/mods/vr/D3D11Component.cpp#L392-L497).
4. **On OpenVR only the left texture reaches the overlay**, so the screen is flat. The route must use OpenXR (WiVRn directly), never OpenVR through a bridge. The engine already writes `Frontend_RequestedRuntime=openxr_loader.dll`.
5. **The screen is fixed in the room** when `UI_FollowView=false`: the layer is placed in stage space at the standing origin, `UI_Distance` metres away and `UI_Size` metres tall — [OverlayComponent.cpp, lines 840-866](https://github.com/praydog/UEVR/blob/4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/src/mods/vr/OverlayComponent.cpp#L840-L866).
6. **The HUD sits at screen depth** in both eyes, because the same UI texture is drawn onto both screens.
7. **Depth strength is `VR_WorldScale`** (range 0.01 to 10): it multiplies the eye separation — [VR.hpp, line 951](https://github.com/praydog/UEVR/blob/4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/src/mods/VR.hpp#L951).
8. It works with native stereo and with synchronized sequential; in sequential mode the two eyes arrive on alternate frames.

Point 3 depends on WiVRn honouring per-eye visibility on quad layers, which the VR stack note lists as not verified.

### The community profile: keton/ace-combat-uevr

- Version v0.2.0, 2024-03-18; last commit 2024-03-28; MIT; free; one asset `Ace7Game.zip` (288,182 bytes) — [repository](https://github.com/keton/ace-combat-uevr); [release v0.2.0](https://github.com/keton/ace-combat-uevr/releases/tag/v0.2.0). Observed. The pin file lists its licence as unstated; it is MIT.
- It needs UEVR nightly 846 or newer — [release notes template](https://github.com/keton/ace-combat-uevr/blob/master/release-template.md).
- Contents: `config.txt`, `cameras.txt`, two ImGui layout files and `plugins/ace_combat_plugin.dll`.

Settings in its `config.txt` that differ from a bare profile or matter here — [profile_template/config.txt](https://github.com/keton/ace-combat-uevr/blob/master/profile_template/config.txt). Observed.

| Key | Value | Meaning for us |
|---|---|---|
| `VR_RenderingMethod` | `0` | Native stereo |
| `VR_SyncedSequentialMethod` | `1` | Skip draw, if sequential is chosen |
| `VR_2DScreenMode` | `false` | The profile is for head-tracked VR; we turn this on |
| `VR_DecoupledPitch` | `false` | The known fix for clouds and cockpit ([tools survey](../stereo_tools_landscape.md)) |
| `VR_GhostingFix` | `false` | |
| `VR_DisableHDRCompositing`, `VR_DisableHZBOcclusion`, `VR_DisableInstanceCulling` | `true` | |
| `VR_HorizontalProjectionOverride`, `VR_VerticalProjectionOverride` | `1` | Symmetric projection on both axes |
| `VR_UncapFramerate` | `true` | |
| `UI_FollowView` | `false` | Screen fixed in the room |
| `UI_Distance`, `UI_Size` | `2.0`, `2.0` | A screen 2 m tall, 2 m away |
| `UI_OverlayType` | `0` | Flat quad, not a cylinder |
| `FrameworkConfig_L3R3LongPress` | `true` | The UEVR menu needs a long press, because a short press of both sticks fires flares in the game |
| `VR_WorldScale` | `1.0` | True scale |
| `Frontend_RequestedRuntime` | `unset` | We set OpenXR |

What the plugin does — [src/Plugin.cpp](https://github.com/keton/ace-combat-uevr/blob/master/src/Plugin.cpp). Observed in the source.

- When the camera type becomes **cockpit**, it waits for the aircraft to move (5 second timeout), stops all camera shakes, then starts the game's looping test camera shake at a large negative scale. The release notes credit this "shake" trick to an earlier mod author as what makes the cockpit render correctly in 3D. On leaving the cockpit view or entering a cutscene it stops the shakes.
- **It does nothing in any other camera view.**
- The gamepad Back button re-runs that cockpit set-up ("unstick instruments").
- An optional stick remap (four radio-control layouts), off by default.
- A debug panel with "complete mission" and "god mode" buttons and an auto-complete tick box. These are cheats; they are off unless clicked.

Cockpit view prerequisites, from the release notes. Observed.

- Four pak mods: an "additional skin slots" mod and an add-on from two Nexus pages (mods 2179 and 2274), plus a "VR planes" pak and a "VR patch" pak that are posted only in a Discord channel.
- The campaign finished once, because the tenth extra skin slot selects the VR cockpit of a plane.
- Not together with the older compatibility pak (Nexus mod 2387); the two conflict.
- Known quirks: throttle up at once on stationary take-offs; press Back after a checkpoint retry.

The Nexus pages return HTTP 403 to a plain fetch, so their version, date and price are still unconfirmed (same gap as group A).

**Inferred:** the pak mods and the plugin exist to make the cockpit instruments and cockpit camera work in VR. The chase view and the HUD-only first-person view draw no cockpit, so they should need only UEVR and the config values. A fixed 3D TV played with a gamepad suits those two views, and the game needs no modification. This is the main simplification over the current A2 session, which asks the owner to install the four paks and finish the campaign first.

### The engine's own stereo path

- `-emulatestereo` on the command line makes Unreal 4 create its fake stereo device: two views side by side in the game window, a fixed eye offset, and a projection meant for a headset lens (inferred from the engine version; the strings are in the exe).
- It is free, needs no injected code, and its output is a side-by-side picture that the SBS viewer built for the Geo-11 route can show.
- Faults to expect (inferred): the wide, off-centre projection will look wrong on a flat screen; the HUD may be drawn once across both halves; the eye offset is not adjustable without a hook.

### Other tools

| Tool | True stereo | State for this game | Mark |
|---|---|---|---|
| UEVR + VRto3D | yes | The PCGamingWiki route (group A). Needs SteamVR, which the VR stack note advises against on SteamOS. | observed listing |
| Geo-11 / 3DMigoto fix | yes | None exists: the Helix Mod blog has no post for this game — [blog search](https://helixmod.blogspot.com/search?q=ace+combat). A generic Geo-11 attempt on Unreal 4 deferred rendering without a fix would have broken shadows and lights. | observed absence, inferred outcome |
| Dedicated PC VR mod | no such mod | The PS4 VR missions were never ported and no mod restores them (group A). The VR code is in the exe (section 2). | observed |
| vorpX | geometry mode possible | Paid, Windows-only driver; not considered | inferred |
| ReShade depth shaders (SuperDepth3D and similar) | **no** | One camera plus a depth-buffer warp. Named for completeness; does not count. | — |

## 4. Recommended route and fallback

### Recommended: UEVR, OpenXR, native stereo, 2D Screen Mode, no pak mods

Settings, all in the UEVR profile folder `AppData/Roaming/UnrealVRMod/Ace7Game/` inside the game's prefix:

1. UEVR nightly 01143 or newer, injected into `Ace7Game` after the main menu shows.
2. Runtime OpenXR (WiVRn). Never OpenVR: the screen would be flat.
3. `VR_RenderingMethod=0`, `VR_2DScreenMode=true`.
4. Keep every other value of the keton `config.txt` (table above), above all `VR_DecoupledPitch=false`, `UI_FollowView=false` and `FrameworkConfig_L3R3LongPress=true`.
5. Screen: start from `UI_Size=2.0`, `UI_Distance=2.0`. A 16:9 picture is then about 3.6 m wide and spans about 83 degrees, close to the 90 degree projection, so shapes are near true scale.
6. Depth: start at `VR_WorldScale=1.0`, then raise it until the player's aircraft and nearby aircraft show clear depth (expect a value between 2 and 10).
7. In the game: chase view or HUD-only view. Do not install the pak mods for the first tests.
8. Steam launch options: only the engine's own wrapper; remove the frame-generation layer and the fps cap for 3D launches.
9. If native stereo shows broken clouds (the sky comes from a third-party plugin that may not draw the second view), switch to `VR_RenderingMethod=1` with `VR_SyncedSequentialMethod=1`.

Whether to ship the plugin DLL: it is harmless outside the cockpit view, but it carries cheat buttons and takes over the Back button. For a no-pak route I would leave the DLL out and ship only the config values (inferred; needs test 5 below).

### Fallback: the engine's own two views plus the SBS viewer

1. Add `-emulatestereo` to the launch options and look at the window. This costs one launch.
2. If the picture is two usable halves, show it through the SBS viewer.
3. If the projection or eye offset is wrong, replace the fake stereo device's view-offset and projection functions with our own, as planned for Zero Company (#34). Without a PDB the two functions must be found by pattern scan; UEVR's scanner already finds the same functions in 4.18, so the patterns exist.

### Risks

1. **Flat-looking picture.** At true scale the eye separation is about 6 cm, so anything beyond a few hundred metres has no visible depth, and most of a flight game is beyond that. Depth will come from the player's aircraft, close passes and the ground at low level. Raising `VR_WorldScale` is the lever. The whole scene also sits in front of the screen plane (parallel cameras), which can tire the eyes at high values.
2. **Per-eye layers on WiVRn** are unverified (VR stack note). If WiVRn shows both layers to both eyes, the route fails and the shim from the VR stack note is needed.
3. **The engine may replace the profile's `config.txt`.** In `crates/engine/src/injectors/uevr.rs` the profile text has three to five keys. If that text replaces the pinned profile's file and is not merged into it, the keton values are lost: the UEVR menu would then open on a short press of both sticks, which is the flare button. Check the merge (inferred from the code, not run).
4. **Frame generation on the final picture.** The current launch options run lsfg-vk over the game. On a UEVR session it would touch only the desktop mirror, but on the fallback's side-by-side picture it would smear across the eye seam.
5. **Injection timing.** `uevr-delay: 15` may fire before the main menu: the game shows logos first and an old report has a 30 second freeze there. The profile author injects at the main menu.
6. **Clouds and sky** are drawn by a third-party plugin (trueSKY) with its own DX11 renderer; it is the likeliest part to break in native stereo.
7. **Cutscenes and briefings** are video streams and 2D menus: they will be flat, at screen depth. In-engine cutscenes change cameras quickly; the plugin stops shakes for them only in the cockpit path.
8. **Gamepad.** Steam's own compatibility note says external controllers are not supported for the first player, and the Proton thread has pad detection faults.
9. **UEVR age and licence.** The last nightly is from 2026-08-30 and the licence allows no redistribution, so the pin must fetch from upstream.

## 5. Proposed seed recipe

Same keys as the current seed in `crates/engine/tests/fixtures/recipe-seeds/502500-ace-combat-7.recipe`; the engine version is now exact and the date is today's. Not added as a file.

```text
# Seed (OD6): untested until sessions/a3-sortie.toml passes; real recipe lands in recipes/ at works-now.
name: ACE COMBAT 7: SKIES UNKNOWN
stereo-status: untested
stereo-route: uevr 2d screen mode + keton profile
stereo-engine: unreal 4.18
stereo-api: dx11
stereo-mod: https://github.com/keton/ace-combat-uevr
stereo-free: yes
stereo-safe: yes
stereo-checked: 2026-11-11
uevr-exe: Ace7Game
uevr-rendering: native
uevr-2d-screen: yes
uevr-delay: 15
uevr-profile: ac7-profile
```

Pin facts for `ac7-profile` in `packaging/pins` (not changed here): version `v0.2.0`, URL `https://github.com/keton/ace-combat-uevr/releases/download/v0.2.0/Ace7Game.zip`, size 288182, licence MIT. GitHub publishes no digest for this asset, so the SHA-256 has to be taken at pin time.

The locked test in #47 expects other values (`stereo-engine: unreal-4`, `stereo-route: uevr: native stereo + 2d screen`, `stereo-mod` pointing at UEVR). See the open questions.

## 6. Tests still needed on the device, cheapest first

1. Start the game with the network off and load the save (#40).
2. Start once with `-emulatestereo` and take a screenshot: two views or not, and where the HUD lands.
3. Time from Play to the main menu, three starts, to set `uevr-delay`.
4. Inject UEVR over OpenXR with `VR_2DScreenMode=true` and capture each eye: are the two pictures different, and is the screen fixed when the head moves?
5. Fly a mission in chase view and in HUD-only view with no pak mods and no plugin: HUD readable, radar and ammo counts present, clouds and sky correct in both eyes.
6. Raise `VR_WorldScale` in steps (1, 2, 4, 8) and note where depth is clear and where it becomes uncomfortable.
7. Compare native stereo and synchronized sequential for clouds, shadows and frame rate.
8. Press both sticks briefly (flares) and confirm the UEVR menu stays closed; confirm what the engine wrote to `config.txt`.
9. Fly a full sortie with the frame-time log: 30+ fps, no crash (#42).
10. Only if the owner wants the cockpit view: install the four paks and the plugin, select the tenth skin slot, and repeat tests 5 and 9.

Proposed issues (titles only):

- ACE COMBAT 7: check 2D Screen Mode gives two different eye pictures on WiVRn
- ACE COMBAT 7: `-emulatestereo` probe and screenshot
- ACE COMBAT 7: measure time to main menu and set `uevr-delay`
- ACE COMBAT 7: depth strength (`VR_WorldScale`) for the fixed screen
- UEVR route: merge the engine's keys into a pinned profile's `config.txt`, never replace it
- ACE COMBAT 7: drop the pak mods from the A2 session unless the cockpit view is wanted
- Pins: fill in `ac7-profile` (v0.2.0, MIT)
- UEVR route: clear frame-generation launch options on a 3D launch

## 7. Open questions for the owner

1. Is the chase view or the HUD-only view acceptable for 1.0, or is the cockpit view a must? The cockpit view needs four pak mods (two from a Discord channel), the plugin with its cheat buttons, and a finished campaign.
2. Has the campaign been finished on this save? It decides whether the cockpit view is possible at all without replaying.
3. Exaggerated depth (a larger eye separation, which makes the world look like a model) or true scale with little depth: which is preferred if both cannot be had?
4. May the 3D launch drop the frame-generation layer and the 90 fps cap that the game's launch options use today?
5. The #47 locked test and the seed recipe disagree on three values (engine, route and mod strings). Which set is right? The test would need the Project Manager to change it.
6. Which gamepad will be used in the headset session? Steam's note about external controllers makes this worth settling before the sortie.
