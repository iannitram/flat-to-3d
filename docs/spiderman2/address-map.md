# Spider-Man 2 address map (Remastered → Spider-Man 2)

Every game-specific item on Spidy's GAME SCREEN 3D path ([spidy-core-map.md](spidy-core-map.md)), with its Remastered address and, once found, its Spider-Man 2 counterpart as a byte pattern. Plan step 3 (and steps 4–8 after it) works from this table.

This map is the output of Spider-Man 2's hook-point research step, Step 2r in [the plan](plan.md) (#283, ruling 52, 2026-11-12): every SM2 address is found on the installed game with our own tools before Step 3 hooks it, the exe's SHA-256, file version and Steam build go at the top once the step runs, and Spidy's, HEROVR's and other peers' addresses are a checklist to verify, not a source.


<!-- BEGIN GENERATED -->
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

## Next leg: exact steps

1. Wait for both installs: `appmanifest_2651280.acf` and `appmanifest_1817070.acf` with `StateFlags` 4 and no bytes left (`tools/xmatch/filled.py` on each exe must show every block non-zero).
2. Copy both exes to `$FLAT3D_RE_HOME/exe/<game>-<version>/` and record their SHA-256.
3. Run `verify_rem.py` on Remastered. That's the step-0 answer. If the hash is Spidy's, all 31 + 4 + 7 + 4 checks pass by definition.
4. Run `survey.py` on Spider-Man 2. That answers the packer/Denuvo question for this build and lists any built-in multi-view, photo-mode or debug-camera paths.
5. Run `run_match.py` (priority 1 first: `run_match.py A B out.json 1`), then `mkmap.py`, and paste the table in here. Check every priority-1 match by hand (decompile both sides) before step 3 uses it.
