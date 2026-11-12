use flat3d_inject::face::{headset_view, logs};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::runs::{RunMode, RunRecord};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

fn paths(test: &str) -> EnginePaths {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("face_logs")
        .join(test);
    EnginePaths {
        home: root.join("home"),
        uid: 0,
        state_dir: root.join("runtime"),
        config: root.join("config"),
        data: root.join("data"),
        cache_dir: root.join("cache"),
        log_dir: root.join("logs"),
        bin: root.join("bin"),
        recipe_dirs: vec![],
    }
}

fn write_log(path: PathBuf, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn write_record(p: &EnginePaths, appid: u32, pid: u32, mode: RunMode, ts: u64) {
    let path = PathBuf::from(flat3d_inject::state::run_record_path(
        &p.state_dir.to_string_lossy(),
        pid,
    ));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let record = RunRecord { appid, pid, mode };
    let mut file = File::create(&path).unwrap();
    file.write_all(record.to_text().as_bytes()).unwrap();
    file.set_modified(UNIX_EPOCH + Duration::from_secs(ts))
        .unwrap();
}

#[test]
fn merges_three_sources_newest_first() {
    let p = paths("merges_three_sources_newest_first");
    write_log(p.link_log(), "100 a\n300 c\n");
    write_log(p.run_log(), "200 appid=1 pid=2 b\n");
    write_record(&p, 1, 2, RunMode::ThreeD, 250);
    assert_eq!(
        logs::read_logs(&p, 500),
        [
            "300 link: c",
            "250 record: appid=1 pid=2 mode=3d",
            "200 run: appid=1 pid=2 b",
            "100 link: a"
        ]
    );
}

#[test]
fn max_truncates_and_blanks_drop() {
    let p = paths("max_truncates_and_blanks_drop");
    write_log(p.link_log(), "1 a\n\n\r\n2 b\n3 c\n");
    assert_eq!(logs::read_logs(&p, 2), ["3 link: c", "2 link: b"]);
}

#[test]
fn untimed_line_inherits_previous_ts() {
    let p = paths("untimed_line_inherits_previous_ts");
    write_log(p.link_log(), "untimed first\n10 start\ncontinued line\n");
    assert_eq!(
        logs::read_logs(&p, 500),
        [
            "10 link: continued line",
            "10 link: start",
            "0 link: untimed first"
        ]
    );
}

#[test]
fn missing_dirs_give_empty() {
    assert!(logs::read_logs(&paths("missing_dirs_give_empty"), 500).is_empty());
}

#[test]
fn link_log_as_dir_and_garbage_record_are_skipped() {
    let p = paths("link_log_as_dir_and_garbage_record_are_skipped");
    fs::create_dir_all(p.link_log()).unwrap();
    write_log(p.run_log(), "5 appid=1 pid=2 ok\n");
    let run_dir = p.state_dir.join("runs");
    fs::create_dir_all(&run_dir).unwrap();
    fs::write(run_dir.join("run-9.txt"), "garbage\n").unwrap();
    fs::write(
        run_dir.join("notes.txt"),
        RunRecord {
            appid: 9,
            pid: 9,
            mode: RunMode::Flat,
        }
        .to_text(),
    )
    .unwrap();
    assert_eq!(logs::read_logs(&p, 500), ["5 run: appid=1 pid=2 ok"]);
}

#[test]
fn tail_reads_last_64k_without_partial_line() {
    let p = paths("tail_reads_last_64k_without_partial_line");
    let text = (0..1000)
        .map(|i| format!("{i:05} {:0>93}\n", i))
        .collect::<String>();
    assert_eq!(text.len(), 100_000);
    write_log(p.link_log(), &text);
    let lines = logs::read_logs(&p, 10_000);
    assert_eq!(lines.len(), 655);
    assert_eq!(lines.first().unwrap(), &format!("999 link: {:0>93}", 999));
    assert!(lines.last().unwrap().starts_with("345 link: "));
}

#[test]
fn log_lines_matches_newest_first() {
    let text = "a\r\n\n b\n";
    assert_eq!(logs::log_lines(text, 500), [" b", "a"]);
    assert_eq!(
        logs::log_lines(text, 500),
        headset_view::log_lines_newest_first(text, 500)
    );
}

#[test]
fn is_error_line_words() {
    for line in [
        "Command failed: exit 1",
        "Could not run command",
        "exec: No such file",
    ] {
        assert!(logs::is_error_line(line));
    }
    assert!(!logs::is_error_line("100 link: ready"));
}

#[test]
fn logs_rs_has_no_spawn_clock_or_write() {
    let source = include_str!("../src/face/logs.rs");
    for forbidden in [
        "journalctl",
        "Command::new",
        "SystemTime::now",
        "unix_now",
        "fs::write",
        "OpenOptions",
        "create_dir",
        "remove_file",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden source token: {forbidden}"
        );
    }
}
