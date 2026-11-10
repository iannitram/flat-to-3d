> Raw research notes from 2026-10-18, kept as written. Where they conflict with [DECISIONS.md](../DECISIONS.md) (for example paid-tool routes), the decisions win. Some paths and names were generalized when these moved into the repo.

# Linux / SteamOS / Proton VR stack for true-stereo flat games on a Quest 2 (as of 2026-10-18)

Scope: the platform side only. This covers the streamer, the OpenXR runtime, the Proton bridge, injectors under Proton, the fixed giant 3D screen, and building our own stereo. Per-game details are covered by other researchers. Project constraints come from `DECISIONS.md`: Z13 only, SteamOS only, true stereo only, a fixed screen, Quest Tether as the USB link, 30+ fps, free, offline. Quest Tether today runs gnirehtet reverse-tethering over `adb reverse` with its relay on port 31416, and Steam Link measured about 28 Mbit/s Direct over it (the Quest Tether README).

Research budget note: roughly 25 searches and fetches. Several items below are flagged as unverified gaps instead of guessed.

## Q1. VR streamers from a Linux host to a Quest 2: wired support, Quest Tether compatibility, latency and bitrate

### Takeaway
**WiVRn is the strongest fit.** It is an OpenXR runtime and streamer in one, it is on Flathub (so it suits SteamOS's immutable root), it is actively released (v26.9, 12 Sep), it lists Quest 2 as supported, it has a documented wired mode over `adb reverse tcp:9757`, and v26.9 added a "native USB mode" for Meta headsets that needs no adb or developer mode. **Steam Link VR now works on Linux hosts.** It was unofficial from SteamVR beta 2.13.2 (Sep 2025) and became better supported in SteamVR 2.16.3 and 2.17 (Sep 2026, "initial USB streaming", Steam Client Beta required). It ties us to SteamVR on Linux, which is historically rough, and USB streaming to Quest is unconfirmed. **ALVR** has an ADB wired mode, but 2025–26 Linux reports show connect and disconnect loops. **Virtual Desktop** is a Windows-only streamer, so it is out (from general knowledge, not re-verified this session).

### Cited Findings
- WiVRn's README marks **Quest 2 as supported**. It documents a manual wired connection: `adb reverse tcp:9757 tcp:9757`, then a `wivrn+tcp://localhost` intent, with advice to turn off Wi-Fi when using USB. — [WiVRn GitHub](https://github.com/WiVRn/WiVRn)
- WiVRn is "an open-source OpenXR runtime based on Monado that streams VR to a standalone headset over Wi-Fi or cable". The wiki recommends at least a 5 Gb/s cable and port, and a "Connect via USB" dashboard option. Page last modified 2026-09-04. — [vronlinux.org WiVRn page](https://vronlinux.org/docs/fossvr/wivrn/)
- WiVRn v26.9 (12 Sep): "**Native USB mode on Meta headsets, doesn't require adb or developer mode**". Also in that release: a revamped headset GUI, gamepad forwarding from headset to server, and graceful shutdown. The release notes do not say whether Quest 2 is included or how the mode works. — [WiVRn v26.9 release](https://github.com/WiVRn/WiVRn/releases/tag/v26.9)
- Other WiVRn releases:
  - v26.6 added a **Half-FPS mode** to reduce load, plus better image quality when content doesn't fill the FOV.
  - v25.12 (19 Dec) made the default encoder **Vulkan Video / H.265 on AMD or Intel**, and added in-stream bitrate adjustment and keyboard and mouse forwarding.
  - v26.2.2 fixed a crash on connection with xrizer.
  - v26.2.1 fixed the OpenXR cylinder layer aspect ratio.

  — [WiVRn releases](https://github.com/WiVRn/WiVRn/releases)
- The WiVRn Flatpak (`io.github.wivrn.wivrn` on Flathub) bundles **xrizer and OpenComposite**. Sandboxing may limit Lighthouse devices and the forwarded virtual input devices. Steam Flatpak needs `flatpak override` filesystem grants to reach WiVRn. — [WiVRn GitHub](https://github.com/WiVRn/WiVRn); [vronlinux WiVRn](https://vronlinux.org/docs/fossvr/wivrn/)
- **Steam Link VR on a Linux host:**
  - Sep 2025: SteamVR beta 2.13.2 let Quest connect to a Linux PC through Steam Link, but Valve had not announced it. Game support was mixed: Half-Life: Alyx, Beat Saber and Boneworks worked; No Man's Sky, Synth Riders and others failed. A fix for one failure was deleting `~/.config/openvr/openvrpaths.vrpath`. — [PCGH, 2025-09-14](https://www.pcgameshardware.de/Virtual-Reality-Hardware-258542/News/steam-link-vr-mit-meta-quest-laeuft-jetzt-auf-linux-1482047/)
  - Steam Flatpak is "not a supported configuration" for it; you need a proper Steam package. SteamOS ships native Steam, so this is fine for us. — [GamingOnLinux, 2025-09](https://gamingonlinux.com/2025/09/steam-link-vr-for-meta-quest-on-linux-now-appears-to-work-in-the-latest-updates)
- SteamVR beta 2.16.3 (Apr 2026) had VRLink (Steam Link) fixes on Linux:
  - It picks the best GPU instead of the first one enumerated.
  - It adds a workaround for 240 Mbit/s streaming on NVIDIA.
  - It adjusts packet timing to reduce latency spikes.

  The article still calls Linux VR "in quite a rough spot". — [GamingOnLinux, 2026-04](https://www.gamingonlinux.com/2026/04/steamvr-beta-2-16-3-brings-linux-fixes-for-vrlink/)
- SteamVR 2.17 went stable on 2026-09-10. It brings:
  - "**Adds initial support for USB streaming**", which needs the Steam Client Beta.
  - On Linux: wired headsets work without third-party extension layers, Steam Link stream recovery is better, a crash on invalid textures is fixed, and games that invert nearZ/farZ are handled.
  - Contrast Adaptive Sharpening for Steam Link.
  - SteamVR can be set as the default OpenXR runtime.
  - Support for 32-bit OpenXR apps.

  Neither article says whether USB streaming covers Quest or only the Steam Frame. — [GamingOnLinux, 2026-09-11](https://www.gamingonlinux.com/2026/09/steamvr-2-17-arrives-ready-to-go-for-the-steam-frame/); [PCGH, 2026-09-12](https://www.pcgameshardware.de/Steam-Frame-Hardware-285941/News/SteamVR-2-17-Update-USB-Streaming-Linux-1553574/)
- Steam Frame context: it is Valve's own headset and runs SteamOS. A Linux-host bug was reported where launching a VR game from the headset opens a flat window. The workaround is to start SteamVR on the PC first. This comes from a community wiki only. — [NixOS wiki: Steam Frame](https://wiki.nixos.org/wiki/Steam_Frame)
- **ALVR:**
  - It has an ADB-based wired connection manager and needs a USB3 cable. — [ALVR repo summary (Algolia docsearch index)](https://docsearch.algolia.com/mcp/docs/repo/alvr-org/alvr)
  - A Bazzite user on v20.14.1 reports ALVR "keep connecting and disconnecting" on Quest 2, wired or over LAN. — [ALVR issue #3034](https://github.com/alvr-org/ALVR/issues/3034)
  - A 2025 r/linux_gaming user could not get wired mode working, blaming Meta's developer-mode rules. — [r/linux_gaming mirror](https://reddit.birdcat.cafe/r/linux_gaming/comments/1jxk2qu/oculus_on_linux)

### Inferences
- **WiVRn wired and Quest Tether can share the cable.** `adb reverse` holds several port mappings at once, so `adb reverse tcp:9757 tcp:9757` (WiVRn) should coexist with gnirehtet's reverse tunnel on 31416. WiVRn's own wired path goes straight over the adb tunnel to `localhost`, so it **does not need gnirehtet at all**. Running it *through* gnirehtet (the headset connects to the host's LAN IP via the tun0 VPN) should also work, since gnirehtet relays TCP and UDP. However, gnirehtet is a userspace relay with an unknown throughput ceiling. The 28 Mbit/s Steam Link figure is probably Steam Link's own bitrate choice, not gnirehtet's limit, but that is untested. Recommendation: have Quest Tether add the WiVRn `adb reverse` mapping (or use WiVRn's native USB mode) and keep gnirehtet only for the headset's general internet access.
- **WiVRn v26.9's native USB mode may overlap with Quest Tether's purpose.** It needs no developer mode. If it works on Quest 2, Quest Tether's role shrinks to internet access plus a "Start 3D" orchestrator. This needs a hands-on test.
- **Steam Link VR has a real advantage:** it is already the app I use, and no sideloaded APK is needed. Against it: SteamVR-on-Linux is closed-source and historically buggy, USB streaming is new and unconfirmed for Quest, and Steam Link VR still streams over the network, which would ride gnirehtet. WiVRn is the first choice and Steam Link VR is the fallback.
- **30 fps is a natural fit for WiVRn.** Its Half-FPS mode (v26.6) lets the game render at half the headset refresh rate, for example 36 or 45 of 72 or 90 Hz. The compositor keeps a static quad screen stable on every frame regardless.

### Gaps
- No measured latency or bitrate numbers for WiVRn wired on Quest 2 (or for any Linux streamer) were found.
- Quest 2 coverage of WiVRn v26.9 native USB mode is unknown. Its mechanism (USB accessory mode? its own driver?) is also unknown.
- Whether SteamVR 2.17 USB streaming works with Quest (Steam Link app) on Linux is unknown.
- Whether Meta has ended Quest 2 OS or app support in a way that affects the WiVRn APK or Steam Link was not checked.
- Virtual Desktop being Windows-host-only was not re-verified this session (general knowledge: its streamer app is Windows-only).

## Q2. OpenXR runtimes on Linux and how Proton games reach them

### Takeaway
On SteamOS the practical runtime is **WiVRn**, which is Monado-based (Monado being the reference FOSS OpenXR runtime). Monado alone has no Quest streaming, so it is not a candidate on its own. **SteamVR for Linux** is the alternative, and from 2.17 it can be the default OpenXR runtime. Proton bridges Windows games to Linux runtimes in two ways: the Windows OpenXR API through **wineopenxr**, and OpenVR through Proton's **vrclient** shim to a native Linux OpenVR runtime. With WiVRn, the native OpenVR runtime is a translation layer: **xrizer** (OpenVR reimplemented on OpenXR; Proton 9+ needs xrizer or VapoR) or **OpenComposite**. The game's Steam launch options must expose the runtime into the pressure-vessel container.

### Cited Findings
- OpenVR games and Proton 9+ need a translation layer (xrizer or VapoR) to run on WiVRn. Older Proton versions fail to initialise VR. A Flatpak override can set a PRESSURE_VESSEL environment variable globally so no per-game launch option is needed. — [vronlinux WiVRn](https://vronlinux.org/docs/fossvr/wivrn/)
- xrizer is "a reimplementation of OpenVR on top of OpenXR", used without SteamVR.
  - Launch option: `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 %command%`.
  - `VR_OVERRIDE` is usually unnecessary and has broken games such as VRChat.
  - Its OpenVR implementation is incomplete ("only what most games need").
  - Logs are at `~/.local/state/xrizer/xrizer.txt`.
  - The Flatpak WiVRn already includes xrizer.

  Page last modified 2026-09-04. — [vronlinux xrizer](https://vronlinux.org/docs/fossvr/xrizer/)
- Community guide: kernel 6.15+ and a Proton with ntsync are recommended, with official Proton 11+ preferred. The guide is about 446 days old, and its requirements conflict with the wiki's "Proton 9+". — [chaosmaou/wivrn-guide](https://github.com/chaosmaou/wivrn-guide)
- For OpenXR games under Proton on plain Monado, the NixOS wiki uses `PRESSURE_VESSEL_FILESYSTEMS_RW=$XDG_RUNTIME_DIR/monado_comp_ipc %command%` to expose Monado's IPC socket. It says OpenVR games and OpenXR-through-Proton games go through OpenComposite. — [NixOS wiki VR](https://wiki.nixos.org/wiki/VR/zh)
- At startup Proton probes both OpenVR and OpenXR. Monado logs show **wineopenxr** and OpenComposite each creating an XrInstance, which fails with "Loader does not support simultaneous XrInstances" in one configuration. This is evidence that wineopenxr is the Proton OpenXR bridge. — [OpenOVR issue #416](https://gitlab.com/znixian/OpenOVR/-/issues/416)
- The OpenXR loader finds the runtime through `/etc/xdg/openxr/1/active_runtime.json`, with a per-user override at `~/.config/openxr/1/active_runtime.json`. — [Monado docs (Debian README / monado.freedesktop.org)](https://metadata-backend.ftp-master.debian.org/changelogs/main/m/monado/stable_README.Debian)
- WiVRn v26.6.1 "resets OpenXR active runtime on server start", so WiVRn takes over the active runtime when it runs. — [WiVRn releases](https://github.com/WiVRn/WiVRn/releases)
- SteamVR 2.17 can be set as the default OpenXR runtime and supports 32-bit OpenXR apps. — [GamingOnLinux, 2026-09](https://www.gamingonlinux.com/2026/09/steamvr-2-17-arrives-ready-to-go-for-the-steam-frame/)

### Inferences
- **SteamOS fit.** On SteamOS's immutable root, the per-user `~/.config/openxr/1/active_runtime.json` together with the WiVRn Flatpak avoids touching `/etc`. Native Steam (not Flatpak) needs the pressure-vessel import variable per game or globally. Our future Inject app can write these launch options, the same way the `lossless-frame-gen` skill already edits Steam launch options.
- **Pick each mod's output API to match the runtime.** UEVR picks OpenXR on WiVRn (see Q4). For OpenVR-only mods, xrizer is the default and OpenComposite the fallback when xrizer lacks a call the mod needs.

### Gaps
- No primary documentation was found for `PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES` (Steam Linux Runtime source not reached).
- How current Proton (10/11) exactly routes `openxr_loader.dll` to wineopenxr to the host loader was not confirmed from Proton source.

## Q3. Showing a stereo game image as a fixed giant 3D screen

### Takeaway
**No turnkey Linux tool was found that shows a live SBS game window as a stereo (per-eye) fixed screen on WiVRn or Monado.**
- **WayVR** (formerly wlx-overlay-s, merged Jan 2026) shows desktop windows in VR on Monado, WiVRn and SteamVR. No SBS-stereo mode was found.
- **vr-video-player** does SBS-stereo window capture, but it is OpenVR/SteamVR-only and X11-only.
- **SteamVR Desktop Theater** is unreliable on Linux and was not shown to do stereo.
- **VRto3D** (LGPL, has Linux builds) is the inverse tool: it turns a VR mod's two eyes into an SBS flat image.

The cleanest "giant 3D TV" is something we build: **an OpenXR API layer, or a small OpenXR app, that submits a world-locked quad per eye.** OpenXR composition layers carry an `eyeVisibility` field, so a left-only and a right-only quad give a true stereo fixed screen.

### Cited Findings
- WayVR is an OpenXR/OpenVR overlay to view Wayland/X11 screens and run apps in VR on Monado, WiVRn and SteamVR. WlxOverlay-S, WayVR and WayVR Dashboard merged into one program in **January 2026**. It runs alongside VR games with low overhead. — [wayvr-org/wayvr](https://github.com/wayvr-org/wayvr); [vronlinux WayVR](https://vronlinux.org/docs/fossvr/wayvr/)
- vr-video-player captures an X11 window into VR on OpenVR/SteamVR (based on Valve's hellovr sample).
  - `--flat` handles side-by-side stereo video and games, and `--right-left` swaps the eyes.
  - The screen is movable by trigger or hotkey.
  - Capture is "only on x11", and no OpenXR support is mentioned.
  - The author notes SteamVR-on-Linux issues.
  - It suggests ReShade or vkBasalt plus SuperDepth3D_VR for games without SBS. That is depth reprojection, which our rules disallow.

  — [vr-video-player](https://git.dec05eba.com/vr-video-player/about/)
- SteamVR Desktop Theater on Linux: in a Dec 2024 Ubuntu thread it showed "no desktop found". Disabling SteamVR Home reportedly helped. — [Steam Community thread](https://steamcommunity.com/app/250820/discussions/5/596261865553709652). Theater mode dates from the 2016 beta. — [Slashdot 2016](https://games.slashdot.org/story/16/03/13/2020251/valve-is-bringing-steams-game-library-into-vr-with-desktop-theater-mode)
- VRto3D is an OpenVR/SteamVR driver with a virtual HMD that renders VR-modded games as SBS (also TaB, interlaced and others) for 3D displays.
  - Head tracking is off by default: a fixed pose, with optional right-stick pitch and yaw.
  - It runs "natively on Linux for most output modes".
  - It "does not 'fix' games for 3D, but it allows you to run VR modded (fixed) games on a 3D Display".
  - License: LGPL-3.0.

  — [oneup03/VRto3D](https://github.com/oneup03/VRto3D)
- `XrCompositionLayerQuad` has `space`, `pose`, `size` (in meters) and **`eyeVisibility`** (an XrEyeVisibility value). The quad's pose is evaluated in the given XrSpace, so a LOCAL or STAGE reference space gives a world-locked screen. — [Khronos OpenXR 1.1 spec: XrCompositionLayerQuad](https://registry.khronos.org/OpenXR/specs/1.1/man/html/XrCompositionLayerQuad.html)
- WiVRn fixed the OpenXR cylinder layer aspect ratio in v26.2.1 (so cylinder layers are supported) and fixed view-locked projection layer handling in v26.6.2. — [WiVRn releases](https://github.com/WiVRn/WiVRn/releases)
- Envision is the WiVRn and Monado setup GUI and is documented on the LVRA wiki. Its contents were not fetched. — [vronlinux index](https://vronlinux.org/docs/)

### Inferences (design options for the "giant 3D TV", ranked)
1. **OpenXR API layer, "fixed-screen shim" (build).** It sits between a VR mod (UEVR in OpenXR mode, or any OpenXR/xrizer game) and WiVRn, and does two things:
   - It answers `xrLocateViews` and space locates with a **constant head pose**, so head movement never steers the game camera, as rule 4 requires. It keeps a fixed IPD and FOV.
   - In `xrEndFrame`, it rewrites the game's projection layer into **two quads with left-only and right-only `eyeVisibility`**, world-locked at, for example, 4 m wide and 3 m away. Our build should confirm that WiVRn honours `eyeVisibility`.

   Result: the mod's two true camera views appear as a fixed stereo TV, and WiVRn's compositor handles reprojection, so a 30 fps game still yields a rock-steady screen. This is the inverse of VRto3D (which turns VR into SBS-on-flat), done at the OpenXR level. It runs in the game's process, so for Proton games it must be a *Windows* OpenXR layer DLL loaded by the game's loader, or a native Linux layer applied inside wineopenxr's host-side instance (preferred, because one native `.so` then covers every Proton game). This is the single most valuable piece to build.
2. **SBS-window stereo viewer (build).** This is for games that natively render SBS, or for stereo produced by a DXVK/VKD3D-level approach (Q5). It is a tiny OpenXR client on WiVRn:
   - It captures the game surface: the gamescope PipeWire stream in Game Mode, or a PipeWire screencast or window capture in Desktop Mode.
   - It splits the frame into left and right halves.
   - It submits them as per-eye quads.

   vr-video-player proves the concept but is OpenVR+X11. Porting its idea to OpenXR+PipeWire is small. It adds one capture-copy of latency.
3. **VRto3D + vr-video-player under SteamVR.** It is not recommended. Two HMD drivers (VRto3D's virtual HMD and the Quest stream) cannot both be the SteamVR headset, and it would re-flatten and then re-stereo the image.
- **Mode:** Desktop Mode is the safer first target. vr-video-player is X11-only, and WiVRn tooling expects a desktop session. Game Mode with gamescope support has to be tested (Q6).

### Gaps
- Not verified: whether WiVRn honours per-eye `eyeVisibility` on quad layers.
- Not verified: whether WayVR has any SBS or stereo screen option.
- Stardust XR and xrdesktop were not researched in depth. xrdesktop is believed dormant, but this was not verified.
- No existing OpenXR "head-lock or fixed-screen" API layer for Linux was found. One may exist under another name.

## Q4. Do Windows stereo injectors work under Proton/Wine?

### Takeaway
- **UEVR: yes, with a documented recipe.** Install .NET 6 Desktop Runtime into the game's prefix with protontricks. Launch the injector *inside the same prefix* via `PROTON_REMOTE_DEBUG_CMD`, and select **OpenXR** when on WiVRn/Monado. Known breakages: process detection failures, and a .NET/WPF crash on non-English locales.
- **BepInEx/Unity mods: yes, routinely**, via `WINEDLLOVERRIDES="winhttp=n,b"` (Wine 11.6 may make this unnecessary).
- **3DMigoto / Geo-11: no Linux evidence found.** They are D3D11 wrapper DLLs (`d3d11.dll` proxies). Under DXVK they would need to chain to DXVK's d3d11 and see real D3D11 objects, which is plausible in principle but undocumented, so treat it as unproven.
- **DXVK-level VR mods already exist and run on Proton.** **FNVVR Reborn** is a DXVK fork with stereo for Fallout: New Vegas, loaded as `d3d9=n,b`.

### Cited Findings
- UEVR on Linux, step by step:
  1. Install .NET Desktop Runtime 6.0.25 x64 into the game's prefix with `protontricks-launch --appid <id> windowsdesktop-runtime-6.0.25-win-x64.exe`.
  2. Set the launch option `PROTON_REMOTE_DEBUG_CMD="$HOME/Documents/uevr/UEVRInjector.exe" %command%`, which starts the injector and the game in the same prefix.
  3. Select **OpenVR for SteamVR, OpenXR for Monado/WiVRn**.
  4. Click Inject.

  UEVR targets UE 4.8+, and it may only give look-around without controller support. Page modified 2026-09-04. — [vronlinux UEVR Injector](https://vronlinux.org/docs/games/uevrinjector/)
- Proton issue "UEVR injector for VR enablement": "UEVR fails to detect game's process for injection". Valve closed it as not planned on 2024-12-26 and moved the discussion to the per-game issue (Dead Island 2, #7667). — [Proton #8346](https://github.com/ValveSoftware/Proton/issues/8346)
- The UEVR injector, and any .NET 6 WPF app, crashes on startup under non-English system locales. The cause is a hardcoded en-US tag that Wine's globalization layer fails on. It persisted across several Wine and Proton builds. — [LorenDB/kaon #20](https://github.com/LorenDB/kaon/issues/20) (kaon appears to be a Linux UEVR/VR launcher project)
- BepInEx under Proton: the launch option is `WINEDLLOVERRIDES="winhttp=n,b" %command%` (the official docs write `winhttp.dll=n,b`). It is needed because UnityDoorstop relies on game-folder DLLs loading before system DLLs. — [BepInEx docs: Proton/Wine](https://docs.bepinex.dev/articles/advanced/proton_wine.html); [ProtonDB example](https://www.protondb.com/app/1073440)
- With Wine 11.6, BepInEx games "look like they will run out of the box" without overrides. This is anecdotal. — [GamingOnLinux comment, 2026-04](https://www.gamingonlinux.com/2026/04/wine-11-6-is-an-exciting-release-to-make-modding-windows-games-on-linux-simpler/comment_id=292694)
- FNVVR Reborn is a DXVK fork that adds VR to Fallout: New Vegas: "Stereo rendering with the game drawing one frame per eye".
  - The eye images are Vulkan images handed through DXVK interop to an OpenVR backend.
  - It ships as a 32-bit `d3d9.dll` loaded with `WINEDLLOVERRIDES="d3d9=n,b"`.
  - It runs under Proton on Linux.
  - Camera patching uses game-specific addresses (`vr_game_addresses.h`).

  — [df4l/FNVVR](https://github.com/df4l/FNVVR)
- A search for 3DMigoto with DXVK/Proton returned nothing specific to 3DMigoto. — [search results, e.g. GamingOnLinux forum](https://www.gamingonlinux.com/forum/topic/4421/)

### Inferences
- **UEVR on Proton gives true stereo** (UE renders two views). Combined with the Q3 shim it becomes the giant 3D TV. UEVR's own fixed-screen-like options were not researched. The shim's head-lock removes the head-steered camera regardless.
- **REFramework** (RE Engine VR) and **MelonLoader** follow the same DLL-proxy pattern (`dinput8.dll` / `version.dll` + `=n,b` overrides). They should behave like BepInEx under Proton, but this session did not verify it.
- **3DMigoto/Geo-11 are a poor bet on SteamOS.** Their stereo output relies on NVIDIA 3D Vision or their own SBS present path through DXGI. Even if loading works, the output path is unproven. Prefer VR mods (UEVR, game-specific OpenVR/OpenXR mods) plus the shim.

### Gaps
- No first-hand 2026 success or failure reports for UEVR + WiVRn on SteamOS specifically.
- REFramework and MelonLoader under Proton were not checked.
- 3DMigoto or Geo-11 under DXVK: no reports found either way.

## Q5. Build our own: true stereo at the Vulkan / DXVK / VKD3D level

### Takeaway
It is feasible and has precedent, but each technique has costs.
- **FNVVR** proves a *game-specific* DXVK fork can render per-eye and hand the images to a VR runtime under Proton.
- **VKS3D** (GPLv3, 2026, Linux `.so` builds listed) is a *universal* Vulkan ICD. It forces multiview and patches SPIR-V so each vertex's clip-space x shifts per eye (separation and convergence, scaled by w), and outputs SBS. This is the "3D Vision driver" technique, and it renders true two views, not depth reprojection.
- Since Proton turns D3D9–12 into Vulkan, a VKS3D-style **Vulkan layer** under DXVK/VKD3D could give generic stereo for many games. Caveats: 2D and HUD passes, screen-space effects, shadow and post passes all need per-shader fixes, the same way 3D Vision needed community fixes.

### Cited Findings
- VKS3D details:
  - It is a "Universal stereoscopic 3D Windows Vulkan driver", a Vulkan 1.1 ICD.
  - It uses multiview render passes with a view mask covering both eyes, plus a SPIR-V patcher that shifts clip-space x per eye using separation and convergence values.
  - It doubles the swapchain width and blits the eyes to an SBS image at present.
  - Known limits: no dynamic rendering support yet, and a synchronous fence wait at present.
  - The release table lists Linux x64 and x86 `.so` builds, but there are no Linux install docs.
  - It is GPLv3, has 2,805 commits and 11 stars, and says "Made with AI".

  — [ThreeDeeJay/VKS3D](https://github.com/ThreeDeeJay/VKS3D)
- Vk3DVision is a Vulkan driver for NVIDIA 3D Vision stereo, including VR support. Forum users run it with DXVK translating D3D. — [helifax/Vk3DVision-Public](https://github.com/helifax/Vk3DVision-Public); [MTBS3D thread](https://www.mtbs3d.com/forum/viewtopic.php?t=25068&start=720)
- FNVVR Reborn is a DXVK fork rendering one frame per eye for an OpenVR backend under Proton (see Q4). — [df4l/FNVVR](https://github.com/df4l/FNVVR)
- `VK_NV_display_stereo` exists, but it is an NVIDIA display-stereo extension. It does not apply to the AMD 8060S or to an HMD. — [Vulkan docs](https://docs.vulkan.org/features/latest/features/proposals/VK_NV_display_stereo.html)

### Inferences (feasibility notes)
- **Layer vs ICD.** On Linux, a Vulkan *implicit layer* (enabled by an env var, like vkBasalt) is the friendlier packaging. VKS3D is an ICD that wraps the real driver, which is awkward to chain with RADV under pressure-vessel. Porting VKS3D's multiview + SPIR-V clip-shift into a layer is the main engineering task. Shipping it is easy because layers load per-game from `~/.local/share/vulkan/implicit_layer.d`, with no root needed on SteamOS.
- **Clip-space shift vs a real per-eye view matrix.** A clip-space x shift with convergence is the classic "driver stereo" method used by 3D Vision and Vk3DVision. It is two camera projections with a parallel off-axis offset, and it produces true parallax for geometry. It is not depth reprojection, so it meets rule 3. Its failure modes are known: HUD at the wrong depth, screen-space reflections, SSAO, and shadows per eye. These need per-game shader overrides.
- **Cost on the 8060S.** Multiview roughly doubles vertex and raster work, but not CPU draw-call cost. At 30 fps with lowered settings that is plausible on this iGPU. Untested.
- **Output path.** The layer's SBS swapchain would feed the Q3 option-2 viewer (capture), or better, the layer could export the two eye images straight to an OpenXR session (as FNVVR does with OpenVR), skipping capture.
- **Recommendation.** Treat this as phase-3 research. Lead with existing VR mods (UEVR, BepInEx-based, game-specific DXVK forks) plus the Q3 OpenXR shim. Build or port the VKS3D-style layer only for real-3D-world games with no mod, per rule 9.

### Gaps
- No public DXVK-level or VKD3D-level *generic* stereo project for Linux was found ("dxvk stereo" or "vkd3d stereo" searches returned only VKS3D, Vk3DVision and FNVVR).
- Whether VKS3D's Linux `.so` actually runs on RADV, or under DXVK or VKD3D-Proton, is unverified.

## Q6. Performance on the 8060S under SteamOS for VR streaming; SteamOS-specific issues

### Takeaway
No VR-specific benchmarks exist for Strix Halo or the 8060S on Linux. General Linux gaming performance is strong, and the kernel 6.15+ / Mesa 25.2+ stack improved it. WiVRn's default on AMD is **Vulkan Video H.265 encode**, so encoding runs on the iGPU's VCN, not the shaders. Half-FPS mode suits our 30+ fps target. On SteamOS the open questions are Game Mode (gamescope) versus Desktop Mode, and Flatpak sandbox limits.

### Cited Findings
- Phoronix (May 2025): the Radeon 8060S runs CS2 at 1080p "with ease", and the open-source stack runs it "great". — [Phoronix 8060S review](https://www.phoronix.com/review/amd-radeon-8060s-linux)
- Linux 6.15+ and Mesa 25.2 bring slightly higher frame rates for some Steam Play games on the 8060S. — [Phoronix news](https://www.phoronix.com/news/Linux-6.15-Mesa-25.2-Strix-Halo)
- WiVRn's default on AMD is Vulkan Video encode with H.265 (since v25.12). Vulkan Video encoding performance improved in v26.2.1. — [WiVRn releases](https://github.com/WiVRn/WiVRn/releases)
- WiVRn has a Half-FPS mode (v26.6). — [WiVRn releases](https://github.com/WiVRn/WiVRn/releases)
- The WiVRn Flatpak sandbox may block forwarded virtual input devices (headset gamepad, keyboard). — [WiVRn GitHub](https://github.com/WiVRn/WiVRn)
- Gamescope is Game Mode's compositor. A Flathub discussion notes that Flatpak apps needing gamescope features require the gamescope Flatpak extension, and the socket is `gamescope-0` in Game Mode. This is a forum comment. — [flathub dolphin PR #178](https://github.com/flathub/org.DolphinEmu.dolphin-emu/pull/178)
- In 2023 gamescope gained some VR support (OpenVR backend). — [GamingOnLinux 2023](https://gamingonlinux.com/2023/01/gamescope-used-on-steam-deck-adds-in-some-vr-support)
- Steam Link VR on Linux needs native Steam, not Flatpak. SteamOS's Steam is native. — [GamingOnLinux 2025](https://gamingonlinux.com/2025/09/steam-link-vr-for-meta-quest-on-linux-now-appears-to-work-in-the-latest-updates)

### Inferences
- **The GPU budget is shared.** The Z13 must render the game twice (stereo), encode H.265, and run the compositor, all on one iGPU and in shared memory. The Z13 shares memory with other jobs and the GPU borrows about 31 GiB. Expect to drop to 720p–900p per eye-equivalent at 30–45 fps. The giant-TV approach helps a lot: the quad needs only TV-resolution eye images (about 1280×720 per eye), not full HMD-resolution projection images. The shim (Q3 option 1) can tell the game a smaller recommended swapchain size.
- **Start in Desktop Mode.** WiVRn server + dashboard, PipeWire capture and X11/Wayland tools all assume a desktop session. Game Mode is the end goal for the Decky "Start 3D" button: the WiVRn server can run headless as a user service, and the game still launches from Steam in Game Mode. Whether WiVRn's compositor coexists with gamescope's direct scanout in Game Mode has to be tested.
- **Steam Link (flat) and WiVRn will conflict.** Both need the headset's foreground app. Quest Tether's "Start 3D" would switch from Steam Link to the WiVRn client.

### Gaps
- No VR benchmarks on the 8060S or Strix Halo, and no SteamOS Game Mode + WiVRn success reports were found.
- Envision on SteamOS (immutable root) was not researched. The WiVRn Flatpak is the assumed install path.

---

## Proposed pipeline (synthesis for the report writer)

1. **Link:** Quest Tether keeps gnirehtet for internet access and adds `adb reverse tcp:9757 tcp:9757`. Alternatively, test WiVRn v26.9's native USB mode on the Quest 2.
2. **Runtime and streamer:** WiVRn Flatpak on the Z13. It is the active OpenXR runtime, with xrizer bundled for OpenVR mods and the H.265 Vulkan Video encoder.
3. **Stereo source:** an existing VR mod running under Proton. Options:
   - UEVR (injector in the prefix via `PROTON_REMOTE_DEBUG_CMD`, OpenXR mode).
   - BepInEx/MelonLoader Unity VR mods (`winhttp=n,b`).
   - Game-specific DXVK forks (FNVVR pattern).
4. **Giant 3D TV, build #1:** an OpenXR API layer that head-locks the game's view poses and re-presents its two eye images as world-locked per-eye quads (`eyeVisibility` LEFT/RIGHT).
5. **Universal fallback, build #2 (later phase):** a VKS3D-style Vulkan implicit layer under DXVK/VKD3D for real-3D games with no mod. It outputs either SBS (shown by a small OpenXR SBS viewer, build #3) or two eye images direct to OpenXR.
6. **Fallback streamer:** Steam Link VR (SteamVR 2.17+, native Steam) if WiVRn fails on Quest 2. This path is more closed and less predictable on Linux.
