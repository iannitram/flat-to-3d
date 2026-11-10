# WP-24 recheck: proposed edits (not applied, because plan mode is on)

Target: roadmap-v2.md (WP-24 section)

Checked and found OK:
- Wave 2. The depends (WP-01, WP-25a) are both wave 1.
- No other WP owns `sessions/`, `tools/session` or `tools/stereo`.
- No Rust code and no locked tests are touched.
- Python 3.14 on the host. SpidyVR `spidy-wivrn.sh` honours `SPIDY_DIR`. The submodule is at aed5c0c.

Edits to make in WP-24.md:

1. **Private words.** WP-25a's private rule has `(?i)craft-build` and `/home/[a-z_]…`, and it scans every tracked file except `**/tests/**`.
   - The device enum becomes {z13, quest2, build-box, ci}.
   - `craft-build` appears only on the single `FLAT3D_BUILD_BOX` line.
   - test_session builds `"/home/" + "x"` at runtime.
   - session.py's own regex is written as `/home/[^/\s{]+`.
   - Done-when text that hits the private rule is replaced by "the owner".
2. **render** leaves `{home}`, `{state}`, `{re_home}`, `{build_box}` and `{ans:*}` unfilled, so the output is deterministic and private-safe. It strips the `{repo}/` prefix and fills `[vars]`.
   - The header becomes `<!-- generated; edit sessions/<name>.toml -->`, which WP-34 step 7 requires.
   - The output goes to stdout.
3. **run semantics:**
   - cwd is `{repo}` = `Path(__file__).resolve().parents[2]`.
   - There is no shell, so globs need `find … -name`.
   - A `who=you` cmd is printed, not executed, and answered y/n/s.
   - Only ask, check and `who=you` cmd steps need a `--script` answer. A measure answer is optional: a path, or empty for the newest match.
   - A `s` on an optional step does not change the verdict.
   - A background cmd is ok when it is still alive 2 s after start. It gets SIGTERM, then SIGKILL after 5 s.
   - A timeout is a fail.
   - `expect` is checked with `re.search(..., re.M)`.
   - `--dry-run` prints the filled steps, runs nothing, writes nothing and exits 0.
   - `{{` and `}}` are literal braces. Placeholders are filled only in text/run/expect/input/args, not in `done_when` or trouble.
4. **The evidence line contradicts the test.** The line contains "closes", but the test asserts there is no "closes".
   - The test becomes: no match of `(?i)\b(close[sd]?|fix(e[sd])?|resolve[sd]?)\s+#36\b`.
   - `<appid>` comes from `[vars].appid`, which check requires when `evidence_only` is non-empty.
5. **`issues` may be empty** (q0 and q2 have no issue number).
6. **Rescope q0, q2 and q12 to §5** (the WP's own flag says §5 wins):
   - q0 = `pin.sh --all-tbd`, with the gate grep `\.sha256: TBD-at-pin$` and ok_rc [1]. The key names in WP-02's pins are unknown.
   - q2 = `tools/build-win.sh --fetch-sdk`, then `--install` (WP-15). Gate: the exe exists.
   - q12 = `tools/re/analyze.sh all …` (WP-30; pipeline.sh no longer exists), plus `fill_map.py`. issues=[19].
   - The Decky Game Mode gates move into decky.toml (Q14, as WP-32 expects).
   - The doctor `"ok":\s*true` gate moves into link-smoke (Q4).
7. **tools-build:** drop the `sha256sum` of the zip, which lives in a mktemp dir. build-tools.sh verifies the sha itself (WP-22 step 1).
8. **g2-viewer:** add `--manifest-path {repo}/Cargo.toml`.
   - No WP ships `--dump-eyes`, so the measure becomes an optional disparity on `{state}/captures/viewer-*.png`.
9. **link-smoke:** use `find {state} -maxdepth 1 -name 'soak-*.txt'` with expect `soak-`.
10. **Rules:** tests are written as unittest.TestCase (WP-02).
    - Network: read-only `gh issue view` to copy each Done-when.
11. **Done-check:** add `python3 tools/docs/check.py --only private tools/session/*.py tools/stereo/*.py sessions/*.toml`.

Cross:
- bones §4 index: the WP-24 file list lacks q0/q2/q12.toml, and its depends lacks WP-25a. WP-25 is now split into 25a/25b.
- WP-34 step 8 (allow-list reduced to one line) drops `*\tFLAT3D_BUILD_BOX`. That breaks session.py's default line, and WP-25b's/WP-30's scripts too.
