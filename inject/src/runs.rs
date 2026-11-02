//! RunRecord: which games run in 3D and which flat. docs/code-map.md, Parts.

/// How a running game is being shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Rendered in 3D through the headset link.
    ThreeD,
    /// Rendered flat, as the game does by itself.
    Flat,
}

/// One running game, as the wrapper writes it to `runs/run-<pid>.txt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunRecord {
    /// The Steam app id of the game.
    pub appid: u32,
    /// The process id of the running game.
    pub pid: u32,
    /// Whether the game runs in 3D or flat.
    pub mode: RunMode,
}

impl RunRecord {
    /// The exact text of the run record file.
    pub fn to_text(&self) -> String {
        let mode = match self.mode {
            RunMode::ThreeD => "3d",
            RunMode::Flat => "flat",
        };
        format!(
            "flat3d-run 1\nappid\t{}\npid\t{}\nmode\t{}\n",
            self.appid, self.pid, mode
        )
    }
}

/// Read a run record, or `None` if the text is not one we wrote.
pub fn parse_run_record(text: &str) -> Option<RunRecord> {
    let mut lines = text.split('\n');
    if lines.next()? != "flat3d-run 1" {
        return None;
    }

    let mut appid = None;
    let mut pid = None;
    let mut mode = None;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('\t')?;
        match key {
            "appid" => appid = Some(value.parse().ok()?),
            "pid" => pid = Some(value.parse().ok()?),
            "mode" => {
                mode = Some(match value {
                    "3d" => RunMode::ThreeD,
                    "flat" => RunMode::Flat,
                    _ => return None,
                })
            }
            _ => {}
        }
    }

    Some(RunRecord {
        appid: appid?,
        pid: pid?,
        mode: mode?,
    })
}

/// Whether any live pid runs in 3D, and whether any live pid runs flat.
pub fn running_modes(records: &[RunRecord], alive: &dyn Fn(u32) -> bool) -> (bool, bool) {
    let three_d = records
        .iter()
        .any(|r| r.mode == RunMode::ThreeD && alive(r.pid));
    let flat = records
        .iter()
        .any(|r| r.mode == RunMode::Flat && alive(r.pid));
    (three_d, flat)
}
