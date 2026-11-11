//! Headset and Logs page helpers. docs/code-map.md, Parts.

use serde::Serialize;

use crate::api::LinkState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Buttons {
    pub start: bool,
    pub stop: bool,
}

pub fn headset_buttons(state: LinkState) -> Buttons {
    let up = matches!(state, LinkState::Starting | LinkState::Ready);
    Buttons {
        start: !up,
        stop: up,
    }
}

pub const DEFAULT_LOG_LINES: usize = 500;

pub fn log_lines_newest_first(text: &str, limit: usize) -> Vec<String> {
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect();
    lines.reverse();
    lines.truncate(limit);
    lines
}

pub fn log_copy_text(lines: &[String]) -> String {
    lines.join("\n")
}
