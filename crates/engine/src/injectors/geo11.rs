//! Pure planning and transactional application of the Geo-11 route.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use flat3d_viewer::spec::{Layout, ViewerSpec};

use crate::api::ApiError;
use crate::injectors::uevr::MARKER_FILE;
use crate::injectors::RouteEnv;
use crate::recipes::{RouteSettings, GEO11_KEYS};
use crate::undo::{Replaced, UndoRecord};

#[derive(Debug, Clone, PartialEq)]
pub struct Geo11Settings {
    pub layout: Layout,
    pub separation: u32,
    pub convergence: f32,
    pub d3dx: Vec<(String, String)>,
}

impl Default for Geo11Settings {
    fn default() -> Self {
        Self {
            layout: Layout::Sbs,
            separation: 40,
            convergence: 2.0,
            d3dx: vec![("force_stereo".into(), "2".into())],
        }
    }
}

impl Geo11Settings {
    pub fn from_route(m: &RouteSettings) -> Result<Self, String> {
        let mut s = Self::default();
        for (key, value) in m {
            if !GEO11_KEYS.contains(&key.as_str()) {
                continue;
            }
            match key.as_str() {
                "geo11-layout" => {
                    s.layout = match value.as_str() {
                        "sbs" => Layout::Sbs,
                        "tab" => Layout::Tab,
                        "sbs_reversed" => Layout::SbsRev,
                        "tab_reversed" => Layout::TabRev,
                        _ => return Err(bad_recipe(format!("invalid geo11-layout: {value}; expected sbs, tab, sbs_reversed or tab_reversed"))),
                    };
                }
                "geo11-separation" => {
                    s.separation =
                        value
                            .parse::<u32>()
                            .ok()
                            .filter(|n| *n <= 100)
                            .ok_or_else(|| {
                                bad_recipe(format!(
                                    "invalid geo11-separation: {value}; expected 0..=100"
                                ))
                            })?;
                }
                "geo11-convergence" => {
                    s.convergence = value.parse::<f32>().ok().filter(|n| *n > 0.0)
                        .ok_or_else(|| bad_recipe(format!("invalid geo11-convergence: {value}; expected a number greater than 0")))?;
                }
                "geo11-d3dx" => {
                    for part in value.split(';') {
                        let Some((k, v)) = part.split_once('=') else {
                            return Err(bad_recipe(format!("invalid geo11-d3dx pair: {part}")));
                        };
                        if k.is_empty() {
                            return Err(bad_recipe("invalid geo11-d3dx pair: empty key"));
                        }
                        s.d3dx.push((k.to_owned(), v.to_owned()));
                    }
                }
                "geo11-remove-first" => {
                    if !value.is_empty() {
                        for item in value.split(';') {
                            let p = Path::new(item);
                            if item.is_empty()
                                || p.is_absolute()
                                || item.starts_with(['/', '\\'])
                                || item.as_bytes().get(1) == Some(&b':')
                                || p.components().any(|c| {
                                    matches!(
                                        c,
                                        Component::ParentDir
                                            | Component::RootDir
                                            | Component::Prefix(_)
                                    )
                                })
                            {
                                return Err(bad_recipe(format!(
                                    "invalid geo11-remove-first path: {item}"
                                )));
                            }
                        }
                    }
                }
                // geo11-exe and geo11-fix are consumed by plan_files.
                "geo11-exe" | "geo11-fix" => {}
                _ => {}
            }
        }
        Ok(s)
    }
}

pub fn direct_mode(l: Layout) -> &'static str {
    match l {
        Layout::Sbs => "sbs",
        Layout::Tab => "tab",
        Layout::SbsRev => "sbs_reversed",
        Layout::TabRev => "tab_reversed",
    }
}

pub fn d3dxdm_text(s: &Geo11Settings) -> String {
    let convergence = if s.convergence.fract() == 0.0 {
        format!("{:.1}", s.convergence)
    } else {
        s.convergence.to_string()
    };
    format!(
        "direct_mode={}\ndm_separation={}\ndm_convergence={}\n",
        direct_mode(s.layout),
        s.separation,
        convergence
    )
}

/// Set one INI key in-place while preserving the input's newline convention and separator.
fn set_ini(
    text: &str,
    file: &str,
    key: &str,
    value: &str,
    section: Option<&str>,
    append: bool,
) -> Result<String, String> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let had_final_newline = text.ends_with('\n');
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
        .collect();
    if had_final_newline && lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    let key_eq = |line: &str| {
        let trimmed = line.trim_start();
        if trimmed.starts_with(';') || trimmed.starts_with('#') {
            return false;
        }
        let Some((k, _)) = trimmed.split_once('=') else {
            return false;
        };
        k.trim().eq_ignore_ascii_case(key)
    };
    if let Some(i) = lines.iter().position(|line| key_eq(line)) {
        let line = &lines[i];
        let start = line.len() - line.trim_start().len();
        let trimmed = &line[start..];
        let equal = trimmed.find('=').expect("matched key has equals");
        let left = &trimmed[..equal];
        let sep_start = left.trim_end().len();
        let sep_end = trimmed[equal + 1..].len() - trimmed[equal + 1..].trim_start().len();
        let sep = &trimmed[sep_start..equal + 1 + sep_end];
        lines[i] = format!("{}{}{}{}", &line[..start], left.trim_end(), sep, value);
    } else {
        let commented = lines.iter().position(|line| {
            let t = line.trim_start();
            t.strip_prefix(';')
                .and_then(|s| s.split_once('='))
                .is_some_and(|(k, _)| k.trim().eq_ignore_ascii_case(key))
        });
        if let Some(i) = commented {
            lines.insert(i + 1, format!("{key}={value}"));
        } else if let Some(section) = section {
            if let Some(i) = lines
                .iter()
                .position(|line| line.trim().eq_ignore_ascii_case(&format!("[{section}]")))
            {
                lines.insert(i + 1, format!("{key}={value}"));
            } else {
                if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
                    lines.push(String::new());
                }
                lines.push(format!("[{section}]"));
                lines.push(format!("{key}={value}"));
            }
        } else if append {
            lines.push(format!("{key}={value}"));
        } else {
            return Err(bad_recipe(format!("{file} has no {key} line")));
        }
    }
    let mut out = lines.join(newline);
    if had_final_newline {
        out.push_str(newline);
    }
    Ok(out)
}

pub fn d3dx_patch(ini: &str, s: &Geo11Settings) -> Result<String, String> {
    let mut out = ini.to_owned();
    for (key, value) in &s.d3dx {
        let section = match key.to_ascii_lowercase().as_str() {
            "force_stereo" => Some("Stereo"),
            "get_resolution_from" => Some("Device"),
            _ => None,
        };
        out = set_ini(&out, "d3dx.ini", key, value, section, false)?;
    }
    Ok(out)
}

fn patch_d3dxdm(ini: &str, s: &Geo11Settings) -> Result<String, String> {
    let mut out = ini.to_owned();
    for (key, value) in [
        ("direct_mode", direct_mode(s.layout).to_owned()),
        ("dm_separation", s.separation.to_string()),
        (
            "dm_convergence",
            if s.convergence.fract() == 0.0 {
                format!("{:.1}", s.convergence)
            } else {
                s.convergence.to_string()
            },
        ),
    ] {
        out = set_ini(&out, "d3dxdm.ini", key, &value, None, true)?;
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStep {
    Remove(PathBuf),
    Copy { from: PathBuf, to: PathBuf },
    Write { to: PathBuf, text: String },
}

fn bad_recipe(message: impl AsRef<str>) -> String {
    format!("bad-recipe: {}", message.as_ref())
}
fn not_found(message: impl AsRef<str>) -> String {
    format!("not-found: {}", message.as_ref())
}

fn sorted_entries(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

/// Return the shallowest directory matching `name`, with sorted path order breaking ties.
fn find_dir(root: &Path, max_depth: usize, name: &str) -> io::Result<Option<PathBuf>> {
    if !root.is_dir() {
        return Ok(None);
    }
    let mut level = vec![root.to_path_buf()];
    for depth in 0..=max_depth {
        level.sort();
        if let Some(found) = level.iter().find(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
        }) {
            return Ok(Some(found.clone()));
        }
        if depth == max_depth {
            break;
        }
        let mut next = Vec::new();
        for dir in level {
            for path in sorted_entries(&dir)? {
                if path.is_dir() {
                    next.push(path);
                }
            }
        }
        level = next;
    }
    Ok(None)
}

fn find_file_parent(root: &Path, max_depth: usize, name: &str) -> io::Result<Option<PathBuf>> {
    if !root.is_dir() {
        return Ok(None);
    }
    let mut level = vec![root.to_path_buf()];
    for depth in 0..=max_depth {
        level.sort();
        for dir in &level {
            if sorted_entries(dir)?.iter().any(|p| {
                p.is_file()
                    && p.file_name()
                        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
            }) {
                return Ok(Some(dir.clone()));
            }
        }
        if depth == max_depth {
            break;
        }
        let mut next = Vec::new();
        for dir in level {
            for path in sorted_entries(&dir)? {
                if path.is_dir() {
                    next.push(path);
                }
            }
        }
        level = next;
    }
    Ok(None)
}

fn regular_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let entries = sorted_entries(&dir)?;
        for path in entries.into_iter().rev() {
            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn root_ignored(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("uninstall") && n.ends_with(".bat")
        || n.starts_with("readme")
        || n.ends_with(".url")
        || n == ".flat3d-fetch"
}

fn existing_case_path(dir: &Path, wanted: &Path) -> PathBuf {
    let Some(name) = wanted.file_name().and_then(|n| n.to_str()) else {
        return wanted.to_path_buf();
    };
    if let Ok(entries) = sorted_entries(dir) {
        if let Some(found) = entries.into_iter().find(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
        }) {
            return found;
        }
    }
    wanted.to_path_buf()
}

pub fn plan_files(
    cache_dir: &Path,
    game_dir: &Path,
    m: &RouteSettings,
) -> Result<Vec<FileStep>, String> {
    let settings = Geo11Settings::from_route(m)?;
    let exe = m
        .get("geo11-exe")
        .ok_or_else(|| bad_recipe("geo11-exe is missing"))?;
    let exe_path = game_dir.join(exe);
    if !exe_path.is_file() {
        return Err(not_found(format!(
            "Geo-11 executable not found at {}",
            exe_path.display()
        )));
    }
    let game_root = exe_path.parent().unwrap_or(game_dir);
    let remove_paths = m
        .get("geo11-remove-first")
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let mut steps = Vec::new();
    for rel in remove_paths.split(';').filter(|s| !s.is_empty()) {
        let wanted = game_root.join(rel);
        if wanted.exists() {
            steps.push(FileStep::Remove(existing_case_path(
                game_root,
                Path::new(rel),
            )));
        }
    }

    let fix_name = m.get("geo11-fix");
    let fix_root = if let Some(name) = fix_name {
        let pin_root = cache_dir.join(name);
        if !pin_root.is_dir() {
            return Err(not_found(format!(
                "{name} is not fetched; run flat3d-inject setup fetch {name}"
            )));
        }
        let found = find_file_parent(&pin_root, 3, "d3dx.ini")
            .map_err(|e| io_error("search", &pin_root, &e))?
            .or(find_file_parent(&pin_root, 3, "d3d11.dll")
                .map_err(|e| io_error("search", &pin_root, &e))?);
        if found.is_none() {
            return Err(not_found(format!(
                "{name} is not fetched; run flat3d-inject setup fetch {name}"
            )));
        }
        found
    } else {
        None
    };

    let fix_has_dm = fix_root
        .as_ref()
        .is_some_and(|r| r.join("d3dxdm.ini").is_file());
    let overlay = if fix_has_dm {
        None
    } else {
        let root = cache_dir.join("geo11");
        let found = find_dir(&root, 3, "x64").map_err(|e| io_error("search", &root, &e))?;
        if found.is_none() {
            return Err(not_found(
                "geo11 is not fetched; run flat3d-inject setup fetch geo11",
            ));
        }
        found
    };

    let mut copies = Vec::<(PathBuf, PathBuf, bool)>::new();
    if let Some(root) = &fix_root {
        for from in regular_files(root).map_err(|e| io_error("search", root, &e))? {
            let rel = from.strip_prefix(root).expect("walked below root");
            if rel.components().count() == 1
                && from
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(root_ignored)
            {
                continue;
            }
            let target = game_root.join(rel);
            copies.push((from, target, false));
        }
    }
    if let Some(root) = &overlay {
        for name in ["d3d11.dll", "nvapi64.dll", "d3dxdm.ini"] {
            let from = root.join(name);
            if from.is_file() {
                copies.push((from, game_root.join(name), true));
            }
        }
        if fix_root
            .as_ref()
            .is_none_or(|r| !r.join("d3dx.ini").is_file())
        {
            let from = root.join("d3dx.ini");
            if from.is_file() {
                copies.push((from, game_root.join("d3dx.ini"), true));
            }
        }
    }

    // Resolve case-insensitive collisions; overlay entries replace fix entries.
    copies.sort_by(|a, b| a.1.cmp(&b.1));
    let mut resolved = Vec::<(PathBuf, PathBuf, bool)>::new();
    for item in copies {
        if let Some(pos) = resolved.iter().position(|old| {
            old.1
                .to_string_lossy()
                .eq_ignore_ascii_case(&item.1.to_string_lossy())
        }) {
            if item.2 {
                resolved[pos] = item;
            }
        } else {
            resolved.push(item);
        }
    }
    for (from, to, _) in resolved {
        let to = existing_case_path(game_root, &to);
        let lower = to
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        if lower == "d3dx.ini" {
            let text = fs::read_to_string(&from)
                .map_err(|e| format!("{}: cannot read {}: {e}", "io", from.display()))?;
            steps.push(FileStep::Write {
                to,
                text: d3dx_patch(&text, &settings)?,
            });
        } else if lower == "d3dxdm.ini" {
            let text = fs::read_to_string(&from)
                .map_err(|e| format!("io: cannot read {}: {e}", from.display()))?;
            steps.push(FileStep::Write {
                to,
                text: patch_d3dxdm(&text, &settings)?,
            });
        } else {
            steps.push(FileStep::Copy { from, to });
        }
    }
    let split = steps
        .iter()
        .position(|s| !matches!(s, FileStep::Remove(_)))
        .unwrap_or(steps.len());
    let (removes, rest) = steps.split_at_mut(split);
    removes.sort_by_key(|s| match s {
        FileStep::Remove(p) => p.clone(),
        _ => unreachable!(),
    });
    rest.sort_by_key(|s| match s {
        FileStep::Copy { to, .. } | FileStep::Write { to, .. } => to.clone(),
        _ => unreachable!(),
    });
    Ok(steps)
}

pub fn viewer_check(viewer_exe: &Path) -> Result<(), ApiError> {
    let executable = viewer_exe.is_file() && {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(viewer_exe).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
        }
        #[cfg(not(unix))]
        {
            true
        }
    };
    if executable {
        Ok(())
    } else {
        Err(ApiError::new("not-found", &format!("flat3d-viewer not found at {}; reinstall flat-to-3d (install, AppImage or Decky zip)", viewer_exe.display())))
    }
}

fn rollback(record: &UndoRecord) {
    for r in record.replaced.iter().rev() {
        let _ = fs::rename(&r.backup, &r.path);
    }
    for p in record.created.iter().rev() {
        let _ = fs::remove_file(p);
    }
}

fn io_error(action: &str, path: &Path, e: &io::Error) -> String {
    format!("io: cannot {action} {}: {e}", path.display())
}

pub fn apply(
    cache_dir: &Path,
    game_dir: &Path,
    appid: u32,
    m: &RouteSettings,
    viewer_exe: &Path,
) -> Result<(Applied, RouteEnv), String> {
    viewer_check(viewer_exe).map_err(|e| format!("{}: {}", e.code, e.message))?;
    let steps = plan_files(cache_dir, game_dir, m)?;
    let exe = m.get("geo11-exe").expect("plan_files checked exe");
    let e = game_dir
        .join(exe)
        .parent()
        .unwrap_or(game_dir)
        .to_path_buf();
    let marker = e.join(MARKER_FILE);
    if marker.exists()
        || fs::read_dir(&e)
            .map_err(|err| io_error("read", &e, &err))?
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .ends_with(".flat3d-backup")
            })
    {
        return Err(format!(
            "refused: Geo-11 is already set up in {}; undo it first: flat3d-inject undo {appid}",
            e.display()
        ));
    }
    let mut record = UndoRecord {
        appid,
        launch_options_before: String::new(),
        created: Vec::new(),
        replaced: Vec::new(),
    };
    let result = (|| {
        fs::write(&marker, "").map_err(|err| io_error("write", &marker, &err))?;
        record.created.push(marker.clone());
        for step in &steps {
            match step {
                FileStep::Remove(path) => {
                    if path.exists() {
                        let backup =
                            PathBuf::from(format!("{}.flat3d-backup", path.to_string_lossy()));
                        fs::rename(path, &backup).map_err(|err| io_error("back up", path, &err))?;
                        record.replaced.push(Replaced {
                            path: path.clone(),
                            backup,
                        });
                    }
                }
                FileStep::Copy { from, to } => {
                    if let Some(parent) = to.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|err| io_error("create", parent, &err))?;
                    }
                    if to.exists() {
                        let backup =
                            PathBuf::from(format!("{}.flat3d-backup", to.to_string_lossy()));
                        let partial =
                            PathBuf::from(format!("{}.partial", backup.to_string_lossy()));
                        if let Err(err) =
                            fs::copy(to, &partial).and_then(|_| fs::rename(&partial, &backup))
                        {
                            let _ = fs::remove_file(&partial);
                            return Err(io_error("back up", to, &err));
                        }
                        record.replaced.push(Replaced {
                            path: to.clone(),
                            backup,
                        });
                    } else {
                        record.created.push(to.clone());
                    }
                    fs::copy(from, to)
                        .map(|_| ())
                        .map_err(|err| io_error("write", to, &err))?;
                }
                FileStep::Write { to, text } => {
                    if let Some(parent) = to.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|err| io_error("create", parent, &err))?;
                    }
                    if to.exists() {
                        let backup =
                            PathBuf::from(format!("{}.flat3d-backup", to.to_string_lossy()));
                        let partial =
                            PathBuf::from(format!("{}.partial", backup.to_string_lossy()));
                        if let Err(err) =
                            fs::copy(to, &partial).and_then(|_| fs::rename(&partial, &backup))
                        {
                            let _ = fs::remove_file(&partial);
                            return Err(io_error("back up", to, &err));
                        }
                        record.replaced.push(Replaced {
                            path: to.clone(),
                            backup,
                        });
                    } else {
                        record.created.push(to.clone());
                    }
                    fs::write(to, text).map_err(|err| io_error("write", to, &err))?;
                }
            }
        }
        Ok::<(), String>(())
    })();
    if let Err(err) = result {
        rollback(&record);
        return Err(err);
    }
    let layout = Geo11Settings::from_route(m)?.layout;
    let mut sidecar = vec![viewer_exe.to_string_lossy().into_owned()];
    sidecar.extend(
        ViewerSpec {
            layout,
            ..ViewerSpec::default()
        }
        .to_args(),
    );
    let env_value = if steps.iter().any(|s| match s {
        FileStep::Copy { to, .. } | FileStep::Write { to, .. } => to
            .file_name()
            .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("dxgi.dll")),
        _ => false,
    }) {
        "d3d11,nvapi64,dxgi=n,b"
    } else {
        "d3d11,nvapi64=n,b"
    };
    Ok((
        Applied { record },
        RouteEnv {
            env: vec![("WINEDLLOVERRIDES".into(), env_value.into())],
            unset: Vec::new(),
            sidecar: Some(sidecar),
        },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub record: UndoRecord,
}
