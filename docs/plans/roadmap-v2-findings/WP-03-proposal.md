# WP-03 recheck: proposed edits (plan mode, WP-03.md not edited)

Verified (read-only):
- Issue #76 body fetched live; the WP-03 awk extract gives sha256 f7f72d10…c1b4, 68 lines, 0 CR: matches.
- Helpers and line refs at main@2812f64 are correct: status.rs:14,26,67,73; link_view.rs:29 (texts 31-47); switches.rs:9,27; api.rs:6 (Game derives Clone), LinkState api.rs:47; serde_json is a normal dep (inject/Cargo.toml:20).
- Sort order works-now < likely < untested (status.rs STATUSES), so the locked test order 1,2,3 holds.
- Wave 2, depends WP-01 (wave 1): OK. No other wave-2 WP owns face/rows.rs or tests/rows.rs (WP-21 only cites it). WP-28 (wave 3) depends on WP-03.

Proposed edits to WP-03.md:
1. Input: drop "the WP-00 pre-step writes" and the review-time ENOENT note. WP-00 has no such pre-step. Make it self-contained:
   `export SCRATCH=<agent scratch dir>`; if `$SCRATCH/issues/76.md` is missing:
   `mkdir -p "$SCRATCH/issues" && gh issue view 76 -R iannitram/flat-to-3d --json body -q .body > "$SCRATCH/issues/76.md"` (GitHub read only, network).
   Keep the awk line and the "gh fallback" wording, because WP-04 and WP-05 cite them.
2. Depends: "WP-01. Network: one GitHub read of #76 if no snapshot exists."
3. Signatures: `use super::link_view::flat_reason;`. It resolves with either face.rs+face/ or face/mod.rs.
4. Done-check: `export CARGO_TARGET_DIR="$SCRATCH/target"` up front, so clippy and check.sh share it. Add a verbatim-check:
   `rustfmt --edition 2021 < <(awk … "$SCRATCH/issues/76.md") | diff - crates/engine/tests/rows.rs`
5. State "Wave: 2" explicitly.
