> Raw research notes from 2026-10-18, kept as written. Where they conflict with [DECISIONS.md](../DECISIONS.md) (for example paid-tool routes), the decisions win. Some paths and names were generalized when these moved into the repo.

# Library engine scan: true-stereo potential of every installed game (Z13, SteamOS, Proton), October 2026

Method: on 2026-10-18 I read every install folder in `library-scan.tsv` read-only on the Z13. I checked file signatures (UnityPlayer.dll/.so, GameAssembly.dll, `*_Data/Managed`, MonoBleedingEdge, `Engine/Binaries`, `*.pak`, `data.win`, `*.pck`, `FNA.dll`/`FNA3D.dll`, `*.xnb`, `love.dll`, `renpy/`, `D3D12/` agility folders). I read Unity versions from `*_Data/globalgamemanagers` headers, read Unreal versions from the `++UE4+Release-4.xx` string in `*-Win64-Shipping.exe`, and read graphics API imports with `objdump -p` on the main exe. I also searched every tree for `EasyAntiCheat*`, `BattlEye*` and `BEClient*` files, plus `EOSSDK*` files, which are Epic Online Services and not an anti-cheat in themselves. I cross-checked against the "Engine / API" line of each page in `my per-game notes` and used web sources for the tools. PCGamingWiki and its API returned HTTP 403 and SteamDB was not reachable this session, so those were not used.

Local evidence paths: install folders listed in `docs/notes/library-scan.tsv`; my per-game notes.

Verdict key:
- **works-now**: a free, maintained stereo injector already supports this engine and version, so only the "show it on a 3D TV in the Quest" glue is missing.
- **likely**: the engine and API fit a free tool, but this exact title is unconfirmed, or the tool is beta.
- **build**: a real 3D world with no free stereo tool for its API, so we build our own.
- **not-safe**: online-only or anti-cheat.
- **wont-work**: flat 2D art.

## Q1. Per-game classification of all 98 rows in library-scan.tsv (plus ACE COMBAT 7)

### Takeaway
Of the 98 scanned rows, 50 are flat 2D art and won't work. The 3D-world games split into 5 that existing tools cover now (4 UE4 games via UEVR, plus Risk of Rain 2), 35 likely (mostly Unity DX11 games that fit Geo-11 or a Unity VR injector, two UE5 games for UEVR, and DX9 games via dgVoodoo2→Geo-11), 7 that need a stereo layer we build ourselves (mostly DX12-only), and 1 not-safe (MTG Arena, always-online). No install contains EasyAntiCheat or BattlEye files. Five games have no per-game notes: Construction Simulator, Keylocker, Ereban: Shadow Legacy, CraftCraft and Baldur's Gate II EE.

### Cited Findings

"No notes" = no matching entry in my per-game notes (name match after normalising punctuation). Style: **3D** = true 3D world; **2.5D** = 3D-rendered but side-on or diorama; **2D** = flat 2D art. Online flag: "none" means no EAC/BattlEye files and no known online requirement. "EOS" means the EOS SDK ships (services such as crossplay or accounts, not anti-cheat). (*) marks the 10 Favorites, which get brief entries only.

| Game | AppID | Engine (+ver) | Style | API | Online / AC | Best stereo route | Verdict |
|---|---|---|---|---|---|---|---|
| NORCO | 1221250 | unknown (folder missing: the manifest says installed, but `common/NORCO` is absent) | 2D pixel art | DX9 min req | none | — | wont-work |
| STAR WARS Zero Company (*) | 2075800 | Unreal 5.6 (my game notes) | 3D | DX12 only | none found | UEVR nightly (UE 5.6 is past UEVR's stated 4.8–5.4; untested) | likely |
| Forza Horizon 6 (*) | 2483190 | ForzaTech | 3D | DX12 | first launch needs sign-in, then offline play works; no AC files | build our own (DX12) | build |
| Marvel's Spider-Man 2 (*) | 2651280 | Insomniac engine (Nixxes) | 3D | DX12 | none | build our own (DX12) | build |
| 007 First Light (*) | 3768760 | IOI Glacier | 3D | DX12 (exe imports d3d12) | none found | build our own (DX12) | build |
| Control Ultimate Edition (*) | 870780 | Remedy Northlight | 3D | DX11 exe and DX12 exe ship | none | Geo-11 on `Control_DX11.exe` | likely |
| Unrailed! | 1016920 | FNA (C#), Content/*.xnb | 3D (low-poly voxel) | FNA3D (D3D11/Vulkan/GL) | online co-op optional | build our own (FNA is open source; patch the camera) | build |
| The Procession to Calvary | 1071130 | Unity 2019.4.18f1 Mono, **native Linux build installed** | 2D cut-out | GL (native) | none | — | wont-work |
| Mini Motorways | 1127500 | Unity 2022.3.62f2 Mono | 2.5D (3D-rendered map) | D3D11 | EOS | Geo-11 or Unity VR injector (low depth value) | likely |
| Monaco | 113020 | custom (Pocketwatch) | 2D top-down | DX9 | none | — | wont-work |
| Hades II (*) | 1145350 | Supergiant engine | 2D art | DX12 | none | — (my ruling) | wont-work |
| Hades (*) | 1145360 | Supergiant engine | 2D art | DX11 / Vulkan build | EOS | — (my ruling) | wont-work |
| I Was a Teenage Exocolonist | 1148760 | Unity 2021.2.15f1 Mono, native Linux | 2D illustrated | GL (native) | none | — | wont-work |
| Golf Gang | 1151050 | Unity 2020.3.31f1 Mono | 3D | D3D11 | **online-multiplayer focused** (no AC files) | Geo-11 / BepInEx Unity VR injector, offline modes only | likely |
| FINAL FANTASY VI | 1173820 | Unity 2019.4.26f1 IL2CPP | 2D pixel | D3D11 | none | — | wont-work |
| OlliOlli World | 1190170 | Unity 2020.3.22f1 IL2CPP | 2.5D | D3D11 | online leaderboards | Geo-11 or UnityVRMod-IL2CPP | likely |
| Sea of Stars | 1244090 | Unity 2021.3.25f1 IL2CPP | 2D pixel | D3D11 | none | — | wont-work |
| Rogue Legacy 2 | 1253920 | Unity 2020.3.47f1 Mono, native Linux | 2D | GL (native) | EOS | — | wont-work |
| Construction Simulator **(no notes)** | 1273400 | Unity 2020.3.49f1 IL2CPP | 3D | D3D11 (d3d9/WinPix also ship) | online co-op optional | Geo-11 or UnityVRMod-IL2CPP | likely |
| Loop Hero | 1282730 | GameMaker (data.win) | 2D | D3D11 | none | — | wont-work |
| Keylocker **(no notes)** | 1325040 | GameMaker (data.win) | 2D pixel | D3D11 | none | — | wont-work |
| Kingdom Rush Vengeance | 1367550 | custom (Ironhide), glew | 2D | OpenGL | none | — | wont-work |
| ANNO: Mutationem | 1368030 | Unity 2021.3.45f2 IL2CPP | 2.5D (pixel sprites in 3D sets) | D3D11 | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| Tactics Ogre: Reborn | 1451090 | Square Enix in-house | 2.5D (3D maps, 2D sprites) | D3D11 | none | Geo-11 | likely |
| DREDGE | 1562430 | Unity 2021.3.5f1 Mono, **32-bit** | 3D | D3D11 | none | Geo-11 (x32) or BepInEx Mono VR mod | likely |
| Blue Prince | 1569580 | Unity 6000.0.58f2 IL2CPP | 3D first-person | D3D11 (Player.log) | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| Until Then | 1574820 | Godot 4.1.4 custom | 2D pixel | Vulkan / DX12 | none | — | wont-work |
| V Rising | 1604030 | Unity 2022.3.58f1 IL2CPP | 3D (iso camera) | D3D11 | EOS; official servers online, private/local play offline; no EAC files | Geo-11 or UnityVRMod-IL2CPP, offline only | likely |
| Planet of Lana (*) | 1608230 | Unity 2020.3.41f1 IL2CPP | 2.5D | D3D11 | none | Geo-11 / UnityVRMod-IL2CPP | likely |
| Core Keeper | 1621690 | Unity 6000.0.59f2 Mono, **native Linux installed** | 2.5D (top-down 3D tiles, pixel sprites) | GL/Vulkan native | EOS, co-op optional | switch to the Windows build, then Geo-11 (low depth value) | likely |
| Nocturnal | 1634080 | Unity 2021.3.45f2 IL2CPP | 2D | D3D11 | EOS | — | wont-work |
| Dome Keeper | 1637320 | Godot 4, native Linux | 2D | GL | PlayFab libs | — | wont-work |
| Ereban: Shadow Legacy **(no notes)** | 1671360 | Unity 2022.3.7f1 IL2CPP | 3D | D3D11 assumed (ships Vulkan renderer plugin, DLSS) | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| BAD END THEATER | 1764390 | Ren'Py | 2D | OpenGL | none | — | wont-work |
| Hell Clock | 1782460 | Unity 6000.0.60f1 Mono | 3D (iso ARPG) | D3D11 (D3D12 folder ships) | EOS | Geo-11 or BepInEx Mono VR injector | likely |
| Vampire Survivors | 1794680 | Unity 6000.0.62f1 IL2CPP | 2D | D3D11 | EOS | — | wont-work |
| Sorry We're Closed | 1796580 | Unity 2021.3.45f1 Mono | 3D (fixed-camera, PS1-style) | D3D11 | EOS | Geo-11 or BepInEx Mono VR injector | likely |
| Nine Sols | 1809540 | Unity 2022.3.62f3 Mono | 2D hand-drawn | D3D11 | none | — | wont-work |
| DAVE THE DIVER | 1868140 | Unity 6000.0.52f1 IL2CPP | 2.5D (pixel sprites, 3D backgrounds) | D3D11 | EOS | Geo-11 (low depth value) | likely |
| Paper Trail | 1889740 | Unity 2020.3.48f1 IL2CPP | 2D | D3D11 | none | — | wont-work |
| Absolum | 1904480 | FNA (FNA3D.dll) | 2D hand-drawn | D3D11 | EOS | — | wont-work |
| Legends of Kingdom Rush | 1904860 | Unity 2020.3.18f1 Mono | 2D | D3D11 | none | — | wont-work |
| Garden Life | 1915380 | **Unreal Engine 5** (my notes say "likely Unity"; wrong, files show `Engine/` + `GardenLife-Win64-Shipping.exe`) | 3D | DX12 default (`D3D12/` folder; `-d3d11` option) | none | UEVR (UE5 minor version unread) | likely |
| Harmony: The Fall of Reverie | 1966420 | Unreal 5.0.3 | 2D illustrated VN | DX11 | none | — | wont-work |
| OCTOPATH TRAVELER II | 1971650 | **Unreal 4.27** (exe string) | 2.5D HD-2D dioramas | DX11 | none | UEVR | works-now |
| Blanc | 1982340 | Unity 2021.3.0f1 IL2CPP | 2D hand-drawn | D3D11 | EOS | — | wont-work |
| Boxes: Lost Fragments | 2019810 | Unity 2021.3.45f2 IL2CPP | 3D | D3D11 | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| Castle Crashers | 204360 | Behemoth custom | 2D | DX9 | online co-op optional | — | wont-work |
| BALL x PIT | 2062430 | Unity 6000.0.62f1 IL2CPP | 2D pixel | D3D11 | none | — | wont-work |
| To the Moon | 206440 | RPG Maker XP (mkxp), native Linux | 2D | OpenGL | none | — | wont-work |
| Tavern Talk | 2076140 | Unity 6000.0.68f1 IL2CPP | 2D VN | D3D11 | none | — | wont-work |
| StarVaders | 2097570 | Unity 2020.3.49f1 IL2CPP | 2D pixel | D3D11 | none | — | wont-work |
| Sektori | 2105620 | Unity 6000.0.69f1 IL2CPP | 2.5D (3D-rendered arena) | D3D11 | none | Geo-11 (low depth value) | likely |
| SteamWorld Build | 2134770 | Unity 2021.3.5f1 IL2CPP | 3D | D3D11 (Player.log) | EOS | Geo-11 or UnityVRMod-IL2CPP | likely |
| Sifu | 2138710 | **Unreal 4.26** (exe string) | 3D | DX11 | none | UEVR | works-now |
| Magic: The Gathering Arena | 2141910 | Unity 6000.3.14f1 Mono | 2D/3D board | D3D11 | **always-online** | — | not-safe |
| Little Nightmares EE | 2149010 | **Unreal 4.27** (exe string) | 2.5D/3D | DX11 | none | UEVR | works-now |
| Tiny Glade | 2198150 | custom Rust/wgpu, native Linux | 3D | Vulkan | none | build our own (Vulkan layer) | build |
| Thomas Was Alone | 220780 | Unity 4.3.1f1 Mono, native Linux 32-bit | 2D | GL | none | — | wont-work |
| The Cave | 221810 | Double Fine custom | 2.5D (3D-rendered) | DX9 | none | dgVoodoo2 (DX9→DX11) + Geo-11, experimental | likely |
| CraftCraft **(no notes)** | 2226430 | Godot (godotsteam, .pck) | 2D (unverified) | OpenGL | none | — | wont-work |
| Station to Station | 2272400 | **Unreal 4.27** (exe string) | 3D diorama | DX11 | none | UEVR | works-now |
| Cat Quest III | 2305840 | Unity 2022.3.14f1 Mono | 2.5D (3D world, flat characters) | D3D11 | EOS | Geo-11 or BepInEx Mono VR injector | likely |
| Balatro | 2379780 | LÖVE | 2D | OpenGL | none | — | wont-work |
| Cabernet | 2401410 | Unity 2022.3.36f1 Mono | 2D | D3D11 | EOS | — | wont-work |
| Neva | 2420660 | Unity 2022.3.50f1 Mono | 2D painterly (layered) | D3D11 | none | — | wont-work |
| Rayman Legends | 242550 | UbiArt | 2D | DX9 | Ubisoft Connect stub | — | wont-work |
| TowerFall Ascension | 251470 | FNA (XNA) | 2D | OpenGL | none | — | wont-work |
| Lost Eidolons: Veil of the Witch | 2530490 | Unity 2022.3.62f2 IL2CPP | 3D tactics | D3D11 | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| GUNTOUCHABLES | 2543510 | Unity 2022.3.15f1 Mono | 3D top-down | D3D11 | online co-op focused (no AC files) | Geo-11 or BepInEx Mono VR injector, solo | likely |
| Hyper Light Drifter | 257850 | GameMaker | 2D | D3D11 | none | — | wont-work |
| Ballionaire | 2667120 | Godot 4, native Linux | 2D | GL/Vulkan | none | — | wont-work |
| BADLAND GOTY | 269670 | custom (Frima), native Linux | 2D | OpenGL | none | — | wont-work |
| Urban Jungle | 2744010 | Unity 2022.3.49f1 Mono | 3D diorama | D3D11 | none | Geo-11 or BepInEx Mono VR injector | likely |
| Beholder: Conductor | 2825530 | Unity 2022.3.62f3 IL2CPP | 3D (unverified) | D3D11 | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| Death Howl | 2825880 | Unity 2022.3.62f2 Mono | 2D hand-drawn | D3D11 | none | — | wont-work |
| Kingdom Rush 5: Alliance TD | 2849080 | LÖVE | 2D | OpenGL | none | — | wont-work |
| Mini Metro | 287980 | Unity 2022.3.62f2 Mono | 2D | D3D11 | EOS | — | wont-work |
| TMNT: Splintered Fate | 2996040 | custom (no Unity files; my notes' "Unity/GDK" is wrong) | 3D iso | D3D12 + OpenGL imports | none | build our own (DX12) | build |
| Geometry Wars 3 | 310790 | custom (Aspyr), **native Linux installed** | 2.5D/3D arenas | GL native; Windows build is D3D11 | none | install the Windows build under Proton, then Geo-11 | likely |
| Puzzle Quest: Immortal Edition | 3236710 | Unity 6000.0.38f1 IL2CPP | 2D | D3D11 | EOS | — | wont-work |
| Vampire Crawlers | 3265700 | Unity 6000.0.62f1 IL2CPP | 3D first-person crawler | D3D11 | none | Geo-11 or UnityVRMod-IL2CPP | likely |
| Tales from the Borderlands | 330830 | Telltale Tool | 3D | DX9 | none | dgVoodoo2 + Geo-11, experimental | likely |
| Passant: A Chess Roguelike | 3353100 | Unity 6000.0.81f1 IL2CPP | 3D board (unverified) | D3D11 | none | Geo-11 (low value) | likely |
| Lara Croft and the Guardian of Light | 35130 | Crystal Dynamics custom | 3D iso | DX9 (exe imports hidden, probably packed) | none | dgVoodoo2 + Geo-11, experimental | likely |
| Stikbold! | 429330 | Unity 5.3.5f1 Mono (32-bit player) | 3D arena | D3D11 | local MP | Geo-11 (x32) | likely |
| We Know the Devil | 435300 | Unity 6000.0.72f1 Mono | 2D VN | D3D11 | none | — | wont-work |
| Tricky Towers | 437920 | Unity 5.5.1p3 Mono | 2D | D3D11 | online MP optional | — | wont-work |
| SteamWorld Dig 2 | 571310 | custom (Image & Form) | 2D | OpenGL | none | — | wont-work |
| Sine Mora EX | 606730 | custom (Digital Reality) | 2.5D 3D-rendered shmup | DX9 | none | dgVoodoo2 + Geo-11, experimental | likely |
| Risk of Rain 2 | 632360 | Unity 2021.3.45f2 Mono | 3D | D3D11 | EOS, online co-op optional; no AC | Resurrected VRMod (BepInEx, OpenXR) or Geo-11 | works-now |
| Star Renegades | 651670 | Unity 2018.4.30f1 Mono | 2D pixel on 3D backdrops | D3D11 | none | — | wont-work |
| LUMINES REMASTERED | 851670 | Unity 5.6.3p4 Mono | 2D | D3D11 | none | — | wont-work |
| Gravity Circuit | 858710 | LÖVE | 2D | OpenGL | none | — | wont-work |
| Flower | 966330 | custom (thatgamecompany PS3 port) | 3D | D3D11 (exe imports) | none | Geo-11 | likely |
| Eastward | 977880 | Moai | 2D pixel | D3D10/GL | none | — | wont-work |
| Alan Wake 2 (*) (Heroic/Epic) | heroic-epic | Remedy Northlight | 3D | DX12 only | EOS (Epic) | build our own (DX12) | build |
| Baldur's Gate II EE **(no notes)** (Heroic/GOG) | heroic-gog | Infinity Engine (Beamdog) | 2D | OpenGL | none | — | wont-work |
| ACE COMBAT 7 (*) (not installed; not in tsv) | 502500 | Unreal 4 (my game notes) | 3D | DX11 | none | UEVR | works-now |

Local-scan facts behind the table:
- No install folder contains `EasyAntiCheat*`, `BattlEye*` or `BEClient*` files. The `EOSSDK-Win64-Shipping.dll` files found (V Rising, Risk of Rain 2, Nocturnal, SteamWorld Build, MTGA, Puzzle Quest, Blanc, Sorry We're Closed, Absolum, Hades, Alan Wake 2) are Epic Online Services runtime libraries, not anti-cheat. Source: local scan of the paths in `library-scan.tsv`.
- Unreal versions read from the shipping exes: Octopath Traveler II = `++UE4+Release-4.27`, Sifu = 4.26, Little Nightmares EE (`Atlas-Win64-Shipping.exe`) = 4.27, Station to Station = 4.27. The UE5 shipping exes (Garden Life, Harmony, SWZC) did not show the string pattern, so their versions come from the wiki or the web. Source: local scan.
- Garden Life is Unreal Engine 5, not Unity as my notes had guessed. The files show this (`Engine/`, `GardenLife-Win64-Shipping.exe`), and so do reviews: "the scalability of Unreal Engine 5 allows Garden Life to perform smoothly on hardware like the Steam Deck" — [Tech-Gaming review](https://www.tech-gaming.com/garden-life-a-cozy-simulator/); [ActuGaming](https://www.actugaming.net/garden-life-a-cozy-simulator-simulation-jardinage-date-sortie-pc-consoles-601142).
- TMNT: Splintered Fate ships no UnityPlayer.dll or `*_Data/` folder. `TMNTSF.exe` imports d3d12.dll, dxgi.dll and OPENGL32.dll, so my notes' "Unity/GDK" engine line looks wrong. Source: local scan.
- Native Linux builds are installed for The Procession to Calvary, Exocolonist, Rogue Legacy 2, Core Keeper, Dome Keeper, Ballionaire, Thomas Was Alone, Tiny Glade, BADLAND, Geometry Wars 3 and To the Moon (`.x86_64`/`UnityPlayer.so`/`.so` libs, no exe). Source: local scan.
- NORCO's `appmanifest_1221250.acf` has StateFlags 4 (installed), installdir `NORCO`, 3.44 GB, but `steamapps/common/NORCO` does not exist on internal storage. NORCO is pixel art ("painterly and cinematic pixel art") with a DirectX 9 minimum — [Epic Games Store, The Art of NORCO](https://store.epicgames.com/p/norco-the-art-of-norco-7eb291?lang=fr).
- Forza Horizon 6 plays offline on a Steam Deck after a first-launch sign-in — [GamingOnLinux comment thread](https://www.gamingonlinux.com/2026/05/forza-horizon-6-is-out-valve-update-proton-hotfix-for-linux-initial-thoughts/comment_id=294767). GamingOnLinux reads the PC specs as having no anti-cheat that blocks Linux. A VKD3D-Proton developer called the game "extremely broken" on Linux, and Valve made a Proton hotfix the default for it — [GamingOnLinux](https://www.gamingonlinux.com/2026/03/forza-horizon-6-confirmed-to-be-playable-on-steam-deck-steamos/comment_id=292213).
- Risk of Rain 2: DrBibop's VRMod is deprecated (last release 2.9.2, Sep 4 2023) and broke after the game's Unity upgrade — [Thunderstore DrBibop/VRMod](https://thunderstore.io/c/riskofrain2/p/DrBibop/VRMod/). Blowntobytes' "Resurrected VRMod" is an MIT-licensed community rebuild for the current Steam version that switches to OpenXR — [Thunderstore Resurrected_VRMod](https://thunderstore.io/c/riskofrain2/p/Blowntobytes/Resurrected_VRMod/).
- areweanticheatyet lists V Rising as "Running", with no anti-cheat named — [areweanticheatyet V Rising](https://areweanticheatyet.com/game/v-rising).

### Inferences
- The library is heavily indie 2D. The real stereo candidates are about 47 titles, and about 30 of them are Unity DX11. Building one Unity path well, Geo-11 or a BepInEx camera mod, covers the most games.
- The five UE4 titles (Octopath II, Sifu, Little Nightmares, Station to Station, ACE COMBAT 7) are the best first tests. UEVR officially supports 4.8–5.4, and its Proton procedure is documented.
- The DX12-only titles (FH6, Spider-Man 2, 007, Alan Wake 2, TMNT, plus SWZC if UEVR fails on 5.6) form the "build our own" bucket. Four of them are Favorites, so the build phase must solve DX12 (via vkd3d-proton) rather than just DX11.
- Games with a native Linux build installed must be switched to the Windows build (Steam → Compatibility → force Proton) before any Windows injector (Geo-11, UEVR, BepInEx-Win) can be used.
- 32-bit titles (DREDGE, Stikbold!) need the x32 variants of Geo-11 or BepInEx.

### Gaps
- NORCO's engine and real install location could not be confirmed. The folder is missing and PCGamingWiki returned 403.
- The UE5 minor versions for Garden Life and SWZC were not read from the binaries. SWZC's 5.6 comes from my notes only.
- Render style for CraftCraft, Beholder: Conductor and Passant is from general knowledge of the titles, not checked this session.
- I checked no per-game Geo-11 fix or UEVR profile for any title. "likely" means the engine and API fit, not that a test passed.
- SteamDB "Technologies" and PCGamingWiki were not reachable (403), so engine lines without a local signature (Flower, Monaco, Tales, Lara Croft, Sine Mora) rest on the exe imports plus my existing notes.

## Q2. Per-engine "how to check any game" method and stereo route

### Takeaway
Detection is reliable from files alone: the Unity, Unreal, GameMaker, Godot, FNA, LÖVE and Ren'Py signatures plus the exe's DLL imports are enough. For stereo: Unreal 4.8–5.4 → UEVR (documented Proton procedure); Unity Mono and IL2CPP on DX11 → Geo-11 or a BepInEx VR injector (UUVR via Rai Pal, UnityVRMod); any other DX11 → Geo-11 (helixmod says Linux/Proton works, output SBS/TAB); DX9 → dgVoodoo2 to DX11, then Geo-11 (experimental). For DX12, Vulkan-native and OpenGL there is no free true-stereo injector, so we build our own. 2D engines (GameMaker, LÖVE, Ren'Py, most Godot and FNA) won't work.

### Cited Findings

**Detection recipe (run from the install folder):**
1. `ls` the top level. `UnityPlayer.dll` + `<Name>_Data/` = Unity. Then `GameAssembly.dll` present = **IL2CPP**; `MonoBleedingEdge/` or `<Name>_Data/Managed/*.dll` = **Mono**. `UnityPlayer.so` = native Linux Unity. Version: the first 2 KB of `<Name>_Data/globalgamemanagers` hold the string (e.g. `2022.3.62f2`, `6000.0.58f2`). Source: local scan of this library.
2. `Engine/` + `<Proj>/Binaries/Win64/<Proj>-Win64-Shipping.exe` + `Content/Paks/*.pak` (UE5: also `.utoc/.ucas`) = **Unreal**. Version: `grep -a -o -m1 '++UE4+Release-4\.[0-9]*' <Shipping.exe>` worked for UE4 here. UE5 exes did not match that pattern, so use the exe file-properties version, the wiki, or UEVR's own log. Source: local scan.
3. `data.win` = **GameMaker**; `*.pck` (+ `godotsteam.dll`) = **Godot**; `FNA.dll`/`FNA3D.dll` or `Content/*.xnb` = **FNA/XNA**; `love.dll` = **LÖVE**; `renpy/` + `game/` = **Ren'Py**; `Game.rgssad` = **RPG Maker**; `re_chunk_000.pak` = **RE Engine** (none in this library). Source: local scan.
4. API: `objdump -p <main.exe> | grep "DLL Name"`. d3d9 / d3d10 / d3d11 / d3d12 / vulkan-1 / opengl32. Unreal and Unity load their RHI dynamically, so also look for a `D3D12/` agility folder (it means DX12 is possible, not that it is the default) and the Unity `Player.log` "Direct3D 11" line. Source: local scan; my game notes (e.g. Blue Prince ships the D3D12 Agility SDK but Player.log shows D3D11).
5. Anti-cheat: `find . -iname 'EasyAntiCheat*' -o -iname 'BattlEye*' -o -iname 'BEClient*'`. `EOSSDK-*.dll` alone is services, not AC. Then check [areweanticheatyet.com](https://areweanticheatyet.com/game/v-rising).

**Unreal 4/5 → UEVR**
- UEVR is the "Universal Unreal Engine VR Mod". The README lists supported versions 4.8–5.4 and three render modes: Native Stereo, Synchronized Sequential (both eyes rendered on the same tick with the world paused between them) and AFR (warned to cause eye desync) — [praydog/UEVR](https://github.com/praydog/UEVR).
- Stable release UEVR 1.05 (16 Nov; adds UE5 crash fixes and a UE5.4 head-aim fix); 1.04 added automatic nightly builds — [UEVR releases](https://github.com/praydog/UEVR/releases). Latest nightly seen: build 01143, 30 Aug — [UEVR-nightly releases](https://github.com/praydog/UEVR-nightly/releases). Neither page mentions UE 5.5/5.6.
- Proton procedure: install .NET 6.0 Desktop Runtime into the game prefix with `protontricks-launch --appid <id> windowsdesktop-runtime-6.0.25-win-x64.exe`; launch option `PROTON_REMOTE_DEBUG_CMD="$HOME/Documents/uevr/UEVRInjector.exe" %command%`; pick the process; choose OpenVR (SteamVR) or OpenXR (Monado/WiVRn); Inject. Windows builds only — [VR on Linux wiki, UEVR Injector](https://vronlinux.org/docs/games/uevrinjector/).

**Unity (Mono and IL2CPP)**
- UUVR ("Universal VR mod for Unity games", GPL-3.0) is installed through Rai Pal — [Raicuparta/uuvr](https://github.com/Raicuparta/uuvr). Rai Pal is a mod manager for universal mods (UEVR, UUVR). It detects games from Steam, GOG, Epic, Itch and Xbox, identifies engine and version, and **ships a Linux AppImage** — [Raicuparta/rai-pal](https://github.com/Raicuparta/rai-pal). The fetched pages do not document UUVR's Mono/IL2CPP coverage.
- UnityVRMod supports **both Mono and IL2CPP** (separate builds), Unity ~5.6 to latest, needs **BepInEx 6** (pre-release 2 or bleeding-edge build ≥647), and lists **DirectX 11 Win64 only**. It is head-tracked 6DOF VR, with no fixed-screen mode documented. Latest release is v0.1.0-beta — [NewUnityModder/UnityVRMod](https://github.com/NewUnityModder/UnityVRMod).
- BepInEx under Proton needs the winhttp proxy override: `WINEDLLOVERRIDES="winhttp=n,b" %command%` — [BepInEx docs, Proton/Wine](https://docs.bepinex.dev/articles/advanced/proton_wine.html). BepInEx 5 is Mono-only; IL2CPP games need the BepInEx 6 "Unity IL2CPP" build — [BepInEx Unix/interop docs](https://docs.bepinex.dev/master/articles/advanced/steam_interop.html); [Nexus BepInEx installer](https://nexusmods.com/site/mods/115).
- Zyringe is a Linux-native GUI (beta) for injecting .NET mods into Mono Unity games and for setting up BepInEx/MelonLoader for IL2CPP — [TheHolyOneZ/Zyringe](https://github.com/TheHolyOneZ/Zyringe).
- Per-game precedent: the Risk of Rain 2 Resurrected VRMod (BepInEx, OpenXR) — [Thunderstore](https://thunderstore.io/c/riskofrain2/p/Blowntobytes/Resurrected_VRMod/).

**Custom DX11 (and Unity/UE on DX11) → Geo-11 (3DMigoto-based)**
- Geo-11 is a stereo 3D driver by davegl1234, written from scratch on 3DMigoto's hooking. It is **DX11 only**; DX12 is "not supported and has no current plans". It runs on any GPU (AMD testing works), with output modes SBS (default), TAB (preferred), reversed, interlaced, checkerboard, anaglyph, `katanga_vr` (HelixVision), Leia SR and 3D Vision. 3D TVs use SBS/TAB. Existing HelixMod DX11 fixes load directly; add geo-11 by copying its DLL + `nvapi64.dll` + `d3dxdm.ini` and setting `force_stereo=2`. Beta quality. **"Linux with Proton works."** Downloads are linked directly, and an open-source non-commercial release is planned — [HelixMod blog, Announcing geo-11](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html).
- Community reports: a Geo-11 fix for Hollow Knight: Silksong (Unity) released Sept 2025; one user said it crashed on Windows 10 but ran on a Steam Deck. Geo-11 needs the Windows build, and Proton Experimental is advised — [r/Stereo3Dgaming via search summary](https://twt.omada.cafe/r/Stereo3Dgaming) (mirror, certificate error on fetch; snippet only).
- A Steam thread reports Geo-11 failing on RoadCraft after it moved to DX12 — [Steam discussion](https://steamcommunity.com/app/2104890/discussions/0/600779252329148550).

**DX9 → dgVoodoo2 (DX9→DX11) + Geo-11**
- A CombatAce guide uses Geo-11 + dgVoodoo2 for stereo in VR in Strike Fighters 2 — [CombatAce file 18070](https://combatace.com/files/file/18070-method-for-stereoscopic-3d-in-vr-with-headtracking-in-strike-fighters-2-geo11-and-dgvoodoo2/). A VOGONS thread reports dgVoodoo2 + Geo-11 crashing on an RTX 4060 with one driver but working on older GPUs — [VOGONS](https://www.vogons.org/viewtopic.php?p=1419194). One Steam post says geo-11's dx9 development has stopped — [Steam HelixVision discussion](https://steamcommunity.com/app/1127310/discussions/0/3726197825948436143).

**Custom DX12 → build our own**
- No free DX12 geometry-stereo tool turned up. Geo-11 explicitly excludes DX12 — [HelixMod blog](https://helixmod.blogspot.com/2022/06/announcing-new-geo-11-3d-driver.html). HelixVision listings say "No DX12 games are currently working in 3D" — [search summary of HelixVision listings](https://www.dekudeals.com/items/helixvision).

**Godot → build our own (if 3D)**
- Godot modding tooling exists: GDRETools recovers a project from a PCK; GodotPckTool injects files into PCKs (used by ModLoader); GDWeave and GodotModding ModLoader are loaders; Godot loads extra packs at runtime via `ProjectSettings.load_resource_pack` — [GDWeave MODS.md](https://github.com/NotNite/GDWeave/blob/main/MODS.md); [GodotModding](https://www.Github.com/GodotModding); [Godot docs: exporting packs](https://docs.godotengine.org/fr/4.5/tutorials/export/exporting_pcks.html).

**GameMaker, LÖVE, Ren'Py, RPG Maker** — 2D engines in this library; no route needed (my ruling 9, [DECISIONS.md](../DECISIONS.md)).

### Inferences
- **Unreal 4/5**: detect `Engine/` + Shipping exe. Route: UEVR via Rai Pal or the VR-on-Linux launch-option recipe. For a fixed 3D TV, we would need glue that turns off head tracking or feeds UEVR's two eye views into a virtual screen; no source documents a UEVR fixed-screen mode. UE 5.5/5.6 titles: try the nightly, treat as "likely". Proton: documented working.
- **Unity Mono**: easiest to mod (BepInEx 5 or 6, managed DLLs readable). Route: a BepInEx plugin that adds a second camera and renders SBS is the cheapest "build", or Geo-11 on DX11. Proton: BepInEx works with the winhttp override.
- **Unity IL2CPP**: BepInEx 6 IL2CPP (or MelonLoader) + UnityVRMod/UUVR, or Geo-11 (API-level, so runtime-agnostic). Geo-11 is the simpler route for IL2CPP because it never touches managed code.
- **GameMaker / LÖVE / Ren'Py / RPG Maker**: always 2D here → wont-work.
- **Godot**: all four Godot games in this library are 2D → wont-work. For a future 3D Godot game, inject a script via a PCK (ModLoader/GDRETools) that drives two cameras.
- **XNA/FNA**: managed C#, so MonoMod/Harmony patching is possible. The only 3D one (Unrailed!) → build.
- **RE Engine**: none installed. The usual route is praydog's REFramework (not verified this session).
- **Source**: none installed. No source fetched.
- **Custom DX11**: Geo-11. Under Proton the game's d3d11 calls go to DXVK, and the helixmod post says Proton works, so Geo-11 hooks the game's d3d11.dll (native override needed; exact DLL-override string unverified).
- **Custom DX12**: build our own, most plausibly a hook at the vkd3d-proton/Vulkan layer or per-engine camera patching. This is the largest engineering item and covers 4 Favorites.
- **Vulkan-native** (Tiny Glade) and **OpenGL** (none 3D here): build our own; no free stereo injector found.

### Gaps
- UUVR's Mono vs IL2CPP coverage and Unity version range: the README fetched is a stub.
- UEVR support for UE 5.5/5.6 is not documented in any page I reached.
- Whether Geo-11's DLL must be set as `d3d11=n,b` in WINEDLLOVERRIDES under Proton is not stated in the source.
- No fixed-screen / SBS-output mode is documented for UEVR, UUVR or UnityVRMod; all three target head-tracked VR runtimes. Geo-11 is the only tool with documented SBS/TAB output.
- REFramework, Source-engine tools, any DX12/Vulkan/OpenGL stereo injector: no sources found or fetched; R.E.A.L. VR's R.E.A.L. mods were not researched (believed paid, so out of scope).
- dgVoodoo2 under Proton: not verified.
- Whether Geo-11 is still developed in 2026: no 2026 release notes found; newest community fix seen was Sept 2025.

## Q3. Library summary counts by verdict

### Takeaway
Across the 98 rows of library-scan.tsv: **works-now 5 · likely 35 · build 7 · not-safe 1 · wont-work 50** (total 98). ACE COMBAT 7, a Favorite not currently installed, would add a 6th works-now. That leaves 47 fixable 3D or 2.5D games; 33 of them depend on Geo-11, a Unity injector, or dgVoodoo2+Geo-11.

### Cited Findings
Counted mechanically from the Q1 table (source: local install scan + `my per-game notes` pages):
- **works-now (5)**: Octopath Traveler II, Sifu, Little Nightmares EE and Station to Station (UEVR, UE 4.26/4.27); Risk of Rain 2 (Resurrected VRMod — [Thunderstore](https://thunderstore.io/c/riskofrain2/p/Blowntobytes/Resurrected_VRMod/)). Not in the tsv: ACE COMBAT 7 (UE4, UEVR).
- **likely (35)**: STAR WARS Zero Company, Garden Life (UE5 → UEVR, version unverified); Control, Tactics Ogre: Reborn, Flower, Geometry Wars 3 (custom DX11 → Geo-11); The Cave, Tales from the Borderlands, Lara Croft GoL, Sine Mora EX (DX9 → dgVoodoo2+Geo-11); Unity DX11 (Geo-11 or a BepInEx VR injector): Mini Motorways, Golf Gang, OlliOlli World, Construction Simulator, ANNO: Mutationem, DREDGE, Blue Prince, V Rising, Planet of Lana, Core Keeper, Ereban, Hell Clock, Sorry We're Closed, DAVE THE DIVER, Boxes, Sektori, SteamWorld Build, Cat Quest III, Lost Eidolons, GUNTOUCHABLES, Urban Jungle, Beholder: Conductor, Vampire Crawlers, Passant, Stikbold!.
- **build (7)**: Forza Horizon 6, Marvel's Spider-Man 2, 007 First Light, Alan Wake 2, TMNT: Splintered Fate (DX12); Tiny Glade (Vulkan, Rust/wgpu); Unrailed! (FNA).
- **not-safe (1)**: Magic: The Gathering Arena (always-online).
- **wont-work (50)**: all 2D-art titles, including Hades and Hades II per my ruling.
- **No notes (5)**: Construction Simulator, Keylocker, Ereban: Shadow Legacy, CraftCraft, Baldur's Gate II EE. (ACE COMBAT 7 has a page but is not in the tsv.)

### Inferences
- Route totals: UEVR 6 (4 UE4 + 2 UE5; 7 with ACE COMBAT 7); Unity Geo-11/injector 25; custom DX11 Geo-11 4; DX9 dgVoodoo2+Geo-11 4; BepInEx per-game mod 1 (RoR2); build DX12 5, Vulkan 1, FNA 1.
- Among the 10 Favorites: works-now 1 (ACE COMBAT 7), likely 3 (SWZC, Control, Planet of Lana), build 4 (FH6, Spider-Man 2, 007, Alan Wake 2), wont-work 2 (Hades, Hades II).
- Priority order for the build phase: (1) UEVR on UE4 titles under Proton, (2) Geo-11 under Proton on one Unity DX11 game (Blue Prince or DREDGE) to prove the 25-game Unity bucket, (3) a DX12 approach for the four Favorites.
- Low depth value even if they work: Mini Motorways, Core Keeper, DAVE THE DIVER, Sektori and Passant are top-down or flat-ish and count as "likely" only technically.

### Gaps
- Counts rest on engine and API fit, not test runs. A Z13 test per game is needed to promote a game to works-now.
- Borderline 2D/2.5D calls (CraftCraft, Beholder: Conductor, Passant, Neva, Star Renegades, DAVE THE DIVER) could move one or two games between likely and wont-work.
- NORCO's folder is missing despite an "installed" manifest, so the game was classified from web info only.
