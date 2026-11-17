//! Locked test (WP-31, refs #306): the sidecar spawn moves out of `flat3d-run` into
//! `wrapper::spawn_sidecar_with`. A fake sidecar (a `/bin/sh` script in a temp
//! dir) proves the argv with `--until-pid`, stdin from /dev/null, output to viewer.log, the
//! flat fallback on a spawn error, and a start with null output when viewer.log cannot open.
#![cfg(target_os = "linux")]

use flat3d_inject::paths::EnginePaths;
use flat3d_inject::runs::{parse_run_record, RunMode, RunRecord};
use flat3d_inject::state;
use flat3d_inject::wrapper::{flat_after_sidecar_error, spawn_sidecar_with, ExecSpec, Launch};
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

const NOW: u64 = 1000;

/// The fake sidecar. `$1` is a marker file; every later argument goes to stdout as
/// `arg <word>`, one line to stderr, then the targets of fds 0, 1 and 2 go to the marker.
const FAKE_SIDECAR: &str = r#"mark=$1
shift
for a in "$@"; do echo "arg $a"; done
echo "to stderr" >&2
i=$(readlink /proc/$$/fd/0)
o=$(readlink /proc/$$/fd/1)
e=$(readlink /proc/$$/fd/2)
printf 'stdin %s\nstdout %s\nstderr %s\n' "$i" "$o" "$e" > "$mark.tmp"
mv "$mark.tmp" "$mark"
"#;

struct Tmp(PathBuf);

impl Tmp {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("wrapper-sidecar")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Tmp(root)
    }

    fn paths(&self) -> EnginePaths {
        let env = BTreeMap::from([
            (
                "HOME".to_string(),
                self.0.join("home").to_string_lossy().into_owned(),
            ),
            (
                "XDG_RUNTIME_DIR".to_string(),
                self.0.join("run").to_string_lossy().into_owned(),
            ),
        ]);
        EnginePaths::from_env(&env, 1000).unwrap()
    }

    /// The fake sidecar script (run as `sh <script>`, so it needs no exec bit) and its marker.
    fn script(&self) -> (String, String) {
        let script = self.0.join("fake-viewer.sh");
        std::fs::write(&script, FAKE_SIDECAR).unwrap();
        let mark = self.0.join("mark.txt");
        (
            script.to_string_lossy().into_owned(),
            mark.to_string_lossy().into_owned(),
        )
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn s(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

/// Wait (up to 20 s) for the fake sidecar's marker and return it.
fn wait_for(mark: &str) -> String {
    let start = Instant::now();
    loop {
        if let Ok(text) = std::fs::read_to_string(mark) {
            return text;
        }
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "the sidecar never wrote {mark}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn canonical(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

/// Run test `name` again in a child test process whose stdin is a pipe, so an inherited stdin
/// can never look like /dev/null. Returns true inside that child (run the body there).
fn in_child_with_piped_stdin(name: &str) -> bool {
    if std::env::var_os("FLAT3D_SIDECAR_TEST_CHILD").is_some() {
        return true;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env("FLAT3D_SIDECAR_TEST_CHILD", "1")
        .stdin(Stdio::piped())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("1 passed"), "{stdout}");
    false
}

fn three_d_launch() -> Launch {
    Launch {
        command: ExecSpec {
            argv: s(&["proton", "waitforexitandrun", "game.exe"]),
            env: vec![("WINEDLLOVERRIDES".into(), "d3d11=n,b".into())],
            unset: s(&["XR_RUNTIME_JSON"]),
            sidecar: Some(s(&["flat3d-viewer", "--layout", "sbs"])),
        },
        record: RunRecord {
            appid: 42,
            pid: 7,
            mode: RunMode::ThreeD,
        },
        message: "3d: geo-11".to_string(),
    }
}

/// `err` from a failed spawn goes through `flat_after_sidecar_error`: flat record, reason
/// `sidecar`, the error as the note, and the bare game command.
fn assert_flat_fallback(paths: &EnginePaths, err: &str) {
    let spec = flat_after_sidecar_error(paths, &three_d_launch(), NOW, err);
    assert_eq!(spec.argv, s(&["proton", "waitforexitandrun", "game.exe"]));
    assert!(spec.env.is_empty());
    assert!(spec.unset.is_empty());
    assert_eq!(spec.sidecar, None);
    let saved = std::fs::read_to_string(state::run_record_path(
        &paths.state_dir.to_string_lossy(),
        7,
    ))
    .unwrap();
    assert_eq!(parse_run_record(&saved).unwrap().mode, RunMode::Flat);
    assert!(
        saved.lines().any(|line| line == "reason\tsidecar"),
        "{saved}"
    );
    let note = format!(
        "note\tsidecar: {}",
        err.strip_prefix("sidecar: ").unwrap_or(err)
    );
    assert!(saved.lines().any(|line| line == note), "{saved}");
}

#[test]
fn the_sidecar_gets_until_pid_null_stdin_and_viewer_log() {
    if !in_child_with_piped_stdin("the_sidecar_gets_until_pid_null_stdin_and_viewer_log") {
        return;
    }
    let tmp = Tmp::new();
    let paths = tmp.paths();
    assert!(!paths.log_dir.exists());
    let (script, mark) = tmp.script();
    let sidecar = vec![
        "sh".to_string(),
        script.clone(),
        mark.clone(),
        "--layout".to_string(),
        "sbs".to_string(),
        "a b".to_string(),
    ];

    let (pid, argv) = spawn_sidecar_with(&paths, &sidecar, Some(Path::new("/bin")), 4242).unwrap();

    assert_eq!(
        argv,
        vec![
            "/bin/sh".to_string(),
            script,
            mark.clone(),
            "--layout".to_string(),
            "sbs".to_string(),
            "a b".to_string(),
            "--until-pid".to_string(),
            "4242".to_string(),
        ]
    );
    assert!(pid > 0);
    assert_ne!(pid, std::process::id());
    let marker = wait_for(&mark);
    let log = paths.log_dir.join("viewer.log");
    let log_name = canonical(&log);
    assert_eq!(
        marker,
        format!("stdin /dev/null\nstdout {log_name}\nstderr {log_name}\n")
    );
    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        "arg --layout\narg sbs\narg a b\narg --until-pid\narg 4242\nto stderr\n"
    );
}

#[test]
fn viewer_log_is_appended_not_replaced() {
    let tmp = Tmp::new();
    let paths = tmp.paths();
    std::fs::create_dir_all(&paths.log_dir).unwrap();
    let log = paths.log_dir.join("viewer.log");
    std::fs::write(&log, "earlier\n").unwrap();
    let (script, mark) = tmp.script();

    let (_, argv) = spawn_sidecar_with(&paths, &s(&["/bin/sh", &script, &mark]), None, 1).unwrap();

    assert_eq!(argv, s(&["/bin/sh", &script, &mark, "--until-pid", "1"]));
    wait_for(&mark);
    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        "earlier\narg --until-pid\narg 1\nto stderr\n"
    );
}

#[test]
fn an_unopenable_viewer_log_still_starts_the_sidecar_with_null_output() {
    let tmp = Tmp::new();
    let paths = tmp.paths();
    // The log folder is a regular file, so viewer.log cannot be created under it.
    std::fs::create_dir_all(paths.log_dir.parent().unwrap()).unwrap();
    std::fs::write(&paths.log_dir, "not a folder\n").unwrap();
    let (script, mark) = tmp.script();

    let (pid, argv) =
        spawn_sidecar_with(&paths, &s(&["/bin/sh", &script, &mark, "x"]), None, 77).unwrap();

    assert!(pid > 0);
    assert_eq!(
        argv,
        s(&["/bin/sh", &script, &mark, "x", "--until-pid", "77"])
    );
    assert_eq!(
        wait_for(&mark),
        "stdin /dev/null\nstdout /dev/null\nstderr /dev/null\n"
    );
    assert!(paths.log_dir.is_file());
    assert_eq!(
        std::fs::read_to_string(&paths.log_dir).unwrap(),
        "not a folder\n"
    );
}

#[test]
fn a_missing_program_is_an_error_and_the_launch_falls_back_to_flat() {
    let tmp = Tmp::new();
    let paths = tmp.paths();
    let missing = tmp.0.join("no-such-viewer").to_string_lossy().into_owned();

    let err = spawn_sidecar_with(&paths, &s(&[&missing, "--layout", "sbs"]), None, 7).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);

    // A relative program found neither beside the exe nor in paths.bin is missing too.
    let empty = tmp.0.join("empty-exe-dir");
    std::fs::create_dir_all(&empty).unwrap();
    let relative = spawn_sidecar_with(&paths, &s(&["flat3d-viewer"]), Some(&empty), 7).unwrap_err();
    assert_eq!(relative.kind(), ErrorKind::NotFound);

    assert_flat_fallback(&paths, &err.to_string());
}

#[test]
fn a_non_executable_program_is_an_error_and_the_launch_falls_back_to_flat() {
    let tmp = Tmp::new();
    let paths = tmp.paths();
    let plain = tmp.0.join("plain-viewer");
    std::fs::write(&plain, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();

    let err = spawn_sidecar_with(&paths, &s(&[&plain.to_string_lossy()]), None, 7).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::PermissionDenied);

    assert_flat_fallback(&paths, &err.to_string());
}

#[test]
fn an_empty_sidecar_is_invalid_input_and_the_launch_falls_back_to_flat() {
    let tmp = Tmp::new();
    let paths = tmp.paths();

    let err = spawn_sidecar_with(&paths, &[], Some(Path::new("/bin")), 7).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
    assert_eq!(err.to_string(), "sidecar: empty argv");

    assert_flat_fallback(&paths, &err.to_string());
    let saved = std::fs::read_to_string(state::run_record_path(
        &paths.state_dir.to_string_lossy(),
        7,
    ))
    .unwrap();
    assert!(saved
        .lines()
        .any(|line| line == "note\tsidecar: empty argv"));
}
