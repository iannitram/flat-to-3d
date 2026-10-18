# Spider-Man 2 address map (Remastered → Spider-Man 2)

Every game-specific item on Spidy's GAME SCREEN 3D path ([spidy-stereo-core-map.md](spidy-stereo-core-map.md)), with its Remastered address and, once found, its Spider-Man 2 counterpart as a byte pattern. Plan step 3 (and steps 4–8 after it) works from this table.

Started 2026-10-18 on Hyperion.

## Where this stands (2026-10-18, first scan leg)

**No exe has been analysed yet. Every SM2 column below says `pending`.** Both games were still downloading on Hyperion for the whole leg (14:41–16:00 MDT):

- Steam suspended Remastered (app 1817070) at 14:40 and downloaded Spider-Man 2 (app 2651280) first. Steam's own log says `AppID 1817070 update canceled : Priority (Suspended)`.
- Download speed was 0–280 Mbps, about 150 on average. At that rate Spider-Man 2's 97 GB takes about 90 minutes and Remastered's 56 GB about 50 more. That's an estimate from the log, not a promise.
- Steam pre-allocates every file at full size and fills it later. Both `Spider-Man2.exe` (194,262,992 bytes) and `Spider-Man.exe` (121,901,008 bytes) existed on disk from 14:40, but every byte of both was still zero at 15:50. That's checked over 1 MiB blocks with `tools/xmatch/filled.py`. **A file existing is not the same as it being downloaded.**

What this leg did instead:

1. **Wrote the matcher** (`tools/xmatch/`, below) and tested it on a Steam DLL. It's ready to run the moment both exes are on disk.
2. **Listed all 85 items** on the stereo path, with the bytes Spidy checks at each, into `tools/xmatch/targets.py`. That's the Remastered column of the table below.
3. **Read the two games' shipped file lists** from Steam's pre-allocation. That turned up four facts that change the plan ([below](#what-the-file-lists-already-show)).

### One thing in Remastered already measured

`Spider-Man.exe` from Steam is **121,901,008 bytes, exactly the size of 4.630.0.0**, the build Spidy 0.3.8 targets (`inspect_game.py`, `docs/REFERENCE.md`). The same size strongly suggests the same build. It doesn't prove it: the SHA-256 check (`verify_rem.py`) is the proof and has not run yet.

## What the file lists already show

These come from the files Steam created for each game, by name and size. That's measured. What each file means is read from its maker's public documentation, not from the game. Nothing here has looked inside an exe.

| Spider-Man 2 ships | Remastered ships | What it means for the port |
|---|---|---|
| `cohtml.WindowsDesktop.dll`, `RenoirCore.WindowsDesktop.dll`, `v8.dll` (Coherent Gameface: HTML and JavaScript UI) | none of these | **Spidy's HUD hooks have no counterpart.** Spidy moves the HUD through Scaleform: its render command (`0x1d2b6c0`), its RTT stream (`0x2104400`) and the Scaleform settings menu (`game_menu.cpp`, 11 hooks). Spider-Man 2 draws its UI with a different library. The HUD-panel gate has to go (step 6 already plans that), and the SPIDY VR settings tab can't be ported. Make GAME SCREEN the only mode. `PlayerModelHudFollower` is still in SM2's type names (Z13 scan), so the panel may still be a 3D model: worth checking in step 7. |
| `PsPcSdkRuntimeInstaller.msi`, `PsPcSdkRuntimeManager.msi` (PlayStation PC SDK) | no | This is the PlayStation Network sign-in runtime. **Ground rule check:** before injecting anything, confirm the game plays fully offline with no PSN sign-in. Public reports say PSN is optional for SM2 on PC. That's read, not tested here. |
| `NRD.dll`, `NRI.dll` (NVIDIA real-time denoisers), `nvngx_dlssd.dll` (DLSS ray reconstruction) | no | Ray-traced effects keep their own denoiser history per view. An extra eye view may need its own denoiser state, or ray tracing off for the eyes. This is plan risk "Ray tracing tied to the main view", now more likely. |
| `dstoragecore.dll` (DirectStorage) | no | Asset streaming only. It adds copy and compute GPU queues. Spidy's `render_probe` already picks the *direct* queue the exe submits from, so it should still find the right one. Low risk. |
| `sl.interposer.dll`, `nvngx_dlssg.dll` (Streamline, DLSS frame generation) | **same** in both | Streamline wraps DXGI. Spidy already hooks Present through that wrapper on Remastered, so this isn't new. Frame generation stays off for the eyes. |
| `D3D12/D3D12Core.dll` (Agility SDK) | same | DX12 in both. No DX11 path to fall back to. |
| No anti-cheat files (`EasyAntiCheat`, `BattlEye`) | same | As expected for offline single-player. The Z13 scan of SM2 2.810.0.0 found no packer sections. Whether this Hyperion build matches that needs the exe (`survey.py` reports sections, entropy and protection strings). |

## How the matcher works (`tools/xmatch/`)

Read-only, offline and low-memory (mmap). It never touches the installed game. Copy the exes to `B:/private/flat-to-3d-re/` first.

- **`verify_rem.py <Spider-Man.exe>`**: the step-0 risk check. It gives the SHA-256 against Spidy's `e297d4d9…81ad1`, the version, `SizeOfImage` against Spidy's hard-coded 140,496,896, and for every item the bytes Spidy checks there (match or not), whether hooked functions start where `.pdata` says, and each vtable's RTTI name.
- **`survey.py <exe> out.json`**: one exe's identity (version, hash, sections with entropy, imports, TLS), protection strings (Denuvo, VMProtect, Themida, anti-cheat, debugger checks), RTTI vtables for the camera, HUD and player types, plan-relevant feature strings (stereo, split screen, photo mode, debug or free camera, multi-view, OpenXR) and the functions that use Spidy's anchor strings (`ViewContextManager`, `Adapted Lum`, `kTaaTarget0`, `RenderAlloc`, `TemporalAa`, `PrepareSrgbOverlay`, `TimeScaleSystem Update`…).
- **`run_match.py <Spider-Man.exe> <Spider-Man2.exe> out.json`**: the diff. For each Remastered function:
  1. Masked byte windows (rel32 branch targets and RIP-relative displacements made wildcards) are searched in SM2 and vote for the SM2 function they land in.
  2. Each candidate is scored by how closely its instruction sequence (mnemonics and registers, numbers blanked) matches Remastered's. Confidence: **high** means ≥ 0.85 similarity and clearly ahead of the runner-up; **medium** means ≥ 0.65; else **low**.
  3. Aligned instruction pairs map **callees** (call-graph propagation, which upgrades a weak match when a confident caller agrees) and **globals** (RIP-relative refs; `ViewContextManager`, the game clock and the render allocator come out this way).
  4. Vtables are matched by their RTTI type name, which SM2 keeps.
  5. Call sites and return addresses (`0x164703f`, `0x1647044`, `0x175aaa9`, `0x186c325`…) map through the aligned parent function.
  6. For every match it emits the **shortest pattern unique in SM2's `.text`** (`48 89 5c 24 ?? …`), which is what plan step 7 needs.
- **`mkmap.py out.json`** renders the table below from the result.

Expected cost: about 1–2 GB of memory and minutes, not hours (the matcher is targeted, not a whole-program Ghidra analysis). Ghidra Version Tracking is still the cross-check for anything that comes out low.

## The map

Kinds: **hook** = Spidy hooks it; **call** = Spidy only calls it; **check** = Spidy only checks its bytes; **site** = a call site or return address; **data** = a global; **vtable**; **gate** = removable gate extra (swing bridge, player watch). Priority 1 is what plan steps 1–3 need.

Spidy's checked bytes are from the SpidyVR v0.3.8 sources (`stereo_probe.cpp:1694-1783`, `native_hud.cpp:281-294`, `game_time.cpp:20-36`, `native_render_memory.cpp:118-137`, `game_bridge.cpp:264-277`). The gate rows' bytes are truncated prefixes.

### Priority 1. Steps 1-3: camera, frame entry, view setup

| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found |
|---|---|---|---|---|---|---|---|
| stock camera fn (calls submit) | call | `0x1646e10` | - | pending | - | pending | - |
| camera submit call site | site | `0x164703f` | `e86c2a2500` | pending | - | pending | - |
| camera submit return | site | `0x1647044` | - | pending | - | pending | - |
| view descriptor submit | hook | `0x1899ab0` | `483bd1747c488bc1` | pending | - | pending | - |
| lens ctor (12 args) | call | `0x187bb00` | `488bc44889580855` | pending | - | pending | - |
| pose setter | call | `0x187ca10` | `40534883ec600f1002` | pending | - | pending | - |
| frame function | call | `0x175a060` | - | pending | - | pending | - |
| frame -> offscreen call site | site | `0x175aaa9` | `e892571c00` | pending | - | pending | - |
| offscreen render setup (placeEyes hook) | hook | `0x1920240` | `48895c24084889742410574883ec2040b701` | pending | - | pending | - |
| history shift | call | `0x18a13c0` | - | pending | - | pending | - |
| history shift per-view copy | check | `0x189ee00` | `48895c2408574883ec20488bf941b830050000` | pending | - | pending | - |
| view maintenance (ViewContextManager) | hook | `0x18a0bb0` | `48895c240848896c2410` | pending | - | pending | - |
| create offscreen view | call | `0x18a09c0` | `4c8bdc49895b10` | pending | - | pending | - |
| ViewContextManager global | data | `0x7a34dd0` | - | pending | - | pending | - |
| view pool mask | data | `0x7a34dd8` | - | pending | - | pending | - |
| offscreen slots (mgr+0x30) | data | `0x7a34e00` | - | pending | - | pending | - |

### Priority 2. Steps 4-5: eye views, render jobs, buffers, memory, pause gate

| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found |
|---|---|---|---|---|---|---|---|
| view update (exposure copy) | hook | `0x189bd30` | `48895c240848897c2410` | pending | - | pending | - |
| occlusion builder | call | `0x189fed0` | `48895c24104889742418574883ec40488bf94883b9a01b000000` | pending | - | pending | - |
| texture wrapper vtable | vtable | `0x4e56c80` | - | pending | - | pending | - |
| copy view into render job | hook | `0x19223e0` | `48895c2420555657` | pending | - | pending | - |
| render job begin | hook | `0x189e310` | `48895c2408574883ec30` | pending | - | pending | - |
| render job end | hook | `0x189e3a0` | `48895c2408574883ec30` | pending | - | pending | - |
| buffer-set init | hook | `0x1873470` | `40534883ec30448b4a04` | pending | - | pending | - |
| buffer-set init return gate | site | `0x186c325` | - | pending | - | pending | - |
| buffer-set sizes | hook | `0x1874f40` | `48895c2408488974241048897c2418554154` | pending | - | pending | - |
| target kind test | check | `0x1875270` | `440fb6d241b1010fbae20e` | pending | - | pending | - |
| kind-mask table | data | `0x3d11b18` | `ff000000ff0000002000000028000000` | pending | - | pending | - |
| TemporalAa | check | `0x1842e10` | `488bc455488da8f8feffff` | pending | - | pending | - |
| display setup | hook | `0x1920310` | `4d85c00f847f030000` | pending | - | pending | - |
| enqueue render command | call | `0x187e130` | `48895c240848896c2410` | pending | - | pending | - |
| tone map | hook | `0x1846c20` | `488bc448895010` | pending | - | pending | - |
| adapted-lum read | check | `0x189c710` | `488d81f0160000488b10` | pending | - | pending | - |
| RenderAlloc ring create | hook | `0x1872d90` | `48895c240848896c2410488974241857415641574883ec30` | pending | - | pending | - |
| ring create call site | site | `0x188a195` | `488d0de4e60a06e8ef8bfeff` | pending | - | pending | - |
| RenderAlloc per-frame rollover | hook | `0x1872b90` | `48895c2408574883ec3083792800488bd98b411cc6414200` | pending | - | pending | - |
| rollover call site | site | `0x187e623` | `488d0d56a20b06e86145ffff` | pending | - | pending | - |
| RenderAlloc allocator | data | `0x7938880` | - | pending | - | pending | - |
| TimeScaleSystem::Update | hook | `0x19bb430` | `488bc448895818555657415441554156` | pending | - | pending | - |
| frame game time (pause gate) | data | `0x7a7fbd8` | - | pending | - | pending | - |

### Priority 3. Step 6-7 and removable: HUD, game-time checks, shadows, diagnostics, gate extras

| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found |
|---|---|---|---|---|---|---|---|
| SubmitScene (shadows) | hook | `0x19206a0` | `44884c242044894424184889542410555356574154` | pending | - | pending | - |
| FitCascades | hook | `0x170fbc0` | `488bc44c8948204c894018488950104889480855535657` | pending | - | pending | - |
| RenderActor | hook | `0x17991a0` | `488bc4448840184889501048894808` | pending | - | pending | - |
| CopyFinal (diag) | hook | `0x186cc00` | `40534883ec30f6817c4e010020` | pending | - | pending | - |
| readback init (diag) | call | `0x186c670` | `48895c24084889742410` | pending | - | pending | - |
| DrawOff | call | `0x191afb0` | `8b415ca820752983c82089415c` | pending | - | pending | - |
| DrawOn | call | `0x191b880` | `8b415ca820742983e0df89415c` | pending | - | pending | - |
| frame allocate (diag) | call | `0x1872860` | - | pending | - | pending | - |
| render-instance table | data | `0x7a436e8` | - | pending | - | pending | - |
| render-instance count | data | `0x7a43704` | - | pending | - | pending | - |
| HUD follower placement | hook | `0x73ab80` | `405553488dac24c8feffff4881ec38020000` | pending | - | pending | - |
| instance SetTransform | call | `0x191c0e0` | `40534883ec400f100233c0` | pending | - | pending | - |
| HUD follower vtable | vtable | `0x38929c8` | - | pending | - | pending | - |
| RTT stream update | hook | `0x2104400` | `48895c2410488974241848897c24205541544156` | pending | - | pending | - |
| RTT stream vtable | vtable | `0x4f87960` | - | pending | - | pending | - |
| Scaleform render command | hook | `0x1d2b6c0` | `48895c241048894c24085556574154415541564157` | pending | - | pending | - |
| marker projection XY | hook | `0x1f10ad0` | `48895c24084889742410574883ec4048` | pending | - | pending | - |
| marker projection | hook | `0x1f10b60` | `48895c2408574883ec40488bfa0f2974` | pending | - | pending | - |
| HUD second movie | data | `0x7be3e00` | - | pending | - | pending | - |
| HUD res factor | data | `0x7a4ae50` | - | pending | - | pending | - |
| HUD frame counter | data | `0x7a30b00` | - | pending | - | pending | - |
| HUD list frames | data | `0x7be5740` | - | pending | - | pending | - |
| HUD counts | data | `0x7be3f38` | - | pending | - | pending | - |
| HUD items | data | `0x7be3f40` | - | pending | - | pending | - |
| time: movss [rbx+18] | check | `0x19bb6ca` | `f30f114318` | pending | - | pending | - |
| time: movsd clock | check | `0x19bb6d3` | `f20f1105b5440c06` | pending | - | pending | - |
| time: test physics flag | check | `0x19bb6db` | `f683d007000001` | pending | - | pending | - |
| time: movss [rbx+1c] | check | `0x19bb70f` | `f30f11431c` | pending | - | pending | - |
| time: lea havok step | check | `0x19bb717` | `488d0d42ee6d04` | pending | - | pending | - |
| havok step setter | check | `0x1822670` | `f30f5949044c894108f30f1109c3` | pending | - | pending | - |
| clock scale | data | `0x7a7fb90` | - | pending | - | pending | - |
| clock 0x7a7fbf0 | data | `0x7a7fbf0` | - | pending | - | pending | - |
| havok step | data | `0x609a560` | - | pending | - | pending | - |
| follow-camera commit | gate | `0x1e1d600` | `4883ec38f20f` | pending | - | pending | - |
| clear input | gate | `0x1ce2f40` | `48895c241048896c2418` | pending | - | pending | - |
| input query A | gate | `0x1cdfa70` | `48895c240848897424` | pending | - | pending | - |
| input query B | gate | `0x1cdf840` | `48895c240848897424` | pending | - | pending | - |
| combat camera update | gate | `0x5c2630` | `4885d20f844809` | pending | - | pending | - |
| camera setter | gate | `0x191c590` | `0f1051304c8bc18b02` | pending | - | pending | - |
| FollowCamera vtable | vtable | `0x3871fd8` | - | pending | - | pending | - |
| CombatCamera vtable | vtable | `0x38720d0` | - | pending | - | pending | - |
| HeroLocal vtable | vtable | `0x38a93c8` | - | pending | - | pending | - |
| player vtable 0x38b2c98 | vtable | `0x38b2c98` | - | pending | - | pending | - |
| MoverStandard vtable | vtable | `0x4f70168` | - | pending | - | pending | - |
| component registry | data | `0x7a44320` | - | pending | - | pending | - |
| component registry 2 | data | `0x7a44340` | - | pending | - | pending | - |

## Next leg: exact steps

1. Wait for both installs: `appmanifest_2651280.acf` and `appmanifest_1817070.acf` with `StateFlags` 4 and no bytes left (`tools/xmatch/filled.py` on each exe must show every block non-zero).
2. Copy both exes to `B:/private/flat-to-3d-re/` and record their SHA-256.
3. Run `verify_rem.py` on Remastered. That's the step-0 answer. If the hash is Spidy's, all 31 + 4 + 7 + 4 checks pass by definition.
4. Run `survey.py` on Spider-Man 2. That answers the packer/Denuvo question for this build and lists any built-in multi-view, photo-mode or debug-camera paths.
5. Run `run_match.py` (priority 1 first: `run_match.py A B out.json 1`), then `mkmap.py`, and paste the table in here. Check every priority-1 match by hand (decompile both sides) before step 3 uses it.
