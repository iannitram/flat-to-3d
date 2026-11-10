> Raw research notes from 2026-10-18, kept as written. Where they conflict with [DECISIONS.md](../DECISIONS.md) (for example paid-tool routes), the decisions win. Some paths and names were generalized when these moved into the repo.

# True stereo 3D on SteamOS: games group A (ACE COMBAT 7, STAR WARS Zero Company, Planet of Lana, Forza Horizon 6, 007 First Light)

Researched 2026-10-18 on the Z13 (SteamOS). Web sources plus a read-only look at the install folders (paths from `library-scan.tsv`). Where a finding comes from the local files and not the web, it is cited as "local inspection" with the path. ProtonDB tiers come from ProtonDB's public summary API on 2026-10-18.

Abbreviations: **UEVR** = praydog's Unreal Engine VR injector. **VRto3D** = oneup03's SteamVR driver that shows a VR mod's two eye views as a flat SBS/TAB 3D picture. **Geo-11** = Helix/3DMigoto-based DX11 driver that draws the scene in true geometric stereo and outputs SBS/TAB. **SBS** = side-by-side.

## Shared tooling: what carries across these five games

### Takeaway
There are two proven free routes to a flat two-camera 3D picture: **UEVR + VRto3D** for Unreal games (VRto3D now runs natively on Linux SteamVR) and **Geo-11** for DX11 games (it says Proton works). Neither is proven end to end on SteamOS with a Quest viewer, so each needs a Z13 test. DX12 games (Zero Company, FH6, 007) have no free generic stereo injector.

### Cited Findings
- VRto3D is an OpenVR (and OpenXR) driver for SteamVR that lets VR-modded flat games run on 3D displays. It renders one SBS frame and repacks it to SbS, TaB, FramePacked, interlaced, anaglyph, Virtual Desktop and other modes. By default there is no motion tracking (a fixed head), with optional right-stick pitch/yaw emulation and OpenTrack input — [VRto3D GitHub](https://github.com/oneup03/VRto3D)
- VRto3D runs natively on Linux with SteamVR (about 2.16+) on a Vulkan GPU, X11 or Wayland. LeiaSR, 3D Vision and WibbleWobble modes are not available on Linux, and hotkeys need the user in the `input` group. Licence LGPL-3.0 — [VRto3D GitHub](https://github.com/oneup03/VRto3D)
- VRto3D latest GitHub releases: `V5.0.0` (2026-07-18) and a rolling `latest` (2026-10-12) — [VRto3D releases API](https://api.github.com/repos/oneup03/VRto3D/releases)
- VRto3D can show "half SbS/TaB" at half resolution per eye to save GPU time. Temporal effects (TAA, DLSS) can cause halos — [VRto3D GitHub](https://github.com/oneup03/VRto3D)
- PCGamingWiki's "Native 3D" route for UE4 games is "UEVR with VRto3D" (there is a UEVR QuickStart in the VRto3D wiki) — [PCGamingWiki: Ace Combat 7](https://www.pcgamingwiki.com/wiki/Ace_Combat_7:_Skies_Unknown); [VRto3D UEVR QuickStart](https://github.com/oneup03/VRto3D/wiki/UEVR-QuickStart)
- UEVR under Proton: install the .NET 6.0 desktop runtime into the game's prefix with protontricks. Then launch the injector inside the same wineserver with `PROTON_REMOTE_DEBUG_CMD="$HOME/Documents/uevr/UEVRInjector.exe" %command%`. Pick the process, then pick **OpenVR for SteamVR** or **OpenXR for Monado/WiVRn**, and inject. Nightly builds are recommended. Page last modified 2026-09-04 — [VR on Linux wiki: UEVR Injector](https://vronlinux.org/docs/games/uevrinjector/)
- Proton 9+ is required for Proton games to start VR. xrizer (OpenVR→OpenXR) is recommended for Proton/OpenVR games under WiVRn/Monado, and the WiVRn Flatpak bundles xrizer and OpenComposite — [VR on Linux wiki: WiVRn](https://vronlinux.org/docs/fossvr/wivrn/); [WiVRn GitHub](https://github.com/WiVRn/WiVRn)
- The latest UEVR nightly is 01143 (2026-08-30) — [UEVR-nightly releases](https://github.com/praydog/UEVR-nightly/releases); [GitHub API](https://api.github.com/repos/praydog/UEVR-nightly/releases)
- The documented UEVR engine range is UE 4.8–5.4 (README, cached ~April 2025). I found no statement about 5.5/5.6 — [awesome.ecosyste.ms UEVR mirror](https://awesome.ecosyste.ms/projects/github.com%2Fpraydog%2Fuevr)
- Geo-11 "does create geometric 3D for DX11", built on 3Dmigoto hooking and shader manipulation. It needs "no NVidia software" and "has been shown to run on AMD cards". Output modes sbs/tab/interlaced/checkerboard/anaglyph/katanga_vr need no 3D Vision driver. The page says "Linux with Proton also works". Files: `d3d11.dll`, `nvapi64.dll`, `d3dxdm.ini`. The cost claim is "higher performance than 3D Vision", with one profile at "max 1.5% GPU" (overhead only) — [Helix Mod: Announcing Geo-11](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)
- None of the five games appears in the AreWeAntiCheatYet database (games.json, master, 2026-10-18), so it tracks no client anti-cheat for them — [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json)

### Inferences
- The "giant fixed 3D TV" fits well: run SteamVR + VRto3D (fixed head, SBS out) on the Z13. Show the SBS window/desktop in the Quest on a virtual 3D screen in SBS mode (e.g. a viewer that supports SBS video/desktop). A second path is UEVR straight to the headset with head tracking turned off or locked, which is harder because UEVR is built around head tracking. VRto3D was made for the fixed-screen case.
- Geo-11 renders the scene twice with per-eye shader corrections. Its "1.5% GPU" figure is overhead only: the second eye still costs roughly the full scene again, so plan for about 1.6–2× GPU frame time at the same resolution. Half-SBS at half width per eye cancels much of that.
- Frame generation (lsfg-vk), which my game notes pages already use, works on the final image. Running it over an SBS image is plausible, but it is not tested and could smear the eye seam.

### Gaps
- No source confirms VRto3D on SteamOS (Valve's Linux SteamVR inside gamescope/Game Mode) as opposed to a desktop Linux distro.
- No source confirms Geo-11 under the DXVK that ships with Proton, beyond the one-line "Linux with Proton also works".
- I found no confirmation that UEVR supports UE 5.6.

## ACE COMBAT 7: Skies Unknown

### Takeaway
**Verdict: likely with work (best candidate of the five).** UE4 + DX11, Denuvo removed, no anti-cheat, single-player campaign. There is a dedicated free UEVR compatibility mod, and PCGamingWiki lists "UEVR with VRto3D" as its true-stereo 3D route. The PSVR-exclusive VR missions never came to PC. Not installed now (probably on `sandisk_4tb`).

### Cited Findings
- Engine/API: Unreal Engine 4; Direct3D 11 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Ace_Combat_7:_Skies_Unknown); my game notes also list UE4, DX11 → DXVK, process `Ace7Game.exe` — my notes on Ace Combat 7
- DRM: Steam. Denuvo Anti-Tamper was removed on 2021-10-19 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Ace_Combat_7:_Skies_Unknown)
- Online: the game has 8-player online Versus with a lobby browser. The campaign and single-player missions are offline — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Ace_Combat_7:_Skies_Unknown)
- Anti-cheat: none listed (not in AreWeAntiCheatYet) — [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json)
- ProtonDB: Platinum (211 reports, trending Platinum) — [ProtonDB](https://www.protondb.com/app/502500)
- PCGamingWiki VR rows: OpenXR "hackable" and SteamVR "hackable" via the [UEVR Compatibility Mod (VR patch)](https://www.nexusmods.com/acecombat7skiesunknown/mods/2387). Native 3D is "hackable": "Dual-View Rendering — use UEVR with VRto3D, import the UEVR Compatibility Mod config", SBS or TAB by default. 3D Vision is hackable through the same path — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Ace_Combat_7:_Skies_Unknown)
- User report (Dec 2025): the whole campaign plus two of three SP missions were played in UEVR with few problems — [r/acecombat comment](https://redlib.hackliberty.org/r/acecombat/comments/1pij7dz/ac7_in_vr_kinda_rips/ntdw4rw/?context=3)
- In UEVR the official VR cockpits (F-22, A-10, Su-30, F/A-18) "work flawlessly". Other planes often lack radar/ammo readouts, and radio subtitles and portraits do not show — [r/acecombat comment](https://redlib.hackliberty.org/r/acecombat/comments/1pij7dz/ac7_in_vr_kinda_rips/nt7euyw/?context=3)
- Setup with a Quest 2 was "straightforward". Menus follow the head, which UEVR's 2D-screen hotkey works around for menus and cutscenes — [Steam discussion](https://steamcommunity.com/app/502500/discussions/0/6597293740385910423)
- The PSVR content was three VR missions plus Free Flight and Air Show, PS4-only at launch — [Push Square](https://www.pushsquare.com/news/2019/01/hands_on_ace_combat_7_skies_unknown_in_psvr_passes_with_flying_colours)
- The PS4 box said "VR Missions exclusive to PS4 until January 18, 2020", which led to speculation about a later PC VR release — [UploadVR](https://uploadvr.com/ace-combat-7-vr-pc-2020/); [GamesBeat](https://gamesbeat.com/ace-combat-7s-vr-missions-could-hit-oculus-rift-and-vive-pc-headsets-in-2020/)
- A Helix/3D Vision fix: PCGamingWiki lists no standalone Helix/Geo-11 fix for AC7. Its 3D Vision row points to the UEVR+VRto3D path — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Ace_Combat_7:_Skies_Unknown)
- UEVR on Proton: the generic `PROTON_REMOTE_DEBUG_CMD` method (see Shared tooling) — [VR on Linux wiki](https://vronlinux.org/docs/games/uevrinjector/)

### Inferences
- The route is: AC7 under Proton → UEVR (nightly) via `PROTON_REMOTE_DEBUG_CMD`, OpenVR runtime → SteamVR with the VRto3D driver (fixed head, SBS) → the SBS picture shown on a virtual 3D screen in the Quest. Import the Nexus compatibility-mod profile. Use VRto3D's fixed pose, not head steering, which matches the brief.
- "Free" caveat: the Nexus mod page could not be fetched (HTTP 403), so I could not confirm whether it is free or tied to Patreon. Nexus mods are usually free downloads.
- Performance: AC7 already reaches a 90 fps base on the Z13 at 1920×1200 Epic (my game notes, local). Two full eyes at 1920×1080 each, or half-SBS, should hold 30–45+ fps at Medium/High. UE4 DX11 is light. This is an estimate, not measured.
- No PC mod restores the PSVR missions that I could find. The missions' cockpit assets do exist in the PC build (UEVR users see "official VR cockpits" for four planes), which suggests the VR assets shipped but the PSVR mission mode did not.

### Gaps
- Not found: any mod that unlocks the three PSVR missions / Free Flight / Air Show on PC. Searches returned only 2019–2020 news.
- I could not read the Nexus mod page (403): author, version, date and price are not confirmed.
- No report of AC7 + UEVR under Proton specifically. No report of VRto3D + UEVR on Linux for this game.

## STAR WARS Zero Company

### Takeaway
**Verdict: likely with work, uncertain (UE5.6 is past UEVR's documented range).** Shipped 2026, UE 5.6.1, DX12 only, no client anti-cheat found, single-player tactics game with a real 3D world. I found no UEVR profile or report. Test UEVR nightly + VRto3D. If injection fails, "build our own" means UEVR-style work on UE 5.6's stereo path.

### Cited Findings
- Engine: Unreal Engine 5, build 5.6.1.0 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Star_Wars_Zero_Company). my game notes: UE 5.6, **DX12 only** → vkd3d-proton (no `-dx11`), real process `SWZeroCompany\Binaries\Win64\SWZeroCompany.exe`, asks once to link an EA account, ProtonDB Platinum — my notes on Star Wars Zero Company
- API: Direct3D 12 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Star_Wars_Zero_Company)
- Local inspection: the install has the standard UE layout (`Engine/`, `SWZeroCompany/Content/Paks/pakchunk*-Windows.pak`, `Binaries/Win64/SWZeroCompany.exe` + `.pdb`). There is no EasyAntiCheat/BattlEye/anti-cheat folder or file — local: `<Steam library>/steamapps/common/Star Wars Zero Company/`
- The shipped `SWZeroCompany.pdb` (debug symbols) is in `Binaries/Win64` — local inspection
- Anti-cheat: the PCGamingWiki anti-cheat field is blank. Not in AreWeAntiCheatYet — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Star_Wars_Zero_Company); [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json)
- EA's Javelin anti-cheat blocks Linux, and all new EA-published *multiplayer* titles are expected to use it — [GamingOnLinux](https://www.gamingonlinux.com/2025/04/ea-rebrand-and-refresh-their-anti-cheat-into-ea-javelin-anticheat-still-blocks-linux-steam-deck/page=1). Zero Company runs on Proton (Platinum on my game notes), so Javelin evidently does not block it.
- UEVR: no search found any Zero Company UEVR profile, report or Flat2VR mention — [search: mixed-news UEVR coverage](https://mixed-news.com/en/uevr-profiles-website/) (general UEVR, no Zero Company entry)
- UEVR's documented support was 4.8–5.4 as of ~April 2025 — [awesome.ecosyste.ms UEVR mirror](https://awesome.ecosyste.ms/projects/github.com%2Fpraydog%2Fuevr)

### Inferences
- It is a turn-based tactics game with an orbiting overhead camera and a full 3D world, so true stereo would give real depth (a tabletop look), and frame rate barely matters.
- The shipped PDB would make a custom hook or UEVR fix much easier (exact symbols for the renderer and `FSceneView` / stereo functions).
- The EA-account link is a one-time online login, not always-online play, so it does not by itself make the game "not safe". Treat it as online at first launch only. Unconfirmed: whether the game checks the link every launch.
- Performance: the game is already CPU-heavy in hub areas. The Z13 runs a 60 base with FSR Quality at 1920×1200. UEVR's synchronized/AFR stereo modes cost about 1.5–2× GPU. A 30 fps stereo target at FSR Performance/Balanced, Medium settings looks reachable. This is an estimate.
- Build-our-own path if UEVR does not support 5.6: UE5 still has the `IStereoRendering` / `ISpectatorScreenController` paths. A custom injector would (a) hook into the engine as UEVR does, using the PDB for symbol addresses, (b) register a stereo device that gives two eye views with a fixed IPD and a locked pose, (c) write the two views SBS into the back buffer (no OpenXR needed). That is about UEVR-plugin-sized work.

### Gaps
- No UEVR compatibility data for UE 5.6 or for this game was found. A Z13 test is needed.
- Whether the EA account link needs a connection on every launch is not documented.
- I could not check the Flat2VR Discord (login required).

## Planet of Lana

### Takeaway
**Verdict: likely with work (good candidate).** Unity 2020.3.41f1, IL2CPP, built-in render pipeline, DX11. Its scenes are 3D-rendered (a Geo-11 "dual-view rendering" fix exists, from Dec 2025), so real depth is possible. Geo-11 outputs SBS and says Proton works. UUVR (Rai Pal) is a weaker fallback because its current release is Mono-only.

### Cited Findings
- Engine: Unity 2020.3.41f1 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Planet_of_Lana); confirmed locally: the `globalgamemanagers` header reads `2020.3.41f1` — local: `<Steam library>/steamapps/common/Planet of Lana/Planet of Lana_Data/globalgamemanagers`
- Scripting backend: **IL2CPP** (`GameAssembly.dll` + `il2cpp_data/` present, no `Managed/` folder) — local inspection of `<Steam library>/steamapps/common/Planet of Lana/`
- Render pipeline: built-in (no Universal/HD RenderPipeline assembly in `ScriptingAssemblies.json`). It uses `Unity.Postprocessing.Runtime` (PPv2), ProBuilder, FBX runtime and Burst — local: `Planet of Lana_Data/ScriptingAssemblies.json`
- API: Direct3D 11 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Planet_of_Lana); my game notes: DX11 → DXVK (Player.log confirms D3D11), ProtonDB Platinum, Deck Verified — my notes on Planet of Lana
- Anti-cheat/online: none. Single-player. Also sold DRM-free on GOG — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Planet_of_Lana); [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json)
- PCGamingWiki Native 3D is "hackable", "Dual-View Rendering — use Geo-11", SBS by default (`direct_mode=sbs`), TAB with `direct_mode=tab` — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Planet_of_Lana)
- Helix Mod Planet of Lana Geo-11 fix by cicicleta (2025-12-22), with credits to DarkStarSword, DHR, 4everAwake, Masterotaku, Losti. It fixes decals, water reflections, water caustics and a sky dust shader. Install: extract into the game folder and set `direct_mode` in `d3dxdm.ini` (default sbs). Commenters use half-SBS through Virtual Desktop, or full SBS via `katanga_vr` + Katanga/VrScreenCap — [Helix Mod: Planet of Lana Geo-11 fix](https://helixmod.blogspot.com/2025/12/planet-of-lana-geo-11-fix.html)
- Geo-11 works on AMD, needs no NVIDIA software for SBS/TAB, and "Linux with Proton also works" — [Helix Mod: Geo-11 announcement](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)
- UUVR (Universal Unity VR, GPL-3.0) is installed through Rai Pal. v0.4.0 (2025-07-03) is "only for uuvr-mono". The il2cpp-legacy assets are kept only so `/latest` works. v0.3.x (June 2024) were earlier builds — [UUVR GitHub](https://github.com/Raicuparta/uuvr); [UUVR releases API](https://api.github.com/repos/Raicuparta/uuvr/releases)
- No Planet of Lana UUVR/VR mod was found in search — [search results, none relevant] (no source to cite)

### Inferences
- "Real depth possible": yes. A Geo-11 fix that corrects decals, reflections and caustics per eye only makes sense with real 3D geometry under a perspective camera (Geo-11 makes geometric stereo, not depth reprojection). The ProBuilder/FBX assemblies point the same way. The look is a side-scroller on a 3D set (2.5D), so depth will be layered and diorama-like, not deep.
- Route: copy the Helix fix's `d3d11.dll`/`nvapi64.dll`/`d3dx*.ini` into the game folder and launch with `WINEDLLOVERRIDES="d3d11,nvapi64=n,b" %command%` so the 3Dmigoto proxy loads and chains to DXVK's d3d11. Set `direct_mode=sbs`, then show the SBS picture on the Quest's 3D screen. Geo-11 does not need SteamVR.
- Performance: the Z13 runs this game well (target 180 native at 1920×1200). Two eyes should easily hold a steady 60, and 30 with lots of headroom. This is an estimate.
- Risk: an `nvapi64.dll` override under Proton on an AMD GPU might clash with Proton's own nvapi handling (DXVK-NVAPI is normally off on AMD). The fix may need its bundled nvapi stub loaded native. Test it.

### Gaps
- No first-hand report of the Planet of Lana Geo-11 fix under Proton/DXVK.
- Whether UUVR's legacy IL2CPP build works on Unity 2020.3 IL2CPP is unknown, and Rai Pal is Windows-first.

## Forza Horizon 6

### Takeaway
**Verdict: not safe (as defined in DECISIONS.md) / build-our-own only if I rule otherwise.** FH6 has no client anti-cheat that blocks Proton. It is an online-first game (PCGamingWiki: "Broadband Internet connection required", Horizon Life multiplayer world, PlayFab cloud saves tied to the Xbox account), and my own wiki forbids injected DLLs here. The one known true-stereo VR mod is paid (Ko-fi) and unfinished. DX12 + ForzaTech has no free stereo injector.

### Cited Findings
- Engine/API: ForzaTech; Direct3D 12 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Forza_Horizon_6); my game notes: DX12 → vkd3d-proton, process `forzahorizon6.exe`. **Online game: keep changes to Proton-level FSR 4 + frame gen; no injected DLLs (OptiScaler etc.)**, and PlayFab saves tied to the Xbox account — my notes on Forza Horizon 6
- PCGamingWiki: "Broadband Internet connection required". Gaming Services 36.113.2002+ is required on Windows. Online play is "true". The anti-cheat field is blank — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Forza_Horizon_6)
- Local inspection: the install ships `PlayFabCore/GameSave/Multiplayer/Services.dll`, `xgameruntime.dll`, `XCurl.dll`, `libHttpClient.dll`, `networkmanifest.xml`, `MicrosoftGame.Config`. There is no EAC/BattlEye folder — local: `<Steam library>/steamapps/common/ForzaHorizon6/`
- Not in AreWeAntiCheatYet — [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json)
- Before launch GamingOnLinux noted "no shenanigans with anti-cheat or logins" — [GamingOnLinux (2026-03)](https://www.gamingonlinux.com/2026/03/forza-horizon-6-confirmed-to-be-playable-on-steam-deck-steamos). Valve made Proton Hotfix the default for FH6 on SteamOS after vkd3d-proton's developer called it "extremely broken" without workarounds — [GamingOnLinux (2026-05)](https://gamingonlinux.com/2026/05/proton-is-getting-some-horrible-workarounds-for-forza-horizon-6-on-linux)
- ProtonDB: Gold (1,082 reports, best Platinum, trending Gold) — [ProtonDB](https://www.protondb.com/app/2483190)
- Offline: a retailer listing says an internet connection is required at first launch. A blog says races and progression then work offline with AI Drivatars (low-quality source: a credits-selling site) — [GameStop](https://www.gamestop.com/products/forza-horizon-6/443392.html); [U4N](https://www.u4n.com/news/can-forza-horizon-6-be-played-offline.html)
- Stereo mod: a Reddit user showed FH6 in "real stereo 3D" VR with a motion rig. It is described as **a paid mod**, still in development and glitchy (jitter, distortion, eye strain), sold on Ko-fi. The creator is not named — [MIXED (2026-06-29)](https://mixed-news.com/en/watch-forza-horizon-6-in-vr-with-a-motion-simulator-looks-absolutely-insane/); [Reddit post](https://www.reddit.com/r/virtualreality/comments/1u19sn3/my_life_is_complete_forza_horizon_6_in_vr_motion/); [Ko-fi download](https://ko-fi.com/s/03bdcc5fe9)
- FH4/FH5: no official VR, and Playground does not plan VR. VorpX with Microsoft Store builds was described as poor. No 3D Vision/the R.E.A.L. VR author work for Forza turned up — [Windows Central](https://www.windowscentral.com/will-forza-horizon-5-support-vr); [Steam discussion](https://steamcommunity.com/app/1551360/discussions/0/4425436537579807508). PCGamingWiki's FH5 VR rows are empty — [PCGamingWiki FH5](https://www.pcgamingwiki.com/wiki/Forza_Horizon_5)

### Inferences
- DECISIONS.md rule 7 says any game with anti-cheat **or always-online play** is "not safe". FH6 needs broadband, runs a shared online world, and has PlayFab save sync that my game notes already warns about. Injecting a stereo DLL risks an Xbox-account ban and save desync even without a client anti-cheat. Mark it **not safe**.
- If I ever override this for offline-only play, the only true-stereo path known is the paid Ko-fi mod (flag: paid, unfinished). A self-built DX12 stereo hook for ForzaTech would be the hardest job here: no engine stereo path, a closed engine, D3D12 command lists to duplicate per eye. FH6 is also already CPU-bound on the Z13 (render thread ~78 avg / ~51 low base, from my game notes), and doubling draw submissions would drop it below 30.
- FH4/FH5 have no stereo mods to carry over.

### Gaps
- No primary Playground/Xbox statement on whether FH6 has a true offline mode on PC, or on its ban policy for modified clients.
- The Ko-fi mod's method (true two-view vs depth) and price are not confirmed.

## 007 First Light

### Takeaway
**Verdict: build our own (with a strong lead), single-player and safe after Denuvo's removal.** Glacier 2, DX12, Denuvo removed 2026-07-24, no anti-cheat, single-player. I found no stereo/VR mod. The shipping exe still contains Glacier's **OpenVR VR mode and a "Stereoscopic" display option** (strings plus `openvr_api.dll` in `Retail/`), inherited from Hitman 3's PC VR. If that dormant code can be switched on (by config or a patch), it is a native two-camera path, the best kind.

### Cited Findings
- Engine/API: Glacier 2; Direct3D 12 — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/007_First_Light); my game notes: process `Retail\007FirstLight.exe`, scales poorly (Low only ~25% faster than Ultra), FSR 4 swap works — my notes on 007 First Light
- Denuvo was added just before launch, with online check-ins reported — [Tom's Hardware](https://www.tomshardware.com/video-games/pc-gaming/007-first-light-drops-last-minute-denuvo-drm-bombshell-leaving-pre-order-customers-and-fans-furious-bond-title-risks-fps-drops-and-strict-online-check-ins-pirates-eagerly-await-repack-as-day-one-cracks-become-the-norm). It was **removed on 2026-07-24** (update 1.1.0, not stated in the patch notes) — [PCGamingWiki (SteamDB ref)](https://www.pcgamingwiki.com/wiki/007_First_Light); [Hardware Busters](https://hwbusters.com/news/007-first-light-drops-denuvo-after-two-months-and-the-patch-notes-never-mentioned-it/)
- Anti-cheat: the PCGamingWiki field is blank, and the game is not in AreWeAntiCheatYet — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/007_First_Light); [AreWeAntiCheatYet games.json](https://github.com/AreWeAntiCheatYet/AreWeAntiCheatYet/blob/master/games.json)
- ProtonDB: Platinum (268 reports, trending Platinum) — [ProtonDB](https://www.protondb.com/app/3768760)
- Modding exists: FOV mod, ultrawide cutscene mod (a hex-edited exe), the open-source First Light Mod Manager for `.rpkg` mods, bHaptics memory-reading mod — [PCGamingWiki](https://www.pcgamingwiki.com/wiki/007_First_Light); [VGTimes FLMM](https://vgtimes.ru/games/007-first-light/files/94959-first-light-mod-manager-flmm.html); [Nexus FOV mod](https://www.nexusmods.com/007firstlight/mods/15)
- **Local inspection of the exe** (`strings` on `<Steam library>/steamapps/common/007 First Light/Retail/007FirstLight.exe`, plus `openvr_api.dll`, 819,664 bytes, in the same folder): the strings include `UI_OPTION_DISPLAY_STEREOSCOPIC`, `UI_OPTION_DISPLAY_STEREO_DEPTH`, `UI_OPTION_DISPLAY_STEREO_STRENGTH`, `stereo3d`, `$vrmode`, `SetVrModeActive`, `ZVRMode`, `UI_OPTION_GRAPHICS_VR_QUALITY`, `UI_OPTION_VR_HEAD_BASED_MOVEMENT`, `UI_VR_DEVICE_DETECTED_TITLE`, `HMD not found.`, `[VR] Dummy device`, `SUIOpenVRComponent`, `vr_quality`, `vr_StepVRCamera`, and "Oculus VR, LLC". The debug path shows a `PRJ_KNT\nsw2_launch` branch
- Hitman 3 (Glacier 2) added PC VR on all PC platforms with patch 3.100 (2022-01-20), via SteamVR, and covers Hitman 1–3 — [Road to VR](https://www.roadtovr.com/hitman-3-pc-vr-support-release/); Quest/Quest 2 via Link is supported — [PCGamingWiki: Hitman WoA (IOI support ref)](https://www.pcgamingwiki.com/wiki/Hitman_World_of_Assassination)
- Hitman 3/WoA lists native 3D and 3D Vision as false, but Hitman (2016, Glacier 2, DX11) has a Helix 3D Vision fix — [PCGamingWiki: Hitman WoA](https://www.pcgamingwiki.com/wiki/Hitman_World_of_Assassination); [PCGamingWiki: Hitman 2016](https://www.pcgamingwiki.com/wiki/Hitman_(2016)); [Helix Mod: Hitman 2016 DX11](https://helixmod.blogspot.com/2016/11/hitman-2016-dx11.html)
- Hitman 3 itself runs well under Proton — [GamingOnLinux](https://gamingonlinux.com/2022/01/hitman-3-arrives-on-steam-and-works-flawlessly-on-linux-with-proton). Hitman 3's VR mode under Proton: no report found — (search, no result)
- No VR/stereo mod for 007 First Light was found, and no public mention of its dormant VR code — (searches 2026-10-18: "007 First Light VR mode PC OpenVR stereoscopic", "First Light openvr_api.dll VR hidden Glacier modding")

### Inferences
- The VR/stereo code and the OpenVR runtime DLL shipped in 007 First Light probably come from the shared Glacier 2 codebase (Hitman 3 PC VR). They may be compiled in but turned off by game data, or reachable through a launch arg/config key. The `UI_OPTION_DISPLAY_STEREOSCOPIC/DEPTH/STRENGTH` strings point to a separate flat "stereoscopic display" option (as for 3D TVs), not just HMD VR. That option is exactly the output this project wants.
- The cheapest test (read-only first): look for Glacier config keys (`vrmode`, `stereo3d`) in the game's registry settings / `thumbs.dat`. Then try launching with SteamVR + VRto3D running so `openvr_api.dll` finds an HMD ("HMD not found" suggests it looks for one at startup). If the VR mode starts, VRto3D gives the fixed SBS screen with no code injection.
- If the paths are compiled out: build our own. That means a D3D12/vkd3d hook that redraws each frame from two camera matrices (Glacier's camera entity is findable via the FOV mod's memory work), or patching the dormant VR mode on via a Glacier `.rpkg`/exe patch, using the community's Glacier tooling (Simple Mod Framework, FLMM, Quartermaster). Both are major reverse-engineering projects.
- Performance: the Z13 holds a ~60 base at FSR 4 Quality, CPU-bound in crowds, and the game "scales poorly". A native VR/stereo path renders two eyes through the engine's own instanced path, so a 30 fps stereo target at FSR Performance + Low/Medium looks reachable. A brute-force double-draw hook would hit the CPU bound and probably not reach 30 in crowds. This is an estimate.
- Online: the game has no always-online play and no anti-cheat now that Denuvo is gone, so injection is "safe" under DECISIONS rule 7. IOI menus offer an IOI account sign-up (R hotkey), which is optional.

### Gaps
- Whether 007 First Light's VR/stereoscopic mode is reachable (config key, hidden menu, launch flag) is unknown. Nothing public was found, so this needs a hands-on Z13 test.
- No confirmation that Hitman 3's OpenVR mode works under Proton/SteamVR on Linux, which would show whether a revived Glacier VR path would run on SteamOS.
- Exact Glacier settings keys for VR/stereo are not documented publicly.
