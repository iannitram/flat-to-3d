//! E5h: unknown extra lines in a run record are ignored, with or without a tab.

use flat3d_inject::runs::{parse_run_record, RunMode, RunRecord};

fn expected() -> RunRecord {
    RunRecord {
        appid: 2075800,
        pid: 4242,
        mode: RunMode::ThreeD,
    }
}

#[test]
fn an_extra_line_without_a_tab_is_ignored() {
    let text = "flat3d-run 1\nappid\t2075800\nnote without tab\npid\t4242\nmode\t3d\n";
    assert_eq!(parse_run_record(text), Some(expected()));
}

#[test]
fn an_extra_line_with_an_unknown_key_is_ignored() {
    let text = "flat3d-run 1\nappid\t2075800\npid\t4242\nmode\t3d\nfuture\t value\n";
    assert_eq!(parse_run_record(text), Some(expected()));
}
