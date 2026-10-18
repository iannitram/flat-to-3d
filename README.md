# Flat to 3D

Play installed PC games in **true stereoscopic 3D**, where the game itself draws two camera views. The picture appears on a giant fixed virtual 3D TV in a Quest 2, with a controller in hand. Everything runs on SteamOS (ROG Flow Z13) under Proton, and the headset connects over USB through Quest Tether (`~/dev/quest-tether`).

Status: **planning.** There's no working code yet.

## What it will be

- A command-line **engine** that finds installed games (Steam, Heroic), reads each game's 3D recipe, and injects the right stereo tool.
- A **Decky plugin** for Game Mode: your installed games, each with an **Inject** button, plus "Start 3D", which brings up Quest Tether and the VR streamer.
- **Per-game stereo cores** where no tool exists. The first is a port of [SpidyVR](https://github.com/IlyaMez/SpidyVR)'s stereo-screen mode from Spider-Man Remastered to Spider-Man 2.

Inspiration: [Rai Pal](https://github.com/Raicuparta/rai-pal).

## Future features

- **Steam Frame compatibility.** Support Valve's Steam Frame headset alongside the Quest 2. Keep the 3D-screen output on standard OpenXR, not tied to WiVRn, so it can also run through SteamVR, which streams to the Frame (SteamVR 2.17 added Frame support and initial USB streaming).

## Ground rules

- True stereo only. Depth-buffer reprojection doesn't count.
- Free and open-source tools only. Nothing paywalled.
- Offline and single-player games only. Never inject into a game with anti-cheat or always-online play.
- Flat 2D-art games are out of scope.

## Where the planning lives

Decisions, the research report and the per-game recipes live in Ian's Nextcloud vault, not here:

- `Workspaces/Flat to 3D/DECISIONS.md`: settled requirements and the recipe-key schema.
- `Workspaces/Flat to 3D/reports/`: research.
- `Wiki/Technology/Gaming/<Game>.md`: each game's `stereo-*` frontmatter keys are the recipe this app reads.

## License

GPL-3.0. The project builds on GPL-3.0 code (SpidyVR, Rai Pal).
