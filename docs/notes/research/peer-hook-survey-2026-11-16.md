# Peer VR-mod hook survey (2026-11-16)

Leads for #283 and #285-#289 (ruling 52). Every row is a **peer lead**: a checklist item to verify on the installed game, never a source of addresses. Nothing here was copied; rows hold identifiers, RVAs and short factual descriptions only. Source was read through `gh api` and shallow clones in `/tmp/claude-1000/flat3d-peers/`; no binary was downloaded or run, and no account or post was used.

Search coverage: GitHub (`gh search repos`, tree and file reads), the VRto3D repo, the Helix Mod blog search, and general web search. Code search on `gh` returned nothing, so absence of a hit is weak evidence. Nexus Mods, GitLab, Codeberg, the Flat2VR Discord mirrors, r/flatscreen2vr and the 3DJ forums were not reachable or returned nothing useful through the search tool; they stay open.

Format of the tables: `Agrees / differs / new` compares a lead with our merged map where one exists (only Zero Company so far, `docs/hooks/2075800-star-wars-zero-company.md`, main 608a8bd).

## Summary of the best leads

| Game | Best leads |
|---|---|
| Zero Company | joeyhodge/UEVR `SWZeroCompanyBinary.hpp` (per-build RVAs and byte checks); oneup03/UU3D (the same code with a screen-3D output, no headset) |
| ACE COMBAT 7 | Beren5556/AC7VRDLSS-Downloads (cloud-material stereo fault, camera and aircraft-visibility helpers; native sources private); keton/ace-combat-uevr (known) |
| Control | nothing new beyond Geo-11 and Losti's fix |
| 007 First Light | RealChrizzl/hitman-vr-foveation-fix and piepieonline/h3_vr_foveation (Hitman 3's VR device layout and signatures, Glacier-shared); OrfeasZ/ZKntSDK (render and camera hooks on the 007 exe) |
| Spider-Man 2 | HEROVR (latest v0.0.81): its SM2 RVAs; IlyaMez/SpidyVR now v0.8.7 |
| Alan Wake 2 | no public source; NoMoreFlat's AW2 page and VRto3D's `AlanWake2.exe` profile only |
| Planet of Lana | nothing new |

## 1. STAR WARS Zero Company (UE 5.6.1, app 2075800) for #285

Compared with `docs/hooks/2075800-star-wars-zero-company.md` (our build: CL 201250, SHA-256 `60831de4...`). The fork's newest entry is CL 199049, so every address differs by build; what matters is which hook sites the fork needs.

| Peer lead | Source URL | Licence | Build / last activity | Hook point (identifier) | Vs our map |
|---|---|---|---|---|---|
| joeyhodge/UEVR, branch `ue57performance`, file `src/mods/vr/SWZeroCompanyBinary.hpp`, `docs/sw-zero-company-ue561.md` | https://github.com/joeyhodge/UEVR | NOASSERTION (UEVR terms; no copying) | CL 196320, 196985, 197649, 199049; pushed 2026-11-16; no CL 201250 entry | `InitializeHMDDevice` with the fake stereo device constructor inlined: stereo vtable (CL 199049 `0xD8D33E0`), render-target-manager vtable (`0xD8D3360`), assignment site | agrees (same functions; addresses differ, map already tabulates) |
| same | same | same | same | stereo vtable slots 1, 4, 8, 9, 11, 12, 14 (199049: `0x4189F20`, `0x797C040`, `0x797C070`, `0x797C0C0`, `0x797C2C0`, `0x797C710`, `0x797CA30`) | agrees (map has slots 1, 3, 4, 8, 9, 11, 12, 14 in 201250) |
| same | same | same | same | stereo vtable slots 0, 5 and 16 (199049: `0x797D020`, `0x797C050`, `0x4183010`) | new (not in map; slot 0 and 5 sit beside `GetDesiredNumberOfViews`; names unknown, look them up in the PDB) |
| same | same | same | same | render-target-manager vtable slots 0, 2 and 8 (199049: `0x797CE10`, `0x797D010`, `0x797CE40`); `RenderTexture_RenderThread` is stereo slot 14 | new for the manager slots; slot 14 agrees |
| same | same | same | same | `FSlateRHIRenderer::DrawWindow_RenderThread` (199049 `0x627CBE0`) plus the argument-byte checks at `+0x45` | agrees (map checks the `+0x2D` and `+0x45` bytes and finds them present) |
| same | same | same | same | `RegisterExternalTexture` entry (199049 `0x5783A10`) hooked from Slate output; the viewport's force-separate-target accessor (`0x7718180`) and the desktop-copy clamp (`0x5F9E230`) | new (map lists neither) |
| same | same | same | same | Nanite hologram path: dispatch (`0x5A220E0`) and caller `RenderHologramCaptureInner` (return `0x5542D94`), checks of `VisibleClustersSWHW` (+0x38) and `ShadingMask` (+0x60) | new; Native Fix only, probably not needed for Synchronized Sequential |
| same | same | same | same | `FRHITexture` vtables (199049 `0xD21F9B8`, back buffer `0xD2267F8`), slot 5 native D3D12 resource getter (`0x5F9ECA0`) | new; Z2b needs the native resource to read the two eye targets |
| oneup03/UU3D (branch `flat3d-afw`, fork of the above) | https://github.com/oneup03/UU3D | custom non-commercial terms (`LICENSE-UU3D.md`); no copying | nightly 00054 on 2026-11-16; repo pushed 2026-11-15; Zero Company builds 197649 and 199049 in its CHANGELOG | Stereo-3D output to a screen with no headset (SBS, TAB, anaglyph, 3D TV); same Zero Company hooks as the fork | new peer. It is what Flat to 3D does on Windows: a direct lead for #33 (2D Screen Mode depth) and for the generic 3D TV path |
| Dekita/SWZeroCompanySDK | https://github.com/Dekita/SWZeroCompanySDK | none stated | pushed 2026-09-09 | UE 5.6 class declarations rebuilt from the game's UE4SS dump; first-party plugins `BitReactorCore`, `BitReactorGame` | new; for naming game classes if the HUD draw needs a game class. No engine stereo classes found |
| klukule/SWZCModKit, arctco/zcom-mod-manager | https://github.com/klukule/SWZCModKit, https://github.com/arctco/zcom-mod-manager | none stated; GPL-3.0 | both 2026-09-09 | mod loading (IoStore, UE4SS), not stereo | new; not hook leads |
| Noniv/UEVR | https://github.com/Noniv/UEVR | NOASSERTION | 2026-09-21 | a Mono rendering method (one view shown to both eyes) | not a lead for stereo; a fallback idea |
| praydog/UEVR issues #422, #389 | https://github.com/praydog/UEVR/issues/422, https://github.com/praydog/UEVR/issues/389 | n/a | open; #422 2026-06-06 | doubled stereo image on UE 5.5/5.6 (zero-based eye index misclassified as a full pass); right-eye tint on 5.6 | new for the map's `CalculateStereoViewOffset` (eye index handling) check |

Not found: any public UEVR profile for Zero Company, or any record of a build newer than CL 199049 in the fork. The `Dekita` and `klukule` repos have no licence file, so read only.

## 2. ACE COMBAT 7 (UE 4.18, app 502500) for #286

| Peer lead | Source URL | Licence | Build / last activity | Hook point | Vs notes |
|---|---|---|---|---|---|
| Beren5556/AC7VRDLSS-Downloads | https://github.com/Beren5556/AC7VRDLSS-Downloads | MIT (profile); native and Python sources private; bundled runtimes keep their own licences | UEVR nightly 01143, profile 0.8.6, pushed 2026-10-10 | D3D11 Native Stereo; a cloud material that is not stereo-projected (the profile removes its contribution: "cloud mode A"); camera selection with a forward offset; aircraft geometry toggle; plugin `AC7_ResearchTools.dll` and script `AC7_Research.lua` (not readable) | new peer. It names three concrete faults to check on `Nimbus` rendering: clouds, camera 1, local aircraft |
| keton/ace-combat-uevr | https://github.com/keton/ace-combat-uevr | MIT | last push 2024-03-28 | SDK headers for `NimbusPlayerCameraManager`, `NimbusPlayerController`, `NimbusCheatManager`, `AcePlayerPawn`, `CameraViewComponent`, `Mission` | known peer; nothing newer from it |
| UEVR upstream, profile folder `Ace7Game` | https://github.com/praydog/UEVR | NOASSERTION | nightly 01143 (2026-08-30) | the profile folder name, `Ace7Game`, equals the exe name | agrees with the notes |
| oneup03/UU3D | https://github.com/oneup03/UU3D | custom non-commercial | 2026-11-15 | handles UE4 D3D11 (README notes on UE4 windowed fullscreen and float projection paths); no ACE7 entry in its docs | new; a route for an on-screen 3D output on UE 4.x |

Not found: any source naming the fake stereo device functions or RVAs for the 4.18 exe.

## 3. Control (Northlight, DX11, app 870780) for #287

| Peer lead | Source URL | Licence | Notes |
|---|---|---|---|
| Geo-11, Losti and Masterotaku's fix | already in the notes | n/a | known |
| VRto3D profile list | https://github.com/oneup03/VRto3D (`vrto3d_profiles/`) | LGPL-3.0 | no Control profile; its `AlanWake2.exe` and `Spider-Man.exe` profiles are display settings, not hooks |

Nothing new: no Control VR mod, no UEVR use (not Unreal), and no further Helix Mod post turned up (search "Control Remedy" on helixmod.blogspot.com returned none).

## 4. 007 First Light (Glacier, app 3768760) for #288

Glacier 2 is shared with Hitman 3, so these are leads for the pattern shape; every address must be re-found on the 007 exe (`ZKntSDK` targets it).

| Peer lead | Source URL | Licence | Build / last activity | Hook point | Vs notes |
|---|---|---|---|---|---|
| RealChrizzl/hitman-vr-foveation-fix (`docs/HOW-IT-WORKS.md`, `docs/UPDATING.md`) | https://github.com/RealChrizzl/hitman-vr-foveation-fix | MIT | Hitman WoA 3.270.1 and 3.290.1, v1.6.2; pushed 2026-11-07 | the VR device pointer is found through a RIP-relative global in a short locator signature; the WNO flag byte at device `+0x31B` (3.290.1 OpenVR field `0x8E0` instead of `0x950`); logical view count 4 against two physical eyes; masks at device `+0x4C0` and `+0x4C4`, scale at `+0x490` to `+0x49C`; refraction depth copy that multiplies its draw range by the view count; Oculus (`LibOVR`) and OpenVR backends, no OpenXR; also a Linux `Linux-HitmanVRFoveationFix.py` (experimental, Proton, untested on new builds) | new. A map of Hitman 3's `ZRenderVRDevice` layout. Our notes say the 007 exe has no OpenVR code, so the layout is a lead only for the dummy device's fields |
| piepieonline/h3_vr_foveation | https://github.com/piepieonline/h3_vr_foveation | MIT | tracks the project above at commit `672eae4`; pushed 2026-11-07 | a ZHMModSDK mod, `src/Layout.h` holds the signatures and device layout, `src/CodePatcher.cpp`, `src/RefractionFix.cpp` | new; proves Glacier mods can hook the VR path through ZHMModSDK |
| piepieonline/H3-VR-Redux | https://github.com/piepieonline/H3-VR-Redux | none stated | last push 2024-12-26 | VR tweaks for Hitman 3 | new; stale |
| OrfeasZ/ZKntSDK | https://github.com/OrfeasZ/ZKntSDK | other (see its `LICENSE`) | pushed 2026-10-08 | pattern hooks `UpdateStreamlineConstants(SRenderCameraData const&, uint32_t)` (camera position, up, right, forward and view-to-clip fields), `SPassExecution_ExecutePass`, globals `RenderManager`, `CameraManagerMain`, D3D12 proxy in `DirectInputProxy/Src/Rendering/` | known peer; these hook names are new to our notes: they are the 007 camera and pass entries, a better place than the VR device to inject two views |
| OrfeasZ/ZHMModSDK | https://github.com/OrfeasZ/ZHMModSDK | NOASSERTION (see its `LICENSE`) | pushed 2026-11-16; branch `master` only (no 007 branch) | no `ZRenderVRDevice` type; has `ZRenderManager`, `ZRenderGraphNodeCamera`, `ZCameraEntity` | new for Hitman 3 only |
| OrfeasZ/ZHMTools (KNT folder) | https://github.com/OrfeasZ/ZHMTools | LGPL-3.0 | pushed 2026-10-08 | enums `EVRRenderDeviceType` (dummy, Oculus, PSVR, OpenVR) and `ERenderVRTrackingID` | known (in the notes) |
| glacier-modding/Bond-Hashes, thepeacockproject/Octopussy | https://github.com/glacier-modding/Bond-Hashes, https://github.com/thepeacockproject/Octopussy | none stated | 2026-11-13, 2026-06-21 | resource hashes and a server emulator | not hook leads |

Not found: any mod that turns on Glacier's dummy VR device or a `stereo3d` mode in 007 First Light. The search results for "007 First Light mod" are mostly trainer spam, so ignore them.

## 5. Marvel's Spider-Man 2 (Insomniac, app 2651280) for #283

Compared with `docs/spiderman2/address-map.md` (all SM2 addresses are still pending).

| Peer lead | Source URL | Licence | Build / last activity | Hook point (SM2 RVA, peer's naming) | Note |
|---|---|---|---|---|---|
| lecutis7of9/HEROVR (`src/stereo_probe.cpp`, `BUILD-SM2.md`) | https://github.com/lecutis7of9/HEROVR | GPL-3.0 | `Spider-Man2.exe` 2.810.0.0, v0.0.81 (2026-11-12) | lens constructor `0x2c75560` (peer-marked confirmed), pose setter `0x2c76310` (third argument passed 0), view submit `0x2d016d0` (fifth argument), `SubmitScene` `0x2d01b30` (fifth argument and full 64-bit fourth), `TemporalAa` `0x2c2dc60` (no arguments), world-light fill `0x2c67f40`, cascade fit `0x2a8dc80`, main camera slot `base+0xc2ca220`, view table `base+0xc2ca250` and `base+0xc2d2d08`, view-copy function `0x2c9b8e0` | known peer; these are the RVAs it holds today, to check against our Step 2r rows (lens ctor, pose setter, view descriptor submit, TemporalAa target kinds) |
| IlyaMez/SpidyVR | https://github.com/IlyaMez/SpidyVR | GPL-3.0 | v0.8.7 (2026-11-16); `submitScene` hooks `19206a0` and `170fbc0` in Remastered | our pinned v0.3.8 and read v0.7.0 are behind: v0.8.7 exists | known; re-diff `stereo_probe.cpp` against v0.7.0 |
| howesjamie5-code/MilesVR | https://github.com/howesjamie5-code/MilesVR | GPL-3.0 | 2026-11-11 | SpidyVR port to Miles Morales (same Remastered engine build family) | new; same engine as the Remastered rows, a second data point for matching |
| tyronmiller55-spec/SpidyVR-NativeAnimation, SpidyVR-Upstream-Features | https://github.com/tyronmiller55-spec/SpidyVR-NativeAnimation | GPL-3.0 | 2026-11-14 | player-side feature layers, no new hooks | not leads |

## 6. Alan Wake 2 (Northlight, DX12, Epic) for #289

| Peer lead | Source URL | Licence | Notes |
|---|---|---|---|
| NoMoreFlat (Mutar), AW2 page | https://oneup03.github.io/VRto3D/wiki/NoMoreFlat | closed (Patreon package `aw2vr-openxr`) | known; says AW2 uses Alternate Eye Rendering. Docs only |
| Helix Mod post "Alan Wake 2 [NoMoreFlat + VRto3D/WWVR]" (2025-04-28, OneUp03) | https://helixmod.blogspot.com/2025/04/alan-wake-2-nomoreflat-vrto3d.html | n/a | the same mod, with ray tracing reported working; no hooks named |
| oneup03/VRto3D profile `AlanWake2.exe_config.json` | https://github.com/oneup03/VRto3D | LGPL-3.0 | display settings only |
| praydog/alan-wake-2-mod-loader | https://github.com/praydog/alan-wake-2-mod-loader | MIT | loose-file loading, 2023-11; no stereo code |

Not found: public source of any AW2 VR or 3D mod, nothing on `rend::View` or `renderWorldInternal`, and no Luke Ross AW2 code (his mods stay closed; the web search found no AW2 release at all).

## 7. Planet of Lana (after 1.0)

Nothing new beyond the Helix Mod fix by cicicleta already in `games_group_a.md`.

## What to do with this

1. Zero Company (#285, merged map): add the "new" rows above to the next pass of the map only if Z2b is needed; the stereo vtable slots 0, 5 and 16 and the `FRHITexture` native-resource getter are the cheapest.
2. For #33 and the 3D TV: read UU3D's README (stereo outputs) before building the fixed-screen layer; its terms forbid copying.
3. 007 (#288): use the Hitman 3 device-locator shape and `ZKntSDK`'s `UpdateStreamlineConstants`/`ExecutePass` names as search targets; our notes' finding (no OpenVR code in the exe) stands.
4. Spider-Man 2 (#283): diff HEROVR v0.0.81's SM2 RVAs against our Step 2r rows.
