//! Locked test: the code map's error table (docs/code-map.md, "API shape") copies
//! `api::ERROR_CODES`, `api::error_help` and `cli::exit_code` exactly. `api.rs` owns the values;
//! the table is the copy, so a drift is fixed in the table.

use flat3d_inject::api::{error_help, ApiError, ERROR_CODES};
use flat3d_inject::cli::exit_code;

const HEADER: &str = "| Code | Exit | Retryable | Fix (`api::error_help`) |";

/// One table row: (code, exit, retryable, fix with backticks removed).
fn rows() -> Vec<(String, i32, bool, String)> {
    let map = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/code-map.md");
    let text = std::fs::read_to_string(map).unwrap();
    let mut lines = text.lines().skip_while(|line| line.trim_end() != HEADER);
    assert_eq!(
        lines.next().map(str::trim_end),
        Some(HEADER),
        "error table header"
    );
    assert_eq!(
        lines.next().map(str::trim_end),
        Some("|---|---:|---|---|"),
        "separator"
    );
    let mut out = Vec::new();
    for line in lines {
        let line = line.trim_end();
        if !line.starts_with('|') {
            break;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        assert_eq!(cells.len(), 4, "row has four cells: {line}");
        let code = cells[0].trim_matches('`').to_string();
        let exit: i32 = cells[1].parse().unwrap();
        let retryable = match cells[2] {
            "yes" => true,
            "no" => false,
            other => panic!("retryable cell must be yes or no, got {other:?}"),
        };
        let fix = cells[3].replace('`', "");
        out.push((code, exit, retryable, fix));
    }
    out
}

#[test]
fn the_table_lists_every_error_code_in_api_order() {
    // Catches: a code missing, added or reordered in the table.
    let codes: Vec<String> = rows().into_iter().map(|r| r.0).collect();
    assert_eq!(codes, ERROR_CODES.map(String::from).to_vec());
}

#[test]
fn every_exit_cell_matches_cli_exit_code() {
    // Catches: an exit cell that disagrees with cli::exit_code (2 for usage and not-built, else 1).
    for (code, exit, _, _) in rows() {
        assert_eq!(exit, exit_code(&ApiError::new(&code, "x")), "{code}");
    }
}

#[test]
fn every_retryable_cell_matches_error_help() {
    // Catches: a yes/no that disagrees with error_help(code).0.
    for (code, _, retryable, _) in rows() {
        assert_eq!(retryable, error_help(&code).0, "{code}");
    }
}

#[test]
fn every_fix_cell_matches_error_help_word_for_word() {
    // Catches: the not-built and not-found cells as they are on main (shortened copies).
    for (code, _, _, fix) in rows() {
        assert_eq!(fix, error_help(&code).1, "{code}");
    }
}

#[test]
fn the_not_built_cell_is_the_full_help_text() {
    // Catches: a fix in error_help instead of the table (api.rs owns the text).
    let row = rows().into_iter().find(|r| r.0 == "not-built").unwrap();
    assert_eq!(
        row.3,
        "this part is not built yet; the message names the work package that builds it"
    );
}

#[test]
fn the_not_found_cell_is_the_full_help_text() {
    // Catches: a fix in error_help instead of the table (api.rs owns the text).
    let row = rows().into_iter().find(|r| r.0 == "not-found").unwrap();
    assert_eq!(
        row.3,
        "check the name or path in the message; flat3d-inject list shows the installed games"
    );
}
