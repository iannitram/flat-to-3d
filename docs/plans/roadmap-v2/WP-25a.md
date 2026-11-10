<!-- split from roadmap-v2.md lines 3646-3712 at c0e6874 -->
### WP-25a: Docs check (wave 1)

**Closes:** T2 (the checker). Runs in **wave 1**, in parallel with WP-01/WP-02, so that every docs-writing WP in wave 2 can use it as a done-check.

**Files (exclusive):** `tools/docs/check.py`, `tools/docs/test_check.py`, `tools/docs/private-allow.txt`.

**check.py** (`python3 tools/docs/check.py [--root DIR] [--only RULES] [--allow FILE] [--json] [--changed BASE] [PATHS...]`):
- Rule names are frozen: `status`, `links`, `mermaid`, `usage`, `paths`, `ids`, `generated`, `private`. `--only` takes a comma list of them. An unknown name exits 2 and prints the valid names.
- Output: one line per finding, `path:line: rule: msg (fix: <hint>)`, sorted by path, line and rule. Each rule's hint is given below.
  - `--json` prints one JSON array instead, `[{"path","line","rule","msg","fix"}]`, on stdout. The exit codes do not change.
  - Exit 0 clean, 1 findings, 2 usage/IO error. A usage error names the fix on stderr (the valid rule names, or the missing path).
- `--root` defaults to `Path(__file__).resolve().parents[2]`.
- Files:
  - `PATHS` (repo-relative) limits the check to those files. Docs-writing WPs check only their own files, before WP-25b has scrubbed the rest. A `PATHS` entry that does not exist exits 2.
  - `--changed BASE` checks the files `git -C root diff --name-only --diff-filter=d $(git merge-base BASE HEAD)` lists, so a branch checks only what it changed. It exits 2 when root is not a git repo or BASE does not resolve. `--changed` together with `PATHS` exits 2.
  - With neither, it checks `git -C root ls-files`, or walks the directory when there is no `.git`.
- Always skipped: `docs/plans/` (the plan files quote the rule words, demo ids and marker strings on purpose, so no rule runs on them), `third_party/`, any path with a `tests` component (so `**/tests/**` and `crates/*/tests/`), `Cargo.lock`, `pnpm-lock.yaml`, non-regular files (such as submodule gitlinks), and anything not UTF-8.
- Rule scope:
  - `status` and `usage` run only when `docs/code-map.md` is in the checked set. `paths` runs on `docs/ROADMAP.md` and `docs/code-map.md` when they are in it.
  - `links`, `mermaid`, `ids` and `generated` run on `*.md` only. `generated` also reads `recipes.rs` and runs `session.py render` as inputs; it reports findings only in the `.md` files.
  - `private` runs on every checked UTF-8 file.
- check.py only reads. It has no `--write` and no generator, so a WP that owns a generated block writes the block itself (WP-30's `fill_map.py`, WP-24's `session.py render`, WP-34's hand-written route-keys block).
- check.py has no plan lint. Waves, depends and `files:` lists are checked by `tools/pm/issues_report.py --roadmap` (WP-02). Architecture lints (no I/O in the app, only exec.rs spawns) are WP-02's and WP-01b's.

Rules:
- **status:** the Parts table is the first table under code-map's `## Parts`. Column 1 is the path, column 2 the status. When the status cell starts with `present`, `partly present`, `stub` or `skeleton`, every backticked path in cell 1 must exist.
  - A bare name resolves against the previous path's directory.
  - One level of `{a,b}` braces expands: `crates/engine/src/{api,link,recipes}.rs` is three paths, and each must exist. WP-25b keeps such cells.
  - `*` globs must match something. A trailing `/` means a directory.
  - `planned` rows are not checked.
  - fix: create the path, or mark the row `planned`.
- **links:** for each `[..](t)` outside code fences:
  - skip `http(s):`, `mailto:` and `#…`;
  - strip `#`/`?`;
  - resolve against the file's directory (a leading `/` means the repo root);
  - the target must exist.
  - fix: point the link at an existing file.
- **mermaid:** inside ```` ```mermaid ````:
  - remove `"…"` first, then `|…|`, then the bodies of `[..]`/`(..)`/`{..}`, and ` & `;
  - split on the arrow regex `-{2,}>|={2,}>|-\.+->|-{3,}`;
  - a whitespace-separated token inside an inner segment is a finding ("2 statements on one line").
  - fix: put one statement per line.
- **usage:** every string literal in `crates/engine/src/cli.rs` and `crates/engine/src/cli/*.rs` that matches `usage: flat3d-inject ([^"]*)` is split on `|`.
  - Take each alternative's first word. Skip a word that starts with `{`, `<` or `[` (`cli/link.rs` has `format!("usage: flat3d-inject {form}")`).
  - Each word must appear in column 1 of code-map's `## Commands` table. A column-1 cell gives the first word of each backtick span in it, or the cell's first word when it has no backticks, so `` `inject <appid>` `` gives `inject`.
  - No `## Commands` table is one finding. WP-25b adds the table, so a full run fails here until then.
  - One-way only: K11, and the locked USAGE at cli.rs:50-51 omits `undo`. The check of the table against `cli::COMMANDS` is WP-34's, not check.py's.
  - fix: add a `<word>` row to code-map `## Commands`.
- **paths:** in `docs/ROADMAP.md` and `docs/code-map.md`, flag `(?<![\w./-])(?:inject/|app/src\b)`, except on the lines from a `<!-- mv-map -->` line to the next `<!-- /mv-map -->` line. WP-01a writes that block in code-map; it is the one exemption (§0, bones §2).
  - fix: write the `crates/engine/` or `crates/app/src` path.
- **ids:** every cited spec item and ruling must exist.
  - `\bM(\d+)\b` must have a row `| M<n> |` in `SPEC.md`.
  - `\b[Rr]ulings? ` followed by numbers (`ruling 23`, `rulings 26, 27`, `rulings 24, 25 and 35`, `rulings 24-41`) must each exist in `docs/notes/DECISIONS.md`: a row `| <n> |` in its `## Rulings` table, or a line starting `Ruling <n> ` (a retired ruling, such as 14). A range checks every number in it. A 4-digit number is a date (`ruling 2026-10-31`), not an id, and is skipped.
  - fix: cite an id that exists in SPEC.md or DECISIONS.md.
- **generated:** bones §2 "One owner per fact".
  - Markers: in each `*.md`, the lines `<!-- BEGIN GENERATED -->` and `<!-- END GENERATED -->` (WP-25b's literals, no suffix) must come in one ordered pair. A stray, nested or second pair is a finding.
  - `recipes/README.md`: when it has the block, the block must hold, in backticks, every string in the `UEVR_KEYS` and `GEO11_KEYS` consts of `crates/engine/src/recipes.rs` (bones §2). fix: add the key to the block (WP-34 writes it by hand). Skipped while the file has no block or recipes.rs has no consts.
  - `docs/STEP0-RUNSHEET.md`: when its first line starts `<!-- generated by tools/session/session.py render` (WP-24's header) and `tools/session/session.py` exists, the file must equal the stdout of `python3 tools/session/session.py render sessions/step0.toml`, run with `sys.executable` from root. fix: `python3 tools/session/session.py render sessions/step0.toml > docs/STEP0-RUNSHEET.md`.
  - `docs/spiderman2/address-map.md`: markers only. Its staleness check is `fill_map.py --check`, which `tools/check.sh` runs (WP-02, WP-30).
- **private:** `\bI[a]n\b`, `(?i)hyperion`, `B:/`, `/home/[a-z_][a-z0-9_-]*`, `(?i)qwen`, `FF8S5`, `(?i)sandisk`, `\bMDT\b`, `(?i)craft-build`, `(?<![\w/.])iannitram(?!/)`, and e-mail `[\w.+-]+@[\w-]+\.[a-z]{2,}`. The allow-list can lift any of these.
  - check.py assembles every pattern from fragments at runtime (`"Hyp"+"erion"`), as the tests do, so check.py and test_check.py are clean with no allow line.
  - fix: remove the word, or add an allow line to `tools/docs/private-allow.txt`.

**private-allow.txt:**
- `--allow FILE` picks the file. The default is `<root>/tools/docs/private-allow.txt`; a missing file means an empty list.
- Lines are `<glob>` (whole file) or `<glob>\t<substring>` (only the lines containing it). `#` lines and blank lines are ignored. Globs use `fnmatch` on the repo-relative POSIX path.
- Initial contents:
```
docs/notes/DECISIONS.md
docs/notes/**
tools/docs/private-allow.txt
decky/plugin.json	"author"
.github/ISSUE_TEMPLATE/issue.md	Qwen brief
AGENTS.md	Qwen brief
docs/workbench.md	craft-build
*	FLAT3D_BUILD_BOX
```
- `docs/notes/**`: the research notes carry "sandisk" drive names (research-report.md:73,84,129; games_group_a.md:40). Their archive move is a follow-up.
- `decky/plugin.json` `"author"`: the O-9 default keeps the publishing handle.
- `tools/docs/private-allow.txt` lists the words it lifts, so it allows itself.

**Tests** (`tools/docs/test_check.py`): `unittest.TestCase` classes. Each test builds a temp tree with no `.git` (except the `--changed` test) and runs check.py through `subprocess` with `sys.executable` and `Path(__file__).with_name('check.py')`. Forbidden words are assembled at runtime (`"Hyp"+"erion"`), so the file itself stays clean.
- status: a `present` row with a missing file → finding; `planned` → none; a bare-name cell resolves; a brace cell with one missing member → finding; a `skeleton` row with a missing dir → finding.
- links: a dead link → finding; an anchor-only link and an http link → none; a link inside a code fence is ignored.
- mermaid: the code-map:21 shape, `    Watch --> Link["a: b"]    Watch --> Health["c"]` → finding; a chain `A --> B --> C` → none; a quoted label holding `-->` → none; a `-.->` edge with a `|"…"|` label → none.
- usage: USAGE `list | inject <appid>` with only `list` in the table → finding; an extra table row → none; `format!("usage: flat3d-inject {form}")` → none; a `` `inject <appid>` `` column-1 cell counts as `inject`.
- paths: an `inject/` token in code-map → finding; the same token inside the mv-map block → none; `crates/engine/` and `flat3d-inject/` → none.
- ids: `M99` with no SPEC row → finding; `rulings 26, 27` with both rows → none; `ruling 2026-10-31` → none; a retired `Ruling 14 ` line counts.
- generated: a BEGIN with no END → finding; a recipes/README block missing one `GEO11_KEYS` entry → finding; a runsheet that differs from a stand-in `session.py render` → finding whose fix is the regenerate command.
- private: a hit → finding; a whole-file glob or substring allow → none; a `tests/` path is exempt; a file under `docs/plans/` is exempt from every rule; `iannitram/flat-to-3d` passes; `--allow /dev/null` lifts nothing.
- `PATHS`: a finding in an unlisted file is not reported; a missing `PATHS` entry exits 2.
- `--changed`: in a temp git repo (skipped when `git` is absent), a finding in an unchanged file is not reported; `--changed` with `PATHS` exits 2.
- `--json`: the array holds the same findings as the text output, with the same exit code.
- `--only`: an unknown rule name exits 2.
- Exit codes are 0/1/2.

**Done-check for wave-2 docs writers** (WP-24, WP-25b, and any other WP that writes `.md`): `python3 tools/docs/check.py --changed "${FLAT3D_BASE:-origin/main}"` (the integration line, §0 "Git references in done-checks"; check.py takes the merge-base itself), or `python3 tools/docs/check.py <the .md files it wrote or changed>`. A full-repo `check.py` with no paths is expected to fail until WP-25b lands.

**Depends on:** WP-00 (as the index lists; its tests use temp trees, so it does not wait for WP-01's `crates/` paths).

**Owner device:** none.

**Done-check:**
```sh
export CARGO_TARGET_DIR=$SCRATCH/target \
&& python3 -m unittest discover -s tools/docs -p 'test_*.py' \
&& python3 -W error -m py_compile tools/docs/check.py tools/docs/test_check.py \
&& python3 tools/docs/check.py --only private --allow /dev/null tools/docs/check.py tools/docs/test_check.py \
&& { test ! -e tools/check.sh || tools/check.sh; }
```
- `-W error` matches check.sh and CI, so an invalid escape sequence fails here.
- `--allow /dev/null` proves check.py and test_check.py are clean with no allow line.
- `tools/check.sh` comes from WP-02 in the same wave, so it closes the chain only when it is already on the branch. Its check.py step stays skipped until WP-25b adds `docs/workbench.md` (bones §2).

---
