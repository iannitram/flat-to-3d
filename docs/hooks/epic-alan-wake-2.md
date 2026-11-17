# Alan Wake 2: hook map (WR, #289)

The hook-point research step for Alan Wake 2 (ruling 52; shape in [ROADMAP.md](../ROADMAP.md#hook-point-research-ruling-52)). Every address and pattern below comes from this step's own analysis of the installed exe. The research note ([epic-alan-wake-2.md](../notes/research/games/epic-alan-wake-2.md)) was a checklist only. No code may use a row marked `lead`.

## Build

| | |
|---|---|
| File | `AlanWake2.exe` (x64, 61,416,960 bytes, image base `0x140000000`, `.text` at `0x1000`, size `0x261133f`) |
| SHA-256 | `09135f128c07cfa4598b22dd4754a98b2c1f4ff816bfcb1e336ecfb4462b4163` |
| File version | `0.564.904.9` (product version `2026.2.3.0`, changelist `5649049`, published 2026-10-11, depot `bigfish\release`), from the PE version resource |
| Store build | Epic `1.2.11`, from the Epic manifest in the install, `.egstore/E4DE7D7C40CD3B7BD5EA81E762A8E089.manifest` (its build-version field; the exe's SHA-1 is listed in that manifest). Heroic's `legendaryConfig/legendary/installed.json` is empty (`{}`) on this device, and Heroic's cached manifest is still the `1.2.10` one |
| Symbols | none; no PDB ships, engine code has no exports |

A hook must refuse any exe whose SHA-256 differs from the one above. The previous map (2026-11-16) was for Epic `1.2.10`, SHA-256 `0e7a7b5f...5b8904`, file version `0.559.302.8`; every RVA moved in `1.2.11` (see "Earlier build" at the end).

## How it was found

All on 2026-11-17, on the Z13 (SteamOS), by static analysis of a copy of the exe in a scratch folder (game folder, prefix and backup untouched; the game was not run):

- **Reference:** Control's `renderer_rmdwin10_f.dll` (Steam library `~/.local/share/Steam/steamapps/common/Control/`), which exports `rend::View::setWorldToView` (`0x14100`), `setViewToWorld` (`0x145b0`) and `setViewToClip` (`0x14a50`) by their C++ names; `d3d_rmdwin10_f.dll` exports `d3d::DeviceUtil::present` (`0x31b50`). Their disassembly gave the behaviour to look for.
- **Tools:** Python 3 scripts on `pefile` and `capstone` 5.0.7 (written for this step): `.pdata` function ranges, RTTI walk (type descriptor, complete-object locator, vtable), RIP-relative and `E8` call cross-references, import-table references, disassembly. Craft Anything CLI (`native-strings-find`, `native-bytes-find`, Deep mode) cross-checked the string addresses and every pattern.
- **Method per row** is in the table: RTTI name, string reference, import reference, or structural match against Control (same writes, same inverse, same uploads; byte-level matching does not work, because Control is SSE code and Alan Wake 2 is AVX).
- **Patterns:** bytes from the function start (or call site). Each was searched over the whole file with both tools and matched exactly once, at the RVA given. The two `rend::View` setters are byte-identical except for call targets, so their patterns keep the first `call`'s `rel32` (the call to the `m::View` setter) literally; that is what makes each one unique, so they hold only for this exe.

## Hooks

RVAs are relative to the image base. "Step" is the build step that uses the hook.

| Hook | Step | RVA | Pattern | How found | Status | Checklist |
|---|---|---|---|---|---|---|
| `rend::RendererInterface::renderWorldInternal(const View&, const View&, DirectionalLight*, const FrameCapture&)` | W3, W4 | `0x225ab60` (ends `0x22637f5`) | `4C 89 4C 24 20 4C 89 44 24 18 48 89 54 24 10 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 D8 EE FF FF` | RTTI name: 129 type descriptors containing `renderWorldInternal@RendererInterface@rend`, 67 vtables, referenced from 62 functions; 48 of them are called, and every one of those 48 only from this function. It spills `rdx`, `r8`, `r9` (the three arguments after `this`), which fits the name's signature. Direct callers (`E8`): 2, both in `0x2267230` | `verified` | note's name, two-view signature and "129 lambdas": `agrees` |
| `rend::View::setWorldToView` (writes world-to-view, refreshes the shader copies) | W3 | `0x12bb320` (ends `0x12bb576`) | `48 8B C4 48 89 58 08 55 48 8D 68 A1 48 81 EC C0 00 00 00 C5 F8 29 70 E8 48 8B D9 C5 F8 29 78 D8 E8 1B FF FF FF` | Structural match to Control's `rend::View::setWorldToView`: calls `m::View::setWorldToView` (`0x12bb260`), converts world-to-view (`+0x10`) and view-to-world (`+0x70`) to float into `+0x4b0` and `+0x4f0`, writes `g_vViewPoint` (`+0x7b0`, from `+0xb8`) and `g_vViewDir` (`+0x7c0`, from `+0xa0`), then tail-jumps to `0x21bfeb0`, which refreshes the float view-to-clip and clip-to-view (`+0x530`, `+0x570`) and tail-jumps to the world-to-clip refresh `0x21bf090`. Direct callers: 3, all in `0x21bed10` | `verified` | note's name: `agrees` |
| `m::View::setWorldToView` | W3 | `0x12bb260` | `48 83 EC 28 C5 F8 10 02 C5 F8 11 41 10 C5 FB 10 4A 10` | Structural match: copies the 4x3 argument to `+0x10` and `+0x70`, inverts `+0x70` (`0x3aafb0`), tail-jumps to the derived-matrix update `0x3aa9e0` | `verified` | not in the note |
| `rend::View::setViewToWorld` | W3 | `0x12bb580` (ends `0x12bb7d6`) | `48 8B C4 48 89 58 08 55 48 8D 68 A1 48 81 EC C0 00 00 00 C5 F8 29 70 E8 48 8B D9 C5 F8 29 78 D8 E8 6B F3 0E FF` | Structural match to Control's `setViewToWorld`: the same body as `setWorldToView`, byte for byte except call and jump targets (11 differing bytes, all in `rel32` fields); its first call goes to `m::View::setViewToWorld` (`0x3aa910`). Direct callers (`E8`): 13, in 11 functions | `verified` | note's name: `agrees` |
| `m::View::setViewToWorld` | W3 | `0x3aa910` | `48 83 EC 28 C5 F8 10 02 C5 F8 11 41 70 C5 FB 10 4A 10` | Structural match: copies the argument to `+0x70` and `+0x10`, inverts `+0x10` (`0x3aafb0`), tail-jumps to `0x3aa9e0` | `verified` | not in the note |
| `setViewToClip` | W3 | `0x12bb1d0` (ends `0x12bb25b`) | `40 53 48 83 EC 20 C5 FC 10 02 C5 FC 11 81 D0 00 00 00` | Structural match to Control's `rend::View::setViewToClip`: copies the 4x4 argument to `+0xd0` and `+0x150`, inverts `+0x150` (`0x3ab530`), tail-jumps to `0x3aa9e0`; no float upload. Of its 13 direct call sites, 12 are immediately followed by a call to `rend::View::setViewToWorld`; the 13th (`0x21bff4c`, in `0x21bff30`) is followed by a tail jump to it | `verified` (address and behaviour); whether this is the `rend::View` or the `m::View` method is a `lead` | note's name: `agrees` |
| Present (engine side; below Streamline) | W4 | function `0x204b4e0` (ends `0x204b7f5`); the `Present` call at `0x204b707` | function: `4C 8B DC 49 89 5B 10 49 89 6B 18 56 57 41 54 41 56 41 57 48 83 EC 50`; call site (pattern at `0x204b704`): `49 8B C9 FF 50 40 44 8B F8 41 80 7C 24 68 00 74 48` | Import reference: `CreateDXGIFactory1` is imported from `sl.interposer.dll` (IAT `0x2615338`), called at `0x205df48` (factory stored at device `+0x3bd8`, under the lock at `+0x3bd0`) and at `0x204d6f6`. Swap chain creation `0x204bd30` takes that factory under the lock, calls its `CreateSwapChain` (vtable `+0x50`, through `r13` at `0x204c09f`) into `[[this]+8]`, then `SetColorSpace1` (`+0x130`) and `MakeWindowAssociation(hwnd, 2)` (`+0x40`). `0x204b4e0` calls `GetFullscreenState` (`+0x58`, at `0x204b512`) and `Present` (`+0x40`) on `[[rcx]+8]`, flags `0` or `0x200` (allow tearing): the shape of Control's `d3d::DeviceUtil::present`. New in this build: right after `Present` it fetches a Streamline feature function (`slGetFeatureFunction`, `0x204b742`) and calls it with `5` (inferred: a Reflex/PCL present marker) | `verified` (static); that the swap chain is Streamline's proxy at run time is a `lead` | note's "hook the swap chain that `sl.interposer.dll` returns": `agrees` |
| Swap chain creation | W4 | `0x204bd30` (ends `0x204c261`) | `48 89 5C 24 20 48 89 4C 24 08 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 F0` | As in the row above; one direct caller (`0x12e6b1b`) | `verified` (static) | not in the note |
| FSR frame-generation swap chain | none (W4 turns frame generation off) | inside `0x204bd30`, `0x204bf95`-`0x204c05e` | none | Same function: a second path builds the swap chain through two imported calls with descriptor types `0x30005` (create, at `0x204c000`) and `0x30009` (query, at `0x204c02d`) instead of `CreateSwapChain` | `lead` | not in the note |
| Previous-view copy (name unknown) | W4 | `0x21befc0` (ends `0x21bf086`) | `40 53 48 83 EC 20 48 8B D9 E8` then the `rel32` must reach `0x21bfeb0` (not unique alone; not for code) | Found as a caller of `0x21bfeb0`: refreshes the float view-to-clip, then copies world-to-view `+0x10` to `+0x440` and view-to-clip `+0xd0` to `+0x3c0` (both double) and sets the byte `+0x4a0` to 1. Called from `0x21bed10`, `0x2239730` and `0x2267230` (between the setter calls and the two `renderWorldInternal` calls). Inferred: it saves the previous frame's view for motion vectors, which W4's per-eye previous view must control | `lead` | not in the note |

## View layout

In Alan Wake 2, `rend::View` holds its `m::View` at `+0`. Offsets are from the `rend::View` pointer (the `this` of the setters above).

**Double-precision matrices** (`verified`: the setters above write them, and the view serializer `0x2001100` references the strings `WorldToView`, `ViewToWorld`, `ViewToClip`, `ClipToView` next to the offsets `+0x10`, `+0x70`, `+0xd0`, `+0x150`):

| Offset | Member | Type |
|---|---|---|
| `+0x10` | world-to-view | 4x3, double (`m::Matrix4x3Template<double>`) |
| `+0x70` | view-to-world | 4x3, double |
| `+0xa0` | view direction (row 2 of view-to-world) | 3 doubles |
| `+0xb8` | view point (translation of view-to-world) | 3 doubles |
| `+0xd0` | view-to-clip | 4x4, double (`m::Matrix4Template<double>`) |
| `+0x150` | clip-to-view | 4x4, double |
| `+0x3c0` | previous view-to-clip (`lead`, from `0x21befc0`) | 4x4, double |
| `+0x440` | previous world-to-view (`lead`, from `0x21befc0`) | 4x3, double |

**Float shader copies** (all `verified`). The constant upload function `0x21bff70` hands each float matrix to `0x206b8f0` as `lea r9, [rsi+offset]`. Each upload is guarded by a lazily built static descriptor; the guard variable compared on the main path was matched to the same guard in the first-use block, and that block references the constant's name string:

| Offset | Constant | Guard / first-use block |
|---|---|---|
| `+0x4b0` | `g_mWorldToView` (float4x4) | `0x3c15dc4` / `0x21c0907` |
| `+0x4f0` | `g_mViewToWorld` | `0x3c1617c` / `0x21c09bd` |
| `+0x530` | `g_mViewToClip` (float4x4) | `0x3c15f38` / `0x21c0a73` |
| `+0x570` | `g_mClipToView` | `0x3c160a0` / `0x21c0b29` |
| `+0x5b0` | `g_mWorldToClip` | `0x3c16174` / `0x21c0bdf` |
| `+0x5f0` | `g_mClipToWorld` | `0x3c15dc0` / `0x21c0c95` |
| `+0x630` | `g_mClipToPreviousClip` | `0x3c16178` / `0x21c0d4b` |
| `+0x670` | `g_mViewToPreviousClip` | `0x3c16180` / `0x21c0e01` |
| `+0x6b0` | `g_mPreviousWorldToView` | `0x3c15f10` / `0x21c0eb7` |
| `+0x6f0` | `g_mPreviousViewToView` | `0x3c15fac` / `0x21c0f6d` |
| `+0x730` | `g_mPreviousClipToClipNoJitter` | `0x3c161a8` / `0x21c1023` |
| `+0x770` | `g_mClipToPreviousClipNoJitter` | `0x3c160a4` / `0x21c10d9` |
| `+0x7b0` | `g_vViewPoint` (float4) | `0x3c15d60` / `0x21c118f` |
| `+0x7c0` | `g_vViewDir` (float4) | `0x3c15fa8` / `0x21c121e` |

Each name string is referenced once in the exe, from this function (strings at `0x331a068` onward).

Checklist: the note's constant names (`g_mWorldToView`, `g_mViewToClip`, `g_mWorldToClip`, `g_mClipToView`, the `g_mPrevious...` set) `agree`, except `g_mPreviousViewToClip`, which this function does not upload; `g_mViewToPreviousClip` and `g_mPreviousViewToView` are uploaded.

## Matrix convention

From the CPU code (`verified` for what the code does; the GPU side is a `lead`):

- `g_mWorldToView` is built from the 4x3 world-to-view (converted to float by `0xf63130`) with no transpose: `[m0 m1 m2 0; m3 m4 m5 0; m6 m7 m8 0; m9 m10 m11 1]`. The `1.0` is the float constant at `0x3351f78`, loaded at `0x12bb35c` and stored as element 15. The translation sits in floats 12 to 14, so the engine uses row vectors (`v * M`). In memory that is the same as a column-major matrix used as `mul(M, v)`.
- `g_mViewToClip` is the 4x4 view-to-clip converted element for element (`0x3ab840`, in `0x21bfeb0`), with no transpose, so it has the same convention.
- `g_vViewPoint` is the translation row of view-to-world and `g_vViewDir` is its third row, `+Z`, not negated. So the camera looks down `+Z` in view space: a left-handed view space (inferred).

## Owed: the RenderDoc capture (tests 3 and 4 as the issue counts them)

Not taken: it needs the game running with a display, and this step did not launch it. Until it is taken, these stay `lead`:

1. How the shaders read `g_mWorldToView` and `g_mViewToClip`: row- or column-major packing in the constant buffer. The capture must show, on one world draw, the 16 floats of `g_mWorldToView` with the translation in floats 12 to 14 (matching the CPU layout above) and how the shader reflection declares them.
2. The projection: from `g_mViewToClip`, whether depth runs 0 to 1, whether it is reversed-Z (near maps to 1), whether it is infinite-far, and the jitter terms (elements 8 and 9, or 2 and 6) with upscaling on and off.

Recipe (host shell, not the Heroic flatpak: the flatpak runtime cannot see the host's RenderDoc layer in `/etc/vulkan/implicit_layer.d/renderdoc_capture.json` or `/usr/lib/librenderdoc.so`). Nothing in the game folder or prefix changes; only the environment does:

```bash
export STEAM_COMPAT_DATA_PATH="$HOME/Games/Heroic/Prefixes/Alan Wake 2"
export STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam"
export ENABLE_VULKAN_RENDERDOC_CAPTURE=1          # loads RenderDoc 1.46's Vulkan layer into vkd3d-proton
export RENDERDOC_CAPFILE="$HOME/aw2-captures/aw2" # captures land here as aw2_*.rdc
export VKD3D_CONFIG=force_host_cached            # optional: easier buffer reads; drop if it misbehaves
export DXVK_HDR=0 PROTON_FSR4_UPGRADE=0
mkdir -p "$HOME/aw2-captures"
cd "$HOME/Games/AlanWake2"
"$HOME/.var/app/com.heroicgameslauncher.hgl/config/heroic/tools/proton/Proton-CachyOS-latest/proton" waitforexitandrun ./AlanWake2.exe
```

Before the run, in the game: upscaling and frame generation off, in a lit outdoor scene. Press F12 (RenderDoc's capture key) once in gameplay, quit, then open the `.rdc` (`renderdoccmd` or qrenderdoc), pick a world mesh draw, and read the constant buffer that holds `g_mWorldToView` / `g_mViewToClip`. If the game will not start offline outside Heroic, add the same environment variables (`ENABLE_VULKAN_RENDERDOC_CAPTURE`, `RENDERDOC_CAPFILE`, and optionally `VKD3D_CONFIG`, `DXVK_HDR`, `PROTON_FSR4_UPGRADE`) to Heroic's game settings and run Heroic from a host-installed (non-flatpak) build instead.

## Counts

- `verified`: 8 hook rows (renderWorldInternal, the two `rend::View` setters, the two `m::View` setters, setViewToClip, the present function and call site, swap chain creation), the double layout of the six main matrices, and all 14 float slots.
- `lead`: the `rend::`/`m::` level of setViewToClip, the run-time proxy swap chain, the FSR frame-generation swap chain path, the previous-view copy and its two members, and the GPU-side matrix convention (needs the capture).

## Earlier build (1.2.10, for history only)

The 2026-11-16 map's RVAs for `1.2.10` (SHA-256 `0e7a7b5fe05ada30ac043d96eeb658e934db96b4783479ade64495ed428b5904`, the backup copy on an external drive). On 2026-11-17 the old map's seven patterns and the upload function's prologue were re-scanned over that copy, and each matched once at the RVA below; the swap chain creation RVA is the old map's and was not re-checked (its `1.2.11` prologue pattern does not occur in `1.2.10`). None of these is valid for `1.2.11`. In `1.2.11` the old patterns find the same functions, except `renderWorldInternal`, whose frame size changed (`C8 EE` to `D8 EE`), and the `rend::View` setters, whose shared pattern matched both.

| Hook | 1.2.10 RVA | 1.2.11 RVA |
|---|---|---|
| `renderWorldInternal` | `0x228bff0` | `0x225ab60` |
| `rend::View::setWorldToView` | `0x12cfb40` | `0x12bb320` |
| `m::View::setWorldToView` | `0x12cfa80` | `0x12bb260` |
| `rend::View::setViewToWorld` | `0x12cfda0` | `0x12bb580` |
| `m::View::setViewToWorld` | `0x3b2e70` | `0x3aa910` |
| `setViewToClip` | `0x12cf9f0` | `0x12bb1d0` |
| Present function / call-site pattern start (the `Present` call is 3 bytes later) | `0x207a680` / `0x207a8a4` | `0x204b4e0` / `0x204b704` |
| Swap chain creation | `0x207aec0` | `0x204bd30` |
| Constant upload | `0x21f0110` | `0x21bff70` |
