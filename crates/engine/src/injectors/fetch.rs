//! Pinned, checksum-verified downloads into the per-user cache.

use crate::api::ApiError;
use crate::exec::CommandRunner;
use crate::paths::EnginePaths;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub const PINS: &str = include_str!("../../../../packaging/pins");
pub const TBD: &str = "TBD-at-pin";
pub const MARKER: &str = ".flat3d-fetch";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMode {
    Auto,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pin {
    pub key: String,
    pub version: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub md5: Option<String>,
    pub license: String,
    pub fetch: Option<FetchMode>,
    pub page: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    UnknownKey(String),
    TbdAtPin(String),
    Manual {
        key: String,
        page: String,
        note: String,
    },
    BadPin {
        key: String,
        field: &'static str,
    },
    Scheme(String),
    Command {
        command: String,
        message: String,
    },
    SizeMismatch {
        key: String,
        expected: u64,
        got: u64,
    },
    ShaMismatch {
        key: String,
        expected: String,
        got: String,
    },
    Io(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownKey(key) => write!(f, "unknown pin key {key}"),
            Self::TbdAtPin(key) => write!(f, "{key} has no recorded sha256; run packaging/pin.sh {key}"),
            Self::Manual { key, page, note } => write!(f, "{key} must be fetched manually; see {page}: {note}"),
            Self::BadPin { key, field } => write!(f, "{key} has an invalid or missing {field} pin"),
            Self::Scheme(url) => write!(f, "unsupported URL scheme for {url}"),
            Self::Command { command, message } => write!(f, "{command} failed: {message}"),
            Self::SizeMismatch { key, expected, got } => write!(f, "{key} size mismatch: expected {expected}, got {got}; run packaging/pin.sh {key} --check"),
            Self::ShaMismatch { key, expected, got } => write!(f, "{key} sha256 mismatch: expected {expected}, got {got}; run packaging/pin.sh {key} --check"),
            Self::Io(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for FetchError {}

impl From<FetchError> for ApiError {
    fn from(error: FetchError) -> Self {
        let code = match &error {
            FetchError::UnknownKey(_) => "not-found",
            FetchError::TbdAtPin(_)
            | FetchError::Manual { .. }
            | FetchError::BadPin { .. }
            | FetchError::Scheme(_) => "refused",
            FetchError::Command { .. }
            | FetchError::SizeMismatch { .. }
            | FetchError::ShaMismatch { .. } => "command-failed",
            FetchError::Io(_) => "io",
        };
        ApiError::new(code, &error.to_string())
    }
}

/// Parse the line based pin table. Unknown fields are intentionally ignored.
pub fn parse_pins(text: &str) -> BTreeMap<String, Pin> {
    let mut pins = BTreeMap::<String, Pin>::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((left, raw_value)) = line.split_once(": ") else {
            continue;
        };
        let Some((key, field)) = left.split_once('.') else {
            continue;
        };
        if key == "crate" || key == "npm" {
            continue;
        }
        let value = raw_value
            .split_once(" #")
            .map_or(raw_value, |(value, _)| value)
            .trim();
        let pin = pins.entry(key.to_string()).or_insert_with(|| Pin {
            key: key.to_string(),
            ..Pin::default()
        });
        match field {
            "version" => pin.version = value.into(),
            "url" => pin.url = value.into(),
            "size" => pin.size = value.parse().unwrap_or(0),
            "sha256" => pin.sha256 = value.into(),
            "md5" => pin.md5 = Some(value.into()),
            "license" => pin.license = value.into(),
            "fetch" => {
                pin.fetch = match value {
                    "auto" => Some(FetchMode::Auto),
                    "manual" => Some(FetchMode::Manual),
                    _ => None,
                }
            }
            "page" => pin.page = value.into(),
            "note" => pin.note = value.into(),
            _ => {}
        }
    }
    pins
}

pub fn pin(key: &str) -> Result<Pin, FetchError> {
    pin_in(&parse_pins(PINS), key)
}

pub fn pin_in(pins: &BTreeMap<String, Pin>, key: &str) -> Result<Pin, FetchError> {
    let pin = pins
        .get(key)
        .ok_or_else(|| FetchError::UnknownKey(key.into()))?;
    if pin.fetch == Some(FetchMode::Manual) {
        return Err(FetchError::Manual {
            key: key.into(),
            page: pin.page.clone(),
            note: pin.note.clone(),
        });
    }
    if pin.sha256 == TBD {
        return Err(FetchError::TbdAtPin(key.into()));
    }
    for (field, missing) in [("url", pin.url.is_empty()), ("size", pin.size == 0)] {
        if missing {
            return Err(FetchError::BadPin {
                key: key.into(),
                field,
            });
        }
    }
    if !path_safe(&pin.version) || pin.version.starts_with("TBD") {
        return Err(FetchError::BadPin {
            key: key.into(),
            field: "version",
        });
    }
    if !path_safe(&pin.key) {
        return Err(FetchError::BadPin {
            key: key.into(),
            field: "key",
        });
    }
    if pin.sha256.len() != 64 || !pin.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(FetchError::BadPin {
            key: key.into(),
            field: "sha256",
        });
    }
    Ok(pin.clone())
}

fn path_safe(part: &str) -> bool {
    !part.is_empty() && part != "." && part != ".." && !part.contains(['/', '\\'])
}

pub fn cache_dir(pin: &Pin, p: &EnginePaths) -> PathBuf {
    p.cache().join(&pin.key).join(&pin.version)
}

pub fn is_fetched(pin: &Pin, p: &EnginePaths) -> bool {
    fs::read_to_string(cache_dir(pin, p).join(MARKER))
        .is_ok_and(|marker| marker == format!("{}\n", pin.sha256.to_ascii_lowercase()))
}

#[doc(hidden)]
pub struct FetchOpts {
    pub allow_file: bool,
}

pub fn fetch(pin: &Pin, p: &EnginePaths, r: &mut dyn CommandRunner) -> Result<PathBuf, FetchError> {
    fetch_with(pin, p, r, &FetchOpts { allow_file: false })
}

#[doc(hidden)]
pub fn fetch_with(
    pin: &Pin,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
    o: &FetchOpts,
) -> Result<PathBuf, FetchError> {
    let pin = pin_in(
        &[(pin.key.clone(), pin.clone())].into_iter().collect(),
        &pin.key,
    )?;
    let allow_file = o.allow_file;
    if !(pin.url.starts_with("https://") || allow_file && pin.url.starts_with("file://")) {
        return Err(FetchError::Scheme(pin.url));
    }
    let dir = cache_dir(&pin, p);
    if is_fetched(&pin, p) {
        return Ok(dir);
    }
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(io_error)?;
    }
    let parent = p.cache().join(&pin.key);
    let tmp = parent.join(format!(".{}.tmp", pin.version));
    if tmp.exists() {
        fs::remove_dir_all(&tmp).map_err(io_error)?;
    }

    let result = (|| {
        fs::create_dir_all(tmp.join("out")).map_err(io_error)?;
        let name = pin
            .url
            .split('?')
            .next()
            .unwrap_or(&pin.url)
            .rsplit('/')
            .next()
            .unwrap_or("");
        if name.is_empty() {
            return Err(FetchError::BadPin {
                key: pin.key.clone(),
                field: "url",
            });
        }
        let archive = tmp.join(name);
        let curl = vec![
            "curl".into(),
            "--proto".into(),
            if allow_file { "=https,file" } else { "=https" }.into(),
            "--proto-redir".into(),
            if allow_file { "=https,file" } else { "=https" }.into(),
            "--tlsv1.2".into(),
            "-fL".into(),
            "--retry".into(),
            "3".into(),
            "--max-filesize".into(),
            pin.size.saturating_add(1).to_string(),
            "-o".into(),
            archive.to_string_lossy().into_owned(),
            pin.url.clone(),
        ];
        run(r, &curl)?;
        let got = fs::metadata(&archive).map_err(io_error)?.len();
        if got != pin.size {
            return Err(FetchError::SizeMismatch {
                key: pin.key.clone(),
                expected: pin.size,
                got,
            });
        }
        let hash_output = run(
            r,
            &["sha256sum".into(), archive.to_string_lossy().into_owned()],
        )?;
        let got_sha = hash_output
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let expected = pin.sha256.to_ascii_lowercase();
        if got_sha != expected {
            return Err(FetchError::ShaMismatch {
                key: pin.key.clone(),
                expected,
                got: got_sha,
            });
        }
        if is_archive(name) {
            run(
                r,
                &[
                    "bsdtar".into(),
                    "-xf".into(),
                    archive.to_string_lossy().into_owned(),
                    "-C".into(),
                    tmp.join("out").to_string_lossy().into_owned(),
                ],
            )?;
        } else {
            fs::rename(&archive, tmp.join("out").join(name)).map_err(io_error)?;
        }
        fs::write(tmp.join("out").join(MARKER), format!("{}\n", expected)).map_err(io_error)?;
        fs::rename(tmp.join("out"), &dir).map_err(io_error)?;
        Ok(dir.clone())
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

fn run(r: &mut dyn CommandRunner, argv: &[String]) -> Result<String, FetchError> {
    let output = r.run(argv);
    if output.success {
        Ok(output.stdout)
    } else {
        Err(FetchError::Command {
            command: argv.join(" "),
            message: output.stderr.trim().into(),
        })
    }
}

fn io_error(error: std::io::Error) -> FetchError {
    FetchError::Io(error.to_string())
}

fn is_archive(name: &str) -> bool {
    [
        ".tar.zst", ".tar.gz", ".tar.xz", ".txz", ".tgz", ".7z", ".zip", ".tar",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix))
}
