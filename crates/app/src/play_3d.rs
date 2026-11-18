//! The per-game 3D button: session state and `update`. Pure: no UI, no process spawning and no
//! engine calls. The app shell turns an `Action` into engine work.
//!
//! The game is never closed. No `Action` and no code path here closes, stops or exits a game,
//! because a forced exit can lose progress: ending a session only stops the headset view.
//!
//! The shell's contract: every `StartLinkAndLaunch` and `ResumeIn3d` is answered by exactly one
//! `Started` or `StartFailed`, the engine work behind them is time-limited, and only one is
//! outstanding at a time. `PlayState::awaiting` holds the appid of that unanswered action and a
//! late reply is matched through it, so a second press cannot be taken for the first answer. That
//! is why `Starting` and `Resuming` need no timeout here and messages carry no attempt id.

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
    /// The headset has been lost since the last time it was seen returned.
    pub headset_lost: bool,
    /// The appid whose `StartLinkAndLaunch` or `ResumeIn3d` has not been answered yet.
    pub awaiting: Option<u32>,
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
    /// Stops only the headset view, never the game.
    StopViewer(u32),
}

pub const FLAT_QUALITY_NOTE: &str = "Full flat quality returns the next time the game starts.";

/// The button for `appid`: its own session, another game's, or a fresh play.
pub fn button(state: &PlayState, appid: u32) -> Button {
    match state.session {
        None => {
            if state.awaiting.is_some() {
                Button::Busy
            } else {
                Button::PlayIn3d
            }
        }
        Some(session) if session.appid != appid => Button::OtherGameRunning,
        Some(session) => match session.mode {
            Mode::Starting | Mode::Resuming => Button::Busy,
            Mode::In3d => Button::BringBackToLaptop,
            Mode::OnLaptop { headset_back: true } => {
                if state.awaiting.is_some() {
                    Button::Busy
                } else {
                    Button::BackTo3d
                }
            }
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
/// `error`; a message naming a different appid than the session is ignored. `Started` and
/// `StartFailed` are ignored unless they answer the action named by `awaiting`.
pub fn update(state: &mut PlayState, msg: Msg) -> Option<Action> {
    match msg {
        Msg::PlayPressed(appid) => {
            if state.awaiting.is_some() || state.session.is_some() {
                return None;
            }
            state.session = Some(Session {
                appid,
                mode: Mode::Starting,
            });
            state.awaiting = Some(appid);
            state.error = None;
            Some(Action::StartLinkAndLaunch(appid))
        }
        Msg::Started(appid) => {
            if state.awaiting != Some(appid) {
                return None;
            }
            state.awaiting = None;
            match state.session {
                Some(session) if session.appid == appid => match session.mode {
                    Mode::Starting | Mode::Resuming => {
                        state.session = Some(Session {
                            appid,
                            mode: Mode::In3d,
                        });
                        None
                    }
                    // The viewer came up after the session moved on.
                    Mode::OnLaptop { .. } => Some(Action::StopViewer(appid)),
                    Mode::In3d => None,
                },
                // No session for this game any more, so the late viewer is stopped.
                _ => Some(Action::StopViewer(appid)),
            }
        }
        Msg::StartFailed(appid, error) => {
            if state.awaiting != Some(appid) {
                return None;
            }
            state.awaiting = None;
            let session = match state.session {
                Some(session) if session.appid == appid => session,
                _ => return None,
            };
            match session.mode {
                Mode::Starting => state.session = None,
                // The game is still running, so it stays and falls back to the laptop.
                Mode::Resuming => {
                    state.session = Some(Session {
                        appid,
                        mode: Mode::OnLaptop {
                            headset_back: !state.headset_lost,
                        },
                    });
                }
                Mode::OnLaptop { .. } => {}
                Mode::In3d => return None,
            }
            state.error = Some(error);
            None
        }
        Msg::BringBackPressed(appid) => {
            let session = state.session?;
            if session.appid != appid || session.mode != Mode::In3d {
                return None;
            }
            // The way back stays on offer only while the headset is still attached.
            state.session = Some(Session {
                appid,
                mode: Mode::OnLaptop {
                    headset_back: !state.headset_lost,
                },
            });
            state.error = None;
            Some(Action::BringBack(appid))
        }
        Msg::GraceEnded => {
            state.headset_lost = true;
            if let Some(session) = state.session.as_mut() {
                if matches!(session.mode, Mode::In3d | Mode::Resuming) {
                    // The link service has already done the work, and the headset is gone.
                    session.mode = Mode::OnLaptop {
                        headset_back: false,
                    };
                }
            }
            None
        }
        Msg::HeadsetReturned => {
            state.headset_lost = false;
            if let Some(session) = state.session.as_mut() {
                if let Mode::OnLaptop { headset_back } = &mut session.mode {
                    *headset_back = true;
                }
            }
            None
        }
        Msg::HeadsetLost => {
            state.headset_lost = true;
            if let Some(session) = state.session.as_mut() {
                if let Mode::OnLaptop { headset_back } = &mut session.mode {
                    *headset_back = false;
                }
            }
            None
        }
        Msg::BackTo3dPressed(appid) => {
            if state.awaiting.is_some() {
                return None;
            }
            let session = state.session?;
            if session.appid != appid || session.mode != (Mode::OnLaptop { headset_back: true }) {
                return None;
            }
            state.session = Some(Session {
                appid,
                mode: Mode::Resuming,
            });
            state.awaiting = Some(appid);
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
