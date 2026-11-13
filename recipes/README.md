# Recipes

One file per game, with `name` plus the eight stereo keys (`docs/notes/DECISIONS.md`, "Recipe keys"). This folder holds the six 1.0 games' recipes (SPEC), versioned with the code and read by the Rust engine (`crates/engine/src/recipes.rs`). The file format is set by the recipes issue. For 1.0 the engine needs Spider-Man 2's recipe (SPEC M12); the owner's per-game notes elsewhere stay notes.

## Format

One file per game, named `<appid>-<slug>.recipe`: the Steam appid, a dash, then a short name, for example `2651280-spider-man-2.recipe`. The loader reads every `.recipe` file in this folder and ignores other files, this README included.

Each line is one `key: value` pair, split at the first `:`. A line starting with `#` is a comment, and blank lines are skipped. One pair of double quotes around a value is dropped, so `"yes"` and `yes` are the same.

Every file must give `name` and the eight stereo keys: `stereo-status`, `stereo-route`, `stereo-engine`, `stereo-api`, `stereo-mod`, `stereo-free`, `stereo-safe` and `stereo-checked`. Their allowed values are listed in `docs/notes/DECISIONS.md` ("Recipe keys"). Any other key is a route setting, and the loader ignores it.

A complete example:

```text
# Marvel's Spider-Man 2
name: Marvel's Spider-Man 2
stereo-status: build
stereo-route: build: port spidy stereo screen
stereo-engine: custom
stereo-api: dx12
stereo-mod: none
stereo-free: "yes"
stereo-safe: "yes"
stereo-checked: 2026-10-31
```

## Route settings

A recipe's other keys are its route's settings, checked by `check_route_settings` in `crates/engine/src/recipes.rs`. A UEVR recipe must give `uevr-exe`; a Geo-11 recipe must give `geo11-fix` and `geo11-exe`. A key that is not in its route's list is refused.

<!-- BEGIN GENERATED -->
| Route | Key | Allowed values |
|---|---|---|
| uevr | `uevr-exe` | any non-empty text (required) |
| uevr | `uevr-rendering` | `native` or `synced` |
| uevr | `uevr-2d-screen` | `yes` or `no` |
| uevr | `uevr-native-stereo-fix` | `yes` or `no` |
| uevr | `uevr-delay` | a whole number from 0 to 120 |
| uevr | `uevr-profile` | a pin key: lowercase letters, digits and `-` |
| geo11 | `geo11-fix` | a pin key: lowercase letters, digits and `-` (required) |
| geo11 | `geo11-exe` | any non-empty text (required) |
| geo11 | `geo11-layout` | `sbs`, `tab`, `sbs_reversed` or `tab_reversed` |
| geo11 | `geo11-separation` | a whole number from 0 to 100 |
| geo11 | `geo11-convergence` | a number above 0 |
| geo11 | `geo11-d3dx` | `key=value` pairs separated by `;` |
| geo11 | `geo11-remove-first` | relative paths separated by `;` (no `..`, no leading `/`) |

The port route takes no keys (`PORT_KEYS` is empty).
<!-- END GENERATED -->
