//! Reversible INI edits, pure: text in, text out. docs/code-map.md, Parts.

use crate::undo::{escape, unescape};
use std::collections::BTreeMap;

const HEADER: &str = "flat3d-ini-patch 1";
const CRLF: &str = "\r\n";
const LF: &str = "\n";

/// A file name and its text, or `None` when the file is missing.
pub type IniFiles = BTreeMap<String, Option<String>>;

/// One setting a route wants in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniEdit {
    pub file: String,
    pub section: String,
    pub key: String,
    pub value: String,
}

/// What one edit found, kept so it can be put back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchEntry {
    pub file: String,
    pub section: String,
    pub key: String,
    /// The value before our first edit; `None` when the key was absent.
    pub before: Option<String>,
    /// Whether the file existed before our first edit.
    pub file_existed: bool,
    /// Whether the section existed before our first edit.
    pub section_existed: bool,
    /// Whether our first edit inserted the key line rather than changing one.
    pub inserted: bool,
    /// Whether our first edit had to add a line ending to the old last line.
    pub eol_added: bool,
}

/// The recovery point for a set of INI edits.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatchRecord {
    pub edits: Vec<PatchEntry>,
}

impl PatchRecord {
    /// The exact text of the patch record file.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        text.push_str(HEADER);
        text.push('\n');
        for e in &self.edits {
            text.push_str(&format!(
                "edit\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                escape(&e.file),
                escape(&e.section),
                escape(&e.key),
                flag(e.file_existed),
                flag(e.section_existed),
                flag(e.inserted),
                flag(e.eol_added),
                flag(e.before.is_some()),
                escape(e.before.as_deref().unwrap_or(""))
            ));
        }
        text
    }

    /// Read a patch record, or say why the text is not one.
    pub fn from_text(text: &str) -> Result<PatchRecord, String> {
        let mut lines = text.split('\n');
        if lines.next() != Some(HEADER) {
            return Err("not an ini patch record".to_string());
        }

        let mut edits = Vec::new();
        for line in lines {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.is_empty() {
                continue;
            }
            let mut fields = line.split('\t');
            if fields.next() != Some("edit") {
                return Err(format!("unknown line: {line}"));
            }
            let file = unescape(fields.next().ok_or("bad edit line")?);
            let section = unescape(fields.next().ok_or("bad edit line")?);
            let key = unescape(fields.next().ok_or("bad edit line")?);
            let file_existed = parse_flag(fields.next())?;
            let section_existed = parse_flag(fields.next())?;
            let inserted = parse_flag(fields.next())?;
            let eol_added = parse_flag(fields.next())?;
            let has_before = parse_flag(fields.next())?;
            let before = if has_before {
                Some(unescape(fields.next().unwrap_or("")))
            } else {
                None
            };
            edits.push(PatchEntry {
                file,
                section,
                key,
                before,
                file_existed,
                section_existed,
                inserted,
                eol_added,
            });
        }

        Ok(PatchRecord { edits })
    }
}

fn flag(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

fn parse_flag(value: Option<&str>) -> Result<bool, String> {
    match value {
        Some("1") => Ok(true),
        Some("0") => Ok(false),
        _ => Err("bad flag".to_string()),
    }
}

/// Apply a fresh set of edits and record how to undo them.
pub fn apply(edits: &[IniEdit], files: &IniFiles) -> (IniFiles, PatchRecord) {
    apply_over(&PatchRecord::default(), edits, files)
}

/// Apply edits on top of an existing record, keeping each key's first earlier value.
pub fn apply_over(
    record: &PatchRecord,
    edits: &[IniEdit],
    files: &IniFiles,
) -> (IniFiles, PatchRecord) {
    let mut out = files.clone();
    let mut merged = record.edits.clone();

    let mut groups: BTreeMap<&str, Vec<&IniEdit>> = BTreeMap::new();
    for edit in edits {
        groups.entry(edit.file.as_str()).or_default().push(edit);
    }

    for (file, group) in groups {
        let current = out.get(file).cloned().flatten();
        let mut lines = current.as_deref().map(parse_lines).unwrap_or_default();
        let eol = if current.is_some() {
            file_eol(&lines)
        } else {
            CRLF
        };

        for edit in group {
            let before = earlier_value(&lines, &edit.section, &edit.key);
            let applied = apply_edit(&mut lines, edit, eol);
            if !merged.iter().any(|e| same_setting(e, edit)) {
                merged.push(PatchEntry {
                    file: edit.file.clone(),
                    section: edit.section.clone(),
                    key: edit.key.clone(),
                    before,
                    file_existed: current.is_some(),
                    section_existed: applied.section_existed,
                    inserted: applied.inserted,
                    eol_added: applied.eol_added,
                });
            }
        }

        out.insert(file.to_string(), Some(render_lines(&lines)));
    }

    (out, PatchRecord { edits: merged })
}

/// Put every patched key back, even where the game changed it since.
pub fn restore(record: &PatchRecord, files: &IniFiles) -> IniFiles {
    let mut out = files.clone();

    let mut groups: BTreeMap<&str, Vec<&PatchEntry>> = BTreeMap::new();
    for entry in &record.edits {
        groups.entry(entry.file.as_str()).or_default().push(entry);
    }

    for (file, entries) in groups {
        let current = out.get(file).cloned().flatten();
        let Some(first) = entries.first() else {
            continue;
        };
        let needs_value = entries.iter().any(|e| e.before.is_some());
        if current.is_none() && !needs_value {
            continue;
        }

        let mut lines = current.as_deref().map(parse_lines).unwrap_or_default();
        let eol = if current.is_some() {
            file_eol(&lines)
        } else {
            CRLF
        };

        for entry in &entries {
            match &entry.before {
                Some(value) => restore_key(&mut lines, entry, value, eol),
                None => remove_key(&mut lines, entry),
            }
        }

        for entry in &entries {
            if !entry.section_existed && section_is_empty(&lines, &entry.section) {
                remove_section(&mut lines, entry);
            }
        }

        let restored = if !first.file_existed && lines.is_empty() {
            None
        } else {
            Some(render_lines(&lines))
        };
        out.insert(file.to_string(), restored);
    }

    out
}

fn same_setting(entry: &PatchEntry, edit: &IniEdit) -> bool {
    entry.file == edit.file
        && entry.section.eq_ignore_ascii_case(&edit.section)
        && entry.key.eq_ignore_ascii_case(&edit.key)
}

struct Applied {
    section_existed: bool,
    inserted: bool,
    eol_added: bool,
}

fn apply_edit(lines: &mut Vec<IniLine>, edit: &IniEdit, eol: &str) -> Applied {
    let Some(section) = find_section(lines, &edit.section) else {
        let eol_added = append_line(lines, format!("[{}]", edit.section), eol);
        append_line(lines, format!("{}={}", edit.key, edit.value), eol);
        return Applied {
            section_existed: false,
            inserted: true,
            eol_added,
        };
    };

    let end = section_end(lines, section);
    if let Some(key) = find_key(lines, section + 1, end, &edit.key) {
        replace_value(&mut lines[key], &edit.value);
        return Applied {
            section_existed: true,
            inserted: false,
            eol_added: false,
        };
    }

    let eol_added = insert_line(lines, end, format!("{}={}", edit.key, edit.value), eol);
    Applied {
        section_existed: true,
        inserted: true,
        eol_added,
    }
}

fn restore_key(lines: &mut Vec<IniLine>, entry: &PatchEntry, value: &str, eol: &str) {
    let Some(section) = find_section(lines, &entry.section) else {
        append_line(lines, format!("[{}]", entry.section), eol);
        append_line(lines, format!("{}={}", entry.key, value), eol);
        return;
    };

    let end = section_end(lines, section);
    if let Some(key) = find_key(lines, section + 1, end, &entry.key) {
        replace_value(&mut lines[key], value);
    } else {
        insert_line(lines, end, format!("{}={}", entry.key, value), eol);
    }
}

fn remove_key(lines: &mut Vec<IniLine>, entry: &PatchEntry) {
    let Some(section) = find_section(lines, &entry.section) else {
        return;
    };
    let end = section_end(lines, section);
    if let Some(key) = find_key(lines, section + 1, end, &entry.key) {
        remove_line(lines, key, entry.eol_added);
    }
}

fn section_is_empty(lines: &[IniLine], name: &str) -> bool {
    let Some(section) = find_section(lines, name) else {
        return false;
    };
    let end = section_end(lines, section);
    lines[section + 1..end]
        .iter()
        .all(|line| trimmed(&line.text).is_empty())
}

fn remove_section(lines: &mut Vec<IniLine>, entry: &PatchEntry) {
    let Some(section) = find_section(lines, &entry.section) else {
        return;
    };
    let end = section_end(lines, section);
    lines.drain(section..end);
    if entry.eol_added && section == lines.len() && section > 0 {
        lines[section - 1].ending.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IniLine {
    text: String,
    ending: String,
}

fn parse_lines(text: &str) -> Vec<IniLine> {
    text.split_inclusive('\n')
        .map(|part| {
            if let Some(body) = part.strip_suffix("\r\n") {
                IniLine {
                    text: body.to_string(),
                    ending: CRLF.to_string(),
                }
            } else if let Some(body) = part.strip_suffix('\n') {
                IniLine {
                    text: body.to_string(),
                    ending: LF.to_string(),
                }
            } else {
                IniLine {
                    text: part.to_string(),
                    ending: String::new(),
                }
            }
        })
        .collect()
}

fn render_lines(lines: &[IniLine]) -> String {
    let mut text = String::new();
    for line in lines {
        text.push_str(&line.text);
        text.push_str(&line.ending);
    }
    text
}

fn file_eol(lines: &[IniLine]) -> &'static str {
    for line in lines {
        if line.ending == CRLF {
            return CRLF;
        }
        if line.ending == LF {
            return LF;
        }
    }
    CRLF
}

fn trimmed(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

fn section_name(text: &str) -> Option<&str> {
    let text = trimmed(text);
    let rest = text.strip_prefix('[')?;
    let end = rest.find(']')?;
    Some(rest[..end].trim())
}

fn key_value(text: &str) -> Option<(&str, usize)> {
    let trimmed = trimmed(text);
    if trimmed.is_empty()
        || trimmed.starts_with(';')
        || trimmed.starts_with('#')
        || trimmed.starts_with('[')
    {
        return None;
    }
    let equals = text.find('=')?;
    let key = text[..equals].trim();
    if key.is_empty() {
        return None;
    }
    Some((key, equals + 1))
}

fn find_section(lines: &[IniLine], name: &str) -> Option<usize> {
    lines.iter().position(|line| {
        section_name(&line.text).is_some_and(|found| found.eq_ignore_ascii_case(name))
    })
}

fn section_end(lines: &[IniLine], section: usize) -> usize {
    (section + 1..lines.len())
        .find(|&i| section_name(&lines[i].text).is_some())
        .unwrap_or(lines.len())
}

fn find_key(lines: &[IniLine], start: usize, end: usize, key: &str) -> Option<usize> {
    (start..end).find(|&i| {
        key_value(&lines[i].text).is_some_and(|(found, _)| found.eq_ignore_ascii_case(key))
    })
}

fn earlier_value(lines: &[IniLine], section: &str, key: &str) -> Option<String> {
    let section = find_section(lines, section)?;
    let end = section_end(lines, section);
    let line = find_key(lines, section + 1, end, key)?;
    let (_, value_start) = key_value(&lines[line].text)?;
    Some(lines[line].text[value_start..].trim().to_string())
}

fn replace_value(line: &mut IniLine, value: &str) {
    let Some((_, value_start)) = key_value(&line.text) else {
        return;
    };
    let after = &line.text[value_start..];
    let spacing = after.len() - after.trim_start_matches([' ', '\t']).len();
    line.text.truncate(value_start + spacing);
    line.text.push_str(value);
}

fn append_line(lines: &mut Vec<IniLine>, text: String, eol: &str) -> bool {
    let mut eol_added = false;
    if let Some(last) = lines.last_mut() {
        if last.ending.is_empty() {
            last.ending.push_str(eol);
            eol_added = true;
        }
    }
    lines.push(IniLine {
        text,
        ending: eol.to_string(),
    });
    eol_added
}

fn insert_line(lines: &mut Vec<IniLine>, at: usize, text: String, eol: &str) -> bool {
    if at == lines.len() {
        return append_line(lines, text, eol);
    }
    lines.insert(
        at,
        IniLine {
            text,
            ending: eol.to_string(),
        },
    );
    false
}

fn remove_line(lines: &mut Vec<IniLine>, at: usize, eol_added: bool) {
    lines.remove(at);
    if eol_added && at == lines.len() && at > 0 {
        lines[at - 1].ending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(pairs: &[(&str, Option<&str>)]) -> IniFiles {
        pairs
            .iter()
            .map(|(name, text)| ((*name).to_string(), text.map(str::to_string)))
            .collect()
    }

    fn edit(file: &str, section: &str, key: &str, value: &str) -> IniEdit {
        IniEdit {
            file: file.to_string(),
            section: section.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn patch_changes_only_the_value_and_keeps_the_key_spelling() {
        let input = files(&[(
            "game.ini",
            Some("[Display]\r\nResolution = 1920x1080\r\n; keep me\r\nSound=on\r\n"),
        )]);
        let (out, record) = apply(
            &[edit("game.ini", "display", "RESOLUTION", "2560x1440")],
            &input,
        );
        assert_eq!(
            out["game.ini"].as_deref(),
            Some("[Display]\r\nResolution = 2560x1440\r\n; keep me\r\nSound=on\r\n")
        );
        assert_eq!(record.edits.len(), 1);
        assert_eq!(record.edits[0].before.as_deref(), Some("1920x1080"));
        assert!(record.edits[0].file_existed);
        assert!(record.edits[0].section_existed);
    }

    #[test]
    fn restore_puts_the_key_back() {
        let input = files(&[("game.ini", Some("[Display]\nResolution=1920x1080\n"))]);
        let (patched, record) = apply(
            &[edit("game.ini", "Display", "Resolution", "2560x1440")],
            &input,
        );
        assert_eq!(restore(&record, &patched), input);
    }

    #[test]
    fn round_trip_is_byte_identical() {
        let input = files(&[
            (
                "a.ini",
                Some("; header\r\n[One]\r\nA=1\r\n\r\n[Two]\r\nB=2"),
            ),
            ("b.ini", Some("[X]\nY=keep\n")),
        ]);
        let edits = [
            edit("a.ini", "one", "a", "changed"),
            edit("a.ini", "three", "c", "3"),
            edit("b.ini", "x", "z", "new"),
        ];
        let (patched, record) = apply(&edits, &input);
        assert_eq!(restore(&record, &patched), input);
    }

    #[test]
    fn a_non_patched_key_changed_by_the_game_survives() {
        let input = files(&[("game.ini", Some("[S]\nPatch=old\nGame=before\n"))]);
        let (mut patched, record) = apply(&[edit("game.ini", "S", "Patch", "new")], &input);
        patched.insert(
            "game.ini".to_string(),
            Some("[S]\nPatch=new\nGame=after\n".to_string()),
        );
        let restored = restore(&record, &patched);
        assert_eq!(
            restored["game.ini"].as_deref(),
            Some("[S]\nPatch=old\nGame=after\n")
        );
    }

    #[test]
    fn a_patched_key_rewritten_by_the_game_still_restores() {
        let input = files(&[("game.ini", Some("[S]\nPatch=old\n"))]);
        let (mut patched, record) = apply(&[edit("game.ini", "S", "Patch", "new")], &input);
        patched.insert(
            "game.ini".to_string(),
            Some("[s]\nPATCH=the game wrote this\n".to_string()),
        );
        let restored = restore(&record, &patched);
        assert_eq!(restored["game.ini"].as_deref(), Some("[s]\nPATCH=old\n"));
    }

    #[test]
    fn a_missing_file_is_created_with_crlf_and_removed_again() {
        let empty = files(&[]);
        let (patched, record) = apply(&[edit("new.ini", "Section", "Key", "Value")], &empty);
        assert_eq!(
            patched["new.ini"].as_deref(),
            Some("[Section]\r\nKey=Value\r\n")
        );
        assert!(!record.edits[0].file_existed);
        assert!(!record.edits[0].section_existed);
        let restored = restore(&record, &patched);
        assert_eq!(restored.get("new.ini"), Some(&None));
    }

    #[test]
    fn a_missing_section_is_added_and_removed_again() {
        let input = files(&[("game.ini", Some("[A]\r\nX=1\r\n"))]);
        let (patched, record) = apply(&[edit("game.ini", "B", "Y", "2")], &input);
        assert_eq!(
            patched["game.ini"].as_deref(),
            Some("[A]\r\nX=1\r\n[B]\r\nY=2\r\n")
        );
        assert!(!record.edits[0].section_existed);
        assert_eq!(restore(&record, &patched), input);
    }

    #[test]
    fn crlf_and_lf_are_each_kept() {
        for ending in ["\r\n", "\n"] {
            let before = format!("[S]{ending}A=1{ending}");
            let input = files(&[("game.ini", Some(before.as_str()))]);
            let (patched, record) = apply(&[edit("game.ini", "S", "B", "2")], &input);
            assert_eq!(
                patched["game.ini"].as_deref(),
                Some(format!("[S]{ending}A=1{ending}B=2{ending}").as_str())
            );
            assert_eq!(restore(&record, &patched), input);
        }
    }

    #[test]
    fn applying_twice_keeps_the_first_earlier_value() {
        let input = files(&[("game.ini", Some("[S]\nKey=original\n"))]);
        let edits = [edit("game.ini", "S", "Key", "first")];
        let (once, record) = apply(&edits, &input);
        let edits = [edit("game.ini", "S", "Key", "second")];
        let (twice, record) = apply_over(&record, &edits, &once);
        assert_eq!(twice["game.ini"].as_deref(), Some("[S]\nKey=second\n"));
        assert_eq!(record.edits.len(), 1);
        assert_eq!(record.edits[0].before.as_deref(), Some("original"));
        assert_eq!(restore(&record, &twice), input);
    }

    #[test]
    fn a_patched_file_missing_at_restore_is_recreated_only_for_earlier_values() {
        let input = files(&[("game.ini", Some("[S]\nKey=original\n"))]);
        let (patched, record) = apply(&[edit("game.ini", "S", "Key", "new")], &input);
        let gone = files(&[("game.ini", None)]);
        assert_eq!(
            restore(&record, &gone)["game.ini"].as_deref(),
            Some("[S]\r\nKey=original\r\n")
        );
        assert_eq!(
            restore(&record, &patched)["game.ini"].as_deref(),
            Some("[S]\nKey=original\n")
        );

        let input = files(&[("game.ini", Some("[S]\nOther=keep\n"))]);
        let (patched, record) = apply(&[edit("game.ini", "S", "Added", "new")], &input);
        let gone = files(&[("game.ini", None)]);
        assert_eq!(restore(&record, &gone)["game.ini"].as_deref(), None);
        assert_eq!(
            restore(&record, &patched)["game.ini"].as_deref(),
            Some("[S]\nOther=keep\n")
        );

        let created = files(&[]);
        let (created_patched, created_record) =
            apply(&[edit("new.ini", "S", "Key", "new")], &created);
        let gone = files(&[("new.ini", None)]);
        assert_eq!(restore(&created_record, &gone)["new.ini"].as_deref(), None);
        assert_eq!(
            restore(&created_record, &created_patched)["new.ini"].as_deref(),
            None
        );
    }

    #[test]
    fn a_record_round_trips_through_its_text() {
        let input = files(&[("game.ini", Some("[S]\nKey=a\tb\n"))]);
        let (_, record) = apply(
            &[
                edit("game.ini", "S", "Key", "new"),
                edit("new.ini", "T", "Other", "x"),
            ],
            &input,
        );
        assert_eq!(PatchRecord::from_text(&record.to_text()), Ok(record));
    }
}
