# Alan Wake 2: hook map (WR, #289)

The hook-point research step for Alan Wake 2 (ruling 52; shape in [ROADMAP.md](../ROADMAP.md#hook-point-research-ruling-52)). Every address and pattern below comes from this step's own analysis of the installed exe. The research note ([epic-alan-wake-2.md](../notes/research/games/epic-alan-wake-2.md)) was a checklist only. No code may use a row marked `lead`.

## Build

| | |
|---|---|
| File | `AlanWake2.exe` (x64, 62,304,768 bytes, image base `0x140000000`) |
| SHA-256 | `0e7a7b5fe05ada30ac043d96eeb658e934db96b4783479ade64495ed428b5904` |
| File version | `0.559.302.8` (product version `2026.2.3.0`, changelist `5593028`, published 2026-08-08, depot `bigfish\release`) |
| Store build | Epic, app name `dc9d2e595d0e4650b35d659f90d41059`, version `1.2.10`, label `Live-Windows` (from Heroic's `legendaryConfig/legendary/installed.json`) |
| Symbols | none; no PDB ships, engine code has no exports |

A hook must refuse any exe whose SHA-256 differs from the one above.

## How it was found

All on 2026-11-16, by static analysis of the files on disk (game folders read-only):

- **Reference:** Control's `renderer_rmdwin10_f.dll` and `d3d_rmdwin10_f.dll`, which export `rend::View`'s setters and getters and `d3d::DeviceUtil::present` by their C++ names. Their export tables and disassembly gave the behaviour to look for: which member each setter writes, which inverse it computes, and what it uploads.
- **Tools:** Python scripts on `pefile` and `capstone` (written for this step): `.pdata` function ranges, RTTI walk (type descriptor to complete-object locator to vtable), vtable and call cross-references, RIP-relative string references, import-table references, displacement searches. Ghidra 12.2 headless auto-analysis of both files, used only to cross-check: its function entries and ends agree with `.pdata` for every function row below.
- **Method per row** is in the table: RTTI name, string reference, import reference, or structural match against Control (same writes, same inverse, same uploads; byte-level matching does not work, because Control's build is SSE code and Alan Wake 2's is AVX).
- **Patterns:** bytes from the function start (or call site), with every call/jump `rel32` and every RIP displacement as `??`. Each pattern was scanned over the whole `.text` section (`0x1000`, size `0x264360f`) and matched exactly once, at the RVA given, except where the row says otherwise.

## Hooks

RVAs are relative to the image base. "Step" is the build step that uses the hook.

| Hook | Step | RVA | Pattern | How found | Status | Checklist |
|---|---|---|---|---|---|---|
| `rend::RendererInterface::renderWorldInternal(const View&, const View&, DirectionalLight*, const FrameCapture&)` | W3, W4 | `0x228bff0` (ends `0x2294ca3`) | `4C 89 4C 24 20 4C 89 44 24 18 48 89 54 24 10 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 C8 EE FF FF` | RTTI name: 62 `std::_Func_impl_no_alloc<lambda_..., renderWorldInternal@RendererInterface@rend@@...>` vtables, referenced from 51 helper functions; 48 of those are called only from this function. It spills all four register arguments (`this` and the first three; the fourth is on the stack), which fits the name's signature | `verified` | note's name and two-view signature: `agrees` |
| `rend::View::setWorldToView` (writes the world-to-view matrix and refreshes the shader copies) | W3 | `0x12cfb40` (size `0x256`) | `48 8B C4 48 89 58 08 55 48 8D 68 A1 48 81 EC C0 00 00 00 C5 F8 29 70 E8 48 8B D9 C5 F8 29 78 D8 E8 ?? ?? ?? ?? 48 8D 53 10 48 8D 4D C7 E8`, then the `E8` at +0x20 must call `m::View::setWorldToView` below | Structural match to Control's `rend::View::setWorldToView`: calls the `m::View` setter, converts the new world-to-view and view-to-world to float into the `g_mWorldToView` / `g_mViewToWorld` slots, writes `g_vViewPoint` / `g_vViewDir`, then tail-jumps to the view-to-clip and world-to-clip refresh (`0x21f0050`, then `0x21ef1f0`). Direct `call` sites (E8 scan): 3, all in `0x21eee70` | `verified` | note's name: `agrees` |
| `m::View::setWorldToView` | W3 | `0x12cfa80` | `48 83 EC 28 C5 F8 10 02 C5 F8 11 41 10 C5 FB 10 4A 10` | Structural match: copies the argument to `+0x10` (world-to-view) and `+0x70` (view-to-world), inverts `+0x70` (`0x3b3530`), tail-jumps to the derived-matrix update `0x3b2f40` | `verified` | not in the note |
| `rend::View::setViewToWorld` | W3 | `0x12cfda0` (size `0x256`) | the `rend::View::setWorldToView` pattern; the `E8` at +0x20 must call `m::View::setViewToWorld` below | Structural match to Control's `setViewToWorld`: the same body as `setWorldToView` byte for byte except call targets. Direct `call` sites (E8 scan) in `0x21f00d0`, `0x22269b0`, `0x22986e0` (twice), `0x22e7a50`, `0x2371b20`; Ghidra counts 12 calling functions in all | `verified` | note's name: `agrees` |
| `m::View::setViewToWorld` | W3 | `0x3b2e70` | `48 83 EC 28 C5 F8 10 02 C5 F8 11 41 70 C5 FB 10 4A 10` | Structural match: copies the argument to `+0x70` and `+0x10`, inverts `+0x10`, tail-jumps to `0x3b2f40` | `verified` | not in the note |
| `setViewToClip` | W3 | `0x12cf9f0` (size `0x8b`) | `40 53 48 83 EC 20 C5 FC 10 02 C5 FC 11 81 D0 00 00 00` | Structural match to Control's `rend::View::setViewToClip`: copies the 4x4 argument to `+0xd0` (view-to-clip) and `+0x150` (clip-to-view), inverts `+0x150` (`0x3b3ad0`), tail-jumps to `0x3b2f40`; no float upload, as in Control. At each direct call site of `setViewToWorld` found by the E8 scan, it is called just before | `verified` (address and behaviour); whether this is the `rend::View` or the `m::View` method is a `lead` | note's name: `agrees` |
| Present (engine side; below Streamline) | W4 | function `0x207a680`; the `Present` call at `0x207a8a7` | function: `4C 8B DC 49 89 5B 10 49 89 6B 18 56 57 41 54 41 56 41 57 48 83 EC 50`; call site (`0x207a8a4`): `49 8B C9 FF 50 40 44 8B F8 41 80 7C 24 68 00 74 48` | Import reference: `CreateDXGIFactory1` is imported from `sl.interposer.dll` (IAT `0x2647330`), called at `0x208d281`, and the factory is stored at device `+0x3bd8`. Swap chain creation (`0x207aec0`) takes that factory under the lock at `+0x3bd0`, calls its `CreateSwapChain` (vtable `+0x50`, at `0x207b207`), stores the `IDXGISwapChain` at `[[this]+8]`, then calls `SetColorSpace1` and `MakeWindowAssociation(hwnd, 2)`. `0x207a680` calls `GetFullscreenState` (`+0x58`) and then `Present` (`+0x40`) on `[[r12]+8]` with flags `0` or `0x200` (allow tearing): the same shape as Control's `d3d::DeviceUtil::present` | `verified` (static); that the swap chain is Streamline's proxy at run time is a `lead` | note's "hook the swap chain that `sl.interposer.dll` returns": `agrees` |
| FSR frame-generation swap chain | none (W4 turns frame generation off) | inside `0x207aec0`, `0x207b11c`-`0x207b195` | none | Same function: a second path builds the swap chain through `ffxCreateContext` (descriptor type `0x30005`) and `ffxQuery` (`0x30009`) instead of `CreateSwapChain` | `lead` | not in the note |

## View layout

In Alan Wake 2, `rend::View` holds its `m::View` at `+0`. Offsets are from the `rend::View` pointer (the `this` of the setters above).

**Double-precision matrices** (named by the view serializer `0x20304a0` / `0x2030880`, which writes the strings `WorldToView`, `ViewToWorld`, `ViewToClip`, `ClipToView` next to these offsets; status `verified`):

| Offset | Member | Type |
|---|---|---|
| `+0x10` | world-to-view | 4x3, double (`m::Matrix4x3Template<double>`) |
| `+0x70` | view-to-world | 4x3, double |
| `+0xa0` | view direction (row 2 of view-to-world) | 3 doubles |
| `+0xb8` | view point (translation of view-to-world) | 3 doubles |
| `+0xd0` | view-to-clip | 4x4, double (`m::Matrix4Template<double>`) |
| `+0x150` | clip-to-view | 4x4, double |
| `+0x1d0` | world-to-clip | 4x4, double |
| `+0x250` | clip-to-world | 4x4, double |

Each offset in this double table is Control's minus `0x10` (Control's `rend::View` keeps its `m::View` at `+0x10`). The float copies below do not follow that rule.

**Float shader copies.** The constant upload function `0x21f0110` hands each float matrix to the constant-provider call `0x209a900`. Each constant's name comes from a lazily built static descriptor; for each one, the guarded first-use block was followed to its name string:

| Offset | Constant | Status |
|---|---|---|
| `+0x4b0` | `g_mWorldToView` (float4x4) | `verified` (name followed one by one; written by `setWorldToView`) |
| `+0x4f0` | `g_mViewToWorld` | `verified` (written by both setters) |
| `+0x530` | `g_mViewToClip` (float4x4) | `verified` (name followed one by one; written by `0x21f0050` from `+0xd0`) |
| `+0x570` | `g_mClipToView` | `verified` (written by `0x21f0050` from `+0x150`) |
| `+0x5b0` | `g_mWorldToClip` | `lead` (order of the upload code) |
| `+0x5f0` | `g_mClipToWorld` | `lead` |
| `+0x630` | `g_mClipToPreviousClip` | `lead` |
| `+0x670` | `g_mViewToPreviousClip` | `lead` |
| `+0x6b0` | `g_mPreviousWorldToView` | `lead` |
| `+0x6f0` | `g_mPreviousViewToView` | `lead` |
| `+0x730` | `g_mPreviousClipToClipNoJitter` | `lead` |
| `+0x770` | `g_mClipToPreviousClipNoJitter` | `lead` |
| `+0x7b0` | `g_vViewPoint` (float4) | `verified` (written by both setters from `+0xb8`) |
| `+0x7c0` | `g_vViewDir` (float4) | `verified` (written by both setters from `+0xa0`) |

The same function also uploads `g_uViewTemporalFrameCount`, `g_uViewTemporalFrame`, `g_fNear`, `g_fFar` and `g_fInvNear`.

Checklist: the note's constant names (`g_mWorldToView`, `g_mViewToClip`, `g_mWorldToClip`, `g_mClipToView`, the `g_mPrevious...` set) `agree`. The note's `g_mPreviousViewToClip` is in the exe as a string but is not uploaded by this function; `g_mViewToPreviousClip` and `g_mPreviousViewToView` are.

## Matrix convention

From the CPU code (status `verified` for what the code does; the GPU side is a `lead`):

- `g_mWorldToView` is built from the 4x3 world-to-view with no transpose: `[m0 m1 m2 0; m3 m4 m5 0; m6 m7 m8 0; m9 m10 m11 1]` (the `1.0` is a constant loaded at `0x12cfb84`). The translation sits in floats 12 to 14, so the engine uses row vectors (`v * M`). In memory that is the same as a column-major matrix used as `mul(M, v)`.
- `g_mViewToClip` is the 4x4 view-to-clip converted element for element (`0x3b3de0`), with no transpose, so it has the same convention.
- `g_vViewPoint` is the translation row of view-to-world and `g_vViewDir` is its third row, `+Z`, not negated. So the camera looks down `+Z` in view space: a left-handed view space (inferred).

**Not done: the RenderDoc capture.** The issue asks for one capture under vkd3d-proton to read `g_mWorldToView` and `g_mViewToClip` from a world draw. It was not taken: the game cannot be driven to a world draw and captured unattended, and the research note's open question 5 (a clean-install hold for a graphics-bug investigation) is still open. So these stay `lead`: the packing of the constant buffer as the shaders read it (row- or column-major in HLSL), the projection's depth range and whether it is reversed-Z, and the jitter in view-to-clip.

## Counts

- `verified`: 7 hook rows (renderWorldInternal, the two `rend::View` setters, the two `m::View` setters, setViewToClip, the present function and call site), the double layout, and 6 float slots.
- `lead`: the FSR frame-generation swap chain path, the `rend::`/`m::` level of setViewToClip, the run-time proxy swap chain, 8 float slots, and the GPU-side matrix convention (needs the capture).
