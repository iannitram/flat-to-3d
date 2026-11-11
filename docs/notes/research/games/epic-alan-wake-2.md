# Alan Wake 2: route to true stereo for 1.0

Research date: 2026-11-11. The game joined the 1.0 list on that day (owner ruling: the Steam Favorites are the 1.0 list). This note adds to [games_group_b.md](../games_group_b.md) (section "Alan Wake 2") and does not repeat it. Where the two differ, this note is newer.

Words used: **observed** means read from the installed copy or from a page on 2026-11-11; **inferred** means my reasoning from those facts. The installed copy was inspected read-only: the game was not started, and nothing was written to the game folder or its prefix.

**File name.** The game has no Steam appid (see Identity), so this file is named `epic-alan-wake-2.md` instead of `<appid>-<slug>.md`.

## Short answer

- **Engine and API:** Remedy Northlight, Direct3D 12 only, one monolithic exe.
- **DRM and anti-cheat:** none that blocks a DLL. Epic Online Services only; the game runs offline.
- **Recommended route:** our own in-process D3D12 hook on Northlight's `rend::View` camera, alternate-eye first, packed side by side for the project's viewer.
- **Fallback:** the NoMoreFlat OpenXR mod (alternate-eye) if its author publishes the free build he announced. As of 2026-11-11 it is still not public.
- **Biggest risk:** frame rate. The handheld holds about 55 to 62 fps in mono ([games_group_b.md](../games_group_b.md)), so alternate-eye gives about 28 fps per eye with no headroom, and a true two-pass render is out of reach.
- **Second risk:** the game is not a Steam game. The engine keys recipes by Steam appid and wraps Steam launch options; neither fits a game that Heroic starts.

## 1. Identity

| Item | Value | Source |
|---|---|---|
| Steam appid | **none.** The game is not sold on Steam. A Steam store search for "alan wake" returns the first game (108710), American Nightmare (202750) and their extras, but no Alan Wake 2 | observed: [Steam store search API](https://store.steampowered.com/api/storesearch/?term=alan%20wake&cc=us&l=en); [PCGamingWiki: Alan Wake II](https://www.pcgamingwiki.com/wiki/Alan_Wake_II) (empty `steam appid` field; Epic Games Store is the only store row) |
| Why it is in Steam Favorites | it is a non-Steam shortcut that Heroic added. The shortcut starts Heroic, and Heroic starts the game | observed: the shortcut entry in the owner's Steam shortcut list |
| Engine | Northlight. No public version number; the exe's version resource says file version `0.559.302.8`, product version `2026.2.3.0` | [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Alan_Wake_II); observed |
| Game version | 1.2.10 (PC patch of 2026-08-18: FSR 4, FSR frame generation, Agility SDK for ray-tracing features) | observed: Epic manifest; [alanwake.info patch post](https://www.alanwake.info/2026/08/18th-august-2026-new-alan-wake-2-patch.html) |
| Graphics API | Direct3D 12 only, feature level 12_0, Shader Model 6.5; mesh shaders wanted | [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Alan_Wake_II) |
| Process name | `AlanWake2.exe` (no launcher exe) | observed |
| Launcher and account | Epic Games Store. Heroic stands in for the Epic launcher. DRM-free when started with `-EpicPortal`; DLC needs the launcher | [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Alan_Wake_II) |
| DRM | none found: no Denuvo, VMProtect, Themida or Arxan strings; seven ordinary PE sections | observed (strings and section table only; inferred that there is no packer) |
| Anti-cheat | none. No EasyAntiCheat or BattlEye files or strings; not in the AreWeAntiCheatYet list | observed; [games_group_b.md](../games_group_b.md) |
| DLL injection | not blocked (inferred). Community ASI mods load into this exe, for example [DeadneM/AlanWake2-Preorder-Cosmetics](https://github.com/DeadneM/AlanWake2-Preorder-Cosmetics), and the NoMoreFlat mod ships as DLLs in the game folder ([VRto3D wiki](https://github.com/oneup03/VRto3D/wiki/NoMoreFlat)) | inferred |
| Offline play | yes: the Epic manifest marks it as able to run offline, and it is single-player | observed |
| ProtonDB | no page, because ProtonDB lists Steam appids only. vkd3d-proton logs on the handheld show "SteamAppID 0" | observed |
| Runs under Proton | yes, on the handheld today: a Proton build chosen in Heroic, vkd3d-proton 3.1.0, its own Heroic prefix | observed |

## 2. Installed copy (static inspection)

Installed: **yes**, through Heroic, at `<Heroic library>/AlanWake2/` (109 GB). All of this section is observed unless marked.

- **Layout.** One exe, `AlanWake2.exe` (62 MB, x64, PE time stamp 2026-08-08), with its DLLs in the same folder. Game data is in `data_pack2/` (`.rmdtoc` and `.rmdblob` archives). Compiled shaders are 153 `.binrfx` files in `data/shaders/build/pc_dx12/`. There is no DX11 exe and no second renderer DLL set.
- **Symbols.** No PDB ships. The exe names its PDB as `...\game_x64_final_epic\game\alanwake2.pdb`, so this is the Epic "final" build. C++ type names are **not** stripped: about 4,500 RTTI names are readable. The game code lives in the `bigfish` namespace, shared engine code in `coregame`, `rend`, `d3d`, `ecs`, `m` and `r`.
- **Renderer names that matter for stereo.**
  - `rend::RendererInterface::renderWorldInternal(const rend::View&, const rend::View&, rend::DirectionalLight*, const rend::FrameCapture&)`: the world render. Its name survives because 129 lambdas inside it carry it in their type names. It takes **two** views; inferred: the current and the previous camera (for motion vectors), not a left and a right eye.
  - `coregame::component::CameraView`: the entity component that holds the final camera. `bigfish::cameramixer` and `bigfish::tailcamera` write it; `bigfish::component::PlayerCamera` is the player's camera state.
  - A free camera is compiled in (`coregame::component::FreeCameraState`, `FreeCameraTransformOverride`, `content::FreeCameraComponent`), and so is a photo mode (`ProgramFlowServerStatePhotoMode`). Inferred: the override component is a ready-made place to push a shifted camera.
  - Shader constant names: `g_mWorldToView`, `g_mViewToClip`, `g_mWorldToClip`, `g_mClipToView`, the `g_mPrevious...` set (`g_mPreviousWorldToView`, `g_mPreviousViewToClip`, `g_mClipToPreviousClip`) and `g_vJitterOffset`, `g_vSSAAJitterOffset`.
- **Stereo or VR code compiled in: none.** No OpenXR, OpenVR, HMD, multiview or stereo-render strings. The only "stereo" strings are Wwise audio effects.
- **Upscalers and frame generation.** FSR upscaler DLL 4.1.0 and FSR frame generation DLL 4.0.0 (`amd_fidelityfx_*_dx12.dll`), plus FSR 2 shader passes built into the engine (`rend::FSR2Glue`, `ffx_fsr2_*.binrfx`). DLSS, DLSS frame generation and ray reconstruction 310.1.0 (`nvngx_dlss*.dll`) through Streamline 2.7.0 (`sl.*.dll`). Also NRD and NRI (ray-tracing denoisers).
- **How the exe reaches Direct3D.** It imports `sl.interposer.dll` (Streamline) and has no direct import of `d3d12.dll` or `dxgi.dll`. It ships the Agility SDK (`D3D12Core.dll` 1.619.1, with the `D3D12SDKVersion` and `D3D12SDKPath` export names in the exe). It does import `VERSION.dll`, `WINMM.dll` and `XINPUT1_4.dll` by name.
- **Other plugins.** Coherent Gameface for the UI (`cohtml.WindowsDesktop.dll`, `RenoirCore`), Bink 2 and a media decoder for the live-action video, PhysX 3, NvCloth, Wwise 2022.1.4, V8, DirectStorage strings, Epic Online Services SDK 1.15.1.
- **Config.** Renderer settings are one JSON file, `renderer.ini`, in the prefix under `AppData/Local/Remedy/AlanWake2/` (object type `rend::UserRendererSettings`, version 45). Keys that a stereo route must set: `m_eWindowMode`, `m_eVSync`, `m_eMotionBlur`, `m_bFilmGrain`, `m_bLensDistortion`, `m_bDepthOfField`, `m_eSSAAMethod`, `m_eFSRFGMode`, `m_eDLSSFGMode`, `m_iRenderResolutionX`/`Y`, `m_iOutputResolutionX`/`Y`. No stereo or eye key exists.

### Comparison with Control (same engine family)

Control Ultimate Edition (870780) is installed too and was inspected the same way.

| | Control | Alan Wake 2 |
|---|---|---|
| Graphics API | DX11 and DX12 exes | DX12 only |
| Layout | exe plus about 20 engine DLLs, in a Windows 7 (DX11) set and a Windows 10 (DX12) set | one exe |
| Renderer | `renderer_rmdwin7_f.dll` and `renderer_rmdwin10_f.dll` | inside the exe |
| Named functions | the renderer DLL **exports** its C++ names: `rend::View::setWorldToView`, `setViewToWorld`, `setViewToClip`, their getters, and `rend::RendererInterfaceWrapper::renderWorld(const m::View&, ...)` | no exports for engine code; names survive only in RTTI |
| Shader constants | `g_mWorldToView`, `g_mViewToClip` | the same names |
| Planned route | Geo-11 with the community fix on the DX11 exe ([870780-control.recipe](../../../../crates/engine/tests/fixtures/recipe-seeds/870780-control.recipe)) | cannot use it: Geo-11 is DX11 only ([games_group_b.md](../games_group_b.md)) |

What this gives us (inferred): the two games share the `rend::View` class and the same constant names, four years apart. Control's DX12 renderer DLL is a labelled map of that class. The project's cross-build matcher ([tools/xmatch](../../../../tools/xmatch), written to carry Spider-Man Remastered names into Spider-Man 2) can be pointed at Control's exported `rend::View` setters to find their Alan Wake 2 twins. That is far cheaper than a blind search. The match is not certain: the class may have changed, and small setters may be inlined in the newer build.

## 3. Existing ways to get true stereo

| Tool | Kind | Version and date | Licence and price | Verdict for this game |
|---|---|---|---|---|
| UEVR | Unreal only | n/a | n/a | does not apply: not Unreal |
| Geo-11 / 3DMigoto | geometry stereo, DX11 | n/a | free | does not apply: no DX11 path. No 3DMigoto-style fix exists on the Helix Mod blog; its only Alan Wake 2 post is the NoMoreFlat one below |
| Engine stereo switch | built in | n/a | n/a | none compiled in (section 2) |
| NoMoreFlat `aw2vr` + VRto3D | VR mod, **alternate-eye**, OpenXR and OpenVR builds | Helix Mod asks for `aw2vr-openxr-v1.0.6` or newer (post of 2025-04-28). Search listings show later builds on the author's Patreon; I could not open the Patreon post to confirm the newest | closed source; **Patreon**. A Helix Mod comment of 2026-11-08 still calls it a paid mod | true stereo, but paid as of today, and alternate-eye. See below |
| VRto3D | SteamVR driver that turns a VR mod's two views into SBS | "latest" build 2026-10-12, V5.0.0 on 2026-07-18; ships `vrto3d-linux64.tar.gz` and an `AW2Ini.zip` game config | LGPL-3.0, free | works only together with a VR mod. Needs SteamVR, not WiVRn ([linux_vr_stack.md](../linux_vr_stack.md)) |
| Geo3D | ReShade add-on, geometry stereo by shader patching, DX9 to DX12, alternate-frame | v1.2.5; last push 2023-12-08 | BSD-2-Clause, free | untested on this game and unlikely (inferred): its own wishlist still lists shadows and ray tracing, and it patches vertex shaders while this game draws its geometry with mesh shaders |
| ReShade + SuperDepth3D / Depth3D | depth-buffer fake | current | free | **does not count** as true stereo (ruling 3 in [DECISIONS.md](../../DECISIONS.md)); named only for completeness |
| R.E.A.L. VR | VR mod, alternate-eye | n/a | n/a | does not support this game ([games_group_b.md](../games_group_b.md)) |

Sources: [Helix Mod: Alan Wake 2 NoMoreFlat + VRto3D](https://helixmod.blogspot.com/2025/04/alan-wake-2-nomoreflat-vrto3d.html); [VRto3D wiki: NoMoreFlat](https://github.com/oneup03/VRto3D/wiki/NoMoreFlat) (last edited 2026-07-20); [VRto3D releases](https://github.com/oneup03/VRto3D/releases); [Geo3D](https://github.com/mutars/Geo3D); [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Alan_Wake_II) (rates 3D as "hackable", sequential-view).

**Is the NoMoreFlat build free yet? No (observed 2026-11-11).**
- On 2026-01-23 the author said all his mods would become public, Alan Wake 2 by name, after "some time to clean up" ([FRVR](https://frvr.com/blog/news/starfield-and-stalker-2-vr-mods-made-free-for-all-following-dmcas-to-other-vr-modders)).
- He kept that promise for two other mods: `starfield2vr` has public releases `v2.0.0.Public` (2026-01-26) and `v2.0.1.Public` (2026-05-05), and `anvilengine2vr` has `v2.0.0.Public` (2026-01-25), both MIT with source ([starfield2vr](https://github.com/mutars/starfield2vr), [anvilengine2vr](https://github.com/mutars/anvilengine2vr)).
- There is still **no public Alan Wake 2 repository or release** on that account. The VRto3D wiki still sends readers to Patreon.
- Under the free-only rule in [DECISIONS.md](../../DECISIONS.md) the recipe value is `stereo-free: "no"` for this route until a public build exists.

**Known faults of the NoMoreFlat route** ([VRto3D wiki](https://github.com/oneup03/VRto3D/wiki/NoMoreFlat), [Helix Mod](https://helixmod.blogspot.com/2025/04/alan-wake-2-nomoreflat-vrto3d.html)): 3D looks odd when the camera moves sideways; eye-flips when the frame rate is not steady; some lights flicker with ray tracing; "more 2D scenes than expected" (the live-action video); frame generation, VSync, motion blur, film grain and lens distortion must be off; windowed mode. A Helix Mod commenter spent eight hours on synchronized (both eyes per frame) rendering and gave up (2026-01-27).

**Under Proton with WiVRn: untested, and nobody reports it.** Neither page mentions Linux. Three things stand between the mod and the headset (inferred):
1. The mod is a Windows OpenXR client. It has to reach WiVRn through Proton's OpenXR bridge, which [linux_vr_stack.md](../linux_vr_stack.md) could not confirm from source.
2. The game runs in a Heroic prefix with a Proton build that Heroic starts outside Steam, so whatever Steam does to expose the host OpenXR runtime to a game is not done here.
3. It is a head-tracked VR mod. For a fixed screen it needs its head tracking frozen or its two views taken into the project's viewer, as [linux_vr_stack.md](../linux_vr_stack.md) describes.

## 4. Recommended route and fallback

### Recommended: our own Northlight view hook, alternate-eye, SBS out

The same shape as the Spider-Man 2 port ([spiderman2/plan.md](../../../spiderman2/plan.md)): a DLL inside the game that moves the camera per eye and hands two images to the viewer. It is free, it needs no SteamVR and no OpenXR bridge, and the DX12 capture and SBS packing code is shared with Spider-Man 2.

Hook points, in order:

1. **Load.** A proxy `version.dll` or `winmm.dll` in the game folder (the exe imports both by name), with a `WINEDLLOVERRIDES` entry for it. Not `dxgi.dll` or `d3d12.dll`: the exe does not import them directly, and Streamline sits in between.
2. **Camera.** Find `rend::View`'s setters (`setWorldToView`, `setViewToWorld`, `setViewToClip`) by matching Control's exported functions, or reach them from `renderWorldInternal` through the lambda RTTI names. On each frame shift the first view sideways by half the eye distance along its right vector and replace its view-to-clip matrix with an off-axis one, so both eyes converge on the screen plane. Leave culling volumes wide enough for both eyes.
3. **Alternate.** Even frames render the left eye, odd frames the right; keep the last image of the other eye. The second `View` argument (inferred: the previous camera) must be the previous camera **of the same eye**, or motion vectors and the upscaler will smear.
4. **Temporal history.** FSR and the engine's own anti-aliasing keep one history. With alternating cameras they ghost. First test with `m_eSSAAMethod` set to the plainest mode and frame generation off (`m_eFSRFGMode`, `m_eDLSSFGMode` = 0); then decide whether a per-eye history is needed.
5. **Output.** Copy each finished eye image to its half of an SBS back buffer before present. The present hook must sit below Streamline: hook the swap chain that `sl.interposer.dll` returns.
6. **HUD and video.** The Coherent UI and the Bink video are flat: draw them once, at screen depth, in both halves.

Settings the route sets in `renderer.ini` (and restores on Stop 3D): motion blur, film grain, lens distortion and frame generation off; VSync off.

Risks:
- **Frame rate** (largest). About 28 fps per eye on the handheld, at the edge of the 30 fps floor, with eye-flips on any drop.
- **Whether alternate-eye counts.** [DECISIONS.md](../../DECISIONS.md) ruling 3 leaves that open. A two-pass render (call the world render twice per frame) is true synchronized stereo but doubles a GPU load that is already at its limit.
- **Reverse engineering with no PDB.** Weeks, not days ([games_group_b.md](../games_group_b.md)). The Control match may shorten it; that is unproven.
- **Screen-space effects** (reflections, ambient occlusion, volumetric light) computed for one eye and reused look wrong. Each needs a look.
- **Game patches** move every address. Pin the build by the exe hash and refuse to hook an unknown one.
- **Not a Steam game.** See the launcher gap below.

### Fallback: the NoMoreFlat OpenXR build, if it becomes public

Use it only after a public, free release appears. Then: OpenXR build of the mod in the game folder, the VRto3D wiki's graphics settings, and either WiVRn with the project's fixed-screen view or the mod's two views taken by the viewer. It is also alternate-eye, so it shares the frame-rate risk and adds the three Proton unknowns from section 3. Its value is that the camera work is already done. If the author publishes source, as he did for the two other mods, it also becomes the best guide to the hook points above.

### The launcher gap (applies to both)

- Recipe files are `<appid>-<slug>.recipe` and the loader parses the appid as a number ([recipes/README.md](../../../../recipes/README.md)). This game has none.
- The engine sets Steam launch options. Here the Steam shortcut starts Heroic, so a wrapper in the shortcut's launch options wraps Heroic, not the game. The game's Proton, prefix and environment are Heroic's.
- Ways out (inferred, not designed): key the recipe by the Steam shortcut id, which is local to one machine; or add a recipe key for the store and let the engine write Heroic's per-game wrapper setting; or have the owner add `AlanWake2.exe` to Steam directly with `-EpicPortal`, which loses gamepad support unless `-epicuserid=Default-Epic-User` is also given ([PCGamingWiki](https://www.pcgamingwiki.com/wiki/Alan_Wake_II)).

## 5. Proposed seed recipe

The file name needs the launcher gap settled first; `<id>` stands for whatever key is chosen. The last four lines are route settings, which the loader ignores today.

```text
# Seed: untested. Not a Steam game: installed through Heroic (Epic), started by a non-Steam shortcut.
# Proposed file name: <id>-alan-wake-2.recipe
name: Alan Wake 2
stereo-status: build
stereo-route: build: northlight view hook, alternate eye
stereo-engine: northlight
stereo-api: dx12
stereo-mod: none
stereo-free: "yes"
stereo-safe: "yes"
stereo-checked: 2026-11-11
store: epic
exe: AlanWake2.exe
build-version: 1.2.10
proxy-dll: version.dll
```

## 6. On-device tests still needed, cheapest first

1. Start the game once from the Steam shortcut and record what the engine can see: the process tree, which Proton and prefix are used, and whether a Steam launch-option wrapper reaches `AlanWake2.exe`.
2. Start it with a do-nothing proxy `version.dll` (logs one line, forwards every call) and the DLL override. Confirms the load path and that nothing checks the module list.
3. Take a RenderDoc capture under vkd3d-proton and read `g_mWorldToView` and `g_mViewToClip` from one world draw. Confirms the constant layout and the matrix convention.
4. Run the cross-build matcher from Control's `renderer_rmdwin10_f.dll` exports to `AlanWake2.exe` and count how many `rend::View` functions match.
5. With the hook, shift the camera by a fixed amount for every frame (one eye only). Confirms the right hook point: the whole picture must shift, with correct culling and shadows.
6. Alternate the shift per frame with no upscaler history, and look for ghosting, eye-flips and effects that stay mono.
7. Measure the frame rate in SBS at the settings in [games_group_b.md](../games_group_b.md), with frame generation off.
8. Play one live-action video scene and one menu: both must sit flat at screen depth.
9. Only if a public NoMoreFlat build exists: try its OpenXR build against WiVRn from the Heroic prefix.

Proposed issues (titles only):

- Alan Wake 2: decide how a non-Steam (Heroic) game gets a recipe key and a launch wrapper
- Alan Wake 2: recon the launch path on the device (process tree, Proton, prefix, wrapper reach)
- Alan Wake 2: proxy DLL load check under Heroic's Proton
- Alan Wake 2: match Control's exported `rend::View` functions into the exe and record the address map
- Alan Wake 2: RenderDoc capture of the view constants
- Alan Wake 2: one-eye camera shift through the `rend::View` hook
- Alan Wake 2: alternate-eye render with per-eye previous view, SBS output
- Alan Wake 2: upscaler and anti-aliasing history under alternating eyes
- Alan Wake 2: flat HUD and video at screen depth
- Alan Wake 2: Stop 3D restores `renderer.ini` and removes the proxy DLL
- Alan Wake 2: watch for a public NoMoreFlat release and re-rate the fallback
- SPEC: move Alan Wake 2 from "out of scope" to the 1.0 list

## 7. Open questions for the owner

1. **Does alternate-eye rendering count as true stereo?** Ruling 3 leaves it open, and both routes for this game depend on it. If it does not count, this game needs a two-pass render, which the handheld cannot hold at 30 fps per eye at today's settings.
2. **Is about 28 fps per eye acceptable for this game,** or should the route lower the render resolution further to win headroom?
3. **How should a non-Steam game be handled:** a recipe keyed by the local shortcut id, a store key in the recipe with Heroic support in the engine, or the exe added to Steam directly?
4. **Order.** The ruling gave no order for the three new games. This note assumes they come after the first four, as [SPEC.md](../../../../SPEC.md) and issue #11 had it (007 First Light before Alan Wake 2).
5. **The clean-install hold.** [games_group_b.md](../games_group_b.md) records a request not to mod this game while a separate graphics-bug investigation needs a clean install. Is that hold over?
6. **If the NoMoreFlat build stays on Patreon,** is it ruled out for good under the free-only rule, or may it be bought once as a reference for the hook points?
