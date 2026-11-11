# STAR WARS Zero Company (Steam 2075800): routes to true stereo

Researched 2026-11-11 on the handheld (SteamOS). Web sources plus a read-only look at the installed copy: nothing was launched, written or downloaded. This note builds on [games_group_a.md](../games_group_a.md#star-wars-zero-company) and on the static recon in the comment on [issue #34](https://github.com/iannitram/flat-to-3d/issues/34); it does not repeat them. Decisions in [DECISIONS.md](../../DECISIONS.md) win over anything here.

Marking: **observed** = read from the installed files or from the linked page on 2026-11-11. **inferred** = my reading, not tested. Addresses are RVAs (offsets from the image base `0x140000000`) in the installed build only.

Abbreviations: **UEVR** = praydog's Unreal Engine VR injector. **SBS** = side-by-side. **CL** = the changelist number in the game's build string.

## Takeaway

- The game is single-player, Unreal 5.6.1, DX12 only, with no anti-cheat and a full public PDB. DLL injection is not blocked.
- The engine's own fake stereo device is compiled into the shipping exe. The launch option `-emulatestereo` should give a half-width SBS picture with two real camera views and no injection at all. This is the cheapest test and nobody has reported trying it.
- Upstream UEVR nightly has open right-eye faults on Unreal 5.6.1. A community fork, `joeyhodge/UEVR`, carries explicit Zero Company support and has followed each game patch within days. The installed build (CL 201250, exe dated 2026-11-10) is newer than the newest build in the fork's table (CL 199049).
- The game's own mod framework loads content only, not code. It is not a way in for a stereo hook.
- Recommended for 1.0: UEVR (the fork) in Native Stereo with OpenXR and 2D Screen Mode. Fallback: the engine's `-emulatestereo` SBS picture into our SBS viewer, plus a small hook of our own for eye separation and the HUD.

## 1. Identity

### Cited Findings
- Steam appid 2075800. Released 2026-08-27. Developer Bit Reactor, publisher Electronic Arts. Windows only, DirectX 12, single-player, partial controller support, 49.99 USD. The store's legal notice asks for an internet connection, an EA account and linking the EA and Steam accounts. The store lists no third-party DRM notice — [Steam store API](https://store.steampowered.com/api/appdetails?appids=2075800)
- ProtonDB: tier platinum, confidence strong, 192 reports, trending gold — [ProtonDB summary API](https://www.protondb.com/api/v1/reports/summaries/2075800.json)
- Patch 1.2 (2026-10-15) crashed the game on SteamOS. Valve fixed it by changing the game's default launch options to turn NVAPI off. The same page lists the game as Steam Deck "Unsupported" — [GamingOnLinux](https://www.gamingonlinux.com/2026/10/valve-fix-zero-company-crashing-on-steam-machine-with-the-latest-major-game-update/)
- A community mod guide says the game has no anti-cheat and no ban risk, and installs mods with UE4SS (an injected DLL) and pak files in `Content/Paks/~mods` — [games.gg mod guide](https://games.gg/star-wars-zero-company/guides/star-wars-zero-company-how-to-install-mods/)
- Engine 5.6.1 and the EA account link at first launch: already in [games_group_a.md](../games_group_a.md#star-wars-zero-company).

### Observed (installed copy)
- Real process: `SWZeroCompany/Binaries/Win64/SWZeroCompany.exe`. The root `SWZeroCompany.exe` (186 KB) is the usual Unreal bootstrap.
- The main exe imports 41 DLLs and none is an anti-cheat or DRM module. The exe is Authenticode signed.

### Inferences
- DRM is Steam plus the EA account link. Nothing blocks DLL injection: the community already injects UE4SS, and the UEVR fork below injects into this exact game.
- Offline play after the first link is still not documented. It stays an on-device test ([issue #31](https://github.com/iannitram/flat-to-3d/issues/31)).

## 2. Installed copy (static inspection)

Installed: yes, at `<Steam library>/steamapps/common/Star Wars Zero Company/`. I checked a sample of the recon on issue #34 against the files and it holds, with one correction (the plugin count, below).

### Observed
- Steam build id `26445778`. Game build string `++ProjectBruno+Stable-CL-201250`. Main exe dated 2026-11-10, 267 MB, monolithic (the whole engine is in the one exe), PE32+, 9 sections. SHA-256 `60831de4a410850efca453e62ba5acaf19a37ca37cb542202e72b8e2b56d1e6b`.
- The exe's debug record names `SWZeroCompany.pdb`, GUID `6b3ed970-48df-69ce-914b-a167a168a600`, age 1. The 251 MB PDB beside the exe matches it and has public symbols. `Manifest_DebugFiles_Win64.txt` lists it.
- Plugins: `Manifest_UFSFiles_Win64.txt` lists 275 `.uplugin` entries. They are inside the pak files, not loose files (the recon's wording suggested loose files). None is an XR, OpenXR, SteamVR or Oculus plugin. Loose plugin folders are only `SWZeroCompany/Plugins/External/` `DLSS`, `FSR`, `WwiseSoundEngine` and `XeSS`.
- Listed plugins of interest: `BitReactorModFramework`, `External/ImGui`, `InputDebugger`, `BitReactorBugIt`, `AntiLag2`, `StreamlineDLSSG`, `StreamlineReflex`, `StreamlineDeepDVC`, `FSR`, `XeSS`, `PythonScriptPlugin`, `ConsoleVariablesEditor`. Also `SWZeroCompany/Config/DefaultBitReactorCheats.ini` and `BP_BrunoCheatManager.uasset`.
- Upscaler and frame-generation DLLs shipped: `nvngx_dlss.dll`, `nvngx_dlssd.dll`, `nvngx_dlssg.dll`, `sl.interposer.dll`, `sl.dlss_g.dll`, `sl.reflex.dll`, `amd_fidelityfx_upscaler_dx12.dll`, `amd_fidelityfx_framegeneration_dx12.dll`, `libxess.dll`, `libxess_fg.dll`, `libxell.dll`. The exe has a user setting class named `BitReactorUserSettingFrameGenerationOption`.
- Stereo code compiled in: the engine's fake stereo device, with its two vtables and all its methods named in the PDB.

| What | RVA (CL 201250) | Checked how |
|---|---|---|
| `UEngine::InitializeHMDDevice` | `0x797d700` | disassembled |
| Fake stereo device vtable (stereo rendering) | `0xd8d5e80` | slots read and named from the PDB |
| Fake stereo device vtable (render target manager) | `0xd8d5e00` | slots read |
| `IsStereoEnabled` / `EnableStereo` (slots 1 and 3) | `0x4189fe0` | bytes `b0 01 c3` (always true) |
| `GetDesiredNumberOfViews` (slot 4) | `0x797c6d0` | returns the view count field |
| `AdjustViewRect` (slot 8) | `0x797c700` | disassembled |
| `SetFinalViewRect` (slot 9) | `0x797c750` | disassembled |
| `CalculateStereoViewOffset` (slot 11) | `0x797c950` | named |
| `GetStereoProjectionMatrix` (slot 12) | `0x797cda0` | named |
| `RenderTexture_RenderThread` (slot 14) | `0x797d0c0` | named |
| `FSlateRHIRenderer::DrawWindow_RenderThread` | `0x627d610` | named |
| `GSystemResolution` | `0xf100900` | named |

### How `-emulatestereo` behaves in this build (observed from the disassembly)
- `InitializeHMDDevice` builds the fake stereo device when the command line has `emulatestereo` **or** the console variable `r.EnableStereoEmulation` is not zero. Otherwise it goes on to the `nohmd` and `hmd=` checks. This runs once at engine start.
- The device is made with 2 views. Each view is half the system resolution wide. Field of view, eye offset and two more values come from the project's `GeneralProjectSettings` object: the exe carries the names `EyeOffsetForFakeStereoRenderingDevice`, `FOVForFakeStereoRenderingDevice`, `DifferenceBetweenEyesForFakeStereoRenderingDevice` and `TopFOVRatioForFakeStereoRenderingDevice`.
- `AdjustViewRect` halves the width and puts view 0 on the left and view 1 on the right. So the output is a half-width SBS picture in the game's own window. No VR runtime is involved.
- `SetFinalViewRect` reads `r.StereoEmulationWidth` and `r.StereoEmulationHeight` and clamps them to 100..10000.

### Inferences
- I could not read Epic's 5.6 source (the repository needs an Epic-linked account), so the above is from the shipped binary, not the source. The eye offset value itself was not decoded: it is whatever the developer left in the project settings.
- Setting `r.EnableStereoEmulation=1` from an ini file should work the same as the launch option, because the variable is read at engine start. Not tested.
- The fake device sets its own projection (its own field of view), so the game's camera framing may change. The HUD is drawn once across the whole window, not once per eye, so it will look wrong in an SBS viewer until a hook draws it per eye.

### The game's mod framework, ImGui and cheats (observed from symbols and strings)
- `UBitReactorModRuntimeSubsystem` has `RefreshAvailableMods`, `TryMountMod`, `TryLoadMod`, `InitializeMod`, `TryChangingGameFeatureState` and `OnLoadGameFeaturePlugin`. Mod states are `NotMounted`, `Mounted`, `GameFeatureLoaded`, `GameFeatureActive` and `Error`. There is a `BitReactorGameFeatureAction_DataTableOverlay`, a `Mods/` folder string, debug-menu tabs "Installed Mods" and "External Mods", and an entitlement data type.
- The ImGui plugin registers console commands such as `ImGui.ToggleInput` and `ImGui.ToggleDemo`. A cheat manager and `BitReactorCheat` classes are compiled in.
- The developer has said there will be no official mod support — [PC Gamer](https://www.pcgamer.com/games/rpg/star-wars-zero-company-wont-have-official-mod-support-confirms-bit-reactor-but-it-has-a-lot-of-respect-for-the-modding-community/)

Inferred: the framework mounts pak files as Unreal game-feature plugins and overlays data tables. That is content (assets, data, Blueprints), with no entry for native code. It is not a sanctioned way to load a stereo hook, and it is not officially open to players. A content mod could not turn the fake stereo device on either, because that choice is made at engine start, before any mod mounts. The ImGui overlay and cheats are debug tools; whether the console that reaches them opens in the shipping build is not known, and they do not help stereo.

## 3. Existing ways to true stereo

### Cited Findings
- **UEVR upstream.** Free. Last stable 1.05 (2024-11-16); latest nightly 01143 (2026-08-30). The README still says Unreal 4.8 to 5.4 and says to turn DLSS Frame Generation off. The licence file reads "All rights reserved", so we may fetch it for the user but not ship it — [UEVR](https://github.com/praydog/UEVR); [UEVR-nightly releases](https://github.com/praydog/UEVR-nightly/releases); [LICENSE](https://github.com/praydog/UEVR/blob/master/LICENSE)
- Upstream work on 5.5 and later in 2026: "Harden UE5.5+ DrawWindow_RenderThread" (2026-05-18), a right-eye `FSceneView` regression fix (2026-05-20), "Harden BeginRenderingViewFamily scan" (2026-06-17), "Initial nonstandard fixes for 5.5.4" (2026-07-24), a D3D12 barrier fix (2026-08-27). Nothing after 2026-08-30 — [UEVR commits](https://github.com/praydog/UEVR/commits/master)
- Open upstream faults on these engine versions: Unreal 5.5 games that do not enter VR ([#329](https://github.com/praydog/UEVR/issues/329)); Unreal 5.6 right eye with a blue-green tint ([#389](https://github.com/praydog/UEVR/issues/389)); Unreal 5.6.1 on nightly 01143, right eye drawn in a sub-rectangle when the resolution scale is not 100%, Synchronized Sequential and AFR crash, reproduced under Proton with Monado ([#439](https://github.com/praydog/UEVR/issues/439)); an open fix for a doubled stereo picture on 5.5 and 5.6 ([PR #422](https://github.com/praydog/UEVR/pull/422)). The first 5.6 report was closed as fixed in 2025-09 ([#368](https://github.com/praydog/UEVR/issues/368)).
- 2D Screen Mode: the author says it "IS 3D, fully stereoscopic. Only in OpenXR" — [UEVR #61](https://github.com/praydog/UEVR/issues/61). This answers [issue #33](https://github.com/iannitram/flat-to-3d/issues/33) on paper. The screen can draw wrongly on some runtimes — [UEVR #409](https://github.com/praydog/UEVR/issues/409)
- **UEVR fork `joeyhodge/UEVR`** (branch `ue57performance`, described as 4.8 to 5.9, releases almost daily, last push 2026-11-11). Free. It has had explicit Zero Company support since 2026-08-28: it keeps the game's 10-bit scene format (forcing 8-bit made DX12 fail), copies the scene to its own texture for OpenXR, gives the Slate UI its own 1920x1080 target, and guards a Nanite crash in the second view. The patch is tied to exact game builds — [release SWCZUEVR](https://github.com/joeyhodge/UEVR/releases/tag/SWCZUEVR); [fork notes](https://github.com/joeyhodge/UEVR/blob/ue57performance/docs/sw-zero-company-ue561.md)
- The fork's table admits four game builds only: CL 196320, 196985, 197649 and 199049 (game 1.2, added 2026-10-17). It says unknown builds are not admitted and that in-game confirmation is still pending — [SWZeroCompanyBinary.hpp](https://github.com/joeyhodge/UEVR/blob/ue57performance/src/mods/vr/SWZeroCompanyBinary.hpp)
- UEVR under Proton with WiVRn: the launch recipe is in [games_group_a.md](../games_group_a.md) (VR on Linux wiki). Linux support upstream is a long open thread, not a promise — [UEVR #280](https://github.com/praydog/UEVR/issues/280)

### Other routes
- **Engine switch** `-emulatestereo` / `r.EnableStereoEmulation`: section 2. Free, part of the game, no licence question. No public report for this game.
- **Geo-11 / 3DMigoto / Helix Mod**: DX11 only, so not usable (the game is DX12 only). See [stereo_tools_landscape.md](../stereo_tools_landscape.md).
- **Dedicated VR mod**: none found.
- **Depth fakes** (ReShade SuperDepth3D, and similar): they would run, but they shift one picture by its depth buffer. They do not count as true stereo.

### Inferences
- Observed mismatch: the installed build's addresses (stereo vtable `0xd8d5e80`, `InitializeHMDDevice` `0x797d700`) differ from the fork's CL 199049 row (`0xD8D33E0`, `0x797D070`). So the fork's game-specific patch will most likely refuse the installed build until the fork adds CL 201250. I found no patch notes for the 2026-11-10 build.
- I did not find the fork's licence stated apart from upstream's file. Treat it the same way: fetch, do not ship.

## 4. Recommended route and fallback

### Recommended for 1.0: UEVR fork, Native Stereo, OpenXR, 2D Screen Mode
- Tool: the newest `joeyhodge/UEVR` release whose table has the installed CL. If none has it, wait for the fork or pin the game build (open question 1).
- Settings: rendering method Native Stereo (the other two crash on 5.6.1, per #439). Runtime OpenXR through WiVRn. 2D Screen Mode on. Game resolution scale 100% (per #439). Frame generation off. The fork also asks for `r.PostProcessing.PropagateAlpha 1` and `r.OneFrameThreadLag 0`.
- Launch: the injector inside the game's Proton session, as in games_group_a. Inject delay about 15 seconds, after the EA link screen.
- Risks: every game patch moves the addresses, and the game auto-updates. The fork is one person's work with "confirmation pending". The UEVR licence forbids shipping it. 2D Screen Mode draws differently per runtime (#409), and WiVRn is not one of the tested ones.

### Fallback: engine fake stereo plus a small hook of our own
- Step 1, no injection: add `-emulatestereo` to the Steam launch options. Expected: half-width SBS in the game window, two real views. Our SBS viewer shows it.
- Step 2, our own DLL, only if step 1 draws two views: replace slots 11 and 12 of the vtable at `0xd8d5e80` (`CalculateStereoViewOffset`, `GetStereoProjectionMatrix`) to set eye separation and convergence and keep the game's own field of view; wrap `FSlateRHIRenderer::DrawWindow_RenderThread` (`0x627d610`) to draw the HUD once per eye at a chosen depth. Find the addresses by PDB symbol name at start, not by fixed number, so a game patch does not break the hook (the game ships a matching PDB with every build so far).
- Risks: the fake device's field of view and eye offset may be unusable without step 2. Nanite and Lumen may misbehave in the second view (the fork needed crash guards for a Nanite case). Half width per eye is soft. The temporal upscaler and frame generation see one wide picture with a seam in the middle.

### Frame generation
- Observed: the game ships DLSS, FSR and XeSS frame generation and has a user setting for it. The exe has FSR frame-interpolation present classes and an XeSS swap-chain provider, so frame generation takes over presenting.
- Cited: UEVR says to turn DLSS Frame Generation off, and has an open change to warn about it — [UEVR README](https://github.com/praydog/UEVR); [PR #406](https://github.com/praydog/UEVR/pull/406). With Valve's default launch options NVAPI is off, so the DLSS kinds are not offered on SteamOS anyway (GamingOnLinux, above). FSR frame generation is still offered on AMD hardware.
- Inferred: frame generation must be off on both routes. On the UEVR route it fights UEVR for the swap chain. On the fallback route it would blend across the seam between the two eyes. The recipe should set it off and Link should check it.

## 5. Proposed seed recipe

Changes from the current seed: the mod URL points at the fork, and four route settings are added. Status stays `untested`. The key names after `uevr-delay` are proposals; the loader treats unknown keys as route settings.

```text
# Seed: untested until the on-device tests in the research note pass.
name: STAR WARS Zero Company
stereo-status: untested
stereo-route: uevr 2d screen mode
stereo-engine: unreal 5.6
stereo-api: dx12
stereo-mod: https://github.com/joeyhodge/UEVR
stereo-free: yes
stereo-safe: yes
stereo-checked: 2026-11-11
uevr-exe: SWZeroCompany
uevr-rendering: native
uevr-2d-screen: yes
uevr-delay: 15
uevr-runtime: openxr
game-build: 201250
game-frame-generation: off
game-resolution-scale: 100
fallback-launch-option: -emulatestereo
```

## 6. On-device tests still needed, cheapest first

1. Launch with `-emulatestereo` in the Steam launch options. Is the window a half-width SBS picture? Do the two halves differ (real parallax)? What happens to the HUD and to the field of view?
2. Same, with the upscaler at native, then at Quality, then with FSR frame generation on. Note seams and ghosting.
3. Pull the network after the first EA link and launch. Does the game start offline? (issue #31)
4. Inject upstream UEVR nightly 01143, Native Stereo, OpenXR on WiVRn. Does it enter stereo? Are faults #389 or #439 present?
5. Inject the newest fork release. Does it admit CL 201250? If yes: both eyes correct, HUD readable, no crash on the hologram scenes?
6. On whichever UEVR build works: 2D Screen Mode on WiVRn. Is the flat screen stereo, and is it drawn correctly? (issue #33)
7. Frame rate and frame time on the handheld at the settings in the recipe, in a hub area and in a battle.
8. After the next game patch: does the working route still work, and how long until the fork follows?

Proposed issues (titles only):
- Zero Company: test `-emulatestereo` half-SBS output under Proton
- Zero Company: test UEVR fork build against installed CL 201250 on WiVRn
- Zero Company: own hook for eye separation and per-eye HUD on the fake stereo device
- Zero Company: resolve hook addresses from the shipped PDB at start
- Link: detect and warn when frame generation is on for a stereo recipe
- Link: record the game build (CL) and refuse a UEVR build that does not list it

## 7. Open questions for the owner

1. May Link stop the game from auto-updating (or warn before an update), so a working build stays working? Every patch so far has broken address-tied UEVR patches for a few days.
2. Is a one-person fork of UEVR acceptable as the 1.0 baseline, or must 1.0 rest on upstream UEVR or on our own hook?
3. If `-emulatestereo` gives a good picture with no injection, is half width per eye good enough to make it the 1.0 route, with UEVR as the later upgrade?
4. Is a first launch online (for the EA link) acceptable for a game marked safe, if later launches work offline?

## Gaps

- Epic's 5.6 source was not readable, so engine behaviour is from the binary. The eye offset default is not decoded.
- PCGamingWiki refused the fetch on 2026-11-11, so nothing here is checked against it beyond what games_group_a already cites.
- No patch notes found for the 2026-11-10 build (CL 201250).
- Whether the developer console opens in the shipping build is not known.
- Nothing was run. Every route above is untested on the handheld.
