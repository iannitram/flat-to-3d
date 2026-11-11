//! Locked tests for E5h: the run record text and the running-modes rule.

use flat3d_inject::runs::{parse_run_record, running_modes, RunMode, RunRecord};

fn rec(pid: u32, mode: RunMode) -> RunRecord {
    RunRecord {
        appid: 2075800,
        pid,
        mode,
    }
}

#[test]
fn the_text_is_exactly_this() {
    assert_eq!(
        rec(4242, RunMode::ThreeD).to_text(),
        "flat3d-run 1\nappid\t2075800\npid\t4242\nmode\t3d\n"
    );
    assert_eq!(
        rec(7, RunMode::Flat).to_text(),
        "flat3d-run 1\nappid\t2075800\npid\t7\nmode\tflat\n"
    );
}

#[test]
fn it_reads_back_equal() {
    for m in [RunMode::ThreeD, RunMode::Flat] {
        let r = rec(99, m);
        assert_eq!(parse_run_record(&r.to_text()), Some(r));
    }
}

#[test]
fn extra_lines_are_ignored() {
    let t = "flat3d-run 1\nappid\t5\npid\t6\nmode\tflat\nnote\thello\n";
    assert_eq!(
        parse_run_record(t),
        Some(RunRecord {
            appid: 5,
            pid: 6,
            mode: RunMode::Flat
        })
    );
}

#[test]
fn bad_text_gives_none() {
    assert_eq!(parse_run_record(""), None);
    assert_eq!(
        parse_run_record("flat3d-run 2\nappid\t5\npid\t6\nmode\t3d\n"),
        None
    );
    assert_eq!(parse_run_record("flat3d-run 1\nappid\t5\nmode\t3d\n"), None);
    assert_eq!(
        parse_run_record("flat3d-run 1\nappid\tx\npid\t6\nmode\t3d\n"),
        None
    );
    assert_eq!(
        parse_run_record("flat3d-run 1\nappid\t5\npid\t6\nmode\tvr\n"),
        None
    );
}

#[test]
fn only_live_pids_count() {
    let recs = [rec(1, RunMode::ThreeD), rec(2, RunMode::Flat)];
    assert_eq!(running_modes(&recs, &|_| true), (true, true));
    assert_eq!(running_modes(&recs, &|p| p == 2), (false, true));
    assert_eq!(running_modes(&recs, &|_| false), (false, false));
    assert_eq!(running_modes(&[], &|_| true), (false, false));
}
