# STAR WARS Zero Company (Steam 2075800): hook-point map

Step ZR (#285, ruling 52). It is the first map under [Hook-point research](../ROADMAP.md#hook-point-research-ruling-52), and it comes before Z2 (#32) and Z2b (#34). Every address below was read from the installed game on 2026-11-16 and is an RVA (offset from the image base `0x140000000`) in this one build only. Nothing was launched, written or downloaded.

## Header

| Field | Value |
|---|---|
| File name | `SWZeroCompany.exe` (`SWZeroCompany/Binaries/Win64/`, 267,189,392 bytes) |
| SHA-256 | `60831de4a410850efca453e62ba5acaf19a37ca37cb542202e72b8e2b56d1e6b` |
| File version | `5.6.1.0` (the exe's PE version resource) |
| Steam build id | `26445778` (the install's `appmanifest_2075800.acf`) |
| CL (game build string) | `201250`, from `++ProjectBruno+Stable-CL-201250` |
| Debug record | `SWZeroCompany.pdb`, GUID `6b3ed970-48df-69ce-914b-a167a168a600`, age 1; the PDB beside the exe has the same GUID and age |

The file version came from `pe_version` in `tools/re/exe_id.py`. `exe_id.py` itself exits 3 for this hash, because the hash is not in its `KNOWN_BUILDS` table; this PR does not add it.

## Method

Each hook was resolved by name from the shipped PDB, its bytes were read at that RVA from the exe on disk, and a pattern was written and counted over the whole exe file. Every pattern below matches exactly once in the whole file, at the RVA in its row.

- Names: `llvm-pdbutil dump -publics` on `SWZeroCompany.pdb` (1,238,096 public symbols), each address converted to an RVA with the exe's section table.
- Bytes: read from the exe's section data at the RVA and disassembled with Capstone (x86-64).
- Patterns: hex bytes, `??` is any byte. Every operand that holds a relative displacement (RIP-relative loads) is `??`, so a pattern survives a rebuild that moves data but keeps the code. A pattern holds whole instructions.
- The REA MCP tools (Ghidra provider) were tried first. Both calls on this 267 MB exe hit the provider's startup deadline (330 s), and Ghidra skipped the PDB ("failed to locate PDB file"), so they gave no result. The shared REA session also switched its open target to another game between the two calls. None of this map rests on REA output.

## Hooks

Status: `verified` = the PDB name, the bytes at the RVA and the whole-file pattern count (1) were all checked on this build. `lead` = not fully checked; no code uses a `lead`. No row here is a `lead`.

| Hook | RVA | Pattern (matches once) | How found | Status |
|---|---|---|---|---|
| `FFakeStereoRenderingDevice` stereo vtable (`IStereoRendering`) | `0xd8d5e80` | Anchor in `UEngine::InitializeHMDDevice`, where the device is built, at `0x797e16f`: `44 8B E0 48 8D 05 ?? ?? ?? ?? 48 89 07 48 8D 05 ?? ?? ?? ?? 48 89 47 08 E8 ?? ?? ?? ?? 48 8B D8 48 83 B8 10 01 00 00 00`. The first `lea` (anchor +3) is this vtable; the second (anchor +13) is the render target manager vtable at `0xd8d5e00`. | PDB public symbol `??_7FFakeStereoRenderingDevice@@6BIStereoRendering@@@`; the anchor is the `lea rax,[rip+...]; mov [rdi],rax` pair in `InitializeHMDDevice` that stores it. All slots read from the exe and each mapped back to its PDB name (llvm-pdbutil, bytes read, Capstone, 2026-11-16) | verified |
| slot 1 `IsStereoEnabled` | `0x4189fe0` | The vtable anchor above, then read the pointer in slot 1 (`vtable + 8`). The target holds `B0 01 C3` (`mov al,1; ret`). Those 3 bytes alone are not a pattern (other functions share the same code, which is why the PDB gives several names for this address), so the vtable anchor is the pattern. | PDB public symbol `?IsStereoEnabled@FFakeStereoRenderingDevice@@UEBA_NXZ`; slot 1 read from the vtable; bytes `B0 01 C3` read at the target (llvm-pdbutil, bytes read, 2026-11-16) | verified |
| slot 3 `EnableStereo` | `0x4189fe0` (the same code as slot 1) | As slot 1, with the pointer in slot 3 (`vtable + 24`). It holds the same `B0 01 C3`, so `EnableStereo(bool)` ignores its argument and returns true. | PDB public symbol `?EnableStereo@FFakeStereoRenderingDevice@@UEAA_N_N@Z`; slot 3 read from the vtable (llvm-pdbutil, bytes read, 2026-11-16) | verified |
| slot 11 `CalculateStereoViewOffset` | `0x797c950` | `40 55 56 57 41 56 48 81 EC B8 00 00 00 44 0F 29 5C 24 50 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 44 24 40` | PDB public symbol `?CalculateStereoViewOffset@FFakeStereoRenderingDevice@@UEAAXHAEAU?$TRotator@N@Math@UE@@MAEAU?$TVector@N@34@@Z`; the vtable slot 11 pointer is the same address (llvm-pdbutil, bytes read, Capstone, 2026-11-16) | verified |
| slot 12 `GetStereoProjectionMatrix` | `0x797cda0` | `40 53 48 83 EC 60 F3 0F 10 1D ?? ?? ?? ?? 48 8B DA F3 0F 10 41 10 0F 28 D3 F3 0F 10 69 14 F3 0F 10 49 18` | PDB public symbol `?GetStereoProjectionMatrix@FFakeStereoRenderingDevice@@UEBA?AU?$TMatrix@N@Math@UE@@H@Z`; the vtable slot 12 pointer is the same address (llvm-pdbutil, bytes read, Capstone, 2026-11-16) | verified |
| `UEngine::InitializeHMDDevice` | `0x797d700` | `40 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 08 FD FF FF 48 81 EC F8 03 00 00 0F 29 B4 24 E0 03 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 85 D0 02 00 00 80 3D ?? ?? ?? ?? 00` | PDB public symbol `?InitializeHMDDevice@UEngine@@MEAA_NXZ`; the device constructor is inlined here, and it stores the two vtables at `0x797e172` (llvm-pdbutil, bytes read, Capstone, 2026-11-16) | verified |
| `FSlateRHIRenderer::DrawWindow_RenderThread` | `0x627d610` | `40 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 88 FD FF FF 48 81 EC 78 03 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 85 F0 01 00 00` | PDB public symbol `?DrawWindow_RenderThread@FSlateRHIRenderer@@AEAA?AUFSlateDrawWindowPassOutputs@@AEAVFRDGBuilder@@AEBUFSlateDrawWindowPassInputs@@@Z` (llvm-pdbutil, bytes read, Capstone, 2026-11-16) | verified |
| Game build (CL), the string | `0xcd541a0` | UTF-16 text `++ProjectBruno+Stable-CL-201250`: `2B 00 2B 00 50 00 72 00 6F 00 6A 00 65 00 63 00 74 00 42 00 72 00 75 00 6E 00 6F 00 2B 00 53 00 74 00 61 00 62 00 6C 00 65 00 2D 00 43 00 4C 00 2D 00 32 00 30 00 31 00 32 00 35 00 30 00` | Searched the exe for the build string (no PDB symbol; the text is in `.rdata`). One match in the whole file, and none as 8-bit text (bytes read, 2026-11-16) | verified |
| Game build (CL), the number in code | `0x8bff410` (`UBitReactorCoreStatics::execGetChangelist`; the constant is the `mov edx` at `+0x2F`) | `40 53 48 83 EC 30 48 8B 4A 20 49 8B D8 45 33 C0 48 85 C9 4C 89 44 24 20 41 8B C0 4C 89 44 24 28 0F 95 C0 48 03 C1 48 8D 4C 24 20 48 89 42 20 BA 22 12 03 00` (the last 4 bytes are 201250) | PDB public symbol `?execGetChangelist@UBitReactorCoreStatics@@SAXPEAVUObject@@AEAUFFrame@@QEAX@Z`; searched the code for the 4-byte constant 201250 and took the hit inside the game's own changelist function (llvm-pdbutil, bytes read, Capstone, 2026-11-16) | verified |

### The CL, and what the UEVR fork checks

The two CL rows say where this build holds its own number. They do not show which source the `joeyhodge/UEVR` fork reads, and this map does not claim that. What was read, in the fork's `src/mods/vr/SWZeroCompanyBinary.hpp` (branch `ue57performance`, fetched 2026-11-16):

- Its table has four entries (`revision` 196320, 196985, 197649, 199049). None is 201250.
- A `revision` is a label for a set of addresses and byte checks. The code picks a layout by matching its byte arrays against the running exe (`select_unique_layout`), and its notes say unknown or ambiguous revisions are not admitted. So the gate is the bytes, and the CL number is not itself the test.
- In the installed build, the `DrawWindow_RenderThread` argument bytes the fork expects at `+0x2D` (`4D 8B 79 18 4D 8B E9 49 8B 41 08 4D 8B F0 48 89 95 80 00 00 00`) and at `+0x45` (`48 89 4D 18`) are present (at `0x627d63d` and `0x627d655`), and the `B0 01 C3` stereo stub is present. The fork's 28-byte `InitializeHMDDevice` prologue is present at `0x797d700`, but it also matches 14 other functions, so it cannot find the function alone. The fork's addresses are another build's and are not valid here (below).

## Checklist

`agrees` = the checklist value is what this build has. `differs` = it is not. A checklist value is never a source (ruling 52).

### The research note's table (CL 201250)

| Checklist item | Note's value | Found | |
|---|---|---|---|
| `UEngine::InitializeHMDDevice` | `0x797d700` | `0x797d700` | agrees |
| Fake stereo device vtable (stereo rendering) | `0xd8d5e80` | `0xd8d5e80` | agrees |
| Fake stereo device vtable (render target manager) | `0xd8d5e00` | `0xd8d5e00` | agrees |
| `IsStereoEnabled` / `EnableStereo` (slots 1 and 3) | `0x4189fe0`, bytes `b0 01 c3` | `0x4189fe0`, bytes `B0 01 C3` | agrees |
| `GetDesiredNumberOfViews` (slot 4) | `0x797c6d0` | `0x797c6d0` | agrees |
| `AdjustViewRect` (slot 8) | `0x797c700` | `0x797c700` | agrees |
| `SetFinalViewRect` (slot 9) | `0x797c750` | `0x797c750` | agrees |
| `CalculateStereoViewOffset` (slot 11) | `0x797c950` | `0x797c950` | agrees |
| `GetStereoProjectionMatrix` (slot 12) | `0x797cda0` | `0x797cda0` | agrees |
| `RenderTexture_RenderThread` (slot 14) | `0x797d0c0` | `0x797d0c0` | agrees |
| `FSlateRHIRenderer::DrawWindow_RenderThread` | `0x627d610` | `0x627d610` | agrees |
| `GSystemResolution` | `0xf100900` | `0xf100900` | agrees |
| Game build (CL) | `201250` | `201250` | agrees |
| Steam build id | `26445778` | `26445778` | agrees |
| SHA-256, PDB GUID | as in the header | as in the header | agrees |

Slots 4, 8, 9 and 14 were read from the same vtable. They have no pattern here because no step lists them yet.

### The `joeyhodge/UEVR` table, entry CL 199049 (its newest)

The fork has no entry for CL 201250. Its newest entry is for a different build, so its addresses are expected to differ; that is not a fault in the fork.

| Checklist item | Fork's CL 199049 value | Found (CL 201250) | |
|---|---|---|---|
| Stereo vtable (`primary_rva`) | `0xD8D33E0` | `0xd8d5e80` | differs |
| Render target manager vtable (`secondary_rva`) | `0xD8D3360` | `0xd8d5e00` | differs |
| `InitializeHMDDevice` | `0x797D070` | `0x797d700` | differs |
| Slot 1 `IsStereoEnabled` | `0x4189F20` | `0x4189fe0` | differs |
| Slot 11 `CalculateStereoViewOffset` | `0x797C2C0` | `0x797c950` | differs |
| Slot 12 `GetStereoProjectionMatrix` | `0x797C710` | `0x797cda0` | differs |
| `DrawWindow_RenderThread` (`slate_rva`) | `0x627CBE0` | `0x627d610` | differs |
| `DrawWindow_RenderThread` argument bytes at `+0x2D` and `+0x45` | same bytes | same bytes | agrees |
| `IsStereoEnabled` code | `B0 01 C3` | `B0 01 C3` | agrees |

In this table the code the fork checks has the same shape in CL 201250 and only the addresses moved. Whether the fork's patch would admit this build is not shown here; Z2 tests it.

## Notes for Z2 and Z2b

- Slots 1 and 3 point at one shared stub, and `EnableStereo` ignores its argument. Other functions share that code, so replacing the stub itself would change them too.
- The vtable's slots are absolute pointers in the file, relocated at load. Find the table with the anchor row, not by searching for pointer bytes.
