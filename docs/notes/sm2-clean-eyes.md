# Marvel's Spider-Man 2: clean eye views, anti-aliasing, and render settings for Step 5 (issue #21)

Notes for port Step 5 (issue #21: same-frame pairs and clean images; [SPEC.md](../../SPEC.md#m6) M6).
Research date: 2026-11-12.

These findings come from studying the public documentation and development logs of [lecutis7of9/HEROVR](https://github.com/lecutis7of9/HEROVR/blob/82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859/BUILD-SM2.md) (GPL-3.0, pinned commit `82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859`, specifically `BUILD-SM2.md`).

Standing project rule ([DECISIONS.md](DECISIONS.md) ruling 2026-10-31; R41): learn only, keep our port. Copy no code, copy no addresses (HEROVR's addresses come from the closed SM2VR mod). All addresses for our port must be discovered independently by our own analysis tools ([address map](../spiderman2/address-map.md)).

---

## 1. Summary of findings from peer

In Marvel's Spider-Man 2 (PC build 2.810.0.0), the off-screen eye views need their own anti-aliasing, lighting and ray-tracing settings. The flat-view path does not provide them:

1. **The game's own TAA/DLAA does not run on the eye views.** Its anti-aliasing runs for the flat monitor view only. The peer did not find the sub-pixel jitter and history offsets that the game's temporal pass needs for the eyes, and it did not guess them.
2. **Shimmer on fine detail.** Without temporal AA, fine geometry (window grids, roof edges, fences) aliases and swims while the head moves, and it gets worse with speed. FXAA does not remove this moire.
3. **Mod-side FXAA 3.11, on by default.** The peer runs FXAA 3.11 (quality preset 29: 12 search steps, subpixel 0.5, edge thresholds 0.125 and 0.0625) on the eye images before they reach the headset. Its measured GPU cost was about 0.33 ms mean per pair.
4. **Contrast-adaptive sharpening (CAS), opt-in.** The peer offers CAS after FXAA, with a halo guard against edge ringing. It is off by default.
5. **Per-eye anti-aliasing history, opt-in.** Tier 1: each eye's lens is offset every frame by an 8-step Halton(2,3) sub-pixel pattern, and the peer's reprojected history pass blends the frames (at least 80% history while on). Tier 2: the game's own TAA on each eye, behind a launch option. The peer found why the eyes never got it (the TAA targets are not allowed for the offscreen buffer sets the eyes use), patched the allowed kinds, and kept a fallback. Its disassembly confirms the path; the peer still lists the result as to be proven in a run.
6. **Ray tracing.** The peer switches ray tracing off before each session (reflections, shadows, AO, interiors and ray reconstruction) and restores the player's settings afterwards. Flat-looking windows come from RT Interiors. Matte glass comes from RT reflections being off, and the eyes' screen-space reflection targets are not ported. Its document does not give the registry names it uses, so none are assumed here.
7. **Per-eye shadow cascades.** Each eye fits its own key-light cascades to its own lens. The peer left out sharing one set between the eyes: the scene-submit parts it would hand over are unconfirmed for this build, and an older SM2VR build crashed doing it.
8. **Exposure and tone.** The peer's tone hook copies the eye's view bytes into each eye job. Its logs showed exposure shared by both eyes, and it ruled exposure out as the cause of the eye artifacts. Our tone check should confirm that match, not assume it.

---

## 2. Input to port Step 5 (issue #21)

Spec M6 requires:
> EACH SHOWN PAIR SHALL come from the same frame, with matching brightness and its own anti-aliasing history; ray-traced reflections are off for the eyes and the upscaler is bypassed (port Step 5).

Applying the peer findings to our Linux/Proton port architecture for Step 5:

### Anti-Aliasing Strategy
- **Start with mod-side FXAA 3.11 in the staging pass:** When copying completed eye render targets to the OpenXR swapchain, run a compute or pixel shader pass with FXAA 3.11. Add CAS as an optional pass after it, as the peer does (off by default).
- **Engine TAA on the eyes is a later tier:** The peer built it as an opt-in path with a fallback, and it depends on jitter and history inputs that must be confirmed for each game build. Take it only after the FXAA path and the same-frame gate pass in a run.
- **Draw overlays after AA:** Any HUD or stereo screen framing must be composited after the FXAA (and CAS, if on) pass so text and lines remain sharp. The peer draws its overlay after FXAA for the same reason.

### Ray Tracing Configuration
- **Confirm RT is off before launch and restore it after:** The peer does this with a launch script before each session. Find our own mechanism in Step 5 and check it at launch; do not assume registry key or value names from the peer.
- The peer's reason for this holds for us: RT Interiors makes flat window surfaces, and RT reflections off means matte glass.

### Upscaler Bypass
- **Native eye resolution:** The game's DLAA and DLSS do not run on the eye views (peer finding). The peer renders eyes at 100% by default and treats eye supersampling as an opt-in setting. Render eye targets at full buffer dimensions, and confirm in our own analysis that no upscaling pass touches the eye buffers.

### Brightness & Tone Consistency
- **Check that both eyes get the same exposure:** The peer's tone hook copies the eye's view bytes into each eye job, and its logs showed exposure shared by both eyes. Our tone hook should do the same, and the step's test should compare exposure per pair.

### Frame Pairing & Timing
- **Same-frame gate (our design, not the peer's):** Tag each eye job with the game frame counter; present a stereo frame to OpenXR only when both eye jobs of the same frame number have completed. The peer's own check is the pattern to copy for the test: every eye render used that frame's pose and the eyes were 61.4 mm apart.

---

## 3. Checklist for #21 implementation

- [ ] Find a ray-tracing-off mechanism in our own analysis and check it at launch.
- [ ] Implement the staging shader pass in the eye worker with FXAA 3.11; add CAS as an optional pass, off by default.
- [ ] Verify the tone hook gives both eyes the same exposure, and test it per pair.
- [ ] Hook scene submit and display setup using calling conventions discovered by our analysis tools.
- [ ] Enforce the same-frame pairing gate before submitting swapchain frames to OpenXR.
- [ ] Decide on the engine-TAA tier only after the FXAA path passes in a run.
