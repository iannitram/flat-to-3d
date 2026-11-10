# Recipes

One file per game, with the stereo keys from `docs/notes/DECISIONS.md` ("Recipe keys"). This folder is the canonical home of the recipes, versioned with the code and read by the Inject engine (`inject/src/recipes.rs`): the owner's ruling of 2026-10-26 (#15). The file format is set by the recipes issue. For 1.0 the engine needs Spider-Man 2's recipe (SPEC M12); the owner's per-game notes elsewhere stay notes.

## Format

One file per game, named `<appid>-<slug>.recipe`: the Steam appid, a dash, then a short name, for example `2651280-spider-man-2.recipe`. The loader reads every `.recipe` file in this folder and ignores other files, this README included.

Each line is one `key: value` pair, split at the first `:`. A line starting with `#` is a comment, and blank lines are skipped. One pair of double quotes around a value is dropped, so `"yes"` and `yes` are the same.

Every file must give these nine keys: `name`, `stereo-status`, `stereo-route`, `stereo-engine`, `stereo-api`, `stereo-mod`, `stereo-free`, `stereo-safe` and `stereo-checked`. Their allowed values are listed in `docs/notes/DECISIONS.md` ("Recipe keys"). Any other key is a route setting, and the loader ignores it.

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
