# WP-13 recheck: proposed edits (not applied: plan mode was active)

Target: roadmap-v2.md (WP-13 section)

Verified OK: wave 2 with depends WP-01, WP-02 (both wave 1); files exclusive (no other WP owns them);
WP-01 adds `pub mod fetch;` to injectors.rs with an empty stub; include_str path `../../../../packaging/pins`
resolves to repo root from crates/engine/src/injectors/; repo has `CommandOutput{success,stdout,stderr}`,
unit `SystemRunner`, `ApiError::new(code,msg)`; no locked test globs `injectors`; 8 locked codes are
api_response.rs:10-11.

## Edits to apply

1. Signatures block, add imports line before `pub const PINS`:
   `use std::collections::BTreeMap; use std::path::PathBuf; use crate::exec::CommandRunner; use crate::paths::EnginePaths; use crate::api::ApiError;`
2. Behaviour/parse_pins, add: `size` parses as u64 (absent/unparsable → 0); `fetch: auto|manual` → Some(Auto|Manual), else None.
3. Behaviour/pin_in, add: `size == 0` is missing → `BadPin{field:"size"}`; missing url → `BadPin{field:"url"}`; bad sha → `BadPin{field:"sha256"}`; `fetch` None is treated as Auto.
4. Step 3: "remove it (ignore NotFound), then `create_dir_all(tmp/out)`".
5. Step 4: both `--proto` and `--proto-redir` become `=https,file` under allow_file. Exact argv for test:
   `["curl","--proto","=https","--proto-redir","=https","--tlsv1.2","-fL","--retry","3","--max-filesize","6","-o","<tmp>/ok.zip","https://example.invalid/d/ok.zip?x=1"]` for fixture `ok` (size 5, url as given).
6. Step 6: `ShaMismatch` compares lowercase hex; a failing sha256sum → `Command`.
7. ApiError: `ApiError::new(code, &e.to_string())`; codes are api_response.rs:10-11 (was :10).
8. Fixture: `ok.url: https://example.invalid/d/ok.zip?x=1`, `ok.size: 5`, `ok.sha256: ` + 64 `a`, `ok.version: 1`, `ok.license: MIT`, `ok.fetch: auto`; `nourl.sha256:` 64 `b`.
9. real_pins: assert sha starts_with("4cd6932d") && ends_with("dc6d8") (full sha is not in this file).
10. Live tests: build EnginePaths with the §2 frozen form `EnginePaths::from_env(&BTreeMap::from([("HOME",home),("XDG_CACHE_HOME",home/cache)]), 1000)`; pins are struct literals with `..Default::default()`; size from metadata and sha from `sha256sum` at test time; `second_fetch_runs_no_commands` uses the local recording fake for the second call and asserts it recorded nothing.
11. Inline facts line 87 → "`EnginePaths::cache()` = `$XDG_CACHE_HOME/flat3d`, falling back to `~/.cache/flat3d` (bones §2 / WP-01 accessor table)."
12. Flags: CI flag → runners are `[self-hosted, linux]`; bones §2 apt list lacks `libarchive-tools` (WP-01 has it); live tests skip without bsdtar. uevr flag → `uevr_cache()` (no arg) should be `cache()/uevr`, i.e. cache_dir of the uevr pin's parent; WP-14 `uevr_dir` already follows cache_dir.
13. Done-check: `CARGO_TARGET_DIR=<agent scratch>/target` on both commands ($SCRATCH undefined; clippy lacked it).
