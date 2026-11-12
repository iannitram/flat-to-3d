//! A fetched UEVR profile (the pin a recipe's `uevr-profile` key names), laid out for the
//! game's `UnrealVRMod/<exe stem>/` folder, and our keys merged into its `config.txt`.
//!
//! Third-party notice: the layout rules (one wrapping top folder is stripped, a zip that holds
//! only another zip is not a profile we can read without unpacking it again, names spelled with
//! backslashes are folders) and the copy-with-backup-then-roll-back import are ported from Kaon,
//! https://github.com/LorenDB/kaon, `src/mods/UevrProfiles.cpp` at commit
//! fa4e22a7217d9b3450d42c50b04677d3e424d1e7 (rollback from commit
//! 6061a807f2679fd59c689cc5244c8e00152089f8). Kaon is copyright its authors and licensed under
//! the GNU General Public License version 3; this file is under the same license as the rest of
//! this repository (GPL-3.0-only, see LICENSE).

use crate::injectors::fetch;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The profile file UEVR reads at the top of the profile folder.
pub const CONFIG_FILE: &str = "config.txt";

/// One file of a fetched profile: where it is in the cache, and where it goes under the
/// profile folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFile {
    pub src: PathBuf,
    pub rel: PathBuf,
}

/// The files of a fetched profile and its `config.txt` text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Every file except the top `config.txt`, in path order.
    pub files: Vec<ProfileFile>,
    /// The profile's own `config.txt`.
    pub config: String,
}

/// Read the profile unpacked in `fetched` (a fetch cache folder). Refuses a folder with no
/// top-level `config.txt` (after one wrapping folder is stripped), a zip inside a zip, and any
/// name that would leave the profile folder.
pub fn read(fetched: &Path) -> Result<Profile, String> {
    let mut found = Vec::new();
    walk(fetched, &mut Vec::new(), &mut found)?;
    found.retain(|(_, parts)| parts.as_slice() != [fetch::MARKER]);
    if found.is_empty() {
        return Err(format!("the profile in {} is empty", fetched.display()));
    }
    if let [(_, parts)] = found.as_slice() {
        if parts.len() == 1 && ends_with_ignore_case(&parts[0], ".zip") {
            return Err(format!(
                "the profile in {} is a zip inside a zip ({}); unpack it and pin the inner zip",
                fetched.display(),
                parts[0]
            ));
        }
    }
    let tops: BTreeSet<&str> = found
        .iter()
        .map(|(_, parts)| {
            if parts.len() > 1 {
                parts[0].as_str()
            } else {
                ""
            }
        })
        .collect();
    let strip = tops.len() == 1 && !tops.contains("");
    let mut config = None;
    let mut files = Vec::new();
    for (src, parts) in found {
        let parts = if strip { &parts[1..] } else { &parts[..] };
        if parts.len() == 1 && parts[0].eq_ignore_ascii_case(CONFIG_FILE) {
            let text = fs::read(&src).map_err(|e| format!("cannot read {}: {e}", src.display()))?;
            config = Some(String::from_utf8_lossy(&text).into_owned());
            continue;
        }
        files.push(ProfileFile {
            src,
            rel: parts.iter().collect(),
        });
    }
    files.sort_by(|a, b| a.rel.cmp(&b.rel));
    let config = config.ok_or_else(|| {
        format!(
            "the profile in {} has no config.txt, so it is not a UEVR profile",
            fetched.display()
        )
    })?;
    Ok(Profile { files, config })
}

fn walk(
    dir: &Path,
    prefix: &mut Vec<String>,
    out: &mut Vec<(PathBuf, Vec<String>)>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut entries: Vec<_> = entries
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        // A zip made on Windows may keep `uobjecthook\1_props.json` as one literal name.
        let parts: Vec<String> = name.split('\\').map(str::to_owned).collect();
        if parts
            .iter()
            .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err(format!("the profile has a bad file name: {name}"));
        }
        let depth = prefix.len();
        prefix.extend(parts);
        if path.is_dir() {
            walk(&path, prefix, out)?;
        } else {
            out.push((path, prefix.clone()));
        }
        prefix.truncate(depth);
    }
    Ok(())
}

fn ends_with_ignore_case(name: &str, suffix: &str) -> bool {
    name.len() >= suffix.len()
        && name
            .get(name.len() - suffix.len()..)
            .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

fn key_of(line: &str) -> Option<&str> {
    let (key, _) = line.split_once('=')?;
    let key = key.trim();
    (!key.is_empty()).then_some(key)
}

/// The profile's `config.txt` with our `Key=Value` lines in place of its own lines for the same
/// keys (the first one kept where it was, later repeats dropped) and our other keys added at the
/// end. Every other line of the profile is kept as it was, with its line ending style.
pub fn merge_config(base: &str, ours: &str) -> String {
    let newline = if base.contains("\r\n") { "\r\n" } else { "\n" };
    let ours: Vec<(&str, &str)> = ours
        .lines()
        .filter_map(|line| key_of(line).map(|key| (key, line)))
        .collect();
    let mut used = BTreeSet::new();
    let mut out: Vec<&str> = Vec::new();
    for line in base.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        match key_of(line).and_then(|key| ours.iter().find(|(k, _)| *k == key)) {
            Some((key, mine)) => {
                if used.insert(*key) {
                    out.push(mine);
                }
            }
            None => out.push(line),
        }
    }
    for (key, mine) in &ours {
        if used.insert(*key) {
            out.push(mine);
        }
    }
    let mut text = out.join(newline);
    text.push_str(newline);
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_replaces_our_keys_in_place_and_keeps_the_rest() {
        let base = "VR_RenderingMethod=2\r\nFrontend_RequestedRuntime=openvr_api.dll\r\nVR_RenderingMethod=1\r\nUI_Distance=2.0\r\n";
        let ours = "VR_RenderingMethod=0\nVR_2DScreenMode=true\nFrontend_RequestedRuntime=openxr_loader.dll\n";
        assert_eq!(
            merge_config(base, ours),
            "VR_RenderingMethod=0\r\nFrontend_RequestedRuntime=openxr_loader.dll\r\nUI_Distance=2.0\r\nVR_2DScreenMode=true\r\n"
        );
    }

    #[test]
    fn merge_into_an_empty_profile_is_our_text() {
        assert_eq!(merge_config("", "A=1\nB=2\n"), "A=1\nB=2\n");
        assert_eq!(merge_config("# note\nA = 9", "A=1\n"), "# note\nA=1\n");
    }
}
