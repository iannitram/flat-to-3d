# Device sessions

Each TOML file is a portable run sheet for one owner or device session. The session runner validates the documents, renders a Markdown copy with unresolved machine placeholders, executes scripted checks, records human gates, and stores evidence locally. It never writes to GitHub.

## Commands

Use `python3 tools/session/session.py check [sessions/*.toml]` to validate, `python3 tools/session/session.py render sessions/step0.toml` to render, `python3 tools/session/session.py run sessions/step0.toml` to run, and `python3 tools/session/session.py pending` to list sessions awaiting a passing result.

`check` validates the schema, IDs, dependencies and placeholders. `render` writes a deterministic checklist to stdout; use `--out PATH` to save it. `run` executes script steps and prompts for owner answers. `--script ANSWERS.json` supplies answers by step ID for repeatable runs. `--dry-run` prints the expanded commands without running them. `pending` lists sessions without a passing result and shows whether their prerequisites have passed.

## Session format

TOML documents use `schema = 1` and carry `name`, `title`, `q`, `issues`, optional `spec`, `device`, and `done_when` fields. Optional `after` lists prerequisite sessions, while `needs` records other prerequisites. `evidence_only` marks issue references where a passing session only gathers evidence; an issue in that list also requires `vars.appid`.

Each `[[step]]` has a unique lowercase `id`, a `kind` (`cmd`, `ask`, `check`, or `measure`), `who`, and `text`. A command has an argv array in `run`; it does not invoke a shell unless the argv explicitly starts one. `expect`, `expect_json`, and `expect_code` check command output. A check is a human gate. Measures call `tools/stereo/fps.py` or `tools/stereo/disparity.py` and record their JSON output. Marking a step `optional = true` means a missing input or failed check does not change the session verdict.

Known placeholders include `{repo}`, `{home}`, `{state}`, `{data}`, `{re_home}`, `{build_box}`, user variables from `[vars]`, and `{ans:step-id}` for an earlier answer. Render preserves machine-specific placeholders; run fills them. Set `FLAT3D_RE_HOME` or `FLAT3D_BUILD_BOX` to override their defaults.

## Results

Results go to `${XDG_STATE_HOME:-~/.local/state}/flat3d/sessions/` unless `--state-dir` is passed. Each run creates `<name>-<unix-time>.json` and a paste-ready `<name>-<unix-time>.md`. The JSON records the commit, dirty state, operating system, step statuses, command return codes, short output tails, and measurement metrics. The Markdown has `## Verified` and `## Not verified` sections. Paste that report into the corresponding issue when the owner has completed the device session. Evidence-only issues remain open until their works-now recipe is merged.

## Stereo measurements

`tools/stereo/fps.py CSV --min 30 --json` reads MangoHud's preamble, value row and `fps` header. It reports the median FPS as p50 and p1, using frametime's 99th percentile when available. It exits 0 when p50 meets the threshold, 1 when it does not, and 2 for bad input.

`tools/stereo/disparity.py IMAGE --layout sbs --json` supports non-interlaced PNG and binary PPM/PGM. It compares textured blocks between side-by-side (`sbs`) or top-and-bottom (`tab`) eyes, reports the number of matched blocks, mean absolute disparity and p90. The default pass threshold is mean disparity greater than one pixel. These measurements are supporting evidence; the owner judges headset depth and camera behavior.
