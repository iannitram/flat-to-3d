# Alan Wake 2 outside Steam: the options for a recipe key and the launch wrapper

Date: 2026-11-12. Issue #277 (design note). This note feeds the open decision W1 (#292, the NEW-AW2 decision; [ROADMAP](../ROADMAP.md), Decisions for you, item 6; [SPEC](../../SPEC.md) M32). It is docs only: no code, and it changes no ruling. The decision is the owner's and is left open at the end.

Words used: **read in the code** means I read it in this repo at the base of this branch; **observed** means the [Alan Wake 2 research note](research/games/epic-alan-wake-2.md) saw it on the handheld; **read from Heroic or Kaon** means the [Heroic files note](research/games/epic-alan-wake-2-heroic-files.md) took it from their source; **unchecked** means nobody has looked yet. Nothing here has been checked on the handheld's Heroic install. Every Heroic path, key name and behaviour below is unchecked until the device rows in section 6 are done.

## 1. The problem in four facts

1. **The wrapper needs a Steam appid.** `flat3d-run` takes the id from `SteamAppId`, then `SteamGameId`, as a non-zero `u32`. With neither, it logs `flat: missing or invalid Steam appid` and runs the command unchanged (read in the code, [`wrapper.rs`](../../crates/engine/src/wrapper.rs)). The game's own vkd3d-proton logs show `SteamAppID 0` when Heroic starts it (observed), so today the wrapper would always play it flat.
2. **A wrapper in the Steam shortcut wraps Heroic, not the game.** The Steam Favorites entry is a non-Steam shortcut that starts Heroic, and Heroic starts the game with its own Proton and prefix (observed).
3. **The key is a number in every place that holds one.** [`recipes/README.md`](../../recipes/README.md) names files `<appid>-<slug>.recipe`. `Recipe.appid`, the games list (`api::Game.appid`), the 3D switch (`game-off` lines in the settings file), the run record, the `3D Ready` collection diff and both faces all use `u32`. Two readers turn a file name into the key: `load_recipes` (the list) and `find_recipe` (the wrapper), both read in the code.
4. **The game list is Steam only.** `catalog` joins recipes to `steam::installed_games`, which reads Steam's `libraryfolders.vdf` and `appmanifest_*.acf`. An Epic game, and a Steam shortcut too, is not in it (read in the code).

## 2. What the engine would need to know, whichever way is chosen

| Fact | Where it comes from | Status |
|---|---|---|
| The game's folder and exe | legendary's `installed.json` entry: `install_path`, `executable` | unchecked, read from Kaon ([Heroic files note](research/games/epic-alan-wake-2-heroic-files.md), section 2) |
| The Wine prefix | `GamesConfig/<app_name>.json`, key `winePrefix` | unchecked, read from Kaon |
| Which Proton build | the same file, `wineVersion.bin` and `wineVersion.type` | unchecked, read from Kaon and Heroic |
| The config root | `~/.config/heroic`, or the Flatpak's `~/.var/app/com.heroicgameslauncher.hgl/config/heroic`; for the Flatpak, a path written as the sandbox sees it is mapped to the host path | unchecked, read from Kaon |
| The Epic `app_name` | the key of the `installed.json` entry | unchecked; its spelling is not known, so a dash in it is not ruled out |

The source for the layout is [Kaon](https://github.com/LorenDB/kaon/blob/fa4e22a7217d9b3450d42c50b04677d3e424d1e7/src/stores/Heroic.cpp) (`src/stores/Heroic.cpp` at commit fa4e22a). Kaon is GPL-3.0 and this project is GPL-3.0-only, so it is **learn-only**: the layout and key names are facts, and the engine's reader is written fresh in this project's own words and structure. Kaon also warns that Proton's own `proton` script fails when called directly; a tool that must run something in the prefix uses the build's `wine` binary ([Heroic files note](research/games/epic-alan-wake-2-heroic-files.md), section 4).

The route for this game (SPEC M32: a proxy DLL in the game folder and edits to `renderer.ini`, then SBS frames to the viewer) needs the game folder and the prefix wherever the engine runs it, so the reader in section 2 is needed by every option except where noted.

## 3. The three options

### Option 1: Heroic's per-game wrapper setting

Heroic keeps a list of wrapper programs per game in `GamesConfig/<app_name>.json` (key `wrapperOptions`, a list of `{exe, args}`; read from Heroic, key and format unchecked). The engine would add one entry for itself and run the game the way a Steam launch option does.

- **Recipes.** The recipe needs a store key. Today's key does not exist for this game (section 1, fact 3). See Option 3 for the key. The seed recipe's `store: epic` and `exe: AlanWake2.exe` lines are route settings the loader ignores today.
- **Wrapper (`flat3d-run`).** Needs a way to be told the Epic `app_name`, because Heroic sets no Steam variable. The least guessing is for the engine to write it into the wrapper entry's `args` (for example a `--epic <app_name>` flag ahead of the game command), since an argument the engine wrote cannot be missing. Heroic may also export the game's name in the environment, which would remove the flag; that is not read, so the note does not rely on it. Beyond the id: the wrapper's extra environment for a 3D launch (`PRESSURE_VESSEL_*`, `STEAM_COMPAT_LAUNCHER_SERVICE`) is Steam-specific, and its use under Heroic's Proton is unchecked (inferred: this game's route is a proxy DLL and the viewer, not OpenXR inside the game, so those variables may not be needed; W2 (#293) decides).
- **Reader.** A new, small reader of `installed.json` and `GamesConfig/<app_name>.json` in the engine (section 2), producing the game's name, folder and prefix for the list and for the route.
- **Writer.** The engine would write a file Heroic owns, so it needs an undo like its other route files, and it should refuse to write while Heroic is running (Heroic may rewrite the file from memory). Neither is checked.
- **App.** The Games page needs a row for a Heroic game and a way to switch it, keyed by the store key instead of an appid. The switch is stored in the settings file as `game-off\t<u32>` today.
- **Decky.** The panel cannot write Heroic's files from Steam's client API, so it calls an engine verb (as it does for `launch`) and the engine writes. The `3D Ready` collection holds Steam ids; whether it can hold the Heroic-made shortcut, and how the shortcut is tied to the Epic `app_name`, is unchecked.
- **Open risks.** Whether the wrapper reaches `AlanWake2.exe` or only Heroic's start script (device row 5); how Heroic splits `args` and orders several wrappers; whether the wrapper runs inside or outside Proton. These decide whether `flat3d-run` can be the wrapper at all.
- **Keeps.** Heroic's Proton, prefix, saves, updates and DLC. The game keeps working as it does today (observed).

### Option 2: A Steam shortcut straight to `AlanWake2.exe`

The owner adds `AlanWake2.exe` to Steam directly, picks a Proton build in Steam, and gives it the launch arguments `-EpicPortal` and `-epicuserid=Default-Epic-User`. The first makes the game start without Epic's launcher (the game is DRM-free then, DLC needs the launcher); the second keeps gamepad support. Both are from [PCGamingWiki](https://www.pcgamingwiki.com/wiki/Alan_Wake_II), through the research note, unchecked here. Steam then runs the game, so `flat3d-run %command%` is a normal Steam launch option.

- **Recipes.** The key is the shortcut's Steam id. It is local to one machine (research note, "The launcher gap"), so a recipe file in this repo would hold a value that is true for one install. Whether the id is stable if the shortcut is re-added is unchecked.
- **Wrapper.** No new code if Steam sets `SteamAppId` or `SteamGameId` to a `u32` for a shortcut. What Steam sets for a shortcut is unchecked, and the wrapper fails closed (plays flat) if it is not a `u32`.
- **Reader.** The game list needs the shortcut: Steam keeps shortcuts in a separate file in the user's `userdata` folder (recalled, not read), which `catalog` does not read. The game folder and exe come from that entry, so the Heroic reader is not needed for the list, but it is still needed to find where Heroic installed the game unless the owner supplies the path.
- **App.** A row for a game that is not in `appmanifest_*.acf`; the switch and undo work by the existing numeric id.
- **Decky.** `SetAppLaunchOptions` and the `3D Ready` collection are written for Steam apps by id. Whether both accept a shortcut's id is unchecked.
- **Open risks.** A second Wine prefix (Steam's, new) next to Heroic's, so saves and settings start fresh unless copied. The game then no longer starts from Heroic's library. Gamepad support needs the second argument.
- **Keeps.** Everything the engine does for Steam games today. Cheapest in engine code and, because it is by hand, usable now to test W2 and W3 before W1 is decided (inferred).

### Option 3: A recipe keyed by the Epic app name

This is a key, not a hook. It answers "what replaces the Steam appid for a store game" and is needed by Option 1 and optional for Option 2.

- **Recipes.** Recipe files named `epic-<app_name>-<slug>.recipe` (the README rule becomes `<store key>-<slug>`, where the store key is a number or `epic-` and the app name). If the app name can contain a dash, the name goes in the file instead, as a `store-id:` line, and the file name uses only a slug. The README, `recipes.rs`'s `RecipeError::BadFileName` rule and the `Recipe` struct change; the eight stereo keys do not.
- **Wrapper.** `appid()` becomes a game key that is either a Steam id or a store id (the flag from Option 1, or Steam's variables). The run record (`runs/run-PID.txt`) and the log line (`appid=...`) carry it.
- **Settings.** The 3D switch needs a line for a store key (today `game-off\t<u32>`); an old file must still parse.
- **Faces.** The shared JSON (`api::Game`, `api::Recipe`) gets a key that is not always a number; the App (`crates/app`) and the Decky types read it. This is the widest part of the change, because both faces read the same shape ([code map](../code-map.md), API shape).
- **Cost.** Wider than Option 2 in engine code. Sizes are not estimated here; they are set with the pipeline's sizing rubric once the option is chosen.

### What does not work

A wrapper in the Heroic-made shortcut's launch options (the current setup) wraps Heroic only (observed). Keying a recipe by that shortcut's id does not help for the same reason.

## 4. Side by side

| | Option 1 + 3: Heroic wrapper, Epic key | Option 2: Steam shortcut to the exe |
|---|---|---|
| Hook | the wrapper entry in `GamesConfig/<app_name>.json` | the shortcut's own launch options |
| Recipe key | Epic `app_name` (shared across machines) | shortcut id (one machine) |
| New engine code | store key, Heroic reader and writer, undo, wrapper flag, two faces | shortcut reader; maybe none else |
| Heroic's Proton and prefix | kept | not used; a second prefix |
| Decky writes | via an engine verb | `SetAppLaunchOptions` (unchecked for shortcuts) |
| Hard unknowns | whether the wrapper reaches the exe; the key's name and format | what Steam sets for a shortcut; gamepad and prefix state |
| Usable before the decision | no | yes, by hand |

Option 2 can also carry Option 3's key: put `flat3d-run --epic <app_name> %command%` in the shortcut's launch options. That mix gives a shared key and uses Steam's own launch path; it needs the wrapper flag and the store key but not the Heroic writer.

## 5. Recommended option

**Option 1 with Option 3's key**: key the recipe by the Epic app name and let the engine write Heroic's wrapper setting. Reasons: the recipe stays valid on any install (the file is versioned with the code), the game keeps Heroic's Proton, prefix and saves, which is what runs today (observed), and the reader the Heroic files note describes already answers the install folder and prefix the route needs.

**It depends on two device rows.** If the wrapper key does not exist in the format the Heroic files note expects (row 3), or a wrapper set in Heroic does not reach `AlanWake2.exe` (row 5), take Option 2 instead, with Option 3's key if the wrapper flag is built anyway. If you want the hook work (W2, W3) to start before W1 is decided, Option 2 by hand needs no engine change.

This is a recommendation only. SPEC M9, M30 and M32 and `DECISIONS.md` are not edited here.

## 6. Decision left open for the owner

Nothing below is decided. It belongs to #292 (W1), the NEW-AW2 decision.

1. **Hook:** Heroic's wrapper setting (Option 1), or a Steam shortcut to the exe (Option 2)?
2. **Key:** the Epic `app_name` (Option 3), or the local shortcut id?
3. **Writing Heroic's file:** is the engine allowed to edit `GamesConfig/<app_name>.json`, with an undo?
4. **Wording:** if a store key is accepted, SPEC M9 and M30 ("Steam launch options", "Steam appid") and M32's open problem sentence need a ruling and a rewrite. A ruling goes in `DECISIONS.md` after the owner says it.

## 7. Device rows owed

Same rows as the [Heroic files note](research/games/epic-alan-wake-2-heroic-files.md), section 6, plus two for Option 2. Read-only except where a setting is changed. Record key names and shapes only.

1. `legendaryConfig/legendary/installed.json`: the Alan Wake 2 entry (`app_name`, `install_path`, `executable`, `platform`), and whether the `app_name` has a dash.
2. `GamesConfig/<app_name>.json`: `winePrefix`, `wineVersion.bin`, `wineVersion.type`.
3. The same file's wrapper setting: set one wrapper in Heroic's game settings, save, and note the key name and format.
4. Which config root exists: native, Flatpak or both.
5. Whether a wrapper set in Heroic reaches `AlanWake2.exe` (a log line from the wrapper, not a guess).
6. For Option 2: which of `SteamAppId` and `SteamGameId` Steam sets for a shortcut and its value, and whether it survives re-adding the shortcut.
7. For Option 2: whether the game starts and takes gamepad input with `-EpicPortal -epicuserid=Default-Epic-User` under Steam's Proton.

Until those rows are done, none of the facts marked unchecked is confirmed. This PR owes them to #277's done-when; it does not close the issue.
