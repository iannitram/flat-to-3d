# Marvel's Spider-Man 2: clean eye views, anti-aliasing, and render settings for Step 5 (issue #21)

Notes for port Step 5 (issue #21: same-frame pairs and clean images; [SPEC.md](../../SPEC.md#m6) M6).
Research date: 2026-11-12.

These findings come from studying public documentation and development logs of [lecutis7of9/HEROVR](https://github.com/lecutis7of9/HEROVR/blob/82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859/BUILD-SM2.md) (GPL-3.0, pinned commit `82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859`, specifically `BUILD-SM2.md`).

Standing project rule ([DECISIONS.md](DECISIONS.md) ruling 2026-10-31; R41): learn only, keep our port. Copy no code, copy no addresses (HEROVR's addresses come from the closed SM2VR mod). All addresses for our port must be discovered independently by our own analysis tools ([address map](../spiderman2/address-map.md)).

---

## 1. Summary of findings from peer

In Marvel's Spider-Man 2 (PC build 2.810.0.0), rendering secondary off-screen eye views for stereoscopic 3D introduces specific visual artifacts that differ from Spider-Man Remastered:

1. **The game's native TAA/DLAA skips off-screen views:** The engine's built-in anti-aliasing passes (DLAA, DLSS, TAA) and sub-pixel camera jitter run only for the primary flat monitor view. Secondary off-screen render views (the stereo eye cameras) do not receive temporal anti-aliasing.
2. **Aliasing and shimmer in the headset:** Without anti-aliasing, fine geometric details (building facades, window frames, roof edges, fences) alias into high-frequency moiré and shimmer aggressively during head or camera motion.
3. **Mod-side post-process anti-aliasing (FXAA 3.11 + CAS):** HEROVR worked around the lack of native TAA by inserting screen-space post-processing during the copy from render targets into OpenXR swapchain images:
   - **FXAA 3.11** (Quality Preset 29: 12 search steps, subpixel 0.5, edge threshold 0.125/0.0625) runs on edge luminance with low GPU overhead (~0.33 ms mean per pair).
   - **Contrast-Adaptive Sharpening (CAS)** runs directly after FXAA to restore texture clarity softened by FXAA without reintroducing high-frequency edge ringing.
4. **Ray tracing instability and window corruption:** Ray-traced features (reflections, ambient occlusion, shadows, interiors, ray reconstruction) are tied to the main viewport. In stereo views, RT Interiors causes opaque or flat window surfaces, and RT reflections corrupt off-axis views. HEROVR forces all ray tracing off before launch and restores settings on exit.
5. **Per-eye shadow cascades:** Each eye view fits its own shadow cascades to its frustum; attempting to share cascades causes crashes.
6. **Exposure and tone map matching:** To prevent retinal rivalry, the tone mapping pass must copy adapted luminance and exposure state from the active view descriptor into each eye job.

---

## 2. Input to port Step 5 (issue #21)

Spec M6 requires:
> EACH SHOWN PAIR SHALL come from the same frame, with matching brightness and its own anti-aliasing history; ray-traced reflections are off for the eyes and the upscaler is bypassed (port Step 5).

Applying the peer findings to our Linux/Proton port architecture for Step 5:

### Anti-Aliasing Strategy
- **Do not attempt to hook internal DLAA/TAA jitter for off-screen views initially:** Spider-Man 2's internal temporal pass expects specific motion vectors and jitter offsets that are omitted on off-screen views. Reverse-engineering those internal structures is high risk and fragile across game updates.
- **Implement mod-side FXAA 3.11 + CAS in the staging pass:** When copying completed eye render targets to the OpenXR swapchain, run a compute or pixel shader pass with FXAA 3.11 followed by CAS.
- **Draw overlays after AA:** Any HUD or stereo screen framing must be composited after the FXAA+CAS pass so text and lines remain sharp.

### Ray Tracing Configuration
- **Force RT off in the Proton prefix registry:** Ray tracing must be confirmed disabled prior to launching the game under Proton:
  - Registry key: `HKCU\Software\Insomniac Games\Marvel's Spider-Man 2\Graphics`
  - Values to clear: `RTReflections = 0`, `RTInteriors = 0`, `RTShadows = 0`, `RTAO = 0`.
- This ensures stability, avoids opaque window rendering, and respects the compute budget of integrated graphics on SteamOS.

### Upscaler Bypass
- **Native eye resolution:** Render eye targets at full buffer dimensions; bypass in-engine DLSS/FSR upscaling passes on off-screen targets to prevent scaling artifacts and mismatch between eye buffers.

### Brightness & Tone Consistency
- **Replicate adapted luminance:** The tone map hook in Step 5 must synchronize the exposure state (adapted luminance) across both eye views to prevent luminance flicker or eye rivalry.

### Frame Pairing & Timing
- **GPU marker fences:** Tag each eye job with the game frame counter; only present a stereo frame to OpenXR when both left and right eye jobs from the same frame number have completed.

---

## 3. Checklist for #21 implementation

- [ ] Add registry check in launcher/wrapper to ensure ray tracing is disabled in the Proton prefix.
- [ ] Implement staging shader pass in the eye worker with FXAA 3.11 and optional CAS sharpening.
- [ ] Verify tone map exposure state copy across both eye views.
- [ ] Hook scene submit and display setup using calling conventions discovered by our analysis tools.
- [ ] Enforce same-frame pairing gate before submitting swapchain frames to OpenXR.
