# System profiles

One file per platform, with the facts Flat to 3D needs on that OS and machine setup, and the
problems people already know about. This folder holds the shipped profiles, versioned with the
code and read by the Rust engine (`crates/engine/src/systems.rs`). The format is set by #412.

The engine reads every `.system` file in the folder and ignores other files, this README included.
A file is named `<id>.system`: lower-case letters, digits and `-`, for example
`steamos-strix-halo.system`. The id is the file stem.

## Format

Each line is one `key: value` pair, split at the first `:`. A line starting with `#` is a comment,
and blank lines are skipped. One pair of double quotes around a value is dropped, so `"yes"` and
`yes` are the same. A `|` separates alternatives inside one value.

Every file must give `name`, `match-os`, `desktop-session`, `game-session`, `steam-install` and
`wivrn-install`. Any other single key must be one of the keys below; an unknown key is refused so
a typo does not pass silently.

| Key | Meaning | Allowed values |
|---|---|---|
| `name` | The platform in plain words | any non-empty text |
| `match-os` | The `ID` from `/etc/os-release`, matched without case | lower-case letters, digits, `-`, `_`, `.` |
| `match-version` | An optional range over `VERSION_ID` | see below |
| `match-gpu` | An optional GPU family; `\|` separates names | non-empty names |
| `desktop-session` | The desktop session | any non-empty text, e.g. `plasma` |
| `game-session` | The session a game runs in | any non-empty text, e.g. `gamescope` |
| `steam-install` | How Steam is installed | any non-empty text, e.g. `preinstalled` or `flatpak com.valvesoftware.Steam` |
| `wivrn-install` | How WiVRn is installed | any non-empty text, e.g. `flatpak io.github.wivrn.wivrn` |
| `tool` | One required tool; the key may repeat | any non-empty text |

A `match-gpu` name matches when it appears in the detected GPU name, without case and with `_`,
`-` and `/` read as spaces. This lets `Strix Halo` and `Radeon 8060S` both match a PCI line that
names the chip. A profile with no `match-gpu` accepts any GPU.

### Version ranges

`match-version` uses the dotted numbers of `VERSION_ID` and is inclusive:

- `>=3.0`, `>3.0`, `<=3.7`, `<3.7` or `=3.6` for one bound;
- `3.0..3.7` for both bounds;
- `3.0..` or `..3.7` when one end is open.

A version that cannot be parsed, or a missing `VERSION_ID` when the profile has a range, does not
match. A bare value such as `3.6` means equal.

### Known problems

Each problem starts with `known-problem: <id>` and gives its three fields. The id is lower-case
letters, digits and `-`; it is also the name of the doctor line. The block repeats for every
problem, and every field is required:

```text
known-problem: gamescope-vulkan-crash
problem-summary: gamescope 3.16.30 can crash in the Radeon Vulkan driver; the cause is unknown.
problem-workaround: none yet
problem-status: open
```

`problem-status` is `open` or `worked-around`. A problem that is `open` still prints in the health
check, but it is information: it never turns doctor red. `problem-workaround` says `none yet` when
there is none.

A complete example:

```text
# SteamOS on a Strix Halo handheld (Radeon 8060S class).
name: SteamOS on Strix Halo
match-os: steamos
match-version: >=3.0
match-gpu: Strix Halo | Radeon 8060S | 8060S
desktop-session: plasma
game-session: gamescope
steam-install: preinstalled
wivrn-install: flatpak io.github.wivrn.wivrn
tool: gamescope
tool: pipewire
tool: flatpak

known-problem: pipewire-capture
problem-summary: Capture must use PipeWire because x11grab gives black frames.
problem-workaround: Capture through PipeWire instead of x11grab.
problem-status: worked-around
```

## Matching and search

The engine reads `ID` and `VERSION_ID` from `/etc/os-release` and the GPU name through the
command seam (`lspci -nn`, then `glxinfo -B`, then `vulkaninfo --summary`). It takes the first
profile, in file-name order, whose OS id, version range and GPU family all accept the facts.

Search folders, first match wins: `$FLAT3D_SRC_DIR/systems` (the shipped data in the AppImage and
the Decky plugin) and then `<data>/systems` (`~/.local/share/flat3d/systems` after `install`).
A missing folder is skipped. A malformed file is an error naming the file. No match is not an
error: the engine runs normally and doctor says `unknown system`.

## Packaging

`systems/` ships beside `recipes/`: the AppImage copies it to `usr/systems`, the Decky zip to
`bin/systems`, and `flat3d-inject install` copies it to `~/.local/share/flat3d/systems`.
