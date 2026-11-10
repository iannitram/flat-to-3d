# Flat to 3D (GPL-3.0). Item list and checked bytes derived from SpidyVR v0.3.8 (aed5c0c) by Ilya Mezerowsky (GPL-3.0).
"""Every game-specific Remastered (4.630.0.0) item on Spidy's stereo-screen path,
from docs/spidy-stereo-core-map.md and the SpidyVR v0.3.8 sources it cites.
kind: hook = hooked function, call = called-only function, check = check-only code
signature, site = call site / return address, data = global, vtable = vtable,
gate = removable gate extra. prio: 1 = plan steps 1-3 (camera, render entry,
view setup), 2 = steps 4-5, 3 = HUD / gate / diag.
bytes: the bytes Spidy checks at that RVA (hex, from the cited source line)."""

T = [
    # name, rva, kind, prio, spidy-checked bytes (hex) or ''
    # --- camera (step 3) ---
    ('stock camera fn (calls submit)', 0x1646e10, 'call', 1, ''),
    ('camera submit call site', 0x164703f, 'site', 1, 'e86c2a2500'),
    ('camera submit return', 0x1647044, 'site', 1, ''),
    ('view descriptor submit', 0x1899ab0, 'hook', 1, '483bd1747c488bc1'),
    ('lens ctor (12 args)', 0x187bb00, 'call', 1, '488bc448895808 55'),
    ('pose setter', 0x187ca10, 'call', 1, '4053 4883ec60 0f1002'),
    # --- frame / render entry (steps 2-4) ---
    ('frame function', 0x175a060, 'call', 1, ''),
    ('frame -> offscreen call site', 0x175aaa9, 'site', 1, 'e892571c00'),
    ('offscreen render setup (placeEyes hook)', 0x1920240, 'hook', 1, '48895c240848897424105748 83ec2040b701'),
    ('history shift', 0x18a13c0, 'call', 1, ''),
    ('history shift per-view copy', 0x189ee00, 'check', 1, '48895c2408574883ec20488bf941b830050000'),
    # --- view manager / eye views (step 4) ---
    ('view maintenance (ViewContextManager)', 0x18a0bb0, 'hook', 1, '48895c240848896c2410'),
    ('create offscreen view', 0x18a09c0, 'call', 1, '4c8bdc49895b10'),
    ('view update (exposure copy)', 0x189bd30, 'hook', 2, '48895c240848897c2410'),
    ('occlusion builder', 0x189fed0, 'call', 2, '48895c241048897424185748 83ec40488bf94883b9a01b000000'),
    ('ViewContextManager global', 0x7a34dd0, 'data', 1, ''),
    ('view pool mask', 0x7a34dd8, 'data', 1, ''),
    ('offscreen slots (mgr+0x30)', 0x7a34e00, 'data', 1, ''),
    ('texture wrapper vtable', 0x4e56c80, 'vtable', 2, ''),
    # --- render jobs, buffers, display (step 5) ---
    ('copy view into render job', 0x19223e0, 'hook', 2, '48895c2420555657'),
    ('render job begin', 0x189e310, 'hook', 2, '48895c2408574883ec30'),
    ('render job end', 0x189e3a0, 'hook', 2, '48895c2408574883ec30'),
    ('buffer-set init', 0x1873470, 'hook', 2, '40534883ec30448b4a04'),
    ('buffer-set init return gate', 0x186c325, 'site', 2, ''),
    ('buffer-set sizes', 0x1874f40, 'hook', 2, '48895c240848897424 1048897c2418554154'),
    ('target kind test', 0x1875270, 'check', 2, '440fb6d241b1010fbae20e'),
    ('kind-mask table', 0x3d11b18, 'data', 2, 'ff000000ff0000002000000028000000'),
    ('TemporalAa', 0x1842e10, 'check', 2, '488bc4554 88da8f8feffff'),
    ('display setup', 0x1920310, 'hook', 2, '4d85c00f847f030000'),
    ('enqueue render command', 0x187e130, 'call', 2, '48895c240848896c2410'),
    ('tone map', 0x1846c20, 'hook', 2, '488bc448895010'),
    ('adapted-lum read', 0x189c710, 'check', 2, '488d81f01600004 88b10'),
    ('SubmitScene (shadows)', 0x19206a0, 'hook', 3, '44884c242044894424184889542410555356574154'),
    ('FitCascades', 0x170fbc0, 'hook', 3, '488bc44c8948204c894018488950104889480855535657'),
    ('RenderActor', 0x17991a0, 'hook', 3, '488bc444884018488950104889 4808'),
    ('CopyFinal (diag)', 0x186cc00, 'hook', 3, '40534883ec30f6817c4e010020'),
    ('readback init (diag)', 0x186c670, 'call', 3, '48895c240848897424 10'),
    ('DrawOff', 0x191afb0, 'call', 3, '8b415ca8207529 83c820 89415c'),
    ('DrawOn', 0x191b880, 'call', 3, '8b415ca8207429 83e0df 89415c'),
    ('frame allocate (diag)', 0x1872860, 'call', 3, ''),
    ('render-instance table', 0x7a436e8, 'data', 3, ''),
    ('render-instance count', 0x7a43704, 'data', 3, ''),
    # --- render memory ring ---
    ('RenderAlloc ring create', 0x1872d90, 'hook', 2, '48895c240848896c241048897424185741564157 4883ec30'),
    ('ring create call site', 0x188a195, 'site', 2, '488d0de4e60a06e8ef8bfeff'),
    ('RenderAlloc per-frame rollover', 0x1872b90, 'hook', 2, '48895c2408574883ec30837928004 88bd98b411cc6414200'),
    ('rollover call site', 0x187e623, 'site', 2, '488d0d56a20b06e86145ffff'),
    ('RenderAlloc allocator', 0x7938880, 'data', 2, ''),
    # --- HUD (gate) ---
    ('HUD follower placement', 0x73ab80, 'hook', 3, '40555348 8dac24c8feffff4881ec38020000'),
    ('instance SetTransform', 0x191c0e0, 'call', 3, '40534883ec400f100233c0'),
    ('HUD follower vtable', 0x38929c8, 'vtable', 3, ''),
    ('RTT stream update', 0x2104400, 'hook', 3, '48895c241048897424184889 7c2420554154 4156'),
    ('RTT stream vtable', 0x4f87960, 'vtable', 3, ''),
    ('Scaleform render command', 0x1d2b6c0, 'hook', 3, '48895c241048894c240855565741544155415641 57'),
    ('marker projection XY', 0x1f10ad0, 'hook', 3, '48895c240848897424105748 83ec4048'),
    ('marker projection', 0x1f10b60, 'hook', 3, '48895c2408574883ec40488bfa0f2974'),
    ('HUD second movie', 0x7be3e00, 'data', 3, ''),
    ('HUD res factor', 0x7a4ae50, 'data', 3, ''),
    ('HUD frame counter', 0x7a30b00, 'data', 3, ''),
    ('HUD list frames', 0x7be5740, 'data', 3, ''),
    ('HUD counts', 0x7be3f38, 'data', 3, ''),
    ('HUD items', 0x7be3f40, 'data', 3, ''),
    # --- game time (gate) ---
    ('TimeScaleSystem::Update', 0x19bb430, 'hook', 2, '488bc448895818555657415441554156'),
    ('time: movss [rbx+18]', 0x19bb6ca, 'check', 3, 'f30f114318'),
    ('time: movsd clock', 0x19bb6d3, 'check', 3, 'f20f1105b5440c06'),
    ('time: test physics flag', 0x19bb6db, 'check', 3, 'f683d007000001'),
    ('time: movss [rbx+1c]', 0x19bb70f, 'check', 3, 'f30f11431c'),
    ('time: lea havok step', 0x19bb717, 'check', 3, '488d0d42ee6d04'),
    ('havok step setter', 0x1822670, 'check', 3, 'f30f5949044c894108f30f1109c3'),
    ('clock scale', 0x7a7fb90, 'data', 3, ''),
    ('frame game time (pause gate)', 0x7a7fbd8, 'data', 2, ''),
    ('clock 0x7a7fbf0', 0x7a7fbf0, 'data', 3, ''),
    ('havok step', 0x609a560, 'data', 3, ''),
    # --- removable gate extras (bridge, player) ---
    ('follow-camera commit', 0x1e1d600, 'gate', 3, '4883ec38f20f'),
    ('clear input', 0x1ce2f40, 'gate', 3, '48895c241048896c2418'),
    ('input query A', 0x1cdfa70, 'gate', 3, '48895c240848897424'),
    ('input query B', 0x1cdf840, 'gate', 3, '48895c240848897424'),
    ('combat camera update', 0x5c2630, 'gate', 3, '4885d20f844809'),
    ('camera setter', 0x191c590, 'gate', 3, '0f1051304c8bc18b02'),
    ('FollowCamera vtable', 0x3871fd8, 'vtable', 3, ''),
    ('CombatCamera vtable', 0x38720d0, 'vtable', 3, ''),
    ('HeroLocal vtable', 0x38a93c8, 'vtable', 3, ''),
    ('player vtable 0x38b2c98', 0x38b2c98, 'vtable', 3, ''),
    ('MoverStandard vtable', 0x4f70168, 'vtable', 3, ''),
    ('component registry', 0x7a44320, 'data', 3, ''),
    ('component registry 2', 0x7a44340, 'data', 3, ''),
]


def hexbytes(s):
    return bytes.fromhex(s.replace(' ', ''))
