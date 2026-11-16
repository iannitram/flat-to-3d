# Control (Steam 870780): hook-point map (GR, #287)

Ruling 52 map for the Geo-11 track. Geo-11 and the community fix need no code hooks of ours, so the hooks here are the D3D11 entry Geo-11's proxy `d3d11.dll` takes and the shaders the fix replaces. Researched 2026-11-16 on the installed game (SteamOS); the game folder was read, never written. Every address below comes from our own analysis of the installed files; the research note ([870780-control.md](../notes/research/games/870780-control.md)) is only the checklist column.

## Header

| Field | `Control_DX11.exe` | `d3d_rmdwin7_f.dll` |
|---|---|---|
| File name | `Control_DX11.exe` (original name `Game_rmdwin7_f.exe`) | `d3d_rmdwin7_f.dll` |
| SHA-256 | `0b0d1023d13ccf532d58374f320ea67ee9517465e7015cf4e5e29156afd03160` | `8811716b71ec719c666142a371f8205bb20fc42f2349e4f076437824053ced90` |
| File version | 0.0.518.2177 | 0.0.518.2177 |
| Link time stamp | 2025-03-23 20:25:45 UTC | 2025-03-23 20:17:52 UTC |
| Image base (x64) | `0x140000000` | `0x180000000` |
| Steam build id | 21225456 (`appmanifest_870780.acf`) | same install |

RVAs are relative to the image base of the file named in the row. "Pattern" is the byte pattern in file order, `??` is a wildcard; each was checked to match exactly once in its file.

## Hooks

Tools: `pefile` (imports, sections, exports, version resource), `capstone` (disassembly of `.text`, scan for `call [rip+disp]` against the IAT, `.pdata` function bounds), Python byte search for uniqueness. No Ghidra: the questions were small enough for these. Method date for all rows: 2026-11-16.

| # | What it is | Used by | RVA | Unique byte pattern | How found | Status | Checklist |
|---|---|---|---|---|---|---|---|
| H1 | Import of `d3d11.dll` by `d3d_rmdwin7_f.dll`: exactly two functions, `D3D11CreateDevice` and `D3D11CreateDeviceAndSwapChain` (import-name strings at `0xAC16E`, `0xAC18E`; DLL name at `0xAC1A0`) | G1: the file Geo-11's proxy `d3d11.dll` replaces | name `0xAC1A0` | ASCII `d3d11.dll` + NUL (one hit in the file) | `pefile` import table | verified | agrees (note: "imports `d3d11.dll`, calls both") |
| H2 | IAT slot of `D3D11CreateDeviceAndSwapChain` | G1 | `0x3F2A8` | the 29-byte name string `D3D11CreateDeviceAndSwapChain` + NUL (one hit) | `pefile` import table | verified | agrees |
| H3 | IAT slot of `D3D11CreateDevice` | G1 | `0x3F2A0` | the name string `D3D11CreateDevice` + NUL at `0xAC18E` (one hit; the NUL ends it, so H2's longer name does not match) | `pefile` import table | verified | agrees |
| H4 | The only call site of `D3D11CreateDeviceAndSwapChain` (`call [rip+0x21E05]`): hardware driver type, no adapter, feature levels {11_0, 9_1}, SDK version 7 | G1: where the swap chain the proxy wraps is born | `0x1D49D` | `89 74 24 38 C7 44 24 30 07 00 00 00 C7 44 24 28 02 00 00 00 48 8D 44 24 70 48 89 44 24 20 45 8B CC 45 33 C0 41 8D 50 01 33 C9 FF 15 ?? ?? ?? ??` (from `0x1D473`) | capstone scan of `.text` for `FF 15` calls whose target is the IAT slot (H2) | verified | agrees |
| H5 | The only call site of `D3D11CreateDevice` (`call [rip+0x21CFE]`): same arguments, no swap chain | G1: used when no window handle is passed | `0x1D59C` | `89 5C 24 38 C7 44 24 30 07 00 00 00 C7 44 24 28 02 00 00 00 48 8D 44 24 70 48 89 44 24 20 45 8B CC 45 33 C0 41 8D 50 01 33 C9 FF 15 ?? ?? ?? ??` (from `0x1D572`) | same scan against the slot of H3 | verified | agrees |
| H6 | The device-creation routine holding H4 and H5 (not exported; `.pdata` range `0x1D230` to `0x1D75C`). It picks the swap-chain call when the output window handle (stored at `+0x50` of its swap-chain description) is non-null and the plain call otherwise; sets the debug flag when bit `0x10` of its flag argument is set. The swap-chain path then calls `IDXGISwapChain::GetParent` and `IDXGIFactory::MakeWindowAssociation` through vtables | G1: the one place the engine makes its D3D11 device | `0x1D230` | `4C 8B DC 57 41 54 41 55 41 56 41 57 48 81 EC E0 01 00 00` (one hit) | `.pdata` bounds around H4 and H5, then disassembly | verified | new (no note row) |
| H7 | Its only caller, `call 0x1D230` inside the wrapper at `0x1EFC0` | G1 (context only) | `0x1F024` | `17 48 8B C8 E8 ?? ?? ?? ?? 90 48 8B 5C 24 68 48 83` (from `0x1F020`) | capstone scan for `E8`/`E9` to `0x1D230` | verified | new |
| H8 | The four failure strings the routine logs: `D3D11CreateDeviceAndSwapChain failed` (`0x44F60`), `IDXGISwapChain:GetParent failed` (`0x44F88`), `IDXGIFactory:MakeWindowAssociation failed` (`0x44FA8`), `D3D11CreateDevice failed` (`0x44FD8`) | G1: lines to look for in the Proton log if device creation fails | `0x44F60` | the ASCII text itself (each one hit) | string references from the routine's `lea rcx,[rip+disp]` before each log call | verified | new |
| H9 | Absence: no import of `dxgi.dll`, `nvapi.dll`, `nvapi64.dll` or `d3d11.dll` by `Control_DX11.exe`, nor by any of the 17 `*rmdwin7*` DLLs in the folder except `d3d_rmdwin7_f.dll` (which imports `d3d11.dll` only). No file of the DX11 set holds the text `dxgi.dll`, `nvapi*.dll`, `nvapi_QueryInterface`, `CreateDXGIFactory*` | G1: the plain case a proxy `d3d11.dll` is built for; nothing else to wrap | none | not applicable (absence) | `pefile` import tables for the exe and all 17 `*rmdwin7*` DLLs, plus a raw byte search of each file | verified | agrees (note: "does not import `dxgi.dll` or `nvapi64.dll`") |
| H10 | Contrast: `d3d_rmdwin10_f.dll` (DX12 build) imports `d3d12.dll` and `dxgi.dll` (`CreateDXGIFactory1`) | G1 (why `-dx11` is needed) | none | not applicable | `pefile` import table | verified | agrees |
| H11 | Shipping state: no PDB; the exe names `W:\p7\main\code\remedy\bin\v141\RmdWin7\Final\Game_rmdwin7_f.pdb`; no Steam DRM wrapper section (sections `.text .rdata .data .pdata .rsrc .reloc`) | Context | none | not applicable | `pefile` sections, string search | verified | agrees |
| H12 | Install facts the note states: Steam build id 21225456, file version 0.0.518.2177, exe link time stamp 2025-03-23 | Header | none | not applicable | `appmanifest_870780.acf`, version resource, PE header | verified | agrees |
| H13 | Local shader store, the static source for the shader comparison: `data/shaders/build/pc_dx11/` holds 93 `.obj` files with 8,620 `DXBC` blobs in the clear (`pc_dxil/` is the DX12 set) | the shaders section | n/a (folder) | `44 58 42 43` (`DXBC`) at each blob start | file listing and byte count | lead: 1,448 blobs hashed, none tie to a fix name (see the shaders section) | new |

## The D3D11 entry point

Geo-11's proxy `d3d11.dll` takes the entry `d3d_rmdwin7_f.dll` already asks the system for: the import of `D3D11CreateDevice` (IAT `0x3F2A0`) and `D3D11CreateDeviceAndSwapChain` (IAT `0x3F2A8`), both called only from the routine at `0x1D230` (calls at `0x1D59C` and `0x1D49D`). Found by listing `d3d_rmdwin7_f.dll`'s import table, scanning `.text` for indirect calls through those two slots, and disassembling the routine around them. A proxy named `d3d11.dll` in the game folder answers the import under Windows' normal search order, and under Proton (note, section 3). Nothing else is needed: neither the exe nor any DX11 engine DLL imports `dxgi.dll` or `nvapi64.dll`, so no second DLL has to be handled. Both calls use the hardware driver, a null adapter and feature levels {11_0, 9_1}; the engine then reaches DXGI only through the swap chain's vtable (`GetParent`, `MakeWindowAssociation`). Not tested at run time: whether Geo-11 loads under Proton is G1's job.

## The fix's shaders

Fix 1.01 archive, fetched 2026-11-16 into a scratch folder and read only (never run, never extracted into the game folder): `Control_3D-VisionReadyFix_1.01_by_Losti_Masterotaku.zip`, 3,296,439 bytes (matches the size in the research note), SHA-256 `c5da0ef29396a8b819b830ac022a7057d6b9ad9371271e85e67f3a37d43f410d`. 79 entries; `ShaderFixes/` carries 38 distinct 16-hex-digit hash names (in `ShaderFixes/`, `Fixed/`, `BlindAdded/` and `velocity/`), and its `d3dx.ini` names 11 hashes in `[ShaderOverride*]` sections, 3 of them not among the file names (`51a0e7e40feb54d8`, `8e5f898b6e17813c`, `c8ec58c11309d727`). The fix's hash list is therefore 41 hashes. The fix also uses `Control_ShaderRegEx_Losti.ini` (pattern matches, no hashes), which a changed build can still satisfy.

| Item | Count |
|---|---|
| Fix 1.01 hashes (the list to match) | 41 |
| Matched by the current build's shipped blobs | not established (lead) |
| Missing | not established (lead) |

The game's shipped DX11 shader store, `data/shaders/build/pc_dx11/` (93 `.obj` files), holds 1,448 distinct complete `DXBC` blobs (each `DXBC` magic with version 1 and a size that fits the file; none rejected).

**Algorithm: lead, not proven.** The fix's own `d3dx.ini` says `shader_hash = 3dmigoto`, "unseeded software FNV-1". I implemented 64-bit FNV-1 (checked on the input `a`: `af63bd4c8601b7be`, the published test value) and hashed every blob as the whole blob, without the 32-byte header, the `SHEX` chunk alone, all chunk bodies, and with the checksum zeroed; each also with FNV-1a, plus the first and second half of the embedded checksum as the "embedded" mode. Result: 0 of 41 hashes matched in every variant. No blob could be tied to any fix file name, so the algorithm is not proven on a pair, and "0 matched, 41 missing" would be a guess: it is equally what a wrong algorithm or a blob store that differs from what the game hands `CreateShader` would give. A structural tie by signature and opcode sequence (for `0f85c5964dc694e8-vs.txt`) found no blob with the same declarations either, which fits the shaders having changed since 2019 but does not prove it.

Hashes still unresolved (all 41): 07bbcbe1081b2e36 0c244087e65a8216 0eb60eb0900b8b83 0f85c5964dc694e8 18871c97b0d1329d 2080fecb3bc6e5ce 231b52404fd8c808 258ceaf3cbbbb628 25cfd289ef18d6bd 3080543ba7c4fbca 358ef8dd04db5b13 43215a37c5f86d02 4329e81f4a8dbc2e 43dddbaa9dd3c724 492f2b4d2e5525e0 51a0e7e40feb54d8 658da279b93f8a97 697e1271a2c7e362 6d3b0d226e3088f3 6fd060fea0abbec4 753d2f895722e9cc 75887bd2c74cb817 7b3ac696247a4fee 8e5f898b6e17813c 91f7ac3ed6076ad8 9388cfb4f8322e47 97ee73b31d6636a0 984e68b076e9682a a82f71377859114d bac044a8fd814d90 be918c8aa24d489d c7082e41037b226f c84aa5f5375fd55c c8ec58c11309d727 ce87520b1da360f9 ceaea1e776139ae5 d783b038c947682d da8e42f518ed6870 f33aa45fe10f486e fc040d831798c94d fd964e83e6606243

What is left (a run is now needed): one run of `Control_DX11.exe` under Proton with 3DMigoto's shader dump on a copy of the install, which gives the game's real runtime hashes; comparing them with the 41 settles both the algorithm (any match proves it) and the counts. Needs `gpu.lock`; the dump must not touch the game folder.

## Checklist against the research note

| Note claim | Result |
|---|---|
| File version `0.0.518.2177` for exe and renderer DLL | agrees (exe, `d3d_rmdwin7_f.dll` and `renderer_rmdwin7_f.dll` read) |
| Steam build id 21225456 | agrees |
| Exe link time stamp 2025-03-23 | agrees |
| `Control_DX11.exe` imports only `rmdwin7` DLLs | agrees (plus `CoherentUIGT.dll` and system DLLs) |
| `d3d_rmdwin7_f.dll` imports `d3d11.dll`, calls `D3D11CreateDevice` and `D3D11CreateDeviceAndSwapChain` | agrees (H1 to H5) |
| No `dxgi.dll` or `nvapi64.dll` import | agrees (H9) |
| `d3d_rmdwin10_f.dll` imports `d3d12.dll` and `dxgi.dll` | agrees (H10) |
| No Steam DRM wrapper section; PDB path stripped | agrees (H11) |
| Fix 1.01's shaders still load on this build (from user reports) | not checked: no hash ties to a shipped blob; run needed |
