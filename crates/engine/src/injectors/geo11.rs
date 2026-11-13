//! Pure planning and transactional application of the Geo-11 route.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use flat3d_viewer::spec::{Layout, ViewerSpec};

use crate::api::ApiError;
use crate::injectors::fetch;
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
                    s.convergence = value.parse::<f32>().ok().filter(|n| n.is_finite() && *n > 0.0)
                        .ok_or_else(|| bad_recipe(format!("invalid geo11-convergence: {value}; expected a number greater than 0")))?;
                }
                "geo11-d3dx" => {
                    for part in value.split(';') {
                        let Some((k, v)) = part.split_once('=') else {
                            return Err(bad_recipe(format!("invalid geo11-d3dx pair: {part}")));
                        };
                        // Same rule as recipes::check_route_settings, plus no line breaks,
                        // which would inject extra lines into d3dx.ini.
                        let (k, v) = (k.trim(), v.trim());
                        if k.is_empty()
                            || v.is_empty()
                            || v.contains('=')
                            || part.contains(['\r', '\n'])
                        {
                            return Err(bad_recipe(format!("invalid geo11-d3dx pair: {part}")));
                        }
                        s.d3dx.push((k.to_owned(), v.to_owned()));
                    }
                }
                "geo11-remove-first" => {
                    for item in value.split(';') {
                        if !is_inside_relative(item) {
                            return Err(bad_recipe(format!(
                                "invalid geo11-remove-first path: {item}"
                            )));
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

/// A relative path made only of normal components (no `..`, `.`, root or drive prefix),
/// the rule `recipes::check_route_settings` uses for `geo11-remove-first`.
fn is_inside_relative(item: &str) -> bool {
    let p = Path::new(item);
    !item.is_empty()
        && !p.is_absolute()
        && !item.starts_with(['/', '\\'])
        && item.as_bytes().get(1) != Some(&b':')
        && p.components().all(|c| matches!(c, Component::Normal(_)))
}

fn is_pin_key(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// True for a real directory; a symlink to a directory is not walked (no loops).
fn is_real_dir(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.is_dir())
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

fn is_plain_folder_name(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with("TBD")
        && s != "."
        && s != ".."
        && !s.contains(['/', '\\', ':', '\0'])
}

fn known_pin_version(key: &str) -> Option<String> {
    let pins = fetch::parse_pins(fetch::PINS);
    let pin = pins.get(key)?;
    if is_plain_folder_name(&pin.version) {
        Some(pin.version.clone())
    } else {
        None
    }
}

fn should_skip_root_child(path: &Path) -> bool {
    let Some(name) = path.file_name() else {
        return false;
    };
    let lower = name.to_string_lossy().to_ascii_lowercase();
    if lower.starts_with('.') || lower.ends_with(".tmp") {
        return true;
    }
    if is_real_dir(path) && path.join(fetch::MARKER).is_file() {
        return true;
    }
    false
}

fn cache_search_scope(cache_dir: &Path, key: &str) -> (PathBuf, bool) {
    let key_dir = cache_dir.join(key);
    if let Some(v) = known_pin_version(key) {
        let version_dir = key_dir.join(v);
        if is_real_dir(&version_dir) {
            return (version_dir, false);
        }
    }
    (key_dir, true)
}

/// Return the shallowest directory matching `name`, with sorted path order breaking ties.
fn find_dir(
    root: &Path,
    max_depth: usize,
    name: &str,
    filter_root_children: bool,
) -> io::Result<Option<PathBuf>> {
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
                if filter_root_children && dir == root && should_skip_root_child(&path) {
                    continue;
                }
                if is_real_dir(&path) {
                    next.push(path);
                }
            }
        }
        level = next;
    }
    Ok(None)
}

fn find_file_parent(
    root: &Path,
    max_depth: usize,
    name: &str,
    filter_root_children: bool,
) -> io::Result<Option<PathBuf>> {
    if !root.is_dir() {
        return Ok(None);
    }
    let mut level = vec![root.to_path_buf()];
    for depth in 0..=max_depth {
        level.sort();
        for dir in &level {
            if sorted_entries(dir)?.iter().any(|p| {
                if filter_root_children && dir == root && should_skip_root_child(p) {
                    return false;
                }
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
                if filter_root_children && dir == root && should_skip_root_child(&path) {
                    continue;
                }
                if is_real_dir(&path) {
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
            if is_real_dir(&path) {
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

/// `base/rel`, with each component that already exists under another ASCII case (for
/// example `shaderfixes/` for `ShaderFixes/`) spelled the way it is on disk. An exact
/// match wins over a case-insensitive one; sorted order breaks ties.
fn existing_case_path(base: &Path, rel: &Path) -> PathBuf {
    let mut out = base.to_path_buf();
    let mut on_disk = true;
    for comp in rel.components() {
        let name = comp.as_os_str();
        if on_disk && !out.join(name).exists() {
            let found = sorted_entries(&out).ok().and_then(|entries| {
                entries.into_iter().find(|p| {
                    p.file_name().is_some_and(|n| {
                        n.to_string_lossy()
                            .eq_ignore_ascii_case(&name.to_string_lossy())
                    })
                })
            });
            match found {
                Some(found) => {
                    out = found;
                    continue;
                }
                None => on_disk = false,
            }
        }
        out.push(name);
    }
    out
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
    if !is_inside_relative(exe) {
        return Err(bad_recipe(format!(
            "geo11-exe must be a relative path inside the game folder: {exe}"
        )));
    }
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
        let found = existing_case_path(game_root, Path::new(rel));
        if fs::symlink_metadata(&found).is_ok() {
            steps.push(FileStep::Remove(found));
        }
    }

    let fix_name = m.get("geo11-fix");
    let fix_root = if let Some(name) = fix_name {
        if !is_pin_key(name) {
            return Err(bad_recipe(format!(
                "geo11-fix must be a pin key ([a-z0-9-]+): {name}"
            )));
        }
        let (search_root, filter) = cache_search_scope(cache_dir, name);
        if !search_root.is_dir() {
            return Err(not_found(format!(
                "{name} is not fetched; run flat3d-inject setup fetch {name}"
            )));
        }
        let found = find_file_parent(&search_root, 3, "d3dx.ini", filter)
            .map_err(|e| io_error("search", &search_root, &e))?
            .or(find_file_parent(&search_root, 3, "d3d11.dll", filter)
                .map_err(|e| io_error("search", &search_root, &e))?);
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
        let (search_root, filter) = cache_search_scope(cache_dir, "geo11");
        let found = find_dir(&search_root, 3, "x64", filter)
            .map_err(|e| io_error("search", &search_root, &e))?;
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
            copies.push((from.clone(), rel.to_path_buf(), false));
        }
    }
    if let Some(root) = &overlay {
        for name in ["d3d11.dll", "nvapi64.dll", "d3dxdm.ini"] {
            let from = root.join(name);
            if from.is_file() {
                copies.push((from, PathBuf::from(name), true));
            }
        }
        if fix_root
            .as_ref()
            .is_none_or(|r| !r.join("d3dx.ini").is_file())
        {
            let from = root.join("d3dx.ini");
            if from.is_file() {
                copies.push((from, PathBuf::from("d3dx.ini"), true));
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
        let at_root = to.parent() == Some(game_root);
        if at_root && lower == "d3dx.ini" {
            let text = fs::read_to_string(&from)
                .map_err(|e| format!("{}: cannot read {}: {e}", "io", from.display()))?;
            steps.push(FileStep::Write {
                to,
                text: d3dx_patch(&text, &settings)?,
            });
        } else if at_root && lower == "d3dxdm.ini" {
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
    // Copies sorted by destination, then Writes sorted by destination.
    rest.sort_by_key(|s| match s {
        FileStep::Copy { to, .. } => (false, to.clone()),
        FileStep::Write { to, .. } => (true, to.clone()),
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
    for p in record.created.iter().rev() {
        let _ = fs::remove_file(p);
    }
    for r in record.replaced.iter().rev() {
        let _ = fs::rename(&r.backup, &r.path);
    }
    crate::undo::remove_created_dirs(record);
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
    // A backup left in a subfolder would be overwritten by this apply's backup of the same
    // file, so it counts as "already set up" just like one in E.
    let stale_backup = steps.iter().any(|step| {
        let (FileStep::Remove(to) | FileStep::Copy { to, .. } | FileStep::Write { to, .. }) = step;
        Path::new(&format!("{}.flat3d-backup", to.to_string_lossy())).exists()
    });
    if stale_backup
        || marker.exists()
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
        created_dirs: Vec::new(),
    };
    let result = (|| {
        fs::write(&marker, "").map_err(|err| io_error("write", &marker, &err))?;
        record.created.push(marker.clone());
        for step in &steps {
            match step {
                FileStep::Remove(path) => {
                    if fs::symlink_metadata(path).is_ok() {
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
                        crate::undo::create_dir_all_recorded(parent, &mut record)
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
                        crate::undo::create_dir_all_recorded(parent, &mut record)
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
        FileStep::Copy { to, .. } | FileStep::Write { to, .. } => {
            to.parent() == Some(e.as_path())
                && to
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("dxgi.dll"))
        }
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
