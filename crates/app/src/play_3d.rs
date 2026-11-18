//! The per-game 3D button: session state and `update`. Pure: no UI, no process spawning and no
//! engine calls. The app shell turns an `Action` into engine work.
//!
//! The game is never closed. No `Action` and no code path here closes, stops or exits a game,
//! because a forced exit can lose progress: ending a session only stops the headset view.

/// Where a game is in its 3D session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Starting,
    In3d,
    Resuming,
    OnLaptop { headset_back: bool },
}

/// The one game in a 3D session at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Session {
    pub appid: u32,
    pub mode: Mode,
}

/// The Games page's 3D on/off state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayState {
    pub session: Option<Session>,
    pub error: Option<String>,
}

/// The button a game shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    PlayIn3d,
    Busy,
    BringBackToLaptop,
    BackTo3d,
    OnLaptop,
    OtherGameRunning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    PlayPressed(u32),
    BringBackPressed(u32),
    BackTo3dPressed(u32),
    Started(u32),
    StartFailed(u32, String),
    GraceEnded,
    HeadsetReturned,
    HeadsetLost,
    GameExited(u32),
}

/// Work for the engine. None of these closes or stops the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartLinkAndLaunch(u32),
    BringBack(u32),
    ResumeIn3d(u32),
}

pub const FLAT_QUALITY_NOTE: &str = "Full flat quality returns the next time the game starts.";

/// The button for `appid`: its own session, another game's, or a fresh play.
pub fn button(state: &PlayState, appid: u32) -> Button {
    match state.session {
        None => Button::PlayIn3d,
        Some(session) if session.appid != appid => Button::OtherGameRunning,
        Some(session) => match session.mode {
            Mode::Starting | Mode::Resuming => Button::Busy,
            Mode::In3d => Button::BringBackToLaptop,
            Mode::OnLaptop { headset_back: true } => Button::BackTo3d,
            Mode::OnLaptop {
                headset_back: false,
            } => Button::OnLaptop,
        },
    }
}

/// The note under a game's button: flat quality is back only for a session on the laptop.
pub fn note(state: &PlayState, appid: u32) -> Option<&'static str> {
    match state.session {
        Some(session)
            if session.appid == appid && matches!(session.mode, Mode::OnLaptop { .. }) =>
        {
            Some(FLAT_QUALITY_NOTE)
        }
        _ => None,
    }
}

/// Apply a message. A transition not listed changes nothing; a press that takes effect clears
/// `error`; a message naming a different appid than the session is ignored.
pub fn update(state: &mut PlayState, msg: Msg) -> Option<Action> {
    match msg {
        Msg::PlayPressed(appid) => {
            if state.session.is_some() {
                return None;
            }
            state.session = Some(Session {
                appid,
                mode: Mode::Starting,
            });
            state.error = None;
            Some(Action::StartLinkAndLaunch(appid))
        }
        Msg::Started(appid) => {
            if let Some(session) = state.session.as_mut() {
                if session.appid == appid && matches!(session.mode, Mode::Starting | Mode::Resuming)
                {
                    session.mode = Mode::In3d;
                }
            }
            None
        }
        Msg::StartFailed(appid, error) => {
            let session = state.session?;
            if session.appid != appid {
                return None;
            }
            match session.mode {
                Mode::Starting => state.session = None,
                // The game is still running, so it stays and falls back to the laptop.
                Mode::Resuming => {
                    state.session = Some(Session {
                        appid,
                        mode: Mode::OnLaptop { headset_back: true },
                    });
                }
                _ => return None,
            }
            state.error = Some(error);
            None
        }
        Msg::BringBackPressed(appid) => {
            let session = state.session?;
            if session.appid != appid || session.mode != Mode::In3d {
                return None;
            }
            // The headset is still attached, so the way back stays on offer.
            state.session = Some(Session {
                appid,
                mode: Mode::OnLaptop { headset_back: true },
            });
            state.error = None;
            Some(Action::BringBack(appid))
        }
        Msg::GraceEnded => {
            if let Some(session) = state.session.as_mut() {
                if session.mode == Mode::In3d {
                    // The link service has already done the work.
                    session.mode = Mode::OnLaptop {
                        headset_back: false,
                    };
                }
            }
            None
        }
        Msg::HeadsetReturned => {
            if let Some(session) = state.session.as_mut() {
                if let Mode::OnLaptop { headset_back } = &mut session.mode {
                    *headset_back = true;
                }
            }
            None
        }
        Msg::HeadsetLost => {
            if let Some(session) = state.session.as_mut() {
                if let Mode::OnLaptop { headset_back } = &mut session.mode {
                    *headset_back = false;
                }
            }
            None
        }
        Msg::BackTo3dPressed(appid) => {
            let session = state.session?;
            if session.appid != appid || session.mode != (Mode::OnLaptop { headset_back: true }) {
                return None;
            }
            state.session = Some(Session {
                appid,
                mode: Mode::Resuming,
            });
            state.error = None;
            Some(Action::ResumeIn3d(appid))
        }
        Msg::GameExited(appid) => {
            if let Some(session) = state.session {
                if session.appid == appid {
                    state.session = None;
                }
            }
            None
        }
    }
}
