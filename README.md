# Flat to 3D

Play installed PC games in **true stereoscopic 3D**, where the game draws two camera views. The picture appears on a giant fixed virtual 3D TV in a Quest 2, with a controller in hand. The project targets the ROG Flow Z13 under SteamOS and Proton, with the headset on a USB cable.

## Ground rules

True stereo (alternate-eye rendering counts only for a game with no two-real-views route, at 30+ fps per eye), free and open-source tools, offline single-player games, and a fixed screen with no head-steered game camera. See [the decisions](docs/notes/DECISIONS.md).

## Build and test

- `tools/check.sh`: the one gate that CI and people run (`--only rust|py|decky`).
- `cargo test --workspace --locked`: the Rust suite.
- `cargo test -p flat3d-app`: the desktop app's page logic.
- `cargo build -p flat3d-viewer --features xr,capture`: the viewer's device build, run inside the distrobox build container (it needs the OpenXR and PipeWire libraries).
- `tools/build-win.sh`: cross-builds the Windows binaries (the UEVR injector) with clang-cl and xwin.

## Repository layout

| Path | What |
|---|---|
| `crates/engine/` | the engine: Rust crate `flat3d-inject`, binaries `flat3d-inject` and `flat3d-run` |
| `crates/app/` | the desktop app (iced) for Desktop Mode |
| `crates/viewer/` | `flat3d-viewer`, the SBS screen viewer for Geo-11 games |
| `crates/uevr-inject/` | `flat3d-uevr-inject.exe`, the UEVR injector run under Proton |
| `decky/` | the Game Mode panel |
| `packaging/` | pinned downloads, build and install scripts |
| `sessions/` | on-device check sessions |
| `tools/` | checks, research and build utilities |
| `games/` | per-game profiles, fixes and the Spider-Man 2 port |
| `recipes/` | per-game stereo routes |
| `docs/` | roadmap, code map, rulings and research |
| `.github/` | CI workflows and the issue template |
| `cmake/` | the cross-compilation toolchain |
| `third_party/` | pinned upstream source |

## Credits and license

GPL-3.0. Spider-Man support builds on [SpidyVR](https://github.com/IlyaMez/SpidyVR) by Ilya Mezerowsky (GPL-3.0). The app design follows [Rai Pal](https://github.com/Raicuparta/rai-pal) (GPL-3.0).
