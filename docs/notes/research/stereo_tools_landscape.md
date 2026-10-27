> Raw research notes from 2026-10-18, kept as written. Where they conflict with [DECISIONS.md](../DECISIONS.md) (for example paid-tool routes), the decisions win. Some paths and names were generalized when these moved into the repo.

# Landscape of true-stereo injection tools (for a fixed-screen "3D TV" in the Quest 2)

Researched 2026-10-18. Repo activity, license and star counts come from the GitHub REST API (`api.github.com/repos/<owner>/<repo>`) queried that day. They are cited as "GitHub API, 2026-10-18" with the repo URL. A license of `NOASSERTION` means GitHub could not auto-detect one: a license file exists but has to be read by hand.

## UEVR (praydog): how it works, rendering modes, flat-screen/3D-TV options, UE coverage

### Takeaway
UEVR is the strongest free, true-stereo path for Unreal games. It covers UE 4.8 through 5.4 and renders two real views through Native Stereo or Synchronized Sequential. Its built-in **2D Screen Mode** is very close to the "3D TV" goal: a third-party plugin author describes it as a fixed quad in front of the viewer that keeps per-eye stereo while the headset pose is not applied to the game camera. Other sources disagree about whether that mode keeps stereo depth, so it must be tested.

### Cited Findings
- Three rendering methods:
  - **Native Stereo** uses the engine's own stereo pipeline. It performs best but can crash or glitch.
  - **Synchronized Sequential** is a synced AFR: both eyes render on the same game tick while game time is paused between the two frames. TAA can ghost, so turn motion blur off. Skip Draw and Skip Tick options are available.
  - **AFR** alternates eyes while game time keeps moving. It causes eye desync and is a last resort.
  - Source: [UEVR README](https://github.com/praydog/UEVR)
- Supports UE 4.8 to 5.4 and outputs through both OpenVR and OpenXR. Injection: run UEVRInjector.exe, pick the game process, pick a runtime, and inject. — [UEVR README](https://github.com/praydog/UEVR); [UEVR docs overview](http://docs.uevr.io/usage/overview.html)
- Runtime guidance: OpenVR is usually the most compatible (needs SteamVR). OpenXR is often faster with a native runtime. Virtual Desktop should use OpenXR. — [UEVR docs overview](http://docs.uevr.io/usage/overview.html)
- Release history:
  - Initial Beta (31 Dec) targeted UE 4.8–5.3.
  - Beta 1.01 added a **"Toggle 2D Screen Mode" key** and a Disable VR key.
  - 1.03 added Freeze Position and Freeze Rotation.
  - 1.04 added symmetric projections and Controller Pitch Offset.
  - 1.05 (latest stable, published **2024-11-16**) added a Lua API, an expanded C/C++ plugin API, UE5 crash fixes, D3D12 spectator fixes and a UE5.4 head-aim fix.
  - Nightlies ship from praydog/UEVR-nightly.
  - Sources: [UEVR releases](https://github.com/praydog/UEVR/releases); GitHub API 2026-10-18
- Maintenance: there has been no new stable release since 1.05 (Nov 2024). The main repo and the nightly repo were last pushed **2026-08-30**. The license file is not auto-detected (NOASSERTION). The repo has 4,630 stars. — GitHub API 2026-10-18 ([UEVR](https://github.com/praydog/UEVR), [UEVR-nightly](https://github.com/praydog/UEVR-nightly))
- Decoupled Pitch is a comfort option that levels the view and removes roll. The docs list no general "disable head tracking" switch. — [UEVR docs overview](http://docs.uevr.io/usage/overview.html)
- **2D Screen Mode, conflicting descriptions:**
  - A third-party plugin README describes UEVR's built-in 2D screen as a quad in front of you: "Each eye keeps its own view, so the quad has depth, and the headset pose is not applied to the camera." It also describes a "flat 2D" variant that gives both eyes the same camera. — [LorenDB/uevr-cutscene-comfort](https://github.com/LorenDB/uevr-cutscene-comfort)
  - A Jan 2026 settings guide says 2D Screen Mode "disables stereoscopic depth." — [vrpupu UEVR Advanced Settings](https://vrpupu.com/en/2026/01/uevr-advanced-settings-explained/)
  - **The two sources conflict.** The plugin README is the more technical, first-hand description.
- CutsceneComfort plugin details:
  - What it does: on cutscenes it zeroes the VR camera offsets and switches to a stereo portal (2D screen with depth). It also has an optional "room-anchored 6DOF window."
  - Window implementation: needs a companion Operator fork (`UEVR-6DOF-Window`) that draws a mask into each eye texture before OpenXR submit.
  - Maturity: v0.1.0-alpha.1, MIT, 0 stars. LorenDB's fork was last pushed 2026-10-07. Upstream is elliotttate/uevr-cutscene-comfort, last pushed 2026-08-03.
  - Sources: [LorenDB/uevr-cutscene-comfort](https://github.com/LorenDB/uevr-cutscene-comfort); GitHub API 2026-10-18
- Ace Combat 7 guides tell users to disable decoupled pitch to fix cloud and cockpit rendering. This is game-specific. — [Nexus AC7 UEVR quick start](https://www.nexusmods.com/acecombat7skiesunknown/articles/120)

### Inferences
- In UEVR, "3D TV" mode amounts to Native Stereo or Synced Sequential plus 2D Screen Mode always on. Our app could set this per game through UEVR's `config.txt` or profile keys, or use the plugin/Lua API to force it on at injection. The CutsceneComfort plugin is proof that the plugin API can drive 2D-screen state.
- Synchronized Sequential counts as true stereo in the DECISIONS.md sense: two camera positions, same game tick. Plain AFR does not, and should be off-limits or flagged.
- UEVR emits OpenXR/OpenVR. Under Proton that implies SteamVR for Linux or Monado/WiVRn plus an OpenXR bridge. Another researcher covers that.

### Gaps
- No official praydog document says whether 2D Screen Mode keeps per-eye stereo. The sources conflict, so test it.
- No source here covers how injection works under Proton. UEVRInjector is a Windows exe that injects into a running process. Out of scope; noted only.
- The exact UEVR license text was not read (GitHub reports NOASSERTION).

## Other engine frameworks: REFramework (RE Engine), Unity (UUVR, BepInEx/MelonLoader), Flat2VR

### Takeaway
REFramework (MIT, very active in Oct 2026) gives RE Engine games full VR with true stereo. UUVR (GPL-3.0) is the universal Unity injector, but it has been quiet since July 2025. Neither is known to have a fixed-screen stereo mode, so output would have to be adapted.

### Cited Findings
- REFramework README: VR users should install SteamVR unless they are using OpenXR on a supported headset. OpenXR arrived in REFramework 1.2, along with motion controls for RE7 and RE Village. — [REFramework](https://github.com/praydog/REFramework); [Wccftech](https://wccftech.com/resident-evil-village-and-re7-vr-mods-look-amazing-are-out-soon/amp/)
- REFramework nightlies ship as one monolithic zip covering every supported RE Engine game. Old non-RT builds of RE2, RE3 and RE7 are not covered. — [newreleases.io REFramework-nightly](https://newreleases.io/project/github/praydog/REFramework-nightly/release/nightly-01376-a2025ed81f788c944d17acec2f7bbf96e774128f)
- Road to VR reports a praydog VR mod for Resident Evil Requiem that is gamepad-only for now. — [Road to VR](https://roadtovr.com/resident-evil-requiem-vr-mod-praydog/)
- REFramework: MIT license, last pushed **2026-10-11**, 5,653 stars. — GitHub API 2026-10-18 ([repo](https://github.com/praydog/REFramework))
- **vrframework** (elliotttate) is an engine-agnostic fork of REFramework with an "engine SPI with a StereoView abstraction." License not detected. Last pushed 2026-06-09. 19 stars. — [ecosyste.ms listing](https://awesome.ecosyste.ms/projects/github.com%2Felliotttate%2Fvrframework); GitHub API 2026-10-18
- UUVR (Raicuparta) is a "Universal VR mod for Unity games" with OpenXR, OpenVR, SteamVR, XInput and patcher modules. It installs through Rai Pal. — [UUVR](https://github.com/Raicuparta/uuvr)
- UUVR is GPL-3.0, last pushed **2025-07-03**, 237 stars. — GitHub API 2026-10-18
- A guide says UUVR supports Unity metadata versions 23–29 and fails on newer Unity. — [CompoundVR UUVR guide](https://compoundvr.com/articles/uuvr-guide/)
- Note: that same guide says Raicuparta made UEVR. **That is wrong.** UEVR is praydog's ([UEVR repo](https://github.com/praydog/UEVR)). Treat the guide's other claims with some caution.
- Community forks exist, such as DrapNard/uuvr-MiSide and Kirba7/uuvr. — [DrapNard fork](https://github.com/DrapNard/uuvr-MiSide); [Kirba7 fork](https://github.com/Kirba7/uuvr)

### Inferences
- In Unity, UUVR renders two eyes with Unity's own camera stack, so it is true stereo. A fixed-screen mode would mean ignoring the HMD pose in the camera rig and showing the eye textures on a quad. That is a fork-level change, not a config switch.
- vrframework's StereoView abstraction could be a reusable base for "build our own stereo" (ruling 9). Worth a look in the build phase.

### Gaps
- Not covered within the call budget: BepInEx/MelonLoader per-game VR mods, the Flat2VR Discord/wiki catalog, and Raicuparta's per-game Unity mods (e.g. Two Forks, Firewatch). Their injection method is the BepInEx doorstop (winhttp.dll proxy) loading a managed plugin.
- No source found on whether REFramework has a flat or theater stereo mode.

## 3DMigoto, Geo-11, Helix Mod / 3D Fix Manager / HelixVision, Katanga: DX11 shader-based true stereo and SBS output

### Takeaway
Geo-11 is a free, 3DMigoto-based "3D driver" that makes geometric (true two-view) stereo for **DX11 games only**. It outputs SBS, TAB and other formats directly with no NVIDIA hardware, has been shown on AMD and **under Linux/Proton**, and its SBS window can be shown in a VR theater app. DX12 is not planned. The VR helper app Katanga ships only inside the paid HelixVision.

### Cited Findings
- 3DMigoto is a "DX11 modding wrapper to enable fixing broken stereoscopic effects," now also a general DX11 modding tool. It builds `d3d11.dll`, `nvapi64.dll`, `d3dx.ini` and a ShaderFixes folder, so it is a DLL proxy plus shader replacement. Original author Chiri, maintained by bo3b. The site 3Dmigoto[.]com is a phishing site. — [bo3b/3Dmigoto](https://github.com/bo3b/3Dmigoto)
- 3DMigoto: last pushed 2026-09-06, license detected as NOASSERTION (repo ships COPYING.txt and LICENSE.GPL.txt), 1,246 stars. — GitHub API 2026-10-18; [repo](https://github.com/bo3b/3Dmigoto)
- Geo-11 basics:
  - Announced June 2022 by bo3b on the Helix Mod blog and developed by davegl1234.
  - Creates "geometric 3D" in DX11 games without the game's own stereo support, using 3DMigoto for hooking and shader manipulation.
  - DX12 support "not planned for now."
  - Source: [Helix Mod blog, geo-11 announcement](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)
- Geo-11 output modes (`direct_mode`):
  - sbs, tab, interlaced, checkerboard, anaglyph, plus `_reversed` variants.
  - `katanga_vr`.
  - `simulated_reality` (Leia SR).
  - `nvidia_dx11` (driver ≤452.06) and `nvidia_dx9` (fullscreen).
  - TAB is recommended over SBS because SBS halves horizontal resolution.
  - Source: [Helix Mod blog](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)
- Geo-11 setup: needs `force_stereo=2` in d3dx.ini (other settings go in d3dxdm.ini). No NVIDIA software is needed for the non-NVIDIA modes. The page says it "has been shown to run on AMD cards," and that **Linux with Proton also works**. — [Helix Mod blog](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)
- Geo-11 status: "beta caliber," tested on about 80 games, free download (v0.7.11 at the time, hosted on bo3b's S3). Open-sourcing was planned under a non-commercial license, details TBD. — [Helix Mod blog](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html)
- Steam forum posts (2022–23): `direct_mode=sbs` is the most compatible starting point, and "dx9 development on geo-11 has stopped." Users show SBS output in VR via Virtual Desktop, Bigscreen or VorpX. — [Steam HelixVision discussion](https://steamcommunity.com/app/1127310/discussions/0/3726197825948436143)
- Geo-11 example: a YouTube guide shows The Riftbreaker in SBS / VR180, with the game forced to DX11 instead of its default DX12. — [YouTube](https://www.youtube.com/watch?v=1fPFe8TuAsk)
- **geo-11_SR** (nautymac): a fork whose small SRWeave `dxgi.dll` hooks `IDXGISwapChain::Present` to weave geo-11 SBS for lenticular SR panels (SpatialLabs, Leia). Last pushed 2026-10-15. — [geo-11_SR](https://github.com/nautymac/geo-11_SR); GitHub API 2026-10-18
- Katanga is a Unity VR app inside the HelixVision install (Tools→Katanga) and needs command-line launch parameters. HelixVision is sold on Steam; its free demo "does not allow you to play any games." 3D Fix Manager (3DFM) has no VR support, and HelixVision is a superset of 3DFM. — [Steam HelixVision threads](https://steamcommunity.com/app/1127310/discussions/0/3726197825948436143); [DekuDeals HelixVision](https://www.dekudeals.com/items/helixvision)
- DarkStarSword/3d-fixes holds 3D Vision fixes, Unity 4/5 DX9/DX11 templates, shader tooling and a `3DVision2SBS` folder. Last pushed 2026-03-05. — [repo](https://github.com/DarkStarSword/3d-fixes); GitHub API 2026-10-18

### Inferences
- For DX11 games, geo-11 with `direct_mode=sbs` (or tab), shown in a VR theater/screen app on the Quest, already delivers the "giant 3D TV" with no head-steered camera. That makes it the closest off-the-shelf match to ruling 4. Under Proton, DX11 runs on DXVK, so the d3d11.dll proxy must load as a native DLL override (`WINEDLLOVERRIDES=d3d11=n,b`).
- Katanga is paid (via HelixVision), so it is excluded. A free SBS-to-3D-screen viewer must come from elsewhere: our own app, or a streamer or theater app that supports SBS. Another researcher covers that.
- Existing Helix Mod / 3D Vision per-game shader fixes (DX9/DX11) are a large back catalog that geo-11 or 3DMigoto can reuse for DX11 titles.

### Gaps
- Not confirmed: the current geo-11 version as of 2026, or whether its source was ever opened. The license is unknown beyond "free beta."
- Whether geo-11's stereo is per-vertex shader offset ("3D Vision automatic" style, which is true per-eye geometry) or doubled draw calls. The announcement doesn't say. Both count as true stereo, not depth reprojection.
- No source found on bo3b's own CompanionApp or a geo-11→OpenXR output.

## VorpX, Vireio, the R.E.A.L. VR author R.E.A.L., SuperDepth3D, wiz3D, ReShade/vkBasalt and Vulkan-level stereo

### Takeaway
The excluded or paid tools are VorpX (paid), the R.E.A.L. VR author R.E.A.L. (Patreon, AER) and SuperDepth3D (depth reprojection). The interesting free and new entries for SteamOS are Vulkan-level stereo drivers:
- **VKS3D** (GPL-3.0, active Oct 2026, AI-written): a Vulkan ICD that injects per-eye clip-space offsets into SPIR-V vertex shaders and outputs SBS.
- **Vk3DVision** (helifax): archived.

Under Proton, DX9–12 all become Vulkan through DXVK/VKD3D, so a Vulkan-level stereo hook is in principle a universal path. This is unproven.

### Cited Findings
- **VorpX** (paid, note only): "Geometry 3D" renders the scene once per eye, which is true stereo and doubles the GPU cost. Its Z-buffer modes are pseudo-3D from depth. — [briankoponen.com](https://www.briankoponen.com/should-you-buy-vorpx/)
- **Vireio Perception** is described as injecting "3d + tracking into games." Another user calls it "barely functional and seems abandoned." — [HardForum thread](https://hardforum.com/threads/vireo-perception.1911886/)
- **the R.E.A.L. VR author R.E.A.L. VR** (Patreon, AER, note only):
  - Based on Alternate Eye Rendering, where one eye renders per frame and the other is reprojected.
  - v19.0.0 added AER v2 optical flow and DLSS RR support.
  - Patreon costs $10/month. — [Mixed News](https://mixed-news.com/en/real-vr-mod-dlss-ray-reconstruction/)
  - Jan 2026: more than 40 mods removed after DMCA requests (CD Projekt, the Ghostrunner publisher). — [Game*Spark (JP)](https://www.gamespark.jp/article/2026/01/24/161952.html)
- **SuperDepth3D / Depth3D** (excluded):
  - A ReShade shader that makes "depth map-based 3D," with SBS, anaglyph, VR and Theater outputs.
  - "Free for personal use, but it is not open source."
  - Last pushed 2026-10-13.
  - Sources: [BlueSkyDefender/Depth3D](https://github.com/BlueSkyDefender/Depth3D); GitHub API 2026-10-18
- **VKS3D** (ThreeDeeJay):
  - Mechanism: a Vulkan ICD (loader interface v5) that wraps the real driver. It uses multiview with view mask 0b11 and a SPIR-V patcher that adds a w-scaled horizontal clip-space offset per eye, controlled by separation and convergence.
  - Output: doubled-width SBS only.
  - Platform: install is Windows-registry-based, but a Linux `.so` is listed.
  - Limitations: dynamic rendering is not intercepted yet, and the SPIR-V patcher is minimal.
  - Project: GPL-3.0, "Made with AI," 11 stars, last pushed 2026-10-16.
  - Sources: [VKS3D](https://github.com/ThreeDeeJay/VKS3D); GitHub API 2026-10-18
- **Vk3DVision** (helifax): a Vulkan stereo driver ("single-frame stereo," VR support). The public repo holds only releases and is **archived**. BSD-3-Clause. — [Vk3DVision-Public](https://github.com/helifax/Vk3DVision-Public); [MTBS3D thread](https://www.mtbs3d.com/forum/viewtopic.php?t=25068); GitHub API 2026-10-18
- **wiz3D** (an iZ3D-based universal wrapper): DirectX/OpenGL, outputs half-SBS, anaglyph and more. DX11 works; DX9/10 are "to come." LGPL-2.1. The effcol fork was last pushed 2026-10-17. — [effcol/wiz3D](https://github.com/effcol/wiz3D); GitHub API 2026-10-18
- **NV3D-Lib** (oneup03): a developer library that takes an SBS image from DX11, DX12, GL or Vulkan and outputs to NVIDIA 3D Vision. Not an injector. LGPL-2.1. — [NV3D-Lib](https://github.com/oneup03/NV3D-Lib)
- **Stereix Engine**: a 2026 "6DOF VR injection engine" hooking DX11, DX12 and Vulkan. Its components list "stereo warp, depth reconstruction, and disocclusion fill," which reads as reprojection. Proprietary license, 2 stars. — [Stereix-Engine](https://github.com/sathishssj3/Stereix-Engine)
- An MTBS3D thread mentions a "Stereoscopic App by Vital" claiming DX12, Vulkan and GLES3 support. Not examined. — [MTBS3D](https://www.mtbs3d.com/phpbb/viewtopic.php?t=26671)

### Inferences
- vkBasalt and ReShade addons are post-process tools. Any stereo they produce from a single rendered view is depth reprojection, the SuperDepth3D class, so they are excluded except as an SBS compositor.
- A Vulkan-level vertex-shader offset approach (VKS3D-style) would cover DXVK and VKD3D-Proton titles on SteamOS without per-engine work. That is the most promising "build our own stereo" seam for ruling 9. Two caveats:
  - Vertex-offset stereo is "3D Vision Automatic"-style: it breaks screen-space effects (shadows, SSR, lighting at the wrong depth) unless shaders are fixed per game, as Helix/3DMigoto fixes do.
  - VKS3D on Linux, and on top of DXVK, is untested by any source found.

### Gaps
- No source confirms VKS3D working under Linux/RADV or with DXVK/VKD3D.
- No Vulkan *layer* (as opposed to ICD) stereo project was found.
- Vireio's current repo URL was not confirmed (cybereality/Vireio-Perception returned 404).

## Rai Pal (Raicuparta) and similar "mod manager with Inject button" apps

### Takeaway
Rai Pal is a GPL-3.0 Rust/Tauri mod manager, actively released (v0.24.0 on 2026-10-17), with a Linux AppImage. It detects games from Steam, GOG, Epic, Itch and Xbox, identifies their engine and version, and installs or runs universal mods (UEVR, UUVR) from a remote mod database. It is the direct template for our app.

### Cited Findings
- Rai Pal describes itself as a manager for universal game mods (e.g. UEVR, UUVR). — [rai-pal README](https://github.com/Raicuparta/rai-pal)
- Game detection:
  - Finds installed games from Steam (including non-Steam shortcuts), GOG, Epic and Itch, plus installed moddable PC Xbox games.
  - Also finds owned-but-not-installed games.
  - Manual add works by button or by dropping an exe onto the window.
  - Source: [rai-pal README](https://github.com/Raicuparta/rai-pal)
- Engine detection: engine and version are detected for installed Steam, GOG, Epic and Itch games; Xbox gets Unity only. Owned games use remote data and name matching. — [rai-pal README](https://github.com/Raicuparta/rai-pal)
- Deep links (`rai-pal://`) install, uninstall or run a mod per provider, game and mod ID. Custom mod databases can be added by URL. — [rai-pal README](https://github.com/Raicuparta/rai-pal)
- Platforms: Windows (installer, winget) and **Linux AppImage**. Repo topics are Rust and Tauri, with backend/frontend folders. — [rai-pal README](https://github.com/Raicuparta/rai-pal)
- Repo stats: GPL-3.0, 739 stars, latest release **v0.24.0 published 2026-10-17**. Mod DB repo Raicuparta/rai-pal-db last pushed 2026-10-18. — GitHub API 2026-10-18 ([rai-pal](https://github.com/Raicuparta/rai-pal), [rai-pal-db](https://github.com/Raicuparta/rai-pal-db))
- Raicuparta's Patreon post links Rai Pal with the universal Unity VR mod. — [Patreon post](https://www.patreon.com/raivr/posts/rai-pal-and-vr-93913267)

### Inferences
- Our Decky "Inject" app can mirror Rai Pal's split: a Rust engine for detection, engine fingerprinting and mod install, plus a separate UI. A Decky plugin would replace the Tauri frontend. GPL-3.0 means a fork or code reuse has to stay GPL. The mod-DB-by-URL design maps onto ruling 11 (wiki header keys as the recipe source).
- Rai Pal already runs on Linux, so its Steam library and Proton prefix handling may be directly reusable. Not verified.

### Gaps
- The internals of how Rai Pal injects UEVR or installs UUVR (BepInEx doorstop or similar) were not read from source. The README doesn't describe them.
- The rai-pal-db format and whether the Linux build can inject into Proton games were not examined.
- No other comparable "inject button" managers were surveyed in the call budget. Flat2VR's own tooling is unknown.

## Projects that turn VR-mod stereo into a fixed virtual screen (theater mode)

### Takeaway
The closest finds are UEVR's own 2D Screen Mode, and the alpha CutsceneComfort plugin with its "stereo portal" and "room-anchored 6DOF window." SuperDepth3D has a "Theater" mode, but it is depth-based. No general tool was found that takes any VR mod's stereo eye textures and puts them on a fixed screen.

### Cited Findings
- CutsceneComfort's stereo portal uses stock UEVR's 2D screen with per-eye stereo and no head pose applied to the camera. Its 6DOF window draws a mask into each eye texture before OpenXR submit, via the UEVR-6DOF-Window fork. Alpha, MIT. — [LorenDB/uevr-cutscene-comfort](https://github.com/LorenDB/uevr-cutscene-comfort)
- Depth3D lists VR and Theater output modes, but it is depth-map 3D. — [Depth3D](https://github.com/BlueSkyDefender/Depth3D)
- The geo-11 SBS path relies on an external SBS-capable viewer (Virtual Desktop, Bigscreen, or the paid VorpX/Katanga). — [Steam thread](https://steamcommunity.com/app/1127310/discussions/0/3726197825948436143)

### Inferences
- There are two generic architectures for our app:
  - (a) Mod-native: UEVR 2D Screen Mode, or patched UUVR/REFramework camera rigs that ignore the HMD pose.
  - (b) SBS-native: geo-11, VKS3D or wiz3D render SBS into the game window, and an OpenXR "3D screen" compositor (ours, or the streamer's SBS theater) shows it.
  - Option (b) is engine-agnostic once SBS exists.

### Gaps
- No source found for a standalone OpenXR API layer that intercepts `xrEndFrame` and re-projects eye views onto a quad for an arbitrary VR mod. This looks like greenfield work.

## Recent (2024–2026) GitHub activity in this space

### Takeaway
2026 activity is real but thin. The bigger projects are maintenance-mode or active (REFramework, Rai Pal, 3DMigoto). Several small, new and often AI-assisted repos have appeared: VKS3D, geo-11_SR, wiz3D forks, Stereix (proprietary, reprojection-ish), CutsceneComfort and vrframework. UEVR has had no stable release since Nov 2024, though it has nightly pushes through Aug 2026.

### Cited Findings
- Last-push dates (GitHub API, 2026-10-18):

  | Repo | Last push | Note |
  |---|---|---|
  | rai-pal | 2026-10-17 | |
  | VKS3D | 2026-10-16 | |
  | geo-11_SR | 2026-10-15 | |
  | Depth3D | 2026-10-13 | |
  | REFramework | 2026-10-11 | |
  | NV3D-Lib | 2026-10-08 | |
  | effcol/wiz3D | 2026-10-17 | |
  | LorenDB CutsceneComfort | 2026-10-07 | |
  | 3Dmigoto | 2026-09-06 | |
  | UEVR and UEVR-nightly | 2026-08-30 | |
  | vrframework | 2026-06-09 | |
  | 3d-fixes | 2026-03-05 | |
  | Vk3DVision-Public | 2026-03-05 | archived |
  | uuvr | 2025-07-03 | |

  Repo URLs are in the sections above.

### Inferences
- UUVR's 15-month silence plus Unity metadata limits (v23–29) mean newer Unity 6 games may need other paths.

### Gaps
- The Flat2VR Discord-linked mod lists and Reddit r/flat2vr were not surveyed due to the call budget.
