#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    WaitingHeadset,
    WaitingSource,
    Showing,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Status {
    pub state: State,
    pub fps: f32,
    pub w: u32,
    pub h: u32,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WaitingHeadset => "waiting-headset",
            Self::WaitingSource => "waiting-source",
            Self::Showing => "showing",
            Self::Error => "error",
        }
    }
}

impl Status {
    pub fn to_line(&self) -> String {
        let fps = if self.fps.is_finite() { self.fps } else { 0.0 };
        format!(
            "{{\"state\":\"{}\",\"fps\":{:.1},\"w\":{},\"h\":{}}}",
            self.state.as_str(),
            fps,
            self.w,
            self.h
        )
    }

    pub fn from_line(line: &str) -> Result<Status, String> {
        let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        let inner = compact
            .strip_prefix("{\"state\":\"")
            .and_then(|s| s.strip_suffix('}'))
            .ok_or_else(|| "invalid status line".to_string())?;
        let (state, rest) = inner
            .split_once("\",\"fps\":")
            .ok_or_else(|| "invalid status state".to_string())?;
        let (fps, rest) = rest
            .split_once(",\"w\":")
            .ok_or_else(|| "invalid status fps".to_string())?;
        let (w, h) = rest
            .split_once(",\"h\":")
            .ok_or_else(|| "invalid status dimensions".to_string())?;
        let state = match state {
            "waiting-headset" => State::WaitingHeadset,
            "waiting-source" => State::WaitingSource,
            "showing" => State::Showing,
            "error" => State::Error,
            _ => return Err("unknown status state".to_string()),
        };
        let fps = fps
            .parse::<f32>()
            .map_err(|_| "invalid status fps".to_string())?;
        let w = w
            .parse::<u32>()
            .map_err(|_| "invalid status width".to_string())?;
        let h = h
            .parse::<u32>()
            .map_err(|_| "invalid status height".to_string())?;
        Ok(Status { state, fps, w, h })
    }
}
