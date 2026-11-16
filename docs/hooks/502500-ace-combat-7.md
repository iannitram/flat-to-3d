# ACE COMBAT 7 hook map (AR, #286)

The hook-point research step for ACE COMBAT 7 ([ROADMAP](../ROADMAP.md#hook-point-research-ruling-52), ruling 52). Step A2 (#41) works from this table. It was made on the installed game with our own tools, without running the game. UEVR's scanner, the keton plugin and the [research note](../notes/research/games/502500-ace-combat-7.md) are the checklist at the end, not a source.

| Item | Value |
|---|---|
| Exe | `Ace7Game.exe` (install root), 65,331,992 bytes |
| SHA-256 | `c7da97f5f8a807d4f1264adbb074146fcffe9bdc2ffa98791b822cd28e558f4f` |
| File version | `2.2.1.16` (PE version resource) |
| Store build | Steam build id 9855922 (depot 502501, manifest 788319397026631312) |
| Engine | Unreal 4.18, project `Nimbus`; image base `0x140000000`; no PDB |

**Read this before using a pattern.** `.text` on disk is encrypted by the Steam DRM wrapper (SteamStub variant 3.1, x64, entry point in `.bind`; original entry point RVA `0x22339ec`). Every RVA and pattern below is for the decrypted image, as it sits in memory once the wrapper has run. A scan of the file on disk finds nothing. For the research the wrapper was decrypted into a scratch copy (AES-256 key and IV from the wrapper header); the game folder was only read.

**How found, for every row:** 2026-11-16, Python with capstone 5.0.6 and numpy over the decrypted copy: a scan of every RIP-relative displacement in `.text` for references to a string or global, function bounds from `.pdata` (chained unwind entries followed to the primary one), disassembly of the referring code. REA's Ghidra provider timed out loading the 62 MB exe and its rizin provider does not run on Windows, so neither was used. Each pattern was counted over the whole decrypted image and matches exactly once. `??` is a wildcarded RIP-relative displacement or call/jump target. A pattern marked *exact* has no wildcards and holds only for this build.

## Stereo device

There is no fake stereo device in this exe. The engine's `UEngine::StereoRenderingDevice` slot is there and the renderer calls through it, so a device of our own goes into that slot (what UEVR does for this game; see the checklist).

| Hook | Step use (A2 #41) | RVA | Pattern | How found | Status |
|---|---|---|---|---|---|
| `GEngine` (global `UEngine*`) | where the device pointer lives | `0x3cbbc28` (`.data`) | `48 8B 0D ?? ?? ?? ?? 0F B6 F8 48 85 C9 0F 84 ?? ?? ?? ?? 48 83 B9 C8 0A 00 00 00` at `0x118f45c`; the `??` after `48 8B 0D` resolves to it | String ref: the UTF-16 `Stereo is %s` (`0x2afea38`) is used only by the `vr.bEnableStereo` console handler, function `0x118f3f0`; it loads this global and tests `+0xAC8` | verified |
| `UEngine::StereoRenderingDevice` | the slot our device is written to (`TSharedPtr<IStereoRendering>`, object pointer first) | `GEngine + 0xAC8` (offset) | same pattern as above (`48 83 B9 C8 0A 00 00 00`) | The handler reads `+0xAC8`, calls `[vtbl+0x10]` (EnableStereo) and `[vtbl+0x00]` (IsStereoEnabled) through it; every stereo call site below loads it from `GEngine` | verified |
| `UEngine::XRSystem` | not needed by A2; sanity check of the layout | `GEngine + 0xAD8` (offset) | `48 8B 88 D8 0A 00 00` inside `0x118f3f0` (not unique on its own) | Same handler: `GetHMDDevice` through `[vtbl+0xB8]` | verified |
| `FFakeStereoRendering` vtable | would be patched for the two eyes | none | none | Absent. No string reference to `L"emulatestereo"` (`0x2578838`, `0x2e6d2f8`), `L"nohmd"`, `L"hmd="`, `L"r.EnableStereoEmulation"` (`0x2e6cb58`) or its help text (`0x2e6ca08`), RIP-relative or absolute. The constructor strings UEVR looks for (`r.StereoEmulationHeight`, `r.StereoEmulationFOV`) are not in the exe. The fake device's 0.151976421 constant is not in the exe, and its 3.2 constant (`0x2cf2500`) is used only by unrelated code (`0x1694d00`). No RTTI either: the 658 type descriptors are all third-party (CRI, ICU and others). | lead |
| `FFakeStereoRendering::CalculateStereoViewOffset` | would give each eye its offset | none | none | Absent with the vtable (above). A2 implements this slot in its own device instead | lead |
| `FFakeStereoRendering::GetStereoProjectionMatrix` | would give each eye its projection | none | none | Absent with the vtable (above). A2 implements this slot in its own device instead | lead |
| Device slot 3, `AdjustViewRect` (`vtbl+0x18`) | split the view rect per eye | call at `0x183c2c2` | `4C 8D 4C 24 3C 48 8D 54 24 38 48 89 54 24 20 4C 8D 44 24 34 48 8B 88 C8 0A 00 00 41 8B D5 48 8B 01 FF 50 18` at `0x183c2a1` | Call shape: `edx` = stereo pass, `r8`/`r9` = &X/&Y, stack = &SizeX/&SizeY; device loaded from `GEngine+0xAC8`. In function `0x183c010` (chunk of the scene-view setup; likely `ULocalPlayer::GetProjectionData`) | verified |
| Device slot 5, `CalculateStereoViewOffset` (`vtbl+0x28`) | per-eye camera offset | call at `0x183c4af`; device load at `0x183c468` | `41 8B D5 48 89 4C 24 20 4C 8D 85 FC 00 00 00 48 8B CF F3 0F 10 98 70 04 00 00 FF 53 28` at `0x183c495` (and `48 8B B8 C8 0A 00 00 48 85 FF` at `0x183c468` for the load) | Call shape: `edx` = pass, `r8` = &Rotation, `xmm3` = WorldToMeters (world settings `+0x470`), stack = &Location. Same function `0x183c010` | verified |
| Device slot 6 (`vtbl+0x30`) | none yet | call at `0x183c514` | not made | Same argument shape as slot 5, called right after it; purpose not pinned | lead |
| Device slot 7, `GetStereoProjectionMatrix` (`vtbl+0x38`) | per-eye projection | call at `0x183ca62`; device load at `0x183ca58` | `48 8B 88 C8 0A 00 00 48 8B 01 FF 50 38` at `0x183ca58` | Call shape: `rdx` = &out FMatrix, `r8d` = pass, returns the matrix pointer; device loaded from `GEngine+0xAC8`. Same function `0x183c010` | verified |
| Device slot 13, `GetRenderTargetManager` (`vtbl+0x68`) | null in our device unless A2 renders to its own targets | call at `0x1095a88` | not made | Call through the `+0xAC8` device; the result is used as an object pointer | lead |
| Device slot 0, `IsStereoEnabled` (`vtbl+0x00`) | our device returns true | 11 call sites, e.g. `0x118f4c8` | not made (many sites) | `call [rax]` on the `+0xAC8` device; result in `al` picks `Stereo is %s` text | verified (slot); sites not listed |

The calls through `vtbl+0x70` (12 sites) were seen but not identified.

## Nimbus classes

The plugin and A2 reach these through Unreal's reflection at run time, so A2 needs no address for them. The rows record what is findable statically, to check a running object against. Each class registers in 4.18 style: three copies of its UTF-16 name, one used by the deferred-registration static initializer, one (+2, without the `A`/`U` prefix) by `GetPrivateStaticClass`, one by `FCompiledInDefer`. `GetPrivateStaticClass` caches the `UClass*` in a global, calls `GetPrivateStaticClassBody` (`0xacae10`) with the class size and flags, and passes `StaticRegisterNatives`, whose table pairs an ANSI function name with its exec thunk.

The `GetPrivateStaticClass` patterns are long because these functions differ only in the size and flags at their end.

| Hook | Step use (A2 #41) | RVA | Pattern | How found | Status |
|---|---|---|---|---|---|
| `ANimbusPlayerController::GetPrivateStaticClass` | the player controller (the plugin uses its BP subclass `NimbusPlayerControllerBP_C`) | `0x92dbd0`; cached `UClass*` at `0x3a829a8`; size `0x6e8`, flags `0x10000004`; natives table `0x286aa80` (14 entries) | `4C 8B DC 48 83 EC 78 48 8B 05 ?? ?? ?? ?? 48 85 C0 0F 85 ?? ?? ?? ?? 88 44 24 68 4C 8D 0D ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 43 E8 4C 8D 05 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 43 E0 48 8D 15 ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 43 D8 48 8D 0D ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ?? 49 89 43 D0 48 8D 05 ?? ?? ?? ?? 49 89 43 C8 48 8D 05 ?? ?? ?? ?? 49 89 43 C0 49 C7 43 B8 00 00 00 00 C7 44 24 28 04 00 00 10 C7 44 24 20 E8 06 00 00` | String ref: name copies `0x286a580` (ref `0x251189`), `0x286a5b0` (+2 ref `0x92dc12`), `0x286a5e0` (ref `0x2512a5`) | verified |
| `ANimbusPlayerCameraManager::GetPrivateStaticClass` | the camera manager (the plugin uses its BP subclass `NimbusPlayerCameraManagerBP_C`) | `0x92d4a0`; cached `UClass*` at `0x3a828f0`; size `0x1d70`, flags `0x10000008`; natives table `0x2869f28` (2 entries) | as the player controller's, ending `C7 44 24 28 08 00 00 10 C7 44 24 20 70 1D 00 00` | String ref: `0x2869070` (ref `0x250e09`), `0x2869130` (+2 ref `0x92d4e2`), `0x2869188` (ref `0x251115`) | verified |
| `ANimbusPlayerCameraManager::execTestLoopCameraShakePlay` | the plugin's camera-shake call | `0x92daa0` | *exact* `48 8B 42 20 45 33 C0 48 85 C0 41 0F 95 C0 4C 03 C0 4C 89 42 20 E9 76 4B B0 FF` | Natives table entry 0 | verified |
| `ANimbusPlayerCameraManager::execTestLoopCameraShakePlayAddScale` | the plugin's scaled camera shake | `0x92dac0` | *exact* `48 89 5C 24 10 57 48 83 EC 20 48 83 7A 20 00 0F 57 C0 48 8B F9 F3 0F 11 44 24 30 48 8B DA 48 8B CA 74 10 48 8B 52 18 4C 8D 44 24 30 E8 7F 18 2A` | Natives table entry 1 | verified |
| `APlayerCameraManager::execStopAllCameraShakes` (engine) | the plugin stops shakes in the cockpit | `0x1c0a3e0` or `0x1c13030` | not made | Two engine natives tables (`0x2f82328` and `0x2f81f90` region) carry the same camera-manager function names, each with its own `StopAllCameraShakes` thunk; which table is `APlayerCameraManager`'s was not pinned | lead |
| `ANimbusHUD::GetPrivateStaticClass` | the HUD class (A2: HUD placement, if needed) | `0x925f60`; cached `UClass*` at `0x3a82318`; size `0xcb0`, flags `0x1000000c`; natives table `0x2863b70` (38 entries) | as the player controller's, ending `C7 44 24 28 0C 00 00 10 C7 44 24 20 B0 0C 00 00` | String ref: `0x2862fc8` (ref `0x24eb49`), `0x2863060` (+2 ref `0x925fa2`), `0x2863090` (ref `0x24ee95`) | verified |
| `UCameraViewComponent::GetPrivateStaticClass` | the plugin's cockpit/chase view switch | `0x8bf660`; cached `UClass*` at `0x3a768b0`; size `0x360`, flags `0x10000000`; `StaticRegisterNatives` `0x8bf710` (8 natives) | as the player controller's, ending `C7 44 24 28 00 00 00 10 C7 44 24 20 60 03 00 00 E8 ?? ?? ?? ?? 48 8B 05 ?? ?? ?? ?? 48 83 C4 78 C3 CC CC CC CC CC CC CC CC CC CC CC CC CC CC 48 83 EC 28` | String ref: `0x27897d8`, `0x2789808` (+2 ref, function `0x8bf660`), `0x2789838` | verified |
| `UCameraViewComponent` exec thunks | view type and view switch | `GetCurrentCameraViewType` `0x21d6b90`, `SwitchToCockpitView` `0x8bfb70`, `SwitchToFirstPersonView` `0x8bfb90`, `SwitchToThirdPersonView` `0x8bfc30` | *exact*, in order: `40 53 48 83 EC 20 48 8B 42 20 45 33 C9 48 85 C0 49 8B D8 41 0F 95 C1 4C 03 C8 4C 89 4A 20 E8 2D E4 00 00 88`; `48 8B 42 20 45 33 C0 48 85 C0 41 0F 95 C0 4C 03 C0 4C 89 42 20 E9 F6 2E`; the same with `E9 86 2F`; the same with `E9 26 32` | Natives table entries | verified |
| `AMission::GetPrivateStaticClass` | the plugin's cutscene check | `0x8f11a0`; cached `UClass*` at `0x3a7ccd0`; size `0x8d8`, flags `0x10000000`; `StaticRegisterNatives` `0x8f13f0` (53 natives) | as the player controller's, ending `C7 44 24 28 00 00 00 10 C7 44 24 20 D8 08 00 00` | String ref: `0x27fd918`, `0x27fd9a8` (+2 ref, function `0x8f11a0`), `0x27fd9e0` | verified |
| `AMission` exec thunks | cutscene check and the plugin's mission cheats | `IsInIGC` `0x8f28e0`, `Complete` `0x8f23f0`, `CompleteCoolDownOverride` `0x8f2410`, `ForcePauseMissionTimer_S` `0x8f24d0` | *exact*, in order: `40 53 48 83 EC 20 48 8B 42 20 45 33 C9 48 85 C0 49 8B D8 41 0F 95 C1 4C 03 C8 4C 89 4A 20 E8 DD BB C2`; `48 8B 42 20 45 33 C0 48 85 C0 41 0F 95 C0 4C 03 C0 4C 89 42 20 E9 D6 93`; `48 89 5C 24 08 57 48 83 EC 20 48 83 7A 20 00 48 8B F9 48 8B CA C7 44 24 48 00 00 00 00 48 8B DA 74 10 48 8B 52 18 4C 8D 44 24 48 E8 30 CF`; `48 89 5C 24 08 57 48 83 EC 20 33 FF 48 8B DA 48 8B CA 89 7C 24 38 48 39 7A 20 74 10 48 8B 52 18 4C 8D 44 24 38 E8 76 CE` | Natives table entries | verified |
| `UNimbusCheatManager::GetPrivateStaticClass` | none (the plugin names the class only) | `0x9161e0`; cached `UClass*` at `0x3a81ae8`; size `0x300` | not made | String ref: `0x284eb90`, `0x284eeb8` (+2 ref, function `0x9161e0`), `0x284ef68`. Its natives include `SetStereoDepth` (`0x91eb80`) and `SetStereoDepthDirectly` (`0x91ec00`), left from the PS4 VR mode | lead |

## Checklist

| Item | Source | Our finding | Mark |
|---|---|---|---|
| Exe SHA-256 `c7da97f5...8f4f`, 65,331,992 bytes | research note | same | agrees |
| Steam build 9855922, depot 502501 | research note | same (`appmanifest_502500.acf`) | agrees |
| No PDB; hooks need byte patterns | research note | no PDB; also no game RTTI | agrees |
| "The engine's built-in fake stereo device is very likely compiled in" | research note | the device and its `emulatestereo` path are not reachable: no code reads the strings | differs |
| "Replace the fake stereo device's view-offset and projection functions ... UEVR's scanner already finds the same functions in 4.18" | research note, section 4 | those functions are not in this exe; the device slots are reached through `GEngine+0xAC8` | differs |
| Fake device found by string ref to `L"r.StereoEmulationHeight"` / `L"r.StereoEmulationFOV"` | UEVR `FFakeStereoRenderingHook` @ `4ee5c6b` | strings absent | differs |
| `-emulatestereo` fallback (string ref to `L"emulatestereo"`) | UEVR @ `4ee5c6b` | string present, never referenced | differs |
| The nonstandard path: no fake-device vtable, but `GEngine`'s `StereoRenderingDevice` slot exists, so UEVR writes its own device there | UEVR @ `4ee5c6b` | same | agrees |
| `StereoRenderingDevice` fallback offset `0xAC8` (generic nonstandard hook) | UEVR @ `4ee5c6b` | `0xAC8` | agrees |
| `StereoRenderingDevice` fallback offset `0xAE8` (the 4.18 hook, picked only for file version 4.18.3.0) | UEVR @ `4ee5c6b` | `0xAC8`; the exe's file version is 2.2.1.16, so UEVR does not pick this hook | differs |
| Slot 5 `CalculateStereoViewOffset`, slot 7 `CalculateStereoProjectionMatrix`, slot 13 `GetRenderTargetManager` (generic nonstandard table) | UEVR @ `4ee5c6b` | slots 5 and 7 match the call shapes; slot 13 seen, not confirmed | agrees |
| Slot 6 is the projection (the 4.18 table) | UEVR @ `4ee5c6b` | the projection is slot 7; slot 6 is a view-offset-shaped call | differs |
| The plugin uses `ANimbusPlayerController` | research note; keton plugin @ `e4b26ac` | the plugin finds `NimbusPlayerControllerBP_C` by name; the native class is at `0x92dbd0` | agrees |
| The plugin uses `ANimbusPlayerCameraManager` | research note; keton plugin @ `e4b26ac` | the plugin calls `TestLoopCameraShakePlay`, `TestLoopCameraShakePlayAddScale` (both native here) and `StopAllCameraShakes` (engine) on `NimbusPlayerCameraManagerBP_C` | agrees |
| The plugin hooks `ANimbusHUD` | research note, line 71 | the plugin source never names the HUD class | differs |
| The plugin uses `CameraViewComponent` and `Mission` functions by name | keton plugin @ `e4b26ac` | all seven functions are in the native tables (rows above) | agrees |
| The plugin uses addresses | keton plugin @ `e4b26ac` | none: reflection only, so it does not depend on this build | agrees |
