//! Quest Tether's state, read-only. When Quest Tether is installed, Flat to 3D may show its
//! `state` on the faces. It never starts, stops, heals or configures that link, and no decision
//! in the engine depends on the answer (owner ruling 2026-11-10). The only command is `status`.

use crate::exec::CommandRunner;

/// What Quest Tether's `status --json` says that Flat to 3D keeps: the `state` text. Every
/// other field is ignored, so a field that is added or missing changes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestTetherStatus {
    pub state: String,
}

/// The one command: `quest-tether status --json`.
pub fn status_command() -> Vec<String> {
    ["quest-tether", "status", "--json"]
        .map(String::from)
        .to_vec()
}

/// Read the `state` text out of a `status --json` reply. `None` for anything that is not a JSON
/// object with a non-empty string `state`.
pub fn parse_status(out: &str) -> Option<QuestTetherStatus> {
    let value: serde_json::Value = serde_json::from_str(out).ok()?;
    let state = value.get("state")?.as_str()?;
    if state.is_empty() {
        return None;
    }
    Some(QuestTetherStatus {
        state: state.to_string(),
    })
}

/// Ask Quest Tether for its state. `None` means not installed or no answer: the program is
/// missing, it exited non-zero, or it printed something that is not a status.
pub fn quest_tether_status(r: &mut dyn CommandRunner) -> Option<QuestTetherStatus> {
    let output = r.run(&status_command());
    if !output.success {
        return None;
    }
    parse_status(&output.stdout)
}
