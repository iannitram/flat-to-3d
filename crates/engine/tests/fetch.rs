use flat3d_inject::api::ApiError;
use flat3d_inject::exec::SystemRunner;
use flat3d_inject::injectors::fetch::{self, FetchError, FetchMode, FetchOpts, Pin, MARKER};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::testing::FakeRunner;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const SAMPLE: &str = include_str!("fixtures/pins-sample");
const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn paths(label: &str) -> (EnginePaths, PathBuf) {
    let home = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("fetch-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let env = BTreeMap::from([
        ("HOME".into(), home.to_string_lossy().into_owned()),
        (
            "XDG_CACHE_HOME".into(),
            home.join("cache").to_string_lossy().into_owned(),
        ),
    ]);
    (EnginePaths::from_env(&env, 1000).unwrap(), home)
}

fn sample() -> BTreeMap<String, fetch::Pin> {
    fetch::parse_pins(SAMPLE)
}

#[test]
fn parse_reads_keys_skips_versions_lists() {
    let pins = sample();
    assert_eq!(
        pins.keys().map(String::as_str).collect::<Vec<_>>(),
        ["hand", "nourl", "ok", "tbd"]
    );
    assert_eq!(pins["ok"].fetch, Some(FetchMode::Auto));
    assert_eq!(pins["ok"].sha256, SHA_A);
    assert!(pins["ok"].md5.is_none());
}

#[test]
fn inline_comment_is_stripped() {
    assert_eq!(sample()["ok"].version, "1");
}

#[test]
fn tbd_is_refused_with_pin_sh_hint() {
    let e = fetch::pin_in(&sample(), "tbd").unwrap_err();
    assert!(matches!(e, FetchError::TbdAtPin(_)));
    assert!(e.to_string().contains("run packaging/pin.sh tbd"));
}

#[test]
fn manual_is_refused_with_page() {
    let e = fetch::pin_in(&sample(), "hand").unwrap_err();
    assert!(matches!(e, FetchError::Manual { .. }));
    assert!(e.to_string().contains("https://example.invalid/p"));
}

#[test]
fn unknown_key() {
    assert!(
        matches!(fetch::pin_in(&sample(), "missing"), Err(FetchError::UnknownKey(k)) if k == "missing")
    );
}

#[test]
fn missing_url_is_bad_pin() {
    assert!(matches!(
        fetch::pin_in(&sample(), "nourl"),
        Err(FetchError::BadPin { field: "url", .. })
    ));
}

#[test]
fn real_pins() {
    let uevr = fetch::pin("uevr").unwrap();
    assert_eq!(
        uevr.sha256,
        "4cd6932d1d01168178aef70499d40b101ea6f19868cf76099d8e0087ef8dc6d8"
    );
    assert_eq!(uevr.size, 38974809);
    assert!(matches!(
        fetch::pin("ac7-prereqs"),
        Err(FetchError::Manual { .. })
    ));
}

#[test]
fn every_variant_maps_to_its_contract_code() {
    let cases = [
        (FetchError::UnknownKey("x".into()), "not-found"),
        (FetchError::TbdAtPin("x".into()), "refused"),
        (
            FetchError::Manual {
                key: "x".into(),
                page: "p".into(),
                note: "n".into(),
            },
            "refused",
        ),
        (
            FetchError::BadPin {
                key: "x".into(),
                field: "url",
            },
            "refused",
        ),
        (FetchError::Scheme("x".into()), "refused"),
        (
            FetchError::Command {
                command: "x".into(),
                message: "m".into(),
            },
            "command-failed",
        ),
        (
            FetchError::SizeMismatch {
                key: "x".into(),
                expected: 1,
                got: 2,
            },
            "command-failed",
        ),
        (
            FetchError::ShaMismatch {
                key: "x".into(),
                expected: "a".into(),
                got: "b".into(),
            },
            "command-failed",
        ),
        (FetchError::Io("x".into()), "io"),
    ];
    for (error, expected) in cases {
        assert_eq!(ApiError::from(error).code, expected);
    }
}

#[test]
fn mismatch_names_key_and_pin_sh_check() {
    for error in [
        FetchError::SizeMismatch {
            key: "ok".into(),
            expected: 1,
            got: 2,
        },
        FetchError::ShaMismatch {
            key: "ok".into(),
            expected: "a".into(),
            got: "b".into(),
        },
    ] {
        let message = error.to_string();
        assert!(message.contains("ok"));
        assert!(message.contains("packaging/pin.sh ok --check"));
    }
}

#[test]
fn http_and_file_are_refused_without_allowance_and_run_nothing() {
    let (p, _) = paths("refused");
    let ok = Pin {
        url: "http://example.invalid/d/ok.zip".into(),
        ..sample()["ok"].clone()
    };
    let mut runner = FakeRunner::new();
    assert!(matches!(
        fetch::fetch(&ok, &p, &mut runner),
        Err(FetchError::Scheme(_))
    ));
    let file = Pin {
        url: "file:///tmp/nope".into(),
        ..ok
    };
    assert!(matches!(
        fetch::fetch(&file, &p, &mut runner),
        Err(FetchError::Scheme(_))
    ));
    assert!(runner.ran.is_empty());
}

#[test]
fn curl_argv_is_exact() {
    let (p, _) = paths("argv");
    let mut runner = FakeRunner::new().fail(&["curl"], "boom");
    assert!(fetch::fetch_with(
        &sample()["ok"],
        &p,
        &mut runner,
        &FetchOpts { allow_file: true }
    )
    .is_err());
    let temp = p.cache().join("ok").join(".1.tmp");
    assert_eq!(
        runner.ran[0],
        vec![
            "curl",
            "--proto",
            "=https,file",
            "--proto-redir",
            "=https,file",
            "--tlsv1.2",
            "-fL",
            "--retry",
            "3",
            "--max-filesize",
            "6",
            "-o",
            &temp.join("ok.zip").to_string_lossy(),
            "https://example.invalid/d/ok.zip?x=1"
        ]
    );
}

#[test]
fn curl_failure_names_the_command() {
    let (p, _) = paths("curl-fail");
    let mut runner = FakeRunner::new().fail(&["curl"], "boom");
    let e = fetch::fetch_with(
        &sample()["ok"],
        &p,
        &mut runner,
        &FetchOpts { allow_file: true },
    )
    .unwrap_err();
    assert!(matches!(e, FetchError::Command { .. }));
    assert!(e.to_string().contains("curl"));
}

#[test]
fn is_fetched_reads_the_marker() {
    let (p, _) = paths("marker");
    let pin = sample()["ok"].clone();
    let dir = fetch::cache_dir(&pin, &p);
    fs::create_dir_all(&dir).unwrap();
    assert!(!fetch::is_fetched(&pin, &p));
    fs::write(dir.join(MARKER), "different\n").unwrap();
    assert!(!fetch::is_fetched(&pin, &p));
    fs::write(dir.join(MARKER), format!("{}\n", pin.sha256)).unwrap();
    assert!(fetch::is_fetched(&pin, &p));
}

fn has_tool(tool: &str) -> bool {
    std::process::Command::new("sh")
        .args([
            "-c",
            "command -v \"$1\" >/dev/null 2>&1",
            "check-tool",
            tool,
        ])
        .status()
        .is_ok_and(|s| s.success())
}

fn live_ready(tools: &[&str]) -> bool {
    let missing = tools.iter().find(|tool| !has_tool(tool));
    if let Some(tool) = missing {
        if std::env::var_os("FLAT3D_REQUIRE_HOST_TOOLS").is_some() {
            panic!("missing required host tool: {tool}");
        }
        eprintln!("skip: {tool} is not on PATH");
        false
    } else {
        true
    }
}

fn live_asset(label: &str, content: &[u8], archive: bool) -> Option<(EnginePaths, Pin, PathBuf)> {
    if !live_ready(if archive {
        &["curl", "sha256sum", "bsdtar"]
    } else {
        &["curl", "sha256sum"]
    }) {
        return None;
    }
    let (p, home) = paths(label);
    let source_dir = home.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    let source = if archive {
        source_dir.join("dir/hello.txt")
    } else {
        source_dir.join("plain.bin")
    };
    if let Some(parent) = source.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&source, content).unwrap();
    let asset = if archive {
        let zip = home.join("sample.zip");
        let status = std::process::Command::new("bsdtar")
            .args(["-a", "-cf"])
            .arg(&zip)
            .arg("dir/hello.txt")
            .current_dir(&source_dir)
            .status()
            .unwrap();
        assert!(status.success());
        zip
    } else {
        source
    };
    let size = fs::metadata(&asset).unwrap().len();
    let output = std::process::Command::new("sha256sum")
        .arg(&asset)
        .output()
        .unwrap();
    assert!(output.status.success());
    let sha = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();
    let pin = Pin {
        key: label.into(),
        version: "1".into(),
        url: format!("file://{}", asset.display()),
        size,
        sha256: sha,
        ..Pin::default()
    };
    assert_eq!(pin.sha256.len(), 64, "sha output: {:?}", pin.sha256);
    assert!(
        pin.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "sha output: {:?}",
        pin.sha256
    );
    Some((p, pin, home))
}

fn live_fetch(label: &str, content: &[u8], archive: bool) -> Option<(EnginePaths, Pin, PathBuf)> {
    let (p, pin, _) = live_asset(label, content, archive)?;
    let mut runner = SystemRunner;
    let out = fetch::fetch_with(&pin, &p, &mut runner, &FetchOpts { allow_file: true }).unwrap();
    assert!(fetch::is_fetched(&pin, &p));
    Some((p, pin, out))
}

#[test]
fn zip_unpacks_into_cache_key_version() {
    let Some((_, _, out)) = live_fetch("live-zip", b"hello from archive\n", true) else {
        return;
    };
    assert_eq!(
        fs::read(out.join("dir/hello.txt")).unwrap(),
        b"hello from archive\n"
    );
}

#[test]
fn second_fetch_runs_no_commands() {
    let Some((p, pin, _)) = live_fetch("live-repeat", b"repeat\n", false) else {
        return;
    };
    let mut runner = FakeRunner::new();
    fetch::fetch_with(&pin, &p, &mut runner, &FetchOpts { allow_file: true }).unwrap();
    assert!(runner.ran.is_empty());
}

#[test]
fn plain_file_is_kept_by_name() {
    let Some((_, _, out)) = live_fetch("live-plain", b"plain payload\n", false) else {
        return;
    };
    assert_eq!(fs::read(out.join("plain.bin")).unwrap(), b"plain payload\n");
}

#[test]
fn sha_mismatch_leaves_no_dir() {
    let Some((p, mut pin, _)) = live_asset("live-bad-sha", b"wrong sha\n", false) else {
        return;
    };
    pin.sha256 = SHA_A.into();
    let mut runner = SystemRunner;
    assert!(matches!(
        fetch::fetch_with(&pin, &p, &mut runner, &FetchOpts { allow_file: true }),
        Err(FetchError::ShaMismatch { .. })
    ));
    assert!(!fetch::cache_dir(&pin, &p).exists());
    assert!(!p.cache().join(&pin.key).join(".1.tmp").exists());
}

#[test]
fn size_mismatch() {
    let Some((p, mut pin, _)) = live_asset("live-bad-size", b"size mismatch\n", false) else {
        return;
    };
    pin.size += 1;
    let mut runner = SystemRunner;
    assert!(matches!(
        fetch::fetch_with(&pin, &p, &mut runner, &FetchOpts { allow_file: true }),
        Err(FetchError::SizeMismatch { .. })
    ));
    assert!(!fetch::cache_dir(&pin, &p).exists());
    assert!(!p.cache().join(&pin.key).join(".1.tmp").exists());
}

#[test]
fn size_one_too_small_leaves_no_dir_or_tmp() {
    let Some((p, mut pin, _)) = live_asset("live-small-size", b"size mismatch\n", false) else {
        return;
    };
    pin.size -= 1;
    let mut runner = SystemRunner;
    assert!(matches!(
        fetch::fetch_with(&pin, &p, &mut runner, &FetchOpts { allow_file: true }),
        Err(FetchError::SizeMismatch { .. })
    ));
    assert!(!fetch::cache_dir(&pin, &p).exists());
    assert!(!p.cache().join(&pin.key).join(".1.tmp").exists());
}

#[test]
fn unsafe_version_or_key_is_bad_pin() {
    for version in ["", "TBD-at-pin", "TBDx", ".", "..", "../x", "a/b", "a\\b"] {
        let mut pins = sample();
        pins.get_mut("ok").unwrap().version = version.into();
        assert!(
            matches!(
                fetch::pin_in(&pins, "ok"),
                Err(FetchError::BadPin {
                    field: "version",
                    ..
                })
            ),
            "version {version:?}"
        );
    }
    for key in ["..", "a/b", "a\\b", "."] {
        let mut pins = BTreeMap::new();
        pins.insert(
            key.to_string(),
            Pin {
                key: key.into(),
                ..sample()["ok"].clone()
            },
        );
        assert!(
            matches!(
                fetch::pin_in(&pins, key),
                Err(FetchError::BadPin { field: "key", .. })
            ),
            "key {key:?}"
        );
    }
}
