<!-- split from roadmap-v2.md lines 3646-3712 at c0e6874 -->
### WP-25a: Docs check (wave 1)

**Closes:** T2 (the checker). Runs in **wave 1**, in parallel with WP-01/WP-02, so that every docs-writing WP in wave 2 can use it as a done-check.

**Files (exclusive):** `tools/docs/check.py`, `tools/docs/test_check.py`, `tools/docs/private-allow.txt`.

**check.py** (`python3 tools/docs/check.py [--root DIR] [--only status,links,mermaid,usage,private] [PATHS...]`):
- Output: `path:line: rule: msg`. Exit 0 clean, 1 findings, 2 usage/IO error.
- Files:
  - `PATHS` (repo-relative) limits the check to those files. Docs-writing WPs check only their own files, before WP-25b has scrubbed the rest.
  - With no `PATHS`, it checks `git -C root ls-files`, or walks the directory when there is no `.git`.
- Always skipped: `third_party/`, `**/tests/**`, `Cargo.lock`, `pnpm-lock.yaml`, anything not UTF-8.

Rules:
- **status:** in code-map's Parts table, when the status cell starts with `present`, `partly present` or `stub`, every backticked path in cell 1 must exist.
  - A bare name resolves against the previous path's directory.
  - `*` globs must match something. A trailing `/` means a directory.
  - `planned` rows are not checked.
- **links:** for each `[..](t)` outside code fences:
  - skip `http(s):`, `mailto:` and `#…`;
  - strip `#`/`?`;
  - resolve against the file's directory (a leading `/` means the repo root);
  - the target must exist.
- **mermaid:** inside ```` ```mermaid ````:
  - remove `"…"`, `|…|`, the bodies of `[..]`/`(..)`/`{..}`, and ` & `;
  - split on `-->|==>|-.->|---`;
  - a whitespace-separated token inside an inner segment is a finding ("2 statements on one line").
- **usage:** every literal `usage: flat3d-inject …` in `crates/engine/src/cli.rs` and `cli/*.rs` is split on `|`.
  - Each alternative's first word must appear in column 1 of code-map's `## Commands`.
  - One-way only: K11, and the locked USAGE at cli.rs:50-51 omits `undo`.
- **private:** `\bI[a]n\b`, `(?i)hyperion`, `B:/`, `/home/[a-z_][a-z0-9_-]*`, `(?i)qwen`, `FF8S5`, `(?i)sandisk`, `\bMDT\b`, `(?i)craft-build`, `(?<![\w/.])iannitram(?!/)`, and e-mail `[\w.+-]+@[\w-]+\.[a-z]{2,}`. The allow-list can lift any of these.

`private-allow.txt` lines are `<glob>` (whole file) or `<glob>\t<substring>` (only the lines containing it). Initial contents:
```
docs/notes/DECISIONS.md
docs/notes/**
tools/docs/check.py
tools/docs/test_check.py
tools/docs/private-allow.txt
decky/plugin.json	"author"
.github/ISSUE_TEMPLATE/issue.md	Qwen brief
AGENTS.md	Qwen brief
docs/workbench.md	craft-build
*	FLAT3D_BUILD_BOX
```
- `docs/notes/**`: the research notes carry "sandisk" drive names (research-report.md:73,84,129; games_group_a.md:40). Their archive move is a follow-up.
- `decky/plugin.json` `"author"`: the O-9 default keeps the publishing handle.

**Tests** (`tools/docs/test_check.py`): each test builds a temp tree with no `.git`. Forbidden words are assembled at runtime (`"Hyp"+"erion"`), so the file itself stays clean.
- status: a `present` row with a missing file → finding; `planned` → none; a bare-name cell resolves.
- links: a dead link → finding; an anchor-only link and an http link → none; a link inside a code fence is ignored.
- mermaid: the code-map:21 shape → finding; a chain `A --> B --> C` → none; a quoted label holding `-->` → none.
- usage: USAGE `list | inject <appid>` with only `list` in the table → finding; an extra table row → none.
- private: a hit → finding; a whole-file glob or substring allow → none; a `tests/` path is exempt; `iannitram/flat-to-3d` passes.
- `PATHS`: a finding in an unlisted file is not reported.
- Exit codes are 0/1/2.

**Done-check for wave-2 docs writers** (WP-24, WP-25b, and any other WP that writes `.md`): `python3 tools/docs/check.py <the .md files it wrote or changed>`. A full-repo `check.py` with no paths is expected to fail until WP-25b lands.

**Depends on:** none. Its tests use temp trees, so it does not wait for WP-01's `crates/` paths.

**Owner device:** none.

**Done-check:** `python3 -m unittest discover -s tools/docs -p 'test_*.py' && python3 -m py_compile tools/docs/check.py && python3 tools/docs/check.py --only private tools/docs/check.py tools/docs/test_check.py tools/docs/private-allow.txt`

---
