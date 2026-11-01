# Peers

Projects doing parts of what this one does, approved by the owner on 2026-10-31 (the Project Manager session's question tool). The `peer:` lines are copied in `SPEC.md` `## Prior art`, which the weekly peer scan reads. What we take from them: adopt (port code, only where licenses fit), learn (the idea, written fresh) or drop-ours (use or contribute instead).

## What the 2026-10-31 search found

- **Nobody does the whole thing.** No project runs true stereo on a fixed 3D TV on Linux from Game Mode. The Linux, Decky, Start 3D and fixed-screen pieces are still ours to build.
- **Spider-Man 2 stereo exists on Windows:** HEROVR ported SpidyVR to Spider-Man 2. Its addresses come from a closed mod (SM2VR), so we learn from it and keep our own port and address map (ruling 2026-10-31: "Learn only, keep our port").
- **SpidyVR now runs under Proton + WiVRn** (its `docs/LINUX.md`), which helps Step 0 and the OpenXR worker.
- **Kaon** already manages UEVR under Proton on Linux: the reference for Zero Company and for the engine's Steam scan.
- **No verified report** of UEVR under Proton with WiVRn was found. Zero Company's Z2 will be one of the first.

## Approved peers

peer: IlyaMez/SpidyVR | GPL-3.0 | our upstream; v0.7.0 documents Proton + WiVRn wired (docs/LINUX.md, tools/spidy-wivrn.sh); watch releases, eye-pair and swapchain code
peer: lecutis7of9/HEROVR | GPL-3.0 | SpidyVR ported to Spider-Man 2 on Windows: render probe, eye hooks, FXAA/CAS on the eyes, RT off; learn only (ruling 2026-10-31), its addresses come from the closed SM2VR mod
peer: LorenDB/kaon | GPL-3.0 | Linux UEVR manager for Proton: Steam scan, UEVR + .NET install, launch options; reference for Zero Company Z2 and the Inject engine
peer: WiVRn/WiVRn | GPL-3.0 | our streamer; wired mode and the dashboard's adb automation are what Start 3D wraps
peer: praydog/UEVR | NOASSERTION | the Zero Company route; watch nightlies for UE 5.6 and 2D Screen Mode depth
peer: Blackskydk/FlatToDepth | MIT | Windows: Geo-11 per-eye output on a big OpenXR screen with a Steam scan; closest to the generic 3D TV (#9)
peer: artumino/VRScreenCap | MIT | Rust OpenXR SBS/Katanga screen viewer; reference for the fixed-screen layer and SBS viewer
peer: Raicuparta/rai-pal | GPL-3.0 | the model for game detection and the Inject flow

## Also seen (not scanned)

- nautymac/GeoGamingHub (none): Windows one-app stereo installer that picks Geo-11 / OGL3D / VK3D / DX12_3D per game; DX12 and Vulkan parts are binaries only, unverified
- bo3b/3Dmigoto (NOASSERTION): the DX11 stereo-fix base under Geo-11
- iprjkt/3DMigoto-Linux (NOASSERTION): 3DMigoto for Wine/Proton/DXVK; brand new, stereo not shown
- Supreeeme/xrizer (GPL-3.0): OpenVR-to-OpenXR shim WiVRn recommends for Proton; a dependency if a mod touches OpenVR
- yakuda-stack/yakuda-connect (GPL-3.0): desktop one-click WiVRn + xrizer launcher; SteamOS untested
- Spear0070/HeadStartVR (MIT): Windows UEVR launcher that waits for the headset before injecting; idea for Start 3D
- wayvr-org/wayvr (GPL-3.0): Linux desktop screens inside WiVRn/Monado; overlay patterns
- OmenDjinn/overstrike-linux (GPL-3.0): Linux Insomniac mod manager; mod-install conventions
- ThreeDeeJay/VKS3D (GPL-3.0): Vulkan-level stereo layer (from the 2026-10-18 research); Linux builds unreported
