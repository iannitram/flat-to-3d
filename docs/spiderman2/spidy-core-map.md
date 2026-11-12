# SpidyVR "stereo screen" core: port map for Marvel's Spider-Man 2

Repo: [IlyaMez/SpidyVR](https://github.com/IlyaMez/SpidyVR) at v0.3.8 (GPL-3.0). All paths below are relative to it. Read-only analysis; nothing built or run.

## Version note: SpidyVR v0.3.8 and v0.7.0 (issue #38)

Read on 2026-11-12 from public pages at pinned commits, nothing cloned or built. Our pin is v0.3.8, commit [`aed5c0c`](https://github.com/IlyaMez/SpidyVR/commit/aed5c0c5f4584d7bc231b90fd1200b3ea85728c9) ("Release Spidy 0.3.8"). The newest release is v0.7.0, commit [`57537585`](https://github.com/IlyaMez/SpidyVR/commit/57537585fb3dea806cf426705a5770b860cc1436) ("Release Spidy 0.7.0"), 66 commits ahead. The licence is GPL-3.0 at both ([LICENSE](https://github.com/IlyaMez/SpidyVR/blob/57537585fb3dea806cf426705a5770b860cc1436/LICENSE)).

**Decision: Step 0 and this map stay on v0.3.8. The submodule is not bumped here.** Reasons:

- The Step 0 plan ([plan.md](plan.md)) asks for the official 0.3.8 release, and every line number in this map is a v0.3.8 line. A bump would need a fresh pass over the map first.
- The Proton + WiVRn launch path does not need v0.7.0. The v0.3.8 `docs/LINUX.md` already gives the same WiVRn launch options and the same start order, so the run sheet works on either pin.
- The stereo-screen files are all still present at v0.7.0, but upstream changed behaviour on that path in between (below). Those changes are unread at file level, so a bump now would be a guess.

**What moved on the launch path (v0.3.8 → v0.7.0).** I compared the two `docs/LINUX.md` pages and the v0.7.0 `tools/spidy-wivrn.sh` wrapper. The WiVRn launch options, the start order and the known limits are the same. The v0.7.0 page adds a stated Proton Experimental requirement and puts the troubleshooting into a table. The wrapper only sets the WiVRn runtime and hands off to `spidy-linux.sh`; I did not read that script, so where its start-order and launch-option checks live is unconfirmed at both pins.

**What moved on the stereo-screen path.** Compared by file name in the two git trees and by commit title. I did not open the changed contents, so none of our line citations are re-checked at v0.7.0.

- Still present at v0.7.0 (no rename or removal seen): `src/stereo_probe.cpp`, `src/game_xr.cpp`, `src/native_eye_gpu.cpp`, `src/native_hud.cpp`, `include/spidy/presentation_gate.hpp`, `include/spidy/native_eye_frame.hpp`, `include/spidy/native_eye_history.hpp`, `tools/probe_game_screen.py`, `tools/probe_stereo.py`, `tools/probe_stereo_gpu.py`, `tools/capture_stereo.py`.
- New files near the path at v0.7.0: `include/spidy/vd_eyes.hpp`, `include/spidy/native_stream.hpp`, `tools/probe_streaming.py`, `include/spidy/recording_fov.hpp`.
- Commits whose titles touch the path (most relevant first):
  - `3c0e1b08`: closing a menu no longer ends VR while the last eye pair still shows (#36). Touches the gate and the eye-pair hand-off that §1.5 cites.
  - `85501e29`: eye tracking and streaming data in VR. Touches where the eye distance comes from (§1.3, `presentation_gate.hpp`).
  - `3ce3baa`: "What the eyes draw counts as on screen". Touches the on-screen test that §1.5 gates on (`native_hud::onScreen`).
  - `7eafac2`: the city at the flat game's distances. Relevance unclear; it may touch distances the screen uses.
- Commits that are not on the path: the phone, the web and swing mechanics, the recording FOV, body haptics, the launcher and the Linux phone-screen fix.

**What this means for the map.** Sections §1.3 and §1.5 cite v0.3.8 lines and gate rules that the three commits above may have changed. Before any port work, redo those citations at the chosen pin. If the owner wants v0.7.0 for Step 0, that is a separate decision, and the Step 0 result should name the Spidy version that was run.

The submodule's row in the code map is in [code-map.md](../code-map.md).

## 0. What the feature actually is (correcting one assumption)

- `VR VIEW: GAME SCREEN` = `Values::flatScreen` (`include/spidy/vr_settings.hpp:32-38`, row `src/vr_settings.cpp:143-144`, toggled via the pause menu's first page, `src/game_menu.cpp:72,362,709`). The launcher does **not** persist it: "Each session starts in VR" (`vr_settings.hpp:35`). So in the shipped code the only way into it is the in-game SPIDY VR tab, which is `game_menu.cpp` (11 Scaleform hooks).
- `GAME SCREEN 3D` = `Values::screenDepth` 0-3 -> `screenDepths[] = {0,.5,.7,1}` (`vr_settings.hpp:61-64,94-97`; row `vr_settings.cpp:147-149`). It applies to **every** "game screen" moment with running game time (cutscenes, finishers, deaths) *and* to VIEW: GAME SCREEN.
- In VIEW: GAME SCREEN the XR worker never sends tracked eyes: `vrPlay = motion.active && !flatScreen` (`src/game_xr.cpp:1318`), so it takes the same "scene" path as a cutscene and sends **eye command `enabled = 3`** (`game_xr.cpp:2576-2600`). The `enabled = 2` "flat clone" mode (`game_xr.cpp:2673`) is unreachable from the worker in practice (it is only reached when `vrPlay`, i.e. `!flatScreen`); `tools/probe_game_screen.py` still uses it.
- No head tracking steers the camera in mode 3: `commandHead()` only builds a head view for `enabled == 1` (`src/stereo_probe.cpp:536`); `activeView()` returns false unless `enabled == 1` (`stereo_probe.cpp:1096`); `latchedImmersive` requires `enabled == 1` (`stereo_probe.cpp:1255`). The game's own camera descriptor is read from the primary view and offset per eye.

## 1. End-to-end data flow (GAME SCREEN 3D, mode 3)

### 1.1 Injection / attach
Windows path: `Launch Spidy VR.cmd` -> `tools/launch-game-vr.ps1` -> `tools/run_game_vr.py` (also spawned by `apps/launcher`). Linux: `tools/spidy-linux.sh` runs the same Python inside the game's Proton container: `steam-runtime-launch-client --bus-name=com.steampowered.App1817070 ... -- wine python/python.exe -u tools/run_game_vr.py --xr-runtime auto` (`spidy-linux.sh:12,133-135`); requires launch option `STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%` and the XR runtime started before the game (`:3-7,47-56`).

`run_game_vr.py session()` (`:1603`):
1. `wait_for_game(..., early=render_memory_module)` (`:1670`) -> `vr_launcher.enlarge_render_memory()` (`tools/vr_launcher.py:222`) loads `spidy_render_memory.dll` within ~3 s of process start and calls `SpidyRenderMemoryStart(512)` (RENDER_RING_MB, `vr_launcher.py:21`).
2. "Already patched" check: reads 16 bytes at every `GAME_HOOKS` RVA in memory vs the on-disk PE (`run_game_vr.py:1696-1699`; list `:33-44`).
3. `wait_for_renderer()` (`:1375`, called `:1704`) -> `probe_stereo_gpu.discover_queue()` (`tools/probe_stereo_gpu.py:73`) loads `spidy_render_probe.dll`, `SpidyStart`, sleeps 1 s, reads `SpidyRenderData`, `game_queue()` picks the direct queue whose submit stack has `spider-man.exe+` nearest the top (`probe_stereo_gpu.py:42-70`).
4. Module loading = classic remote `LoadLibraryW`: `bridge_game.prepare()` copies the DLL to `reports/<kind>-modules/<sha256>/`, `CreateRemoteThread(LoadLibraryW, path)`, then resolves exports by loading the same DLL locally and rebasing (`tools/bridge_game.py:29-58`); calls go through `observe_game.call_remote / call_with_payload` (`tools/observe_game.py:71-100`, VirtualAllocEx + WriteProcessMemory + CreateRemoteThread, 15 s timeout).
5. Loads `spidy_bridge.dll` (`SpidyStart` with no player, `:1709-1717`), `spidy_ray_bridge.dll`, `spidy_movement_bridge.dll` (`:1719-1729`), then `spidy_stereo_probe.dll` (`:1730-1735`), packs `XrConfig` v23 / 664 bytes (`:1740-1755`; struct `src/game_xr.cpp:52-121`) incl. `queue`, the three module bases, `screenSize`, `screenDepth`, and calls `SpidyXrStart` (`:1758`). `SpidyXrStart` refuses unless `queue`, `bridgeModule`, `rayModule`, `motionModule` are all non-zero (`game_xr.cpp:3285-3290`) and `GetProcAddress`es ~35 exports from them (`game_xr.cpp:3305-3340`). **A stereo-only build must drop that requirement.**

### 1.2 Device/queue
- `src/render_probe.cpp`: creates its own D3D12 device + direct queue + a 16x16 swapchain (composition; on Proton falls back to a hidden-window HWND swapchain, `:106-124`), takes `IDXGISwapChain` vtable[8] (Present) and `ID3D12CommandQueue` vtable[10] (ExecuteCommandLists) (`:125,128`), MinHooks them, records submitter stacks filtered to the game image range `base .. base+140496896` (`:51`, hard-coded SizeOfImage).
- The queue pointer goes into `XrConfig.queue`; `game_xr.cpp:563-566` gets the device from it; `XrRuntime::initialize(device, queue, eyeSize, renderScale)` binds OpenXR to the game's own device/queue (`game_xr.cpp:575`).

### 1.3 Making the game render two eyes (`spidy_stereo_probe.dll`)
- Lazy start: the GPU bridge starts on first viewing frame (`native_gpu::start`, `game_xr.cpp:2505-2519`, `copyToXr = 2` staged). Native views start only when VR play **or a scene** first needs them (`game_xr.cpp:2523-2575`): `SpidyEyeTemporal(upscale | 7<<8 | reflections)`, `SpidyEyeShadows`, `SpidyMonitorRate`, then `SpidyStart(&views)` with magic `0x53534346`, flags `13|16|32` by default, width/height = **headset eye dimensions** (`game_xr.cpp:2559`).
- `SpidyStart` (`stereo_probe.cpp:1660-1899`): checks `GetModuleHandleW(L"Spider-Man.exe")` (`:1684`), then 31 prologue/data signatures (`:1694-1783`, error 1002 on any mismatch), creates 16 MinHooks (`:1790-1861`), enables them, `native_hud::start(base)` (`:1888-1889`).
- Eye creation (`maintain`, hook on view maintenance `0x18a0bb0`, `:1495-1636`): manager is the static `ViewContextManager` at `base+0x7a34dd0` (`:1496`); creates two **offscreen views** with `Create 0x18a09c0(manager, "Spidy left eye test", near, far, 90.f, width, height, 28, nullptr)` (`:1520-1526`); `giveOcclusion` -> `0x189fed0` (`:330-334,1529-1530`); sets view `+0x14e84 |= 0x480` (active post profile + display conversion, `:1536-1543`); retires by `+0x14e7c |= 1` (`:1564-1579`); ownership = presence in the 8 offscreen slots `base+0x7a34e00` (`:309-316,1597-1610`). Registers each eye's `ID3D12Resource` with the GPU bridge: view `+0x1f70` -> wrapper (vtable must be `base+0x4e56c80`) -> `*(wrapper+0x38)` (`:1612-1619`).
- Private eye buffers: `initBuffers` hook `0x1873470`, gated on return address `base+0x186c325` (`:810-860`): forces render size, no parent, temporarily ORs the offscreen kind mask at `0x3d11b18 + 2*4` (VirtualProtect write) to get TAA (bit 4) and SSR (bit 1) targets; `sizeBuffers` hook `0x1874f40` rewrites `+0x12bf0..+0x12c04` and flag `+0x12c2c` (`:793-809`).
- Per-frame placement (the core): `renderOffscreen` hook `0x1920240` -> `placeEyes()` (`:1487-1494`, `:1232-1393`). For `enabled == 3` (`:1345-1373`): copy primary view descriptor (`0x440` bytes, `include/spidy/native_view.hpp:36-59`), `stereoEyes()` gives separation/shift from the stock lens tangents (`include/spidy/native_eye_frame.hpp:105-117`), translate pose by `±separation/2` along the camera's right row, shift lens tangents `±shift` (off-axis), rebuild with game's lens ctor `0x187bb00` (keeps near/far, jitter from `0x428/0x42c` or `eyeJitter()` Halton) and pose setter `0x187ca10`, submit to the eye with `0x1899ab0` (`:1365-1371`).
- Stereo math (generic): `screenStereo(eyeDistance, screenWidth, 2.5 m, depth) = {convergence = 2.5*depth, parallax = eyeDistance/screenWidth, separation = eyeDistance}` (`native_eye_frame.hpp:66-68`); validity `native_eye_frame.hpp:69-73`; carried in `eyes[0].fov[0..2]` (`:53-60`). Eye distance from tracked eyes, clamp 50-80 mm else 64 mm (`third_party/SpidyVR/include/spidy/presentation_gate.hpp:342-345`).
- Command channel: worker calls exported `SpidySetEyes(&Command)` (208 bytes, magic `0x53455043` v2, `native_eye_frame.hpp:16-26`; `stereo_probe.cpp:1638-1659`): monotonically increasing serial, 250 ms lease.
- Render-job tagging (`eye_job_table`): `copyJob` hook `0x19223e0` (`:897-958`) records `{job = *(settings+0x4b0), serial, generation, eye}` in `EyeJobTable` (`include/spidy/eye_job_table.hpp`, 128 slots, reclaim after 16 generations); also sets render-context AA mode `settings+0x40c = 1` (engine TAA) and `shareExposure()`. `beginJob 0x189e310` / `endJob 0x189e3a0` hooks (`:985-996`) set a thread-local GPU marker `native_gpu::enterMarker(eye, serial, generation, ended)` around the original call.
- Display path fixes: `setupDisplay` hook `0x1920310` (`:759-786`) for views with negative view index (`+0xf90`) enqueues `PrepareSrgbOverlay (0x88)` on `+0x15b0` and `TemporalAa (0x8a)` on `+0x1630` via `Enqueue 0x187e130` (profile `+0x68c`); `tone` hook `0x1846c20` forces `genericView = 0` for eye render copies (owner `+0x1f40`) (`:997-1007`).
- Shadows: `submitScene 0x19206a0` / `fitCascades 0x170fbc0` hooks (`:1401-1485`). In mode 3 `shadowLensSame = desired.enabled >= 2` (`:1315`), so **each eye makes its own cascades** (sharing is immersive-only).

### 1.4 Camera offset (`game_view`, `camera_watch`)
- Neither is in the shipping path (not in `tools/package.ps1:10` / launcher `kModules`, `apps/launcher/system.cpp:24-26`). `src/game_view.cpp` (`spidy_view.dll`) is the earlier bounded experiment that rewrote the stock camera submit (`0x1899ab0`, return `0x1647044`, lens `0x187bb00`, pose `0x187ca10`; `game_view.cpp:46-72,115-119`) - it is the *proof* that the descriptor/lens/pose ABI works, and is a good first SM2 milestone. `src/camera_watch.cpp` is a hardware data-watchpoint (DR0/DR7) tool on one thread for finding who writes a camera field (`:76-101,141-145`); generic apart from the `Spider-Man.exe` check.
- The production offset is entirely `placeEyes()` above; there is no gameplay-camera write in mode 3.

### 1.5 HUD on the stereo screen (`native_hud` + `hudPlace`)
- Remastered's HUD is a 3D model (`PlayerModelHudFollower`, vtable `0x38929c8`) placed ~20 m before the camera with a Scaleform render-to-texture stream (`include/spidy/native_hud.hpp:14-29`). `hudPlace` hook `0x73ab80` (`stereo_probe.cpp:684-723`): in mode 3, `placeScreenHud()` (`:641-681`) pulls the panel toward the camera until its face (bounding sphere `instance+0x40`) sits at the stereo convergence distance and scales it down equally (SetTransform `0x191c0e0`), so HUD/subtitles/prompts have zero parallax and lie on the screen plane; sets `screenHudFrame`, `native_hud::onScreen()`.
- `placeEyes` calls `native_hud::eyesShown(true)` only on frames whose panel was placed (`stereo_probe.cpp:1259-1266`). `native_hud.cpp` hooks the RTT stream update `0x2104400` (vtable `0x4f87960` slot 3, verified `:293`), the Scaleform render command `0x1d2b6c0` (moves the second movie into the panel texture, `:181-215`), and marker projections `0x1f10ad0/0x1f10b60` (`:289-306`).
- **Gate:** `ScenePanel::wanted(scene, native_hud::onScreen(200 ms), now)` drops back to flat after 1 s with no panel (`presentation_gate.hpp:296-339`; `game_xr.cpp:1333-1335`), and the draw callback only shows the pair if `native_hud::onScreen(sceneHoldMs)` (`game_xr.cpp:2756`). **Without a working HUD hook the stereo screen never engages.**

### 1.6 Copy and submit to OpenXR
- GPU bridge (`src/native_eye_gpu.cpp`): hooks `ID3D12GraphicsCommandList::ResourceBarrier` [26], `Reset` [10], `EndQuery` [53], queue `ExecuteCommandLists` [10] (`:1197-1210`), DXGI `Present` [8] / `Present1` [22] (`:856-889`). Barriers on the two registered eye textures are recorded per command list; a `D3D12_QUERY_TYPE_TIMESTAMP` EndQuery recorded while a job marker is current becomes a begin/end event (`:249-266`); on the game queue's `ExecuteCommandLists` those events feed `EyePairState` (`include/spidy/eye_pair_state.hpp`): a pair is ready only when **both** eyes finished the **same {serial, generation}** (`:23-27`), then `stagePair` copies both eyes to private staging textures on the game queue (`:354-380`) with `copyEyeTexture` (`include/spidy/copy_eye_texture.hpp:19-42`, CopyResource + barriers, RGBA8 typeless family check `:7-15`). `std::recursive_mutex recording` for vkd3d-proton re-entrancy (`:49-52`).
- XR side: draw callback (`game_xr.cpp:2754-2795`): `native_gpu::copyLatest(targets[0], targets[1], serial, gen)` (`native_eye_gpu.cpp:1079-1120`) copies staged eyes into the swapchain images; `history.find(serial)` must give a frame saved with `stereoScreen = true` (`game_xr.cpp:2591-2595`, `include/spidy/native_eye_history.hpp:35-37`); then `runtime.presentation(true, screenPose, screenAspect = SpidyFlatAspect(), screenWidth(size), true)`.
- Layers: `XrSession::presentation()` sets `flatScreen_`, `stereoScreen_ = flat && stereo` (`include/spidy/xr_session.hpp:69-77`). End frame (`src/xr_session.cpp:665-683`): one `XrCompositionLayerQuad` with `eyeVisibility = XR_EYE_VISIBILITY_LEFT` showing swapchain 0, a copy with `XR_EYE_VISIBILITY_RIGHT` showing swapchain 1, `size = {width, width/aspect}`; `layerCount = 2` in stereo, 1 (both eyes, left image) for the flat game screen, else the projection layer. Screen pose `screenAhead(head, 2.5 m)` level, heading-only (`presentation_gate.hpp:348-358`; `game_xr.cpp:1397-1398`). Widths 2.4/3.2/4.2 m (`vr_settings.hpp:82-85`).
- Because the eye views are created at the headset eye size but drawn with the **stock camera's lens** (e.g. 21:9), each eye image is anamorphically squeezed; the quad's aspect (`flatAspect`, computed from tangents `0x400..0x40c`, `stereo_probe.cpp:1090-1093,1274-1276`) un-squeezes it.
- Fallback flat game screen (menus): Present hook copies the back buffer (`copyScreen`, `native_eye_gpu.cpp:551-620`; `latestScreen` `:943`), XR blits it with `D3D12Renderer::blit` (`game_xr.cpp:2796-2822`).

### 1.7 Menu / pause gating
- Scene condition (`game_xr.cpp:1326-1335`): `screen && values.screenDepth > 0 && player.hero && bridgeRunning && timeRuns` then `ScenePanel`.
  - `timeRuns`: `SceneClock` true while a frame with `game_time::telemetry().gameFrame > 0` is <= 200 ms old (`presentation_gate.hpp:295-314`). `gameFrame` = double at `base+0x7a7fbd8` (`include/spidy/game_time.hpp:23`, read `src/game_time.cpp:214`), exactly 0.0 in the pause menu (`third_party/SpidyVR/docs/REFERENCE.md:162`). But `installed` must be true, i.e. `game_time::install()` must hook `TimeScaleSystem::Update 0x19bb430` and pass 6 instruction checks (`game_time.cpp:15-38`). For stereo-only you need only the one data RVA.
  - `player.hero`: `game_player::Watch` (component registry `0x7a44320/0x7a44340`, vtables `0x38a93c8 HeroLocal`, `0x38b2c98`, `0x4f70168 MoverStandard`; `src/game_player.cpp:15-16`).
  - `bridgeRunning`: `spidy_bridge.dll` state 1 (`game_xr.cpp:1124-1129`), i.e. its 5 hooks installed (`src/game_bridge.cpp:273-289`).
- `GameScreen` 250 ms delay before the screen comes up (`presentation_gate.hpp:257-290`); without a scene the views get no command and their lease (250 ms) lapses.

### 1.8 Render memory
`src/native_render_memory.cpp` hooks the RenderAlloc ring creator `0x1872d90` (only call `0x188a195`) and per-frame rollover `0x1872b90` (only call `0x187e623`), allocator object `base+0x7938880`, fields at `+0x10` (`include/spidy/native_render_memory.hpp:22-45`), overflow flag `+0x41`; replaces the 128 MB ring with 512 MB before first frame (`native_render_memory.cpp:50-80,130-137`). Needed because 3 scene views overflow the ring (`docs/MILESTONE-1.md:161-172`). In mode 3 the game view still renders (no `restGameView`), so all 3 views run.

## 2. Game-specific dependencies on the stereo-screen path

Location method, globally: **every address is a hard-coded image RVA (`GetModuleHandleW(nullptr) + rva`) validated by comparing 5-26 expected prologue/instruction bytes (`entry()`/`matches()`), never located by pattern scan.** A few are additionally validated by vtable-slot equality or a return-address match. RTTI and string lookups were used only offline (tools) to find them. There is zero runtime signature scanning in the codebase.

### 2.1 `src/stereo_probe.cpp` (required for mode 3 unless noted)
Hooked functions (MinHook; prologue bytes `:1694-1748`, checks `:1749-1780`, hook RVAs `:1790-1805`):

| RVA | Name/role | Needed in mode 3? |
|---|---|---|
| `0x18a0bb0` | view maintenance (`ViewContextManager`): create/retire eyes, register textures | yes |
| `0x1920240` | offscreen render setup -> `placeEyes()` (only call `0x175aaa9` in frame fn `0x175a060`) | yes, core |
| `0x19223e0` | copy view into render job: job table, AA mode, exposure | yes |
| `0x189e310` / `0x189e3a0` | render job begin / end: GPU markers | yes |
| `0x1873470` | buffer-set init (gated by return `0x186c325`) | yes (eye size, TAA/SSR targets) |
| `0x1874f40` | buffer-set sizes | yes if TAA upscale |
| `0x1920310` | display setup: enqueue 0x88/0x8a | yes (HUD overlay pass, TAA) |
| `0x1846c20` | tone map (`genericView` flag) | yes |
| `0x189bd30` | view update: copy exposure `+0x1708/+0x170c` | yes |
| `0x73ab80` | `PlayerModelHudFollower` placement | yes (gate, 1.5) |
| `0x19206a0` / `0x170fbc0` | SubmitScene / FitCascades: shadow sharing | installed; no effect in mode 3 |
| `0x1899ab0` | view descriptor submit (hook acts only on return `0x1647044`, immersive) | called in mode 3; hook immersive-only |
| `0x17991a0` | RenderActor (HUD-off / hero hide) | no (immersive) |
| `0x186cc00` | CopyFinal (CPU readback diag, `createViews == 7`) | no |
| `0x187e130` | Enqueue (diag trace hook `:1980-2001`) | called yes, hook no |

Called-only functions: `0x18a09c0` Create offscreen view; `0x187bb00` lens ctor (12 args); `0x187ca10` pose setter; `0x1899ab0` submit; `0x187e130` enqueue render command; `0x189fed0` occlusion builder (flag 16); `0x191c0e0` instance SetTransform (HUD); `0x186c670` readback init (flag 2, diag); `0x191afb0`/`0x191b880` DrawOff/DrawOn (immersive); `0x1872860` frame allocate (diag) = 11.
Check-only signatures: `0x164703f` (call in `0x1646e10`), `0x175aaa9`, `0x189ee00` (history shift copy), `0x189c710` (adapted-lum read), `0x1875270` (target kind test), `0x1842e10` (TemporalAa), data `0x3d11b18` (kind-mask table `{ff,ff,20,28}`) = 7. Note: all 31 must match or `SpidyStart` fails with 1002, even the immersive-only ones.
Return addresses: `0x1647044`, `0x186c325` = 2.
Globals: `0x7a34dd0` ViewContextManager (deref = primary/active view), `0x7a34dd8` pool mask (bit 0), `0x7a34e00` (= mgr+0x30) 8 offscreen slots, `0x7a436e8`/`0x7a43704` render-instance table/count (HUD `registered()`), `0x7938880` RenderAlloc (diag), `0x3d11b18` kind mask (written) = 7.
Vtables: `0x38929c8` HUD follower, `0x4e56c80` texture wrapper = 2.
Render command IDs: `0x88` PrepareSrgbOverlay, `0x8a` TemporalAa, `0xad` readback (order `0x41a`); create arg `28`; buffer kind index 2, kind bits 4 (TAA) / 1 (SSR).
Struct offsets (~60): view size `0x1f70`; view `+0x30` pos, `+0x460/464` viewport, `+0x560` prev pos, `+0xf90` view index, `+0x1208` renders, `+0x120a`, `+0x15b0` / `+0x1630` command lists, `+0x1640` render buffers, `+0x16f0/+0x16f8` Adapted Lum buffer/SRV, `+0x1708/+0x170c` exposure, `+0x1ba0` occlusion, `+0x1f40` owner, `+0x1f70` texture, `+0x1f80` buffer set, `+0x14e28/2c` size, `+0x14e40/68/70/74` readback, `+0x14e7c` flags (bit 1 retire), `+0x14e84` profile flags `0x480`; descriptor `0x440` (`native_view.hpp:36-59`): pose rows 0-63 (right/down/forward/pos), near `0x3f8`, far `0x3fc`, tangents `0x400-0x40c`, jitter `0x428/0x42c`; buffers `+0x12ae8`, `+0x12bf0..+0x12c04`, `+0x12c2c`; settings `+0x4b0` job, `+0x40c` AA mode; profile `+0x68c`; render context `+8` view; wrapper `+0x38` ID3D12Resource, `+0x40`; instance `+0x0` 4x4, `+0x40` sphere, `+0x4c` radius, `+0x5c` flags (0x20 DrawOff), `+0x64` handle (24-bit index/8-bit gen), `+0x70`, `+0x88`; follower `+0x8`, `+0x48` distance, `+0xc0`; scene-submit result 0x58 bytes; KeyLightContext `+0xd,+0xf,+0x150` stride `0x3e0`.

### 2.2 `src/native_hud.cpp` (required: gate)
Hooks 4: `0x2104400` RTT stream update (verified as vtable `0x4f87960` slot 3, `:293`), `0x1d2b6c0` Scaleform render command, `0x1f10ad0` / `0x1f10b60` marker projections (`:279-306`). Globals 7: `0x7be3e00` second movie, `0x7a4ae50` res factor, `0x7a30b00` frame counter, `0x7be5740/0x7be5744` list frames, `0x7be3f38` counts, `0x7be3f40` items (`:111,138,190-199`). Code ranges for HUD callers `0x72a000-0x7c0000`, `0x1f00000-0x1f10980` (`:221`). Offsets: stream `+0x10,+0x20,+0x78,+0x7c,+0x9c,+0xa0`; target `+0x1cc`; movie `+0xa0`; follower `+0xc0`; Item 24 B.

### 2.3 `src/native_eye_gpu.cpp` (GPU bridge)
No game RVAs. COM vtable slots only (generic D3D12/DXGI ABI). Game-specific *behaviour*: relies on the game issuing a timestamp `EndQuery` inside `0x189e310`/`0x189e3a0`; module check `L"Spider-Man.exe"` (`:1177`).

### 2.4 `src/game_time.cpp` / `game_time.hpp` (gate)
Hook `0x19bb430` + 6 instruction checks (`0x19bb6ca, 0x19bb6d3, 0x19bb6db, 0x19bb70f, 0x19bb717, 0x1822670`) (`game_time.cpp:15-38`); globals `0x7a7fb90` clock scale, `0x7a7fbd8` frame game time (**the only one the gate needs**), `0x7a7fbf0`, `0x609a560` Havok step; offsets `0x18, 0x1c, 0x7d0`.

### 2.5 `src/native_render_memory.cpp`
Hooks `0x1872d90`, `0x1872b90`; call-site checks `0x188a195`, `0x187e623` (24/12-byte patterns incl. RIP-relative displacements, `:118-137`); global `0x7938880`; `Fields` layout `+0x10`, flag `+0x41`. Python: `RENDER_MEMORY_HOOKS` `tools/vr_launcher.py:24`.

### 2.6 `src/render_probe.cpp`
Hard-coded SizeOfImage `140496896` (`:51`); module name (`:145`). Rest generic.

### 2.7 Gate prerequisites dragged in by `game_xr.cpp` (removable in a stripped build)
- `src/game_bridge.cpp`: hooks `0x1e1d600` follow-camera commit, `0x1ce2f40` clear input, `0x1cdfa70`/`0x1cdf840` input queries, `0x5c2630` combat camera update (verified as vtable `0x38720d0` slot 21); setter `0x191c590`; vtables `followCamera 0x3871fd8`, `combatCamera 0x38720d0` (`include/spidy/game_bridge_protocol.hpp:17`, slot check `game_bridge.cpp:279`), `HeroLocal 0x38a93c8` (`game_bridge.cpp:91`). Python `HOOKS` `tools/bridge_game.py:26`.
- `src/game_player.cpp:15-16`: registry `0x7a44320`/`0x7a44340`, vtables `0x38a93c8`, `0x38b2c98`, `0x4f70168`.
- `src/game_menu.cpp` (only way to set VIEW: GAME SCREEN): 11 hooks (`0x1d1cf30, 0x7dc400, 0x72d640, 0x72d650, 0x72e1d0, 0x72d5f0, 0x72d610, 0x730ee0, 0x730ae0, 0x1749970, 0x1749ae0`, `:532-552`), RVAs `0x5d96dd0, 0x7dd9f0, 0x807740, 0x80a220, 0x7dd8ae, 0x7dd7c0, 0x1d29690, 0x389f4c0` + type vtables `0x3a7d6e8, 0x3a7d838`, hashes `0xd0dbcfa5, 0x6d8ed88` (`:52-79`).
- `src/game_steam.cpp:23`: `0x5d69010, 0x2aa63f7, 0x2aa6400` (Steam ticket guard; optional).
- `src/game_pad.cpp`: XInput (`xinput1_4/1_3/9_1_0` `XInputGetState/GetCapabilities`, `:17,118-119`) - generic, needed for gamepad control on the screen.

### 2.8 Config / identity strings
`L"Spider-Man.exe"` in ~25 source files (each DLL's start check); `tools/vr_display.py:26-27` registry `Software\Insomniac Games\Marvel's Spider-Man Remastered\Graphics|Input`; Steam app `1817070` (`tools/vr_launcher.py:25`, `tools/spidy-linux.sh:12`), `pgrep 'Spider-Man\.exe'` (`spidy-linux.sh:45,92,103`), `process_ids('spider-man.exe')` (`tools/capture_game_state.py:85`), `game_queue(image='spider-man.exe')` (`probe_stereo_gpu.py:42`), hard-coded install paths (`tools/research_game.py:21`, `tools/find_method_owners.py:11`), `-nolauncher` (`vr_launcher.py:280`), Python hook tuples (`run_game_vr.py:33-44`, `probe_game_screen.py:46`, `probe_eye_frames.py:29`).

### 2.9 Counts (stereo-screen path, distinct items)
| Category | Must re-find for SM2 | Of which only for immersive/diag (can delete) |
|---|---:|---:|
| Function hooks (prologue-checked RVA) | 16 stereo_probe + 4 HUD + 2 render-mem + 1 time = **23** | 5 (`0x17991a0, 0x186cc00, 0x19206a0, 0x170fbc0, 0x187e130`-as-hook) + time hook if gate reads the global directly |
| Called-only functions | 11 | 4 (`0x186c670, 0x191afb0, 0x191b880, 0x1872860`) |
| Check-only code/data signatures | 7 + 2 call sites + 6 time instrs = 15 | `0x164703f`, time instrs |
| Return-address RVAs | 2 | `0x1647044` |
| Globals / data | 7 + 7 HUD + 1 alloc + 1 time = **16** (+2 HUD code ranges) | `0x7a436e8/0x7a43704` only if HUD dropped |
| Vtables | 2 + 1 HUD stream = 3 | - |
| Render command IDs / enums | ~6 | - |
| Struct field offsets | ~85 | ~25 (hero/shadow/readback) |
| COM vtable indices (generic, keep) | 8 | - |
| Image-size / module-name / registry / app-id / hash | ~6 kinds, ~40 sites | - |
| Gate extras (bridge 5+1+3, player 2+3, menu 11+~12, steam 3) | ~40 | all removable if gate simplified |

Location method breakdown: hard-coded RVA + byte check: ~100%. Signature scan: 0. Runtime RTTI/string: 0 (offline only). Vtable-slot cross-checks: 2 (`native_hud.cpp:293`, `game_bridge.cpp:279`). Return-address gating: 2.

### 2.10 Discovery tooling
- `third_party/SpidyVR/tools/inspect_game.py`: PE32+ parser (`PE`, `:48-93`), MSVC RTTI vtable-to-type (`PE.vtable`, `:83-93`), checks `FUNCTIONS`/`VTABLES` leads (ArkWeb-derived, `:38-46`). Generic except the hash gate (`:9-12,110,129`).
- `tools/discover_render_types.py`: regex over `.?AV...@@` type descriptors -> COL -> vtables + method lists (`:25-54`); default filter `Camera2@|CameraManager|RenderView|SceneView|RenderContext|...` (`:60`). **Fully generic algorithm**; refuses unknown hash (`:62-64`) - delete that check.
- `tools/research_game.py`: capstone disassembly by `.pdata` unwind bounds, rel32/RIP-relative xrefs, string search (`modes function|refs|strings`). Generic; hash check `:26-27`, hard-coded path `:21`.
- `tools/find_method_owners.py`: which RTTI vtable holds a method. Generic; hash + path hard-coded (`:11-14`).
- Runtime probes (game-specific, rewrite per target): `probe_game_screen.py --stereo DEPTH` (drives eye command 3 with no headset and measures parallax - the ideal SM2 acceptance test), `probe_stereo.py`, `probe_stereo_gpu.py`, `capture_stereo.py`, `probe_eye_frames.py`, `probe_exposure.py`, `probe_render_commands.py` (uses `SpidyCommandTrace`), `probe_hud.py`, `probe_vr_load.py`, `trace_scene.py` + `src/scene_trace.cpp`, `src/eye_capture.cpp`, `src/game_observer.cpp` (also SizeOfImage `140496896`, `:173`), `src/camera_watch.cpp` (generic watchpoint).
- Docs say the render facts came from disassembly + live tracing, recorded in `third_party/SpidyVR/docs/VALIDATION.md` (e.g. frame order `:5947-5966`, ViewContextManager `:6292-6316`, cascades `:2416-2430`) and `third_party/SpidyVR/docs/REFERENCE.md:18-60,125-170`. Game strings worth searching in SM2: `Adapted Lum`, `kTaaTarget0`, `kSsrHistory`, `RenderAlloc`, `TimeScaleSystem Update`, `PlayerModelHudFollower`, `ScaleformRTTStream`, `ViewContextManager`, `GlobalWorldCBuffer`/`m_ActiveViewCtxPos` (shader reflection), `TemporalAa`, `PrepareSrgbOverlay`.

## 3. Version gate
- Source of truth: `third_party/SpidyVR/tools/inspect_game.py:9` `EXPECTED_SHA256 = "e297d4d9...81ad1"` and `:12` `EXPECTED_VERSION = "4.630.0.0"` (Steam build 23986256; also `third_party/SpidyVR/docs/REFERENCE.md:20-24`, size 121,901,008 B).
- Readers: launcher parses both constants out of the .py text (`apps/launcher/system.cpp:660-664`, `text::pythonConstant`) and words the error (`include/spidy/launcher_text.hpp:716-748`); `tools/capture_game_state.py:115-118` (`steam_build` = hash match, else version must match or raise); `run_game_vr.py:1676-1679` (warn if version matches but hash not); hard refusals in `discover_render_types.py:62`, `find_method_owners.py:13`, `research_game.py:26`.
- Real runtime gate is in the DLLs: module name + per-RVA byte checks (error codes 1002 `stereo_probe.cpp:1781`, 9501 `native_hud.cpp:294`, 7001 `native_render_memory.cpp:138`, 9701-9707 game_time, 1003 bridge), plus Python's "already patched" compare (`run_game_vr.py:1696-1699`).
- To retarget: change the two constants, the exe name in every `GetModuleHandleW`, `140496896`, app id, registry keys, paths; consider replacing hash gates with a per-build table keyed by `EXPECTED_SHA256`.

## 4. Reuse and minimum build
Engine-agnostic, reusable unchanged (or near):
- `src/xr_session.cpp`, `include/spidy/xr_session.hpp` (+ `controller_layout.hpp`, `tracking.hpp`, `xr_timing.hpp`, `xr_layers.hpp`): OpenXR on an existing D3D12 device/queue, quad stereo layers. Only "Spidy" app name (`xr_session.cpp:82-83`).
- `src/d3d12_renderer.cpp` (blit, capture), `copy_eye_texture.hpp`, `eye_pair_state.hpp`, `eye_job_table.hpp`, `eye_resolution.hpp`, `monitor_rate.hpp`, `presentation_gate.hpp`, `native_eye_frame.hpp` stereo math, `native_view.hpp` math (descriptor *layout* is game-specific), `vr_settings.cpp`.
- `src/native_eye_gpu.cpp` (all D3D12/DXGI vtable hooking, staging, Present copy, recursive mutex for vkd3d-proton), `src/render_probe.cpp` (with HWND fallback for Proton), `src/game_pad.cpp` (XInput).
- Python injection plumbing: `observe_game.py`, `bridge_game.py prepare`, `probe_stereo_gpu.discover_queue/game_queue`, `xr_runtime.py`, `spidy-linux.sh` (change app id/exe).

CMake (`CMakeLists.txt`): everything game-side is under `SPIDY_BUILD_OBSERVER` (needs `third_party/minhook`), XR under `SPIDY_BUILD_XR` (`third_party/OpenXR-SDK`). Relevant targets: `spidy_render_probe` (render_probe.cpp), `spidy_render_memory` (native_render_memory.cpp), `spidy_stereo_probe` = stereo_probe.cpp + native_eye_gpu.cpp + native_webs.cpp + native_body.cpp + native_hud.cpp + (with XR) game_xr.cpp, game_player.cpp, game_pad.cpp, game_menu.cpp, game_time.cpp, game_prompts.cpp, game_steam.cpp, linked to `spidy_core` (17 files incl. swing, body IK...) and `spidy_openxr` -> `spidy_d3d12` -> `spidy_core`. `spidy_bridge`, `spidy_ray_bridge` (9 files), `spidy_movement_bridge` are separate DLLs loaded first.

Minimum "stereo screen only" set:
- DLL `spidy_render_probe`: `src/render_probe.cpp`.
- DLL `spidy_render_memory`: `src/native_render_memory.cpp`, `include/spidy/native_render_memory.hpp` (only if SM2 has the same ring limit).
- DLL `spidy_stereo_core`: `src/stereo_probe.cpp` (strip hero hiding, activeView/restGameView, shadow donor, readback, webs/body calls -> stubs for `native_webs::heroSample/ropeStarts/status/webInstance`, `native_body::drawn/renderFrame`, `native_appearance::Data`), `src/native_eye_gpu.cpp`, `src/native_hud.cpp` (or delete and drop the `ScenePanel`/`onScreen` gate), `src/xr_session.cpp`, `src/d3d12_renderer.cpp`, `src/game_pad.cpp` (+ `prompt_buttons.hpp`/`.cpp` header dep of `game_pad.hpp`), `src/vr_settings.cpp`, a new ~300-line worker replacing `game_xr.cpp` (keep: runtime init `:563-576`, GameScreen/SceneClock/ScenePanel, native_gpu start `:2505-2519`, views start `:2525-2575`, command 3 `:2576-2600`, draw callback `:2754-2822`); a 10-line game-time read instead of `game_time.cpp`. Headers: `native_eye_frame.hpp`, `native_view.hpp`, `math.hpp`, `eye_job_table.hpp`, `eye_pair_state.hpp`, `eye_resolution.hpp`, `copy_eye_texture.hpp`, `presentation_gate.hpp`, `monitor_rate.hpp`, `native_eye_gpu.hpp`, `native_hud.hpp`, `xr_session.hpp`, `controller_layout.hpp`, `tracking.hpp`, `xr_timing.hpp`, `xr_layers.hpp`, `d3d12_renderer.hpp`, `vertex.hpp`, `vr_settings.hpp`, `game_pad.hpp`. Replace `native_eye_history.hpp` (pulls game_swing, web_visual, body_calibration, gadget_wheel...) with a slim `{serial, displayTime, stereoScreen, screenPose, aspect}` ring.
- Drop entirely: game_bridge, ray_bridge, movement_bridge, game_player, game_menu (default `flatScreen = true`, settings from launcher/config), game_prompts, game_steam (re-evaluate for SM2), native_webs, native_body, all of spidy_core except vr_settings (check `d3d12_renderer`'s real use of `spidy_core`; it includes only its own header).
- Simplified gate: `scene = screen && screenDepth > 0 && gameFrame > 0` (+ HUD panel if kept).

## 5. Stereo technique (doc quotes)
- Synchronized, same-frame pair, not alternating: "Both eyes share one simulation update." (`docs/MILESTONE-1.md:40`). Code: both eyes submitted in one `placeEyes()` per game frame (`stereo_probe.cpp:1339-1392`) with one serial/generation; `EyePairState::ready()` requires both finished with equal stamps (`eye_pair_state.hpp:23-27`). "Both eyes share a serial and generation." (`MILESTONE-1.md:179-180`). Placement timing: "Spidy places both eyes in a hook on `1920240`, before the copy." (`MILESTONE-1.md:156-157`), frame order in `third_party/SpidyVR/docs/VALIDATION.md:5947-5966`. Cost: the game renders 3 scene views in a cutscene (`docs/CHANGELOG.md:1750-1752`).
- Held images: last pair re-submitted up to 1000 ms (`presentation_gate.hpp:236-241`; `MILESTONE-1.md:180-182`).
- TAA / upscaler history per eye: each eye is a separate engine view with its own buffer set, so its own TAA history; the game's DLSS/FSR/XeSS is bypassed: "DLSS itself is not used for the eyes" (`docs/DEVELOPMENT.md:878`); mechanism `DEVELOPMENT.md:865-879` (AA mode `+40c` forced to 1 in `copyJob`, TAA targets via `initBuffers`/`sizeBuffers`, Halton jitter). Upscale share "50% to 100% of the eye's size per side, 67% by default" (`DEVELOPMENT.md:742-746`). Correct motion vectors need placement before the history shift (`stereo_probe.cpp:1221-1231`).
- Exposure: shared from the game's main view: "the main camera's exposure, so both eyes are drawn equally bright" (`DEVELOPMENT.md:892-903`; code `stereo_probe.cpp:861-896,1008-1030`). In mode 3 the main view keeps rendering, so both eyes take the stock camera's exposure.
- Shadows: in mode 3 each eye fits its own cascades (`stereo_probe.cpp:203-205,1315`); sharing applies to immersive (`DEVELOPMENT.md:953-960`).
- Eye resolution: headset recommended size x render scale (`DEVELOPMENT.md:219-220,728-738`, `eye_resolution.hpp`), scene at upscale %; on the stereo screen the stock (often ultrawide) lens is squeezed into that eye-sized image and un-squeezed by quad aspect.
- Screen geometry: convergence = 2.5 m x depth; infinity parallax = IPD/screen width; separation capped by lens ("A close-up looks larger than life", `CHANGELOG.md:1743-1745`; `native_eye_frame.hpp:36-52,105-117`). "the game draws it once for each of your eyes, from its own cutscene camera set a little apart" (`CHANGELOG.md:1729-1730`). Flat cases (pause, map, loading) and the pause-menu-as-scene bug that motivated the game-time gate (`CHANGELOG.md:1746-1749,1771-1772`). VIEW: GAME SCREEN in 3D added later (`CHANGELOG.md:209-217`).
- Validation numbers without a headset: far shore 75/77 px apart, HUD 0-1 px (`CHANGELOG.md:1757-1760`); never verified in a real cutscene/headset (`:1762-1769`).

## 6. Port risks (Remastered-only assumptions)
1. **No signature scanning anywhere**: ~23 hooks, ~11 calls, ~16 globals, ~85 offsets must be rediscovered by hand in Spider-Man2.exe; any SM2 patch breaks all of them. Strongly consider adding pattern scans while porting.
2. **Offscreen view API** (`0x18a09c0` create with name/near/far/fov/w/h/flags, 8 offscreen slots at mgr+0x30, view stride `0x1f70`, descriptor `0x440`, lens ctor with 12 args): SM2's renderer (newer engine, RT-heavy) may have changed layout, count of slots, or not expose offscreen views the same way.
3. **Frame ordering** (history shift -> gameplay -> offscreen job setup -> game views -> maintenance in `0x175a060`) is what makes same-frame placement work; must be re-verified.
4. **TAA/upscaler**: the hack depends on the engine's own `TemporalAa` command (0x8a), AA mode at render context `+0x40c`, buffer-kind mask table, and TAA targets by kind. SM2 ships DLSS/FSR/XeSS/PSSR-era paths and frame generation; its native TAA (IGTI?) path may not exist for offscreen views.
5. **GPU pair detection** needs the game to emit timestamp `EndQuery` calls inside render-job begin/end; a build without GPU timing markers yields no pairs (would need Spidy-inserted markers or fences).
6. **HUD**: the stereo screen depends on Remastered's world-space `PlayerModelHudFollower` + Scaleform RTT; if SM2's HUD is screen-space or differs, `ScenePanel` keeps the screen flat after 1 s. Either reimplement or remove that gate (and accept HUD only in the flat game view, or composite it yourself).
7. **Pause gate**: frame game time exactly 0.0 while paused (`REFERENCE.md:162`); SM2 may keep a non-zero time in menus or use a different clock.
8. **Render memory**: 128 MB RenderAlloc ring with 32-bit offsets, replaceable only at creation ~3 s after start; SM2 sizes/structure may differ, and late attach (as `spidy-linux.sh` does, "wait at the main menu") cannot enlarge it.
9. **Exposure/tone**: Adapted Lum at view `+0x16f0`, tone `genericView` flag, profile flags `0x480`, PrepareSrgbOverlay skipped for negative view ids - all Remastered render-graph specifics; wrong values give dark/HDR-gained eyes or missing UI.
10. **Ray tracing**: eyes inherit no RT reflections (launcher turns `RTReflections` off, `vr_display.py`); SM2 has RT on many presets and may require RT resources per view (BVH/TLAS keyed to the active view).
11. **VIEW toggle lives in game_menu Scaleform hooks** (Remastered settings UI internals). Simplest port: make GAME SCREEN the only mode.
12. `game_xr` hard-requires bridge/ray/motion modules and a player (`game_xr.cpp:3289-3290,1326-1335`); those are all swing-feature code tied to Remastered.
13. Identity: exe name in ~25 files, SizeOfImage constant, app id 1817070 (SM2: different), Insomniac registry path (SM2 settings likely under a different key/Nixxes), Steam ticket guard RVAs, launcher-screen skip `-nolauncher`.
14. Anamorphic eye images: horizontal resolution per eye equals the headset eye width regardless of the 16:9/21:9 lens; acceptable but wasteful - a port could create eye views at the stock aspect.
15. Anti-tamper/integrity on SM2's exe (not checked here) could block CreateRemoteThread/LoadLibrary injection or MinHook prologue patches.
