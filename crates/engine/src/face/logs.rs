//! The log lines both faces show. docs/code-map.md, API shape, LogLines.
use crate::paths::EnginePaths;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::time::UNIX_EPOCH;

const TAIL_BYTES: u64 = 64 * 1024;
const ERROR_WORDS: [&str; 6] = [
    "error",
    "failed",
    "could not",
    "refused",
    "not found",
    "exec:",
];

/// Pure: newest first, carriage returns stripped, blank lines dropped, truncated to `max`.
/// Kept lines are not trimmed.
pub fn log_lines(text: &str, max: usize) -> Vec<String> {
    crate::face::headset_view::log_lines_newest_first(text, max)
}

/// Read the two XDG state log files and run records under the runtime `state_dir/runs` directory.
/// Returns the merged lines newest first, without changing either directory.
pub fn read_logs(p: &EnginePaths, max: usize) -> Vec<String> {
    let mut entries: Vec<(u64, usize, String)> = Vec::new();
    let mut seq = 0usize;

    for (path, tag) in [(p.link_log(), "link"), (p.run_log(), "run")] {
        let Ok(mut file) = File::open(path) else {
            continue;
        };
        let Ok(metadata) = file.metadata() else {
            continue;
        };
        let len = metadata.len();
        if len > TAIL_BYTES && file.seek(SeekFrom::Start(len - TAIL_BYTES)).is_err() {
            continue;
        }
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_err() {
            continue;
        }
        if len > TAIL_BYTES {
            if let Some(newline) = bytes.iter().position(|byte| *byte == b'\n') {
                bytes.drain(..=newline);
            } else {
                bytes.clear();
            }
        }

        let text = String::from_utf8_lossy(&bytes);
        let mut previous_ts = 0u64;
        for line in text
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .filter(|line| !line.trim().is_empty())
        {
            let (ts, rest) = match line
                .split_once(' ')
                .and_then(|(token, rest)| Some((token.parse::<u64>().ok()?, rest)))
            {
                Some((ts, rest)) => {
                    previous_ts = ts;
                    (ts, rest)
                }
                None => (previous_ts, line),
            };
            entries.push((ts, seq, format!("{ts} {tag}: {rest}")));
            seq += 1;
        }
    }

    let runs_dir = crate::state::runs_dir(&p.state_dir.to_string_lossy());
    if let Ok(records) = fs::read_dir(runs_dir) {
        for entry in records.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("run-") || !name.ends_with(".txt") {
                continue;
            }
            let path = entry.path();
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Some(record) = crate::runs::parse_run_record(&text) else {
                continue;
            };
            let Ok(modified) = entry.metadata().and_then(|meta| meta.modified()) else {
                continue;
            };
            let Ok(age) = modified.duration_since(UNIX_EPOCH) else {
                continue;
            };
            let mode = match record.mode {
                crate::runs::RunMode::ThreeD => "3d",
                crate::runs::RunMode::Flat => "flat",
            };
            entries.push((
                age.as_secs(),
                seq,
                format!(
                    "{} record: appid={} pid={} mode={mode}",
                    age.as_secs(),
                    record.appid,
                    record.pid
                ),
            ));
            seq += 1;
        }
    }

    entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    entries
        .into_iter()
        .take(max)
        .map(|(_, _, line)| line)
        .collect()
}

pub fn is_error_line(line: &str) -> bool {
    let line = line.to_lowercase();
    ERROR_WORDS.iter().any(|word| line.contains(word))
}
