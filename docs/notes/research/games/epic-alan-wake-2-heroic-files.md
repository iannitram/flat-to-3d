# Alan Wake 2: the Heroic files the engine would read

Research date: 2026-11-12. Issue #277 (from-peer, learn). This note feeds the open decision W1 (#292, [ROADMAP](../../../ROADMAP.md) Decisions item 6) and adds to [epic-alan-wake-2.md](epic-alan-wake-2.md), section "The launcher gap". No engine code follows from it until W1 is decided.

Words used: **read from Kaon** means I read the peer's code at the commit below; **read from Heroic** means I read Heroic's own type declarations; **unverified** means no one has looked at the file on the handheld's Heroic install yet. Every row below is unverified.

## Source and licence

- Kaon, `src/stores/Heroic.cpp` at commit [fa4e22a](https://github.com/LorenDB/kaon/blob/fa4e22a7217d9b3450d42c50b04677d3e424d1e7/src/stores/Heroic.cpp). Epic support arrived in Kaon commit [e7f75a98](https://github.com/LorenDB/kaon/commit/e7f75a98). The peer is listed in [PEERS.md](../../../../PEERS.md) as `peer: LorenDB/kaon`.
- Kaon is GPL-3.0 and this project is GPL-3.0-only. This is a learn item: a file layout and a key name are facts, and the note states them in my own words. No Kaon code is copied here, and the engine's reader will be written fresh.
- Heroic's declaration of its per-game settings is in [`src/common/types.ts`](https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/main/src/common/types.ts) (`GameSettings`, `WrapperVariable`, `WineInstallation`). It was read on the default branch on 2026-11-12, not pinned to a commit.

## 1. Where Heroic keeps its files

Kaon looks in two config roots and uses both when both exist:

| Install kind | Config root | Status |
|---|---|---|
| Native (deb, AppImage, tarball) | `~/.config/heroic` | unverified, from Kaon |
| Flatpak | `~/.var/app/com.heroicgameslauncher.hgl/config/heroic` | unverified, from Kaon |

Which one the handheld uses is not known yet (device row 4 below).

## 2. The files for one Epic game

| What the engine needs | File under the config root | Keys | Status |
|---|---|---|---|
| Is it installed, and where | `legendaryConfig/legendary/installed.json` | one entry per game, named by the game's Epic `app_name`. Each entry has `app_name`, `title`, `install_path`, `executable` and `platform` (`Windows` or `Win32` for a Windows build) | unverified, from Kaon |
| The game's exe | the same entry | `executable` is relative to `install_path`; join the two | unverified, from Kaon |
| The prefix | `GamesConfig/<app_name>.json` | the settings sit under a top-level key equal to the `app_name`; `winePrefix` is the prefix folder | unverified, from Kaon |
| The Proton or Wine build | the same file | `wineVersion.bin` is the build's launcher path; `wineVersion.type` says `proton` or `wine` (the type is read from Heroic) | unverified, from Kaon and Heroic |
| The per-game wrapper (the Steam launch-option equivalent) | the same file | key `wrapperOptions`, a list of `{ "exe": ..., "args": ... }` pairs (read from Heroic; Kaon at fa4e22a does not read it) | unverified; the key name and format are the open part |
| Extra environment variables for the game | the same file | `enviromentOptions` (spelled so in Heroic's source), a list of `{ "key": ..., "value": ... }` | unverified, from Heroic |

Notes on the rows:

- Kaon also reads `legendaryConfig/legendary/metadata/<app_name>.json`, for artwork only. The engine does not need it.
- Alan Wake 2's own `app_name` is a code Epic assigns. It is not in the research note, so the first device row below has to read it.
- Heroic's `GameSettings` also has `launcherArgs` (arguments given to the game), `targetExe` and `beforeLaunchScriptPath` / `afterLaunchScriptPath`. They are named here only because W1 may prefer one of them to `wrapperOptions`; nothing has been read about how Heroic applies them.
- The wrapper pair's meaning is inferred: `exe` is the program put in front of the game's command and `args` are its arguments, as a Steam `%command%` prefix would be. Not read yet: whether Heroic splits `args` on spaces, how it orders several pairs, and whether a wrapper runs inside or outside Proton. Those decide whether `flat3d-run` can be the wrapper.

## 3. Flatpak paths

Heroic in a Flatpak writes the sandbox's view of a path into its JSON files, for example a prefix or install folder under the home directory. The host sees the same files under `~/.var/app/com.heroicgameslauncher.hgl/`. Kaon's rule, which the engine can follow:

1. Only for the Flatpak install: if the path as written does not exist and it starts with the home folder, put `.var/app/<app id>` between the home folder and the rest.
2. Use the rewritten path only if it exists; otherwise keep the original.

It applies to `install_path`, `winePrefix` and `wineVersion.bin`. Kaon also keeps the sandbox spelling, because a command run inside the sandbox needs it.

## 4. Kaon's note on Proton

When `wineVersion.bin` ends in `/proton`, Kaon does not run that file. Its code comment says that calling Proton's wrapper directly does not work, so it runs the `wine` binary that ships inside the same Proton build. It looks for `files/bin/wine` and then `dist/bin/wine` under the build's folder, and tries `files/bin-arm64/wine` first when the machine or the prefix is arm64. The first one that exists wins.

What this means here: a tool that needs the game's prefix and Proton (for example the do-nothing proxy DLL check, device test 2 in the research note) should expect the same failure and use the build's own `wine`, not `proton`. It does not say how Heroic starts the game; Heroic has its own logic for that.

## 5. What this answers for W1, and what it does not

- Answered by the layout, once the rows are confirmed: where the game folder is (`install_path`), which exe (`executable`) and which prefix (`winePrefix`).
- Not answered: how a recipe is keyed (the Epic `app_name` is a natural key that replaces the Steam appid for this store, but that is W1's choice), and whether writing `wrapperOptions` wraps the game itself or only Heroic. The research note's three ways out stay as they were; this note makes the second one (a store key plus Heroic's own setting) concrete enough to cost.
- Writing into `GamesConfig/<app_name>.json` changes a file Heroic owns. If W1 picks that way, the engine needs an undo like its other route files, and should not write while Heroic is running (Heroic may rewrite the file from memory). Neither is checked.

## 6. Device rows owed

Check these on the handheld's Heroic install, read-only, with the game installed. Until each is done, the table rows stay unverified. Record key names and shapes only; do not paste file contents into the repo.

1. `legendaryConfig/legendary/installed.json`: the Alan Wake 2 entry, with its `app_name`, `install_path`, `executable` and `platform`.
2. `GamesConfig/<app_name>.json`: `winePrefix`, `wineVersion.bin` and `wineVersion.type`.
3. The same file's wrapper setting: the key name and format (the open part). Set one wrapper in Heroic's game settings, save, and see which key changes.
4. Which config root exists: native, Flatpak or both.
5. Whether a wrapper set in Heroic reaches `AlanWake2.exe` or only Heroic's start script (device test 1 in the research note).
