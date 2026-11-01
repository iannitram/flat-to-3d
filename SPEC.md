# Spec

**1.0 means:** on the Z13 in Game Mode, I plug in the Quest 2, press **Start 3D** in the Decky plugin, then **Inject** on one of five Favorites: STAR WARS Zero Company, ACE COMBAT 7, Control, Planet of Lana or Spider-Man 2. The game plays in true stereo on a giant fixed 3D TV in the Quest 2 over USB, gamepad in hand, at 30+ fps. **Stop 3D** shuts the link down. In Desktop Mode a Flat to 3D app (Tauri) shows the same library, link and buttons, plus setup, logs and undo. Order: Zero Company, ACE COMBAT 7, the two Geo-11 games, Spider-Man 2.

**Approved by the owner:** 2026-10-31, the owner, question tool: "Approve (Recommended)" for the desktop app (M24), rulings "Desktop app + Decky plugin (Recommended)" and "Tauri (Rust + web UI) (Recommended)"). Previous approval, 2026-10-31: "Approve (Recommended)" (the five games, gamepad only, Start 3D and Stop 3D, M18-M23, #9's SBS viewer in 1.0); order "ZC, AC7, Geo-11 pair, SM2 (Recommended)". Wording fix 2026-10-31, question tool: M10 "Reword to WiVRn wired only (Recommended)". Rulings behind it, the Project Manager session's question tool, 2026-10-31: V1 scope "Add the 3 'likely' ones"; controller "A gamepad (Recommended)"; Stop 3D "Yes (Recommended)".

Earlier approval, 2026-10-31, the owner, in the Project Manager session's question tool: "Approve (Recommended)" (1.0 = Zero Company first, UEVR then our own hook, 2D Screen Mode else the fixed-screen layer, offline after one EA link, plus Spider-Man 2, both from Start 3D and Inject in Game Mode). Rulings behind it, 2026-10-31: "Into 1.0"; route "UEVR first, port fallback (Recommended)"; 3D TV "Try 2D Screen Mode, else layer (Recommended)"; order "Zero Company first (Recommended)"; EA link "OK if it plays offline after (Recommended)".

First approval, 2026-10-26: "Spider-Man 2 + Start 3D (Recommended)".

A Must or a Won't changed after approval sends the line above back to "not yet".

## Musts

Almost every check needs the owner's Z13, the games and the Quest 2, so those checks are marked `needs-owner-device`. The engine's logic tests run in CI.

| # | Must | Done looks like | Check |
|---|---|---|---|
| M1 | WHEN unmodified SpidyVR 0.3.8 runs on Spider-Man Remastered in GAME SCREEN 3D over WiVRn's wired mode, THE SETUP SHALL show the city on a 3D screen in the Quest 2 with visible depth (Step 0, #5). | Depth is visible and the frame rate holds at 30+ fps. A compatibility report goes upstream to SpidyVR (ruling 23). | `docs/STEP0-RUNSHEET.md` (`needs-owner-device`) |
| M2 | BEFORE anything is injected into Spider-Man 2, IT SHALL be shown to play offline (PlayStation PC SDK check, #7, ruling 7). | Spider-Man 2 starts and plays with the network off. | #7's steps (`needs-owner-device`) |
| M3 | THE PORT SHALL get inside Spider-Man 2: injection loads only `render_probe`, the game keeps running, and the exe's hash and version are recorded with auto-updates off (port Steps 1-2). | The log names the game's D3D12 queue. | port Step 2 test (`needs-owner-device`) |
| M4 | THE PORT SHALL move Spider-Man 2's own camera sideways on command (port Step 3). | The picture on the Z13's screen slides left and right. | port Step 3 test (`needs-owner-device`) |
| M5 | THE PORT SHALL render two eye views every frame (port Step 4). | A capture of both eyes shows distant buildings tens of pixels apart and HUD text about 0 apart. | the Spider-Man 2 version of `probe_game_screen.py --stereo` (`needs-owner-device`) |
| M6 | EACH SHOWN PAIR SHALL come from the same frame, with matching brightness and its own anti-aliasing history; ray-traced reflections are off for the eyes and the upscaler is bypassed (port Step 5). | Frame numbers match in every pair, both eyes are equally bright, and edges don't shimmer. | port Step 5 test (`needs-owner-device`) |
| M7 | IN THE QUEST 2, Spider-Man 2 SHALL play on the big fixed 3D screen with the controller, at 30+ fps, and pausing SHALL drop to a flat screen. The output is plain OpenXR (port Step 6, ruling 19). | That play, seen in the headset. | port Step 6 test (`needs-owner-device`) |
| M8 | A FULL PLAY SESSION (swinging, combat, a cutscene, a pause) SHALL run without dropped eyes or crashes, with the HUD on the screen plane and pattern scans instead of fixed addresses (port Step 7). | The session as described. | port Step 7 test (`needs-owner-device`) |
| M9 | THE INJECT ENGINE (command line) SHALL list the installed Steam games with their stereo status from the recipes, and `inject <appid>` SHALL apply the recipe's route for Spider-Man 2 (ruling 10, #10). | `flat3d list` shows Spider-Man 2 as `works-now`; `flat3d inject 2651280` sets up the launch the way the port's launch script does. | engine unit tests in CI; one real inject (`needs-owner-device`) |
| M10 | **Start 3D** SHALL bring up the headset link over WiVRn's wired mode: start the WiVRn server without its desktop window, run `adb reverse tcp:9757 tcp:9757`, open the WiVRn app on the Quest, and say when it is ready (rulings 5, 2026-10-31). Quest Tether is not needed for this and stays optional. | One press, and the Quest shows WiVRn ready over the cable. | engine unit tests for the command sequence; on the device (`needs-owner-device`) |
| M11 | IN GAME MODE THE DECKY PLUGIN SHALL show the installed games with their stereo status, an **Inject** button per game, and **Start 3D** (rulings 5, 10). | The 1.0 sentence, done from the QAM. | on the device (`needs-owner-device`) |
| M12 | Spider-Man 2's recipe keys SHALL read `stereo-status: works-now`, wired to the Inject button (port Step 8). | The recipe is in the repo and the engine reads it. | engine unit test |
| M13 | BEFORE anything is injected into STAR WARS Zero Company (Steam 2075800, UE 5.6, DX12 only), IT SHALL be shown to play offline after its one-time EA account link (ruling 2026-10-31). | With the link done once, the game starts and plays a mission with the network off. | on the device (`needs-owner-device`) |
| M14 | WHEN the UEVR nightly is injected into Zero Company under Proton, IT SHALL render two real views (Native Stereo or Synchronized Sequential; plain AFR doesn't count). IF UEVR can't on UE 5.6, a camera hook of our own, built with the shipped `SWZeroCompany.pdb`, SHALL do it instead (ruling 2026-10-31: UEVR first, port fallback). | A capture of both eyes shows distant scenery apart and the HUD about 0 apart. | on the device (`needs-owner-device`) |
| M15 | THE TWO VIEWS SHALL appear on the giant fixed 3D TV with no head-steered camera: UEVR's 2D Screen Mode if it keeps per-eye depth, otherwise the generic fixed-screen OpenXR layer (#9, pulled into 1.0 only in that case) (ruling 2026-10-31). | In the headset, turning my head doesn't move the game camera and the screen shows depth. | on the device (`needs-owner-device`) |
| M16 | IN THE QUEST 2, Zero Company SHALL play on the fixed 3D screen with the controller at 30+ fps through a full mission (hub, a battle, a cutscene, a pause), without dropped eyes or crashes. | The session as described. | on the device (`needs-owner-device`) |
| M17 | Zero Company's recipe SHALL read `stereo-status: works-now` with the UEVR route (or our hook) and its settings, and `flat3d inject 2075800` SHALL apply it. | The recipe is in the repo; the engine reads and applies it. | engine unit test; one real inject (`needs-owner-device`) |
| M18 | **Stop 3D** SHALL close WiVRn's session, remove the `adb reverse` and say when it is done; pressing Start 3D again SHALL work without a reboot (ruling 2026-10-31). | Start 3D, Stop 3D, Start 3D in a row, all reporting ready/done. | engine unit tests for the command sequence; on the device (`needs-owner-device`) |
| M19 | EVERY 1.0 GAME SHALL play with a gamepad through Steam Input; the Quest Touch controllers are not used (ruling 2026-10-31). | Each game's on-device test is played on the gamepad. | on the device (`needs-owner-device`) |
| M20 | ACE COMBAT 7 (Steam 502500, Unreal 4, DX11) SHALL play through UEVR with two real views on the fixed 3D TV (the same route as Zero Company, M14-M15), a full sortie at 30+ fps, with a `works-now` recipe. | A sortie in the headset with depth; `flat3d inject 502500` applies the recipe. | on the device (`needs-owner-device`); engine unit test |
| M21 | Geo-11 SHALL run under Proton and its side-by-side output SHALL reach the headset as the fixed 3D TV, through an SBS screen viewer (#9's viewer, pulled into 1.0). | A Geo-11 game shows depth on the fixed screen in the Quest 2. | on the device (`needs-owner-device`) |
| M22 | Control (Steam 870780, `Control_DX11.exe`) SHALL play with the community Geo-11 3D fix on the fixed 3D TV at 30+ fps, with a `works-now` recipe. | A mission section in the headset with depth. | on the device (`needs-owner-device`); engine unit test |
| M23 | Planet of Lana (Steam 1608230, Unity 2020.3 IL2CPP, DX11) SHALL play with its Geo-11 fix on the fixed 3D TV at 30+ fps, with a `works-now` recipe. | A level in the headset with depth (expected shallow, diorama-like). | on the device (`needs-owner-device`); engine unit test |
| M24 | IN DESKTOP MODE A FLAT TO 3D APP (Tauri) SHALL show the installed games with their stereo status, Start 3D and Stop 3D with the link state, first-time headset setup, each game's recipe with Inject and an undo back to flat, and the engine's log; it drives the same engine as the Decky plugin (ruling 2026-10-31). | Each of those works from the app window on the Z13. | app build + typecheck in CI; on the device (`needs-owner-device`) |

## Won'ts

- #8's other big-bucket tests (UEVR and Geo-11 on games outside 1.0); #9's fixed-screen OpenXR layer unless UEVR's 2D Screen Mode loses depth (M15) (#9's SBS viewer is in 1.0, M21); #11 own stereo for other DirectX 12 games; #12 Steam Frame support (the plain-OpenXR output in M7 keeps it possible); #13 the Uncharted collection, with #4's pop-out control. All are marked `after-1.0`.
- Any game other than the five (Remastered is only the Step 0 baseline): 007 First Light and Alan Wake 2 (DX12 builds of our own, #11), Forza Horizon 6 (not safe), Hades and Hades II (no 3D world), any game that is online-only or has anti-cheat, and any paid tool (rulings 7, 8, 16).
- Spidy's HUD hooks and settings tab: Spider-Man 2 draws its UI with Coherent Gameface, not Scaleform.
- Windows or Mac as the render machine (rulings 1, 2).

## Problem

Installed PC games only play flat. The free VR mods that exist steer the camera with the head, which isn't wanted. True stereo on a fixed screen needs a per-game port, and nothing launches it from Game Mode.

## Solution

Run Zero Company and ACE COMBAT 7 in stereo through UEVR (our own hook if UEVR can't on Zero Company), Control and Planet of Lana through Geo-11 plus an SBS screen viewer, port SpidyVR's stereo-screen core to Spider-Man 2, then wrap all five in one Rust engine with two faces: a Decky plugin, so Game Mode has one button for the headset link and one for the game, and a Tauri desktop app for Desktop Mode (library, setup, logs, undo).

## User stories

1. As the owner, I want to sit with a controller and play my Favorites (Zero Company, ACE COMBAT 7, Control, Planet of Lana, Spider-Man 2) on a giant 3D TV in the Quest 2, so that the game has real depth without head-steering.
2. As the owner, I want to start it from Game Mode with two presses, so that no terminal is needed.

## Implementation decisions

- The rulings in `docs/notes/DECISIONS.md` are binding: Z13 and SteamOS only, true stereo only, a giant fixed TV, Quest Tether + WiVRn, 30+ fps, offline only, free only, Spidy as the baseline, plain OpenXR.
- Controls: a gamepad through Steam Input for every game (ruling 2026-10-31).
- Geo-11 games need an SBS screen viewer that puts Geo-11's side-by-side picture on the fixed screen. Likely route, unproven: a viewer running in the same Proton session (OpenXR through WiVRn) reading Geo-11's shared texture, as Blackskydk/FlatToDepth and artumino/VRScreenCap (MIT) do on Windows.
- ACE COMBAT 7 is not installed today (its library drive isn't mounted).
- Zero Company comes first (ruling 2026-10-31): offline check, UEVR nightly under Proton, the 3D TV, then its recipe; it proves the headset chain and Inject on a real game while the Spider-Man 2 address mapping goes on.
- The Spider-Man 2 port follows `docs/PORT-SPIDERMAN2.md` Steps 0-8. Its code lives in `games/spiderman2/`, and the DLLs build on the Z13 with clang-cl + xwin (ruling 21).
- **Two faces, one engine** (rulings 2026-10-31, question tool: "Desktop app + Decky plugin (Recommended)", "Tauri (Rust + web UI) (Recommended)"): the Tauri app links the engine crate; the Decky backend calls its binary with `--json`. Shape: `docs/code-map.md`.
- **The engine is Rust** (the owner's ruling, 2026-10-26, question tool: "Rust"): one binary, `flat3d-inject`, a std-only crate in `inject/`. The Decky backend stays Python (Decky backends are Python) and calls the binary as a subprocess. Rai Pal's game detection (Rust, GPL-3.0) is the reference for detection logic.
- **Recipes are a data file in this repo** (the owner's ruling, 2026-10-26, question tool: "Data file in the repo (Recommended)", #15), under `recipes/`, versioned with the code and read by the engine. For 1.0 the engine needs the five games' recipes; the owner's per-game notes elsewhere stay notes.

## Testing decisions

The engine is tested without games or hardware, with `cargo test --manifest-path inject/Cargo.toml` (Rust `#[test]`s, sample files under `inject/tests/fixtures/` as they are added): Steam library parsing on sample `libraryfolders.vdf` and `appmanifest_*.acf` files, recipe loading, and the commands Start 3D would run (asserted, not executed). Every port step has its own on-device test, as in `docs/PORT-SPIDERMAN2.md`.

## Further notes

Platforms (R04): Linux (SteamOS) as the host only. The game DLLs are Windows binaries cross-built on the Z13 and run under Proton. Mac and Windows hosts are out (rulings 1, 2).

## Prior art

Approved 2026-10-31; why each matters and what was also seen: [PEERS.md](PEERS.md).

peer: IlyaMez/SpidyVR | GPL-3.0 | our upstream; v0.7.0 documents Proton + WiVRn wired (docs/LINUX.md, tools/spidy-wivrn.sh); watch releases, eye-pair and swapchain code
peer: lecutis7of9/HEROVR | GPL-3.0 | SpidyVR ported to Spider-Man 2 on Windows: render probe, eye hooks, FXAA/CAS on the eyes, RT off; learn only (ruling 2026-10-31), its addresses come from the closed SM2VR mod
peer: LorenDB/kaon | GPL-3.0 | Linux UEVR manager for Proton: Steam scan, UEVR + .NET install, launch options; reference for Zero Company Z2 and the Inject engine
peer: WiVRn/WiVRn | GPL-3.0 | our streamer; wired mode and the dashboard's adb automation are what Start 3D wraps
peer: praydog/UEVR | NOASSERTION | the Zero Company route; watch nightlies for UE 5.6 and 2D Screen Mode depth
peer: Blackskydk/FlatToDepth | MIT | Windows: Geo-11 per-eye output on a big OpenXR screen with a Steam scan; closest to the generic 3D TV (#9)
peer: artumino/VRScreenCap | MIT | Rust OpenXR SBS/Katanga screen viewer; reference for the fixed-screen layer and SBS viewer
peer: Raicuparta/rai-pal | GPL-3.0 | the model for game detection and the Inject flow
