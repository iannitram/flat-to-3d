//! System profiles: shipped data for an OS and machine setup (#412).
//!
//! One file per platform under `systems/`, in the same `key: value` syntax as the recipes.
//! Each profile gives its match rules, sessions, installs, required tools and known problems.
//! The health check names the matched profile and lists its known problems; an open problem is
//! information, never a failure.

use crate::exec::CommandRunner;
use crate::fsx;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The file extension of one system profile.
pub const EXTENSION: &str = "system";
/// The os-release file read when nothing overrides it.
pub const OS_RELEASE: &str = "/etc/os-release";
/// Allowed `problem-status` values.
pub const PROBLEM_STATUSES: [&str; 2] = ["open", "worked-around"];
/// The single-valued keys a profile may give, other than the problem fields.
pub const KEYS: [&str; 9] = [
    "name",
    "match-os",
    "match-version",
    "match-gpu",
    "desktop-session",
    "game-session",
    "steam-install",
    "wivrn-install",
    "tool",
];
/// The keys one known-problem block gives, after its `known-problem` id line.
pub const PROBLEM_KEYS: [&str; 3] = ["problem-summary", "problem-workaround", "problem-status"];

/// One known problem, in plain words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownProblem {
    /// A short id, used in the health check's line name.
    pub id: String,
    /// A plain summary of what goes wrong.
    pub summary: String,
    /// What to do about it; `none yet` when there is no workaround.
    pub workaround: String,
    /// `open` or `worked-around`.
    pub status: String,
}

impl KnownProblem {
    pub fn is_open(&self) -> bool {
        self.status == "open"
    }
}

/// One shipped platform profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemProfile {
    /// The file stem; set by [`load_systems`].
    pub id: String,
    pub name: String,
    /// `match-os`: the os-release `ID`, compared without case.
    pub os_id: String,
    /// `match-version`: an optional range, e.g. `>=3.0` or `3.0..3.7`.
    pub version_range: Option<String>,
    /// `match-gpu`: family names, any one of which matches the detected GPU.
    pub gpu_families: Vec<String>,
    /// `desktop-session`, e.g. `plasma`.
    pub desktop_session: String,
    /// `game-session`, e.g. `gamescope`.
    pub game_session: String,
    /// `steam-install`: `preinstalled`, `flatpak <id>` or the like.
    pub steam_install: String,
    /// `wivrn-install`: `flatpak <id>` or the like.
    pub wivrn_install: String,
    /// Every `tool` line, in file order.
    pub tools: Vec<String>,
    /// Every `known-problem` block, in file order.
    pub problems: Vec<KnownProblem>,
}

/// The facts detection reads, and the only input to [`matches`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemFacts {
    /// os-release `ID`, lower case; empty when it could not be read.
    pub os_id: String,
    /// os-release `VERSION_ID`, when present.
    pub os_version: Option<String>,
    /// The GPU's name as the driver or PCI bus reports it.
    pub gpu: Option<String>,
}

/// Why a system file did not parse or load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemError {
    /// A line with no `:`.
    BadLine { line: usize, text: String },
    /// A key given twice; `line` is the second one.
    DuplicateKey { key: String, line: usize },
    /// A key the format does not know.
    UnknownKey { key: String, line: usize },
    /// A required key that never appeared.
    MissingKey(String),
    /// A known-problem block missing one of its fields.
    MissingProblemField { id: String, key: String },
    /// Two known-problem blocks with one id.
    DuplicateProblem(String),
    /// A value the key does not allow.
    BadValue { key: String, value: String },
    /// A file whose name is not `<id>.system`.
    BadFileName(String),
    /// A file could not be read.
    Io { path: String, message: String },
    /// A file's own error; `file` is the file name only.
    InFile {
        file: String,
        error: Box<SystemError>,
    },
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemError::BadLine { line, text } => {
                write!(f, "line {line}: not `key: value`: {text}")
            }
            SystemError::DuplicateKey { key, line } => {
                write!(f, "line {line}: duplicate key `{key}`")
            }
            SystemError::UnknownKey { key, line } => write!(f, "line {line}: unknown key `{key}`"),
            SystemError::MissingKey(key) => write!(f, "missing key `{key}`"),
            SystemError::MissingProblemField { id, key } => {
                write!(f, "known problem `{id}`: missing `{key}`")
            }
            SystemError::DuplicateProblem(id) => write!(f, "known problem `{id}` given twice"),
            SystemError::BadValue { key, value } => write!(f, "bad value for `{key}`: `{value}`"),
            SystemError::BadFileName(name) => write!(f, "bad file name: {name}"),
            SystemError::Io { path, message } => write!(f, "{path}: {message}"),
            SystemError::InFile { file, error } => write!(f, "{file}: {error}"),
        }
    }
}

impl std::error::Error for SystemError {}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn is_id(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn take(values: &mut BTreeMap<String, String>, key: &str) -> Result<String, SystemError> {
    values
        .remove(key)
        .ok_or_else(|| SystemError::MissingKey(key.to_string()))
}

fn is_os_id(text: &str) -> bool {
    !text.is_empty()
        && text.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_' | b'.')
        })
}

/// Parse one system file's `key: value` text. Values are checked by [`check_system`].
pub fn parse_system(text: &str) -> Result<SystemProfile, SystemError> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    let mut tools: Vec<String> = Vec::new();
    let mut problems: Vec<KnownProblem> = Vec::new();
    let mut current: Option<KnownProblem> = None;

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            return Err(SystemError::BadLine {
                line,
                text: trimmed.to_string(),
            });
        };
        let key = raw_key.trim();
        let value = unquote(raw_value).to_string();
        match key {
            "known-problem" => {
                if let Some(problem) = current.take() {
                    problems.push(problem);
                }
                if !is_id(&value) {
                    return Err(SystemError::BadValue {
                        key: "known-problem".to_string(),
                        value,
                    });
                }
                if problems.iter().any(|problem| problem.id == value) {
                    return Err(SystemError::DuplicateProblem(value));
                }
                current = Some(KnownProblem {
                    id: value,
                    summary: String::new(),
                    workaround: String::new(),
                    status: String::new(),
                });
            }
            "problem-summary" | "problem-workaround" | "problem-status" => {
                let Some(problem) = current.as_mut() else {
                    return Err(SystemError::BadValue {
                        key: key.to_string(),
                        value: "(not inside a known-problem block)".to_string(),
                    });
                };
                let slot = match key {
                    "problem-summary" => &mut problem.summary,
                    "problem-workaround" => &mut problem.workaround,
                    _ => &mut problem.status,
                };
                if !slot.is_empty() {
                    return Err(SystemError::DuplicateKey {
                        key: key.to_string(),
                        line,
                    });
                }
                *slot = value;
            }
            "tool" => tools.push(value),
            _ if KEYS.contains(&key) => {
                if values.insert(key.to_string(), value).is_some() {
                    return Err(SystemError::DuplicateKey {
                        key: key.to_string(),
                        line,
                    });
                }
            }
            _ => {
                return Err(SystemError::UnknownKey {
                    key: key.to_string(),
                    line,
                });
            }
        }
    }
    if let Some(problem) = current.take() {
        problems.push(problem);
    }

    let profile = SystemProfile {
        id: String::new(),
        name: take(&mut values, "name")?,
        os_id: take(&mut values, "match-os")?,
        version_range: values.remove("match-version").filter(|v| !v.is_empty()),
        gpu_families: values
            .remove("match-gpu")
            .map(|text| {
                text.split('|')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        desktop_session: take(&mut values, "desktop-session")?,
        game_session: take(&mut values, "game-session")?,
        steam_install: take(&mut values, "steam-install")?,
        wivrn_install: take(&mut values, "wivrn-install")?,
        tools,
        problems,
    };
    for problem in &profile.problems {
        for (key, value) in [
            ("problem-summary", &problem.summary),
            ("problem-workaround", &problem.workaround),
            ("problem-status", &problem.status),
        ] {
            if value.trim().is_empty() {
                return Err(SystemError::MissingProblemField {
                    id: problem.id.clone(),
                    key: key.to_string(),
                });
            }
        }
    }
    Ok(profile)
}

/// Check a parsed profile's values; reports the first bad key.
pub fn check_system(profile: &SystemProfile) -> Result<(), SystemError> {
    let bad = |key: &str, value: &str| SystemError::BadValue {
        key: key.to_string(),
        value: value.to_string(),
    };
    for (key, value) in [
        ("name", &profile.name),
        ("desktop-session", &profile.desktop_session),
        ("game-session", &profile.game_session),
        ("steam-install", &profile.steam_install),
        ("wivrn-install", &profile.wivrn_install),
    ] {
        if value.trim().is_empty() {
            return Err(bad(key, value));
        }
    }
    if !is_os_id(&profile.os_id) {
        return Err(bad("match-os", &profile.os_id));
    }
    if let Some(range) = &profile.version_range {
        if !valid_version_range(range) {
            return Err(bad("match-version", range));
        }
    }
    for family in &profile.gpu_families {
        if family.trim().is_empty() {
            return Err(bad("match-gpu", family));
        }
    }
    for tool in &profile.tools {
        if tool.trim().is_empty() {
            return Err(bad("tool", tool));
        }
    }
    let mut seen: Vec<&str> = Vec::new();
    for problem in &profile.problems {
        if seen.contains(&problem.id.as_str()) {
            return Err(SystemError::DuplicateProblem(problem.id.clone()));
        }
        seen.push(&problem.id);
        if !PROBLEM_STATUSES.contains(&problem.status.as_str()) {
            return Err(bad("problem-status", &problem.status));
        }
    }
    Ok(())
}

/// Every `<id>.system` file under `folder`, parsed and checked, keyed by id.
/// Other files are ignored. A file's own error comes back as [`SystemError::InFile`] naming it.
pub fn load_systems(folder: &Path) -> Result<BTreeMap<String, SystemProfile>, SystemError> {
    let entries = std::fs::read_dir(folder).map_err(|e| SystemError::Io {
        path: folder.display().to_string(),
        message: e.to_string(),
    })?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| SystemError::Io {
            path: folder.display().to_string(),
            message: e.to_string(),
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".system") {
            names.push(name);
        }
    }
    names.sort();
    let mut profiles = BTreeMap::new();
    for name in names {
        let stem = name
            .strip_suffix(".system")
            .ok_or_else(|| SystemError::BadFileName(name.clone()))?;
        if !is_id(stem) {
            return Err(SystemError::BadFileName(name));
        }
        let path = folder.join(&name);
        let text = std::fs::read_to_string(&path).map_err(|e| SystemError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let in_file = |error: SystemError| SystemError::InFile {
            file: name.clone(),
            error: Box::new(error),
        };
        let mut profile = parse_system(&text).map_err(in_file)?;
        check_system(&profile).map_err(in_file)?;
        profile.id = stem.to_string();
        profiles.insert(stem.to_string(), profile);
    }
    Ok(profiles)
}

/// Load each folder in order and return the first profile that matches. A missing folder is
/// skipped; a malformed file is an error naming the file. `None` means unknown system.
pub fn find_system(
    dirs: &[PathBuf],
    facts: &SystemFacts,
) -> Result<Option<SystemProfile>, SystemError> {
    for dir in dirs {
        let profiles = match load_systems(dir) {
            Ok(profiles) => profiles,
            Err(SystemError::Io { .. }) if !dir.exists() => continue,
            Err(error) => return Err(error),
        };
        for profile in profiles.into_values() {
            if matches(&profile, facts) {
                return Ok(Some(profile));
            }
        }
    }
    Ok(None)
}

/// Whether a profile's match rules accept the detected facts. Pure, so tests supply fixtures.
pub fn matches(profile: &SystemProfile, facts: &SystemFacts) -> bool {
    if facts.os_id.trim().is_empty() {
        return false;
    }
    if !profile.os_id.eq_ignore_ascii_case(facts.os_id.trim()) {
        return false;
    }
    if let Some(range) = &profile.version_range {
        let Some(version) = &facts.os_version else {
            return false;
        };
        if !version_in_range(version, range) {
            return false;
        }
    }
    if !profile.gpu_families.is_empty() {
        let Some(gpu) = &facts.gpu else {
            return false;
        };
        let gpu = normalise_gpu(gpu);
        if !profile
            .gpu_families
            .iter()
            .any(|family| gpu.contains(&normalise_gpu(family)))
        {
            return false;
        }
    }
    true
}

fn normalise_gpu(text: &str) -> String {
    text.to_ascii_lowercase()
        .replace(['_', '-', '/'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether `version` falls inside `range`. A bare value or `=x` means equal; `a..b` is
/// inclusive at both ends, and either end may be left out.
pub fn version_in_range(version: &str, range: &str) -> bool {
    let Some(current) = parse_version(version) else {
        return false;
    };
    let range = range.trim();
    if let Some((lower, upper)) = range.split_once("..") {
        let above = lower.trim().is_empty()
            || parse_version(lower.trim())
                .is_some_and(|value| cmp_versions(&current, &value) != Ordering::Less);
        let below = upper.trim().is_empty()
            || parse_version(upper.trim())
                .is_some_and(|value| cmp_truncated(&current, &value) != Ordering::Greater);
        return above && below;
    }
    let (operator, rest) = [
        (">=", true),
        ("<=", true),
        (">", false),
        ("<", false),
        ("=", false),
    ]
    .into_iter()
    .find_map(|(operator, longest)| {
        range
            .strip_prefix(operator)
            .filter(|rest| !longest || !rest.is_empty())
            .map(|rest| (operator, rest))
    })
    .unwrap_or(("=", range));
    let Some(target) = parse_version(rest.trim()) else {
        return false;
    };
    let order = cmp_versions(&current, &target);
    let bounded = cmp_truncated(&current, &target);
    match operator {
        ">=" => order != Ordering::Less,
        ">" => order == Ordering::Greater,
        "<=" => bounded != Ordering::Greater,
        "<" => order == Ordering::Less,
        _ => bounded == Ordering::Equal,
    }
}

/// Whether `range` is a shape [`version_in_range`] can use.
pub fn valid_version_range(range: &str) -> bool {
    let range = range.trim();
    if let Some((lower, upper)) = range.split_once("..") {
        return (!lower.trim().is_empty() || !upper.trim().is_empty())
            && (lower.trim().is_empty() || parse_version(lower.trim()).is_some())
            && (upper.trim().is_empty() || parse_version(upper.trim()).is_some());
    }
    for operator in [">=", "<=", ">", "<", "="] {
        if let Some(rest) = range.strip_prefix(operator) {
            return parse_version(rest.trim()).is_some();
        }
    }
    parse_version(range).is_some()
}

fn parse_version(text: &str) -> Option<Vec<u32>> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for part in text.split('.') {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            return None;
        }
        parts.push(digits.parse().ok()?);
    }
    Some(parts)
}

/// Compare `version` with `bound` using only the components `bound` names, so `3.7.13` is
/// equal to `3.7` for `<=`, `=` and the upper end of a range.
fn cmp_truncated(version: &[u32], bound: &[u32]) -> Ordering {
    let version = &version[..version.len().min(bound.len())];
    cmp_versions(version, bound)
}

fn cmp_versions(left: &[u32], right: &[u32]) -> Ordering {
    let len = left.len().max(right.len());
    for index in 0..len {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        match a.cmp(&b) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

/// Read the facts doctor matches on: os-release through the file helper, the GPU name through the
/// command seam. Tests point the first at a fixture and answer the second with a fake runner.
pub fn detect(os_release: &Path, runner: &mut dyn CommandRunner) -> SystemFacts {
    let text = fsx::read_opt(os_release).ok().flatten();
    let (os_id, os_version) = text.as_deref().map(parse_os_release).unwrap_or_default();
    SystemFacts {
        os_id,
        os_version,
        gpu: detect_gpu(runner),
    }
}

/// `ID` and `VERSION_ID` from an os-release text. The id is lower case.
pub fn parse_os_release(text: &str) -> (String, Option<String>) {
    let mut id = String::new();
    let mut version = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = unquote(value).to_string();
        match key.trim() {
            "ID" => id = value.to_ascii_lowercase(),
            "VERSION_ID" => version = Some(value),
            _ => {}
        }
    }
    (id, version.filter(|value| !value.is_empty()))
}

fn detect_gpu(runner: &mut dyn CommandRunner) -> Option<String> {
    let commands: [&[&str]; 3] = [
        &["lspci", "-nn"],
        &["glxinfo", "-B"],
        &["vulkaninfo", "--summary"],
    ];
    for command in commands {
        let argv: Vec<String> = command.iter().map(|part| (*part).to_string()).collect();
        let output = runner.run(&argv);
        if output.success {
            if let Some(name) = parse_gpu_name(&output.stdout) {
                return Some(name);
            }
        }
    }
    None
}

/// The GPU name from one of the tools doctor probes, or `None`.
pub fn parse_gpu_name(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Device:") {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
        if let Some(index) = line.find("deviceName") {
            if let Some((_, value)) = line[index..].split_once('=') {
                let value = value.trim().trim_matches('"').trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        let lower = line.to_ascii_lowercase();
        if lower.contains("vga compatible controller") || lower.contains("3d controller") {
            let rest = line.rsplit_once("]: ").map_or(line, |(_, rest)| rest);
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(match_version: Option<&str>) -> SystemProfile {
        SystemProfile {
            id: "test".into(),
            name: "Test".into(),
            os_id: "steamos".into(),
            version_range: match_version.map(str::to_string),
            gpu_families: vec!["Strix Halo".into()],
            desktop_session: "plasma".into(),
            game_session: "gamescope".into(),
            steam_install: "preinstalled".into(),
            wivrn_install: "flatpak io.github.wivrn.wivrn".into(),
            tools: vec!["gamescope".into()],
            problems: Vec::new(),
        }
    }

    #[test]
    fn a_version_range_matches_inclusively() {
        assert!(version_in_range("3.6", ">=3.0"));
        assert!(version_in_range("3.6.19", "3.0..3.7"));
        assert!(version_in_range("3.7", "3.0..3.7"));
        assert!(!version_in_range("3.8", "3.0..3.7"));
        assert!(version_in_range("3.6", "3.0.."));
        assert!(!valid_version_range("nonsense"));
    }

    #[test]
    fn the_gpu_family_matches_a_detected_name() {
        let facts = SystemFacts {
            os_id: "steamos".into(),
            os_version: Some("3.6".into()),
            gpu: Some("AMD Radeon 8060S Graphics".into()),
        };
        let mut profile = profile(None);
        profile.gpu_families = vec!["Strix Halo".into(), "Radeon 8060S".into()];
        assert!(matches(&profile, &facts));
    }

    #[test]
    fn os_release_and_lspci_fixtures_are_read() {
        assert_eq!(
            parse_os_release("ID=steamos\nVERSION_ID=3.6\n"),
            ("steamos".into(), Some("3.6".into()))
        );
        assert_eq!(
            parse_gpu_name(
                "03:00.0 VGA compatible controller [0300]: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon 8050S/8060S] (rev c1)\n"
            )
            .as_deref(),
            Some("Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon 8050S/8060S] (rev c1)")
        );
    }
}
