# Flat to 3D

Play installed PC games in **true stereoscopic 3D**, where the game draws two camera views. The picture appears on a giant fixed virtual 3D TV in a Quest 2, with a controller in hand. The project targets the ROG Flow Z13 under SteamOS and Proton, with the headset on a USB cable.

## Ground rules

True stereo, free and open-source tools, offline single-player games, and a fixed screen with no head-steered game camera. See [the decisions](docs/notes/DECISIONS.md).

## Build and test

Run `tools/check.sh`; the Rust suite can also be run with `cargo test --workspace`.

## Repository layout

| Path | What |
|---|---|
| `crates/` | Rust engine, desktop app, viewer and injector |
| `decky/` | Game Mode panel |
| `recipes/` | Per-game stereo routes |
| `packaging/` | Build and install scripts |
| `sessions/` | Device test procedures |
| `games/` | Per-game profiles and fixes |
| `tools/` | Checks, research and build utilities |
| `docs/` | Specification, roadmap and technical maps |
| `cmake/` | Cross-compilation toolchain |
| `third_party/` | Pinned upstream source |
| `.github/` | CI workflows |

## Credits and license

GPL-3.0. Spider-Man support builds on [SpidyVR](https://github.com/IlyaMez/SpidyVR) by Ilya Mezerowsky (GPL-3.0). The app design follows [Rai Pal](https://github.com/Raicuparta/rai-pal) (GPL-3.0).
