# Control Ultimate Edition (Steam 870780): Geo-11 under Proton, the first Geo-11 test game

Researched 2026-11-11 on the handheld (SteamOS), for issue #259. Web sources plus a read-only look at the installed copy, its Proton prefix and the installed Proton 11.0 launch script. Nothing was launched, written or downloaded; the two download links were checked with a header request only.

This note builds on [games_group_b.md](../games_group_b.md) (Control section) and [stereo_tools_landscape.md](../stereo_tools_landscape.md) (Geo-11 section) and does not repeat them. Each finding is marked **observed** (seen in the local files on 2026-11-11), **sourced** (a linked page says so) or **inferred** (my reasoning, not yet shown).

Why now: by the owner's ruling of 2026-11-11 Planet of Lana leaves 1.0, so Control is the first game to try Geo-11 on (step G1, #43), not the second.

## Short answer

- The route stays **Geo-11 + the community 3D fix + the SBS viewer**, on `Control_DX11.exe`.
- Use **Geo-11 0.7.12** (2026-11-04) with **Losti and Masterotaku's fix 1.01** (2019-11-30). There is no newer public fix build.
- Three things in the install stand in the way today, all observed: the Steam launch options force `-dx12`, a third-party `dxgi.dll` scaler test sits beside the exe, and the game build is the March 2025 one, which commenters say broke parts of the 2019 fix.
- Under Proton 11.0 no `WINEDLLOVERRIDES` should be needed for Geo-11's `d3d11.dll` and `nvapi64.dll` (inferred from the Proton script and from how the scaler test loads today).
- Biggest risk: the fix's shader matches are from 2019 and the game's shaders changed in March 2025. Depth may be right while shadows and lights are wrong.

## 1. Identity

| Fact | Value | Basis |
|---|---|---|
| Steam appid | 870780, "Control Ultimate Edition", build id 21225456 | observed: `appmanifest_870780.acf` |
| Engine | Remedy Northlight. No public version number; the file version of the exe and renderer DLL is `0.0.518.2177` | observed; engine name sourced in [games_group_b.md](../games_group_b.md) |
| Game version | The March 2025 update series (1.30 on 2025-03-10, small follow-ups after). The exe's link time stamp is 2025-03-23 | observed (time stamp); sourced: [GamingOnLinux, 2025-03-10](https://www.gamingonlinux.com/2025/03/control-ultimate-edition-is-adding-hdr-ultrawide-fov-scaling-updated-dlss-and-dlaa/) |
| Graphics APIs | Direct3D 11 (`Control_DX11.exe`) and Direct3D 12 (`Control_DX12.exe`). The store page lists DirectX 11 as minimum and 12 as recommended | observed; sourced: [Steam store data](https://store.steampowered.com/api/appdetails?appids=870780&l=english) |
| What Steam starts | `Control.exe`, a small .NET launcher (90 KB), with no arguments. It understands `-dx11`, `-dx12` and `-showlauncher`, remembers the last choice, and since 1.30 picks DX11 or DX12 by itself | observed: Steam's app info and the launcher's strings; sourced: the 1.30 notes say "automatic selection between DX11 and DX12" ([GamingOnLinux](https://www.gamingonlinux.com/2025/03/control-ultimate-edition-is-adding-hdr-ultrawide-fov-scaling-updated-dlss-and-dlaa/)) |
| Real process | `Control_DX11.exe` (original name `Game_rmdwin7_f.exe`) or `Control_DX12.exe` | observed |
| Account, launcher | None beyond Steam. The store data has no third-party account or DRM notice | sourced: [Steam store data](https://store.steampowered.com/api/appdetails?appids=870780&l=english) |
| DRM | Steamworks API only (`steam_api64.dll`). Neither exe has the extra section the Steam DRM wrapper adds, so they are not wrapped | observed (section list of both exes) |
| Anti-cheat | None. Single-player; not in the anti-cheat database | sourced in [games_group_b.md](../games_group_b.md) |
| Does anything block DLL injection | No. A third-party proxy `dxgi.dll` already loads in this install (section 2) | observed |
| Offline play | Expected to work: no account, no online feature. Not tested | inferred |
| ProtonDB | Platinum, 528 reports, score 0.92. Steam Deck Verified | sourced: [ProtonDB summary](https://www.protondb.com/api/v1/reports/summaries/870780.json), [GamingOnLinux](https://www.gamingonlinux.com/2025/03/control-ultimate-edition-is-adding-hdr-ultrawide-fov-scaling-updated-dlss-and-dlaa/) |

## 2. Installed copy (static inspection only)

Installed on the internal drive at `<Steam library>/steamapps/common/Control/`, about 45 GB. All of this is observed.

**Layout.** Everything is in the game's root folder: `Control.exe`, `Control_DX11.exe`, `Control_DX12.exe`, and two full sets of engine DLLs, `*_rmdwin7_f.dll` for DX11 and `*_rmdwin10_f.dll` for DX12 (renderer, d3d, coregame, physics, ui and so on). Game data is in `data/` and `data_packfiles/`. Geo-11's files therefore go in the root folder, next to the exes.

**What the DX11 build loads.** `Control_DX11.exe` imports only the `rmdwin7` DLLs. `d3d_rmdwin7_f.dll` imports `d3d11.dll` and calls `D3D11CreateDevice` and `D3D11CreateDeviceAndSwapChain`; it does not import `dxgi.dll` or `nvapi64.dll`. That is the plain case Geo-11's proxy `d3d11.dll` is built for. The DX12 build's `d3d_rmdwin10_f.dll` imports `d3d12.dll` and `dxgi.dll`.

**Symbols.** No PDB ships. The exes name their PDBs (`...\RmdWin7\Final\Game_rmdwin7_f.pdb`), which only confirms a final, stripped build.

**Stereo or VR code.** None found. The DX11 exe holds no OpenVR, OpenXR, NVAPI stereo or multiview strings. The lone words `Stereographic` and `Hmd` appear once each with no context around them; I read them as unrelated (inferred).

**Upscalers shipped by the game.** `nvngx_dlss.dll` (DLSS). No FSR or XeSS files of the game's own.

**Settings file.** `renderer.ini` in the game folder (JSON). Today: 1920x1200 output and render size, window mode 1, VSync off, HDR allowed (`m_eAllowHDR: 1`), film grain on, motion blur on, SSR and global illumination at 2, ray-tracing values set (they only apply in DX12). A sibling backup copy from 2026-10-06 is also there.

**The third-party scaler test (the collision).** Since 2026-10-17 the folder also holds a HelixSR Scaler test, version 1.0.6, listed in its own manifest `HelixSR-Scaler.files` (30 files):

- `dxgi.dll` (26 MB), an OptiScaler-based proxy. This is the file that matters.
- `OptiScaler.ini`, `OptiScaler/` (FSR, XeSS and Streamline DLLs), `HelixSR/` (`helixsr.dll`, `helixsr.ini`) and `Licenses/`.
- `HelixSR-Scaler-remove.sh`, the test's own remover. Its header says it puts back any file the game had before from a `HelixSR-Scaler-backup` folder. No such folder exists, so the test replaced nothing.

`HelixSR/helixsr.ini` says it is rewritten "each time the game starts", and its time stamp matches the last play (2026-10-18). So the proxy did load on the last run.

**Steam settings for the game.** Launch options are `%command% -dx12`. No per-game Proton choice is set, and the prefix was last used with Proton 11.0. No other launch variable is set, so the scaler's `dxgi.dll` loaded with no DLL override.

**The collision, spelled out.**

1. Geo-11 needs the DX11 exe; the scaler test needs DX12 (its payload is DX12 FSR and XeSS, and the launch options force `-dx12`). The two cannot both be on for one launch.
2. With `-dx11` and the scaler's `dxgi.dll` still in place, DXVK's `d3d11.dll` would pull that `dxgi.dll` from the game folder first (inferred: the game folder is searched before the system folder), so Geo-11 and OptiScaler would both wrap the same swap chain. Nobody has reported that pair working.
3. Geo-11 0.7.7 and later also ship an optional `dxgi.dll` of their own, a "loader to force early loads" ([Geo-11 announcement, changelog](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)). It is not one of the three files the install steps name, and the planned route (WP-16, PR #250) copies only `d3d11.dll`, `nvapi64.dll` and `d3dxdm.ini`. Keep it that way: do not copy Geo-11's `dxgi.dll` for Control.
4. So for a 3D launch the scaler's `dxgi.dll` has to be out of the folder and the launch argument has to be `-dx11`; for a flat launch the owner may want both back. Moving the single file `dxgi.dll` aside is enough to switch the test off, because every other file of the test is only loaded by it (inferred). The seed already says `geo11-remove-first: dxgi.dll`.

I removed nothing.

## 3. Ways to get true stereo

| Way | True stereo | Version, date | Licence, price | Under Proton with WiVRn |
|---|---|---|---|---|
| **Geo-11 + Losti and Masterotaku's fix** | Yes, both eyes drawn in the same frame | Geo-11 0.7.12 (2026-11-04); fix 1.01 (2019-11-30) | Both free. Geo-11 has no stated licence (the author plans "open source with a restrictive license"); the fix states none | Geo-11's page says "Linux with Proton also works". It writes SBS into the game's own picture, so it needs our SBS viewer to reach the headset. Not shown on this game |
| vorpX | Yes (its geometry mode) | - | Paid: excluded by the decisions | - |
| Geo-11 `katanga_vr` output | Yes | Part of Geo-11 | The headset app ships only inside the paid HelixVision | Excluded |
| Dedicated VR mod, R.E.A.L. VR, engine stereo switch | - | None found for Control ([games_group_b.md](../games_group_b.md)); no stereo code in the exe (section 2) | - | - |
| UEVR | - | Not an Unreal game | - | - |
| ReShade SuperDepth3D and similar | **No**: one view plus a depth map | - | Free | Named only; does not count |

### Geo-11: the exact builds

- **Geo-11 0.7.12**, dated 2026-11-04: `https://bo3b.s3.us-east-1.amazonaws.com/geo-11/geo-11_v0.7.12.7z`, 6,548,894 bytes, server date 2026-11-04 (header request, observed). This is the version already written in `packaging/pins`. Its changelog since 0.7.7 matters here: SBS became the default output, AMD crashes were fixed in 0.7.9 ("should be fully compatible now"), and 0.7.9's test list has the line "Control (x64) in SR mode. Runs, but had some crashes" — [Geo-11 announcement](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html) (sourced).
- **The fix, 1.01**: `https://losti.s3.amazonaws.com/Control_3D-VisionReadyFix_1.01_by_Losti_Masterotaku.zip`, 3,296,439 bytes, server date 2019-11-30 (header request, observed). It is a 3DMigoto fix: its own `d3d11.dll`, a `d3dx.ini` and a `ShaderFixes` folder — [Helix Mod: Control fix](https://helixmod.blogspot.com/2019/10/control-3d-vision-ready-fix-losti-v100.html) (sourced).
- **No newer fix exists in public.** Losti said in 2021 that Masterotaku had a newer one; Geo-11's author looked in 2022 and could not find it (comments on the fix page, sourced).
- **Install order** from the Geo-11 page (sourced): install the fix, then copy Geo-11's `d3d11.dll`, `nvapi64.dll` and `d3dxdm.ini` from its `x64` folder over it, keep the fix's `d3dx.ini`, and set `force_stereo=2` in it. Output is chosen with `direct_mode` in `d3dxdm.ini` (`sbs` is the default); depth with `dm_separation` (a percentage) and `dm_convergence`.
- Neither pin can be completed from here: taking the SHA-256 needs the download, which this pass was not allowed to do.

### What users say about this fix on current game builds (comments on the fix page, sourced, unverified)

- March 2025: a game update "broke depth for many elements"; a later March update made it work again, but only in Geo-11's `nvidia_dx9` output, with crashes in `sbs`, `tab` and interlaced.
- June 2025: a Steam copy works with Geo-11; one user fetched an older Steam depot to get there. A reversed crosshair went away after deleting the cached shaders.
- October 2025: on AMD and on RTX 40 cards, shadows are broken on the Steam version and fine on Epic. Avoid F10. Borderless works better than exclusive fullscreen.
- December 2025: on Epic, geometry is right but highlights, shadows and lights are wrong, and only `interlaced_reversed` output had any effect.
- March 2026: setting `get_resolution_from = swap_chain` in `d3dx.ini` "fixed shadows and one-eye objects".
- The fix's own list still applies: DX11 only, some ambient shadows wrong, screen-space reflections not exact, HUD depth slips. Its author advises motion blur and film grain off, and global reflections off if they bother.

### Geo-11 under Proton: what is known

- **First-hand reports exist, but not for Control.** A Steam Deck user ran The Witcher 3 with Geo-11 by copying `d3d11.dll`, `nvapi64.dll`, `d3dxdm.ini` and `d3dx.ini` into the game folder, and it "worked out of the box"; Alien: Isolation worked too, and its upscale setting only worked after switching to windowed fullscreen and Proton Experimental (comments of June and July 2023 on the [Geo-11 announcement](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html), sourced). Neither report mentions a DLL override.
- **SteamTinkerLaunch has had a Geo-11 switch since July 2022.** It copies the files from Geo-11's `x32` or `x64` folder next to the exe and can start an SBS VR player ([SteamTinkerLaunch wiki: Geo11](https://github.com/frostworx/steamtinkerlaunch/wiki/Geo11), sourced). Its script sets no DLL override for Geo-11 and its downloader looks for a `.zip` link, while Geo-11 now ships as `.7z`, so its automatic download is probably broken today (read from the script at commit `8550ab26a712`; the second half is inferred).
- **Proton 11.0 already marks the needed DLLs as native** (observed in the installed Proton 11.0 launch script, build `11.0-2e`): it sets `d3d11`, `d3d10core`, `d3d9` and `dxgi` to native for DXVK, and `nvapi64` to native because NVAPI is on by default and Control is not in the script's "disable NVAPI" list. Its own overrides are added after any the user gives.
- **So no override should be needed** (inferred): a native DLL is looked up in the game folder before the system folder, which is how the scaler's `dxgi.dll` loads today with launch options of only `%command% -dx12` (observed). The earlier notes' `WINEDLLOVERRIDES="d3d11=n,b"` is then harmless but not required; `nvapi64=n,b` likewise. The first device test should try with no override.
- **The chain** (inferred): the game's `d3d_rmdwin7_f.dll` asks for `d3d11.dll` and gets Geo-11's from the game folder; Geo-11 loads the real one from the system folder, which under Proton is DXVK 2.7.1 (observed in the prefix). Geo-11's `nvapi64.dll` sits in front of DXVK-NVAPI the same way; on an AMD GPU DXVK-NVAPI does nothing useful, and Geo-11 says it needs no NVIDIA software.
- **Unknown:** whether Geo-11's SBS present path works on DXVK with this game. The crash reports in `sbs` after the March 2025 update are on Windows; nobody has reported Control + Geo-11 on Linux.

## 4. Recommended route for 1.0, and a fallback

### Recommended: Geo-11 0.7.12 + fix 1.01 on `Control_DX11.exe`, SBS, our viewer

Settings and hook points:

1. Move the scaler's `dxgi.dll` aside (with an undo record), nothing else.
2. Unpack fix 1.01 into the game's root folder; copy Geo-11 0.7.12's `x64` `d3d11.dll`, `nvapi64.dll` and `d3dxdm.ini` over it. Do not copy Geo-11's `dxgi.dll` loader.
3. `d3dx.ini`: `force_stereo=2`. If shadows are broken, try `get_resolution_from=swap_chain` (the route's `geo11-d3dx` key already accepts both).
4. `d3dxdm.ini`: `direct_mode=sbs`, starting from `dm_separation=40` and `dm_convergence=2.0` (the page's example values).
5. Launch argument `-dx11` in place of today's `-dx12`. The launcher then starts `Control_DX11.exe`. No `WINEDLLOVERRIDES` on the first try.
6. In the game (or `renderer.ini`): borderless window, HDR off, motion blur and film grain off, MSAA off; global reflections off if the picture swims. Keep 1920x1200 for the first test.
7. The SBS viewer (#44) shows the game window's two halves on the fixed screen.

Risks:

- **The March 2025 build against a 2019 fix** (the biggest). Expect correct geometry depth and possibly wrong shadows, lights and highlights. A game with wrong-eye shadows is not comfortable to play.
- **SBS output crashing** on this build, as two Windows users reported.
- **Frame rate.** Geo-11 draws the scene for both eyes, and the DX11 build has none of the scaler or frame-generation help the owner uses in DX12 today. 30 fps at 1920x1200 is unproven; dropping the render size is the first lever.
- **Half-width eyes.** In `sbs` each eye gets half the window width unless Geo-11's `upscaling` is used, and that setting was the fragile one on the Steam Deck report.
- **The flat game changes too.** While 3D is set up, the DX12 scaler test is off. Undo has to put back both the `dxgi.dll` and the `-dx12` argument.
- **Licence.** Neither Geo-11 nor the fix states one. We fetch them from their authors' own links at install time and ship neither (rulings 8 and 16 are about free, not about redistribution; #46 asks for this check).

### Fallback: the same route on the game build from before March 2025

If the current build's shadows or SBS output cannot be made right with settings, the owner can fetch the pre-1.30 depot of the game with Steam's own depot download (one commenter did exactly that) and point the route at that copy. The fix was reported complete on those builds through 2021. Cost: about 45 GB more disk, no HDR or ultrawide options (not needed here), and Steam must not update that copy. The manifest id of the last pre-1.30 build has to be looked up (SteamDB lists them); I did not confirm one.

If that fails too, the remaining choice is to repair the fix's shaders ourselves with 3DMigoto's hunting mode, which is days of work per effect and after 1.0. Control would then leave 1.0 rather than hold it.

## 5. Proposed seed recipe

This replaces the text of the current seed `crates/engine/tests/fixtures/recipe-seeds/870780-control.recipe`; no fixture is changed in this PR.

```text
# Seed: untested until sessions/g1-geo11.toml and g4-control.toml pass; the real recipe lands in recipes/ at works-now.
# Game build checked: Steam build 21225456 (March 2025 series), Control_DX11.exe file version 0.0.518.2177.
# Geo-11 0.7.12 (pin `geo11`) + Losti and Masterotaku's fix 1.01 (pin `control-fix`).
name: Control Ultimate Edition
stereo-status: untested
stereo-route: geo-11 + sbs viewer (control_dx11.exe)
stereo-engine: northlight
stereo-api: dx11
stereo-mod: https://helixmod.blogspot.com/2019/10/control-3d-vision-ready-fix-losti-v100.html
stereo-free: "yes"
stereo-safe: "yes"
stereo-checked: 2026-11-11
geo11-fix: control-fix
geo11-exe: Control_DX11.exe
geo11-layout: sbs
geo11-separation: 40
geo11-convergence: 2.0
geo11-remove-first: dxgi.dll
# Add only if shadows are wrong on the device:
# geo11-d3dx: get_resolution_from=swap_chain
```

Three things the recipe format cannot say yet:

- **The launch argument.** `-dx11` has no key. Without it the launcher repeats the last choice or picks by itself, and today's Steam launch options say `-dx12`.
- **The name and mod link differ from #49's locked test**, which expects the name `Control Ultimate Edition` (the seed says `Control`) and the Geo-11 announcement as `stereo-mod` (the seed has the fix page). The block above takes #49's name and route text and keeps the fix page; #49 has to settle the link.
- **The pins.** `control-fix` and `geo11` still have `TBD-at-pin` for url, size and sha256. The urls and sizes are in section 3.

## 6. On-device tests still needed (cheapest first)

1. Flat check: launch options `%command% -dx11`, scaler `dxgi.dll` still in place. Does the DX11 build start and reach the menu? (Shows whether the scaler proxy alone breaks DX11.)
2. Move the scaler's `dxgi.dll` aside; start with `-dx11` again; note the flat frame rate at 1920x1200 with HDR off. This is the baseline to halve.
3. Geo-11 0.7.12 alone (no fix), `force_stereo=2`, `direct_mode=sbs`, no override. Does the game start, and is the window two different views? Read `d3d11_log.txt` to confirm Geo-11 loaded in front of DXVK.
4. If step 3 does not load Geo-11: repeat with `WINEDLLOVERRIDES="d3d11,nvapi64=n,b"`.
5. Add fix 1.01. Check shadows, lights, the HUD and the crosshair in the first office area; then try `get_resolution_from=swap_chain`; then delete the fix's shader cache and retry.
6. If `sbs` crashes: try `tab`, then Proton Experimental.
7. Measure: disparity from an SBS screenshot (the `disparity` tool) and frame rate from a MangoHud log, at 1920x1200 and at a lower render size.
8. With the viewer (#44): a mission section in the headset on the gamepad, 30+ fps (M22).
9. Undo: the scaler's `dxgi.dll` and `-dx12` come back, and the DX12 game starts as before.
10. Only if step 5 cannot be made right: the fallback, on a pre-1.30 depot.

Proposed issues (titles only):

- G1: prove Geo-11 under Proton on Control, not Planet of Lana (rewrite #43 and `sessions/g1-geo11.toml`)
- Recipes: a launch-argument key, so a route can ask for `-dx11`
- Geo-11 route: restore the removed proxy DLL and the launch argument on undo, and never copy Geo-11's own `dxgi.dll` loader
- Pins: fill `geo11` 0.7.12 and `control-fix` 1.01 (url, size, sha256)
- Control: settle the recipe name and `stereo-mod` link between the seed and #49's locked test
- Control: fallback on the pre-1.30 game depot (open only if the shadow test fails)

## 7. Open questions for the owner

1. Is the DX12 scaler test in Control still wanted? If yes, the 3D switch has to swap `dxgi.dll` and `-dx11`/`-dx12` on every change between 3D and flat; if no, the owner can run the test's own remover once and the route gets simpler.
2. If the current game build shows wrong shadows with the fix, is a second, older copy of the game (about 45 GB, never updated) acceptable, or does Control then leave 1.0?
3. Is half-width SBS (960x1200 per eye from a 1920x1200 window) good enough for the first pass, or should the test also try Geo-11's `upscaling`?
4. Order: Control now carries G1. Should it also move ahead of ACE COMBAT 7 in the 1.0 order, since the Geo-11 answer unblocks the viewer work (#44)?
5. Geo-11 and the fix state no licence. Fetching them from the authors' links at install time and shipping neither is what the pins assume. Is that enough, or should the authors be asked?
