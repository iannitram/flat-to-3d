# Spider-Man 2 address map (Remastered → Spider-Man 2)

Every game-specific item on Spidy's GAME SCREEN 3D path ([spidy-core-map.md](spidy-core-map.md)), with its Remastered address and, once found, its Spider-Man 2 counterpart as a byte pattern. Plan step 3 (and steps 4–8 after it) works from this table.

This map is the output of Spider-Man 2's hook-point research step, Step 2r in [the plan](plan.md) (#283, ruling 52, 2026-11-12): every SM2 address is found on the installed game with our own tools before Step 3 hooks it, the exe's SHA-256, file version and Steam build go at the top once the step runs, and Spidy's, HEROVR's and other peers' addresses are a checklist to verify, not a source.

## The exe this map is for

| Field | Value |
|---|---|
| File | `Spider-Man2.exe` (Steam app 2651280) |
| SHA-256 | `c9b912eff96facb746a705ac8b0b83c29faa7bbe53d984a0c5eb45efd0b738c9` |
| File version | 2.810.0.0 |
| Steam build id | 24651585 |
| PE timestamp, SizeOfImage | 1786356053, 218533888 |

Every SM2 address below is an RVA in that file. A different SHA-256 means every row must be checked again.

## How to read the tables

- **Status.** `verified`: our own tool output shows the address does this job (the "How found" column says which output). `lead`: our own analysis points at it, but the output does not yet prove the job. `pending`: not started.
- **SM2 pattern.** Bytes from the SM2 address (for a `data` row, from the function the "How found" text names), with every rel32 and RIP displacement as `??`. Each one has exactly one hit in `.text` (`probe.py EXE count PATTERN`).
- **How found.** The tool, the method and the date. Rerun any row with `tools/xmatch/probe.py EXE dis|callers|pattern RVA`.
- **Spidy bytes at SM2 address.** A checklist column only (ruling 52): `agrees` when the bytes Spidy checks on Remastered are also at our SM2 address, `differs` when they are not. It is never a source for a row, and `differs` is expected where the build changed the code.
- Remastered (`Spider-Man.exe`, app 1817070) is not on the research machine yet, so no row comes from Ghidra Version Tracking. That pass runs later, once Remastered is brought in, as a cross-check of these rows.



<!-- BEGIN GENERATED -->
### Priority 1. Steps 1-3: camera, frame entry, view setup

| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found | Status | Spidy bytes at SM2 address |
|---|---|---|---|---|---|---|---|---|---|
| stock camera fn (calls submit) | call | `0x1646e10` | - | pending | - | pending | - | pending | - |
| camera submit call site | site | `0x164703f` | `e86c2a2500` | pending | - | pending | - | pending | - |
| camera submit return | site | `0x1647044` | - | pending | - | pending | - | pending | - |
| view descriptor submit | hook | `0x1899ab0` | `483bd1747c488bc1` | pending | - | pending | - | pending | - |
| lens ctor (12 args) | call | `0x187bb00` | `488bc44889580855` | pending | - | pending | - | pending | - |
| pose setter | call | `0x187ca10` | `40534883ec600f1002` | pending | - | pending | - | pending | - |
| frame function | call | `0x175a060` | - | `0x2afa730` | `488bc448894808555356574154415541564157488da8c8feffff` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): disassembly; calls offscreen render setup at 0x2afb395 and SubmitScene at 0x2afb45e (2026-11-16) | verified | - |
| frame -> offscreen call site | site | `0x175aaa9` | `e892571c00` | `0x2afb395` | `e8????????c78548010000ffffffff488d8d48010000e8????????` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): the call to offscreen render setup inside the frame function; returns to 0x2afb39a (2026-11-16) | verified | differs |
| offscreen render setup (placeEyes hook) | hook | `0x1920240` | `48895c24084889742410574883ec2040b701` | `0x2d01600` | `48895c2408574883ec2033db488d3d????????33c94533c0` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): loops the 8 offscreen slots and calls 0x2c65b50 (view, isLast), which calls SubmitScene (2026-11-16) | verified | differs |
| history shift | call | `0x18a13c0` | - | pending | - | pending | - | pending | - |
| history shift per-view copy | check | `0x189ee00` | `48895c2408574883ec20488bf941b830050000` | pending | - | pending | - | pending | - |
| view maintenance (ViewContextManager) | hook | `0x18a0bb0` | `48895c240848896c2410` | `0x2c9d4f0` | `48895c240848896c24104889742418574883ec508b0d????????` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): called from the frame function at 0x2afac29; updates every live pool view, then every offscreen slot (2026-11-16) | lead | agrees |
| create offscreen view | call | `0x18a09c0` | `4c8bdc49895b10` | pending | - | pending | - | pending | - |
| ViewContextManager global | data | `0x7a34dd0` | - | `0xc2ca220` | `48895c240848896c24104889742418574883ec508b0d????????` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): ViewContextManager init 0x2c9cfc0 (string) stores it; pool views at [mgr], stride 0x1e20; pattern is the referencing function at 0x2c9d4f0 (2026-11-16) | verified | - |
| view pool mask | data | `0x7a34dd8` | - | `0xc2ca228` | `48895c240848896c24104889742418574883ec508b0d????????` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): mgr+8, bt-tested per pool index; pattern is the referencing function at 0x2c9d4f0 (2026-11-16) | verified | - |
| offscreen slots (mgr+0x30) | data | `0x7a34e00` | - | `0xc2ca250` | `48895c240848896c24104889742418574883ec508b0d????????` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): mgr+0x30, 8 offscreen view pointers walked by index; pattern is the referencing function at 0x2c9d4f0 (2026-11-16) | verified | - |

### Priority 2. Steps 4-5: eye views, render jobs, buffers, memory, pause gate

| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found | Status | Spidy bytes at SM2 address |
|---|---|---|---|---|---|---|---|---|---|
| view update (exposure copy) | hook | `0x189bd30` | `48895c240848897c2410` | pending | - | pending | - | pending | - |
| occlusion builder | call | `0x189fed0` | `48895c24104889742418574883ec40488bf94883b9a01b000000` | pending | - | pending | - | pending | - |
| texture wrapper vtable | vtable | `0x4e56c80` | - | pending | - | pending | - | pending | - |
| copy view into render job | hook | `0x19223e0` | `48895c2420555657` | pending | - | pending | - | pending | - |
| render job begin | hook | `0x189e310` | `48895c2408574883ec30` | pending | - | pending | - | pending | - |
| render job end | hook | `0x189e3a0` | `48895c2408574883ec30` | pending | - | pending | - | pending | - |
| buffer-set init | hook | `0x1873470` | `40534883ec30448b4a04` | pending | - | pending | - | pending | - |
| buffer-set init return gate | site | `0x186c325` | - | pending | - | pending | - | pending | - |
| buffer-set sizes | hook | `0x1874f40` | `48895c2408488974241048897c2418554154` | pending | - | pending | - | pending | - |
| target kind test | check | `0x1875270` | `440fb6d241b1010fbae20e` | pending | - | pending | - | pending | - |
| kind-mask table | data | `0x3d11b18` | `ff000000ff0000002000000028000000` | pending | - | pending | - | pending | - |
| TemporalAa | check | `0x1842e10` | `488bc455488da8f8feffff` | `0x2c2dc60` | `488bc4488958085556574154415541564157488da8e8feffff4881ece0010000c5f82970b8c5f82978a8` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): render command registration table (0x2c83840): name TemporalAa, id 0xb1, this handler (2026-11-16) | verified | differs |
| display setup | hook | `0x1920310` | `4d85c00f847f030000` | `0x2d016d0` | `4d85c00f84????????488954241056` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): called from SubmitScene; enqueues cmd 0xaf (PrepareSrgbOverlay) on view+0x1850 and 0xb1 (TemporalAa) on view+0x18d0 (2026-11-16) | verified | differs |
| enqueue render command | call | `0x187e130` | `48895c240848896c2410` | `0x2c77810` | `48895c240848896c2410488974241857415641574883ec200fb7410c` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): (cmdlist, id, r8, r9); writes into the command table 0xc2c4ff8 that the registration table fills (2026-11-16) | verified | agrees |
| tone map | hook | `0x1846c20` | `488bc448895010` | `0x2c37140` | `488bc44889580855565741544155415641574881ec90000000` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): registration table: name ToneMap, id 0xac, this handler; not yet shown to be the hooked function (2026-11-16) | lead | differs |
| adapted-lum read | check | `0x189c710` | `488d81f0160000488b10` | `0x2c98350` | `488bc4488958084889681048897018488978204154415641574883ec70418bd8` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): the one function that references the string Adapted Lum (2026-11-16) | lead | differs |
| RenderAlloc ring create | hook | `0x1872d90` | `48895c240848896c2410488974241857415641574883ec30` | `0x2c6c110` | `48895c240848896c2410488974241857415641574883ec30488bd9e8????????84c0` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): references RenderAlloc; called once from render init 0x2c85720 with the allocator global (2026-11-16) | lead | agrees |
| ring create call site | site | `0x188a195` | `488d0de4e60a06e8ef8bfeff` | `0x2c857b8` | `e8????????84c074ad448b0d????????` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): the call to ring create in render init 0x2c85720 (2026-11-16) | lead | differs |
| RenderAlloc per-frame rollover | hook | `0x1872b90` | `48895c2408574883ec3083792800488bd98b411cc6414200` | `0x2c6bf10` | `48895c2408574883ec3083791800` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): references RenderAlloc; called once per frame from 0x2c77a80 with the allocator global (2026-11-16) | lead | differs |
| rollover call site | site | `0x187e623` | `488d0d56a20b06e86145ffff` | `0x2c77d24` | `e8????????488d0d????????e8????????488d0d????????e8????????c7433801000000` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): the call to the rollover in 0x2c77a80 (2026-11-16) | lead | differs |
| RenderAlloc allocator | data | `0x7938880` | - | `0xc15d780` | `48895c240848896c2410488974241857415641574883ec504c8bf2` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): lea rcx before both RenderAlloc calls (0x2c77d1d, 0x2c857b1); pattern is the referencing function at 0x2c77a80 (2026-11-16) | verified | - |
| TimeScaleSystem::Update | hook | `0x19bb430` | `488bc448895818555657415441554156` | `0x2db5e90` | `488bc44889581855565741544155415641574881ecd0000000c5f82970b8488db968050000` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): the function that references the string TimeScaleSystem Update; not a TimeScaleSystem vtable slot (2026-11-16) | lead | agrees |
| frame game time (pause gate) | data | `0x7a7fbd8` | - | pending | - | pending | - | pending | - |

### Priority 3. Step 6-7 and removable: HUD, game-time checks, shadows, diagnostics, gate extras

| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found | Status | Spidy bytes at SM2 address |
|---|---|---|---|---|---|---|---|---|---|
| SubmitScene (shadows) | hook | `0x19206a0` | `44884c242044894424184889542410555356574154` | `0x2d01b30` | `488bc4488958104889701848897820554154415541564157488da8c8fdffff` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): references the strings Submit Scene %s and Unnamed ViewContext; called from the frame function and from 0x2c65b50 (2026-11-16) | verified | differs |
| FitCascades | hook | `0x170fbc0` | `488bc44c8948204c894018488950104889480855535657` | pending | - | pending | - | pending | - |
| RenderActor | hook | `0x17991a0` | `488bc4448840184889501048894808` | pending | - | pending | - | pending | - |
| CopyFinal (diag) | hook | `0x186cc00` | `40534883ec30f6817c4e010020` | `0x2c656d0` | `40534883ec30f681e4da010020488bd9` | own analysis | tools/xmatch/probe.py (dis, callers, pattern): tests view flag 0x20 at view+0x1dae4 and enqueues on view+0x18d0; called right after SubmitScene in 0x2c65b50 (2026-11-16) | lead | differs |
| readback init (diag) | call | `0x186c670` | `48895c24084889742410` | pending | - | pending | - | pending | - |
| DrawOff | call | `0x191afb0` | `8b415ca820752983c82089415c` | pending | - | pending | - | pending | - |
| DrawOn | call | `0x191b880` | `8b415ca820742983e0df89415c` | pending | - | pending | - | pending | - |
| frame allocate (diag) | call | `0x1872860` | - | pending | - | pending | - | pending | - |
| render-instance table | data | `0x7a436e8` | - | pending | - | pending | - | pending | - |
| render-instance count | data | `0x7a43704` | - | pending | - | pending | - | pending | - |
| HUD follower placement | hook | `0x73ab80` | `405553488dac24c8feffff4881ec38020000` | pending | - | pending | - | pending | - |
| instance SetTransform | call | `0x191c0e0` | `40534883ec400f100233c0` | pending | - | pending | - | pending | - |
| HUD follower vtable | vtable | `0x38929c8` | - | pending | - | pending | - | pending | - |
| RTT stream update | hook | `0x2104400` | `48895c2410488974241848897c24205541544156` | pending | - | pending | - | pending | - |
| RTT stream vtable | vtable | `0x4f87960` | - | pending | - | pending | - | pending | - |
| Scaleform render command | hook | `0x1d2b6c0` | `48895c241048894c24085556574154415541564157` | pending | - | pending | - | pending | - |
| marker projection XY | hook | `0x1f10ad0` | `48895c24084889742410574883ec4048` | pending | - | pending | - | pending | - |
| marker projection | hook | `0x1f10b60` | `48895c2408574883ec40488bfa0f2974` | pending | - | pending | - | pending | - |
| HUD second movie | data | `0x7be3e00` | - | pending | - | pending | - | pending | - |
| HUD res factor | data | `0x7a4ae50` | - | pending | - | pending | - | pending | - |
| HUD frame counter | data | `0x7a30b00` | - | pending | - | pending | - | pending | - |
| HUD list frames | data | `0x7be5740` | - | pending | - | pending | - | pending | - |
| HUD counts | data | `0x7be3f38` | - | pending | - | pending | - | pending | - |
| HUD items | data | `0x7be3f40` | - | pending | - | pending | - | pending | - |
| time: movss [rbx+18] | check | `0x19bb6ca` | `f30f114318` | pending | - | pending | - | pending | - |
| time: movsd clock | check | `0x19bb6d3` | `f20f1105b5440c06` | pending | - | pending | - | pending | - |
| time: test physics flag | check | `0x19bb6db` | `f683d007000001` | pending | - | pending | - | pending | - |
| time: movss [rbx+1c] | check | `0x19bb70f` | `f30f11431c` | pending | - | pending | - | pending | - |
| time: lea havok step | check | `0x19bb717` | `488d0d42ee6d04` | pending | - | pending | - | pending | - |
| havok step setter | check | `0x1822670` | `f30f5949044c894108f30f1109c3` | pending | - | pending | - | pending | - |
| clock scale | data | `0x7a7fb90` | - | pending | - | pending | - | pending | - |
| clock 0x7a7fbf0 | data | `0x7a7fbf0` | - | pending | - | pending | - | pending | - |
| havok step | data | `0x609a560` | - | pending | - | pending | - | pending | - |
| follow-camera commit | gate | `0x1e1d600` | `4883ec38f20f` | pending | - | pending | - | pending | - |
| clear input | gate | `0x1ce2f40` | `48895c241048896c2418` | pending | - | pending | - | pending | - |
| input query A | gate | `0x1cdfa70` | `48895c240848897424` | pending | - | pending | - | pending | - |
| input query B | gate | `0x1cdf840` | `48895c240848897424` | pending | - | pending | - | pending | - |
| combat camera update | gate | `0x5c2630` | `4885d20f844809` | pending | - | pending | - | pending | - |
| camera setter | gate | `0x191c590` | `0f1051304c8bc18b02` | pending | - | pending | - | pending | - |
| FollowCamera vtable | vtable | `0x3871fd8` | - | pending | - | pending | - | pending | - |
| CombatCamera vtable | vtable | `0x38720d0` | - | pending | - | pending | - | pending | - |
| HeroLocal vtable | vtable | `0x38a93c8` | - | pending | - | pending | - | pending | - |
| player vtable 0x38b2c98 | vtable | `0x38b2c98` | - | pending | - | pending | - | pending | - |
| MoverStandard vtable | vtable | `0x4f70168` | - | `0x85d24d8` | `RTTI MoverStandard` | own analysis | tools/xmatch/vt.py: RTTI complete-object-locator walk (2026-11-16) | verified | - |
| component registry | data | `0x7a44320` | - | pending | - | pending | - | pending | - |
| component registry 2 | data | `0x7a44340` | - | pending | - | pending | - | pending | - |

### Checked against HEROVR's list

| HEROVR item | Our row | Check | Evidence |
|---|---|---|---|
| H1 Runtime Direct3D 12 and DXGI queue discovery | - | runtime | - |
| H2 512 MB render memory ring allocator | - | pending | - |
| H3 Scene submit (5th argument) | - | pending | - |
| H4 Display setup (5th argument) | - | pending | - |
| H5 Pose setter (3 arguments) | - | pending | - |
| H6 Game view kept (flag 32 never sent) | - | no-row | - |
| H7 Eye-job offset in copyJob | - | pending | - |
| H8 Buffer-size hook and output sizing | - | pending | - |
| H9 Tone hook: copies the eye's view bytes into the job's view copy | - | pending | - |
| H10 Temporal AA offscreen target kinds | - | pending | - |
| H11 Steam ticket guard (quit to menu) | - | off-path | - |

<!-- END GENERATED -->

### Checked against HEROVR's list

Checklist derived from peer [lecutis7of9/HEROVR](https://github.com/lecutis7of9/HEROVR/blob/82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859/BUILD-SM2.md) (GPL-3.0, commit `82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859`, `BUILD-SM2.md`). Per ruling 2026-10-31: learn only, keep our port. No addresses or code are copied (HEROVR's addresses come from the closed SM2VR mod). Every address in our port must be found by our own analysis tools (Ghidra Version Tracking and matcher, session Q12). For clean-eye findings (TAA/FXAA/CAS and RT settings), see [clean eyes note](../notes/sm2-clean-eyes.md).

The peer file has no numbered hook list, so this one numbers every hook its text documents. H1 to H11 are the items the update notes name first. H12 to H24 are the other hooks the file describes (input, ray, movement, camera fit, bind trace, actor and body hooks). Its section 8 items are documented as not hooked, so they are not in the list.

The list is a checklist for what our analysis must locate, by port step:
- #18 (Step 2: Get inside the game): H1 runtime queue discovery, H6 game view kept, H11 Steam ticket guard (needed to start and quit safely), H19 stereo start and maintain hooks
- #19 (Step 3: Move the camera): H5 pose setter
- #20 (Step 4: Two eyes): H7 eye-job offset in copyJob, H22 render-actor hook (hero hidden per view)
- #21 (Step 5: Same-frame pairs and clean images): H2 render memory ring, H3 scene submit, H4 display setup, H8 buffer-set sizes, H9 tone hook (eye view copy), H10 temporal AA target kinds, H20 render-target and table-bind trace hooks, H21 fit-cascades and IsActiveView hooks
- Step 6 (in the headset) or later, not in #18–#21: H12–H18, H23, H24

| HEROVR item | Our row | Check | Evidence |
|---|---|---|---|
| H1 Runtime D3D12/DXGI queue discovery | - | runtime | `render_probe` |
| H2 512 MB render memory ring allocator | RenderAlloc ring create | pending | pending Q12 analysis |
| H3 Scene submit (5th argument) | SubmitScene (shadows) | pending | pending Q12 analysis |
| H4 Display setup (5th argument) | display setup | pending | pending Q12 analysis |
| H5 Pose setter (3 arguments) | pose setter | pending | pending Q12 analysis |
| H6 Game view kept (flag 32 never sent) | - | pending | pending #18 |
| H7 Eye-job offset in copyJob | copy view into render job | pending | pending Q12 analysis |
| H8 Buffer-size hook and output sizing | buffer-set sizes | pending | pending Q12 analysis |
| H9 Tone hook: copies the eye's view bytes into the job's view copy | tone map | pending | pending Q12 analysis |
| H10 Temporal AA offscreen target kinds | TemporalAa | pending | pending Q12 analysis |
| H11 Steam ticket guard (quit to menu) | - | pending | pending #18 |
| H12 Input commit hook (input bridge, hook 0) | - | pending | pending Step 6 |
| H13 Input clear hook (input bridge, hook 1) | - | pending | pending Step 6 |
| H14 Digital query hook (input bridge, hook 2) | - | pending | pending Step 6 |
| H15 Analog query hook (input bridge, hook 3) | - | pending | pending Step 6 |
| H16 Combat camera update hook (hook 4) | - | pending | pending Step 6 |
| H17 Controller world-ray cast hook | - | pending | pending Step 6 |
| H18 Web-swing mover hook | - | pending | pending Step 6 |
| H19 Stereo start and end-of-maintain hooks | - | pending | pending #18 |
| H20 Render-target, viewport and table-bind trace hooks | - | pending | pending #21 |
| H21 Fit-cascades and IsActiveView hooks (far shadows) | FitCascades | pending | pending #21 |
| H22 Render-actor hook (hero hidden per view) | RenderActor | pending | pending #20 |
| H23 Body pose-writer and toModel hooks (analysis only) | - | pending | pending Step 6 |
| H24 Web-shooter muzzle hooks (wrist ball look) | - | pending | pending Step 6 |

### Peer cross-check (checklist only, ruling 52)

Checked on 2026-11-16 against the peer lists named in #336. None of these is a row's source, and no peer text is copied here.

- **HEROVR v0.0.81** (GPL-3.0): its view submit, SubmitScene and TemporalAa addresses **agree** with our `display setup`, `SubmitScene (shadows)` and `TemporalAa` rows. Its SubmitScene is not the Remastered address that the issue lists. Its lens ctor and pose setter are rows we have not found yet; they stay `pending` until our own tools find them. Which exe build HEROVR targets has not been checked against our SHA-256.
- **SpidyVR** newer than our pinned 0.3.8 (0.7.0 read, 0.8.7 out 2026-11-16) and **MilesVR** (a SpidyVR port to Miles Morales): not yet checked. They are a later checklist pass.

## Next leg: exact steps

1. Finish the rows still `pending` from the SM2 exe alone, in the priority order of #283: FitCascades and RenderActor, then the hooks, calls, globals and field offsets. Use RTTI (`vt.py`), string references and `probe.py`; the render command registration function (`0x2c83840`, name, id and handler for every command) is a good anchor for render rows.
2. Turn each `lead` into `verified` by showing its job in our own output (the caller chain, the field it writes, the string it names).
3. When Remastered is brought in: copy it to `$FLAT3D_RE_HOME/exe/sm-<version>/`, record its SHA-256, run `verify_rem.py`, then Ghidra Version Tracking from Remastered to SM2 as a cross-check of every row. A disagreement goes in the row; it never replaces our own finding without new output.
4. Check every priority-1 row by hand (decompile it) before Step 3 hooks it.
