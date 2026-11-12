//! The Headset page: state, messages, update. docs/code-map.md, Parts, crates/app/src.

use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::headset_view::headset_buttons;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeadsetState {
    pub snapshot: Option<LinkSnapshot>,
    pub busy: bool,
    pub error: Option<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Snapshot(Result<LinkSnapshot, String>),
    StartPressed,
    StopPressed,
    Done(Result<(), String>),
    SetAuto3d(bool),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start3d,
    Stop3d,
    SetSetting { key: &'static str, on: bool },
    Refresh,
}
pub fn update(state: &mut HeadsetState, msg: Msg) -> Option<Action> {
    let link = state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.state)
        .unwrap_or(LinkState::Stopped);

    match msg {
        Msg::Snapshot(Ok(snapshot)) => {
            state.snapshot = Some(snapshot);
            state.error = None;
            None
        }
        Msg::Snapshot(Err(error)) => {
            state.error = Some(error);
            None
        }
        Msg::StartPressed => {
            if state.busy || !headset_buttons(link).start {
                return None;
            }
            state.busy = true;
            Some(Action::Start3d)
        }
        Msg::StopPressed => {
            if state.busy || !headset_buttons(link).stop {
                return None;
            }
            state.busy = true;
            Some(Action::Stop3d)
        }
        Msg::Done(Ok(())) => {
            state.busy = false;
            state.error = None;
            Some(Action::Refresh)
        }
        Msg::Done(Err(error)) => {
            state.busy = false;
            state.error = Some(error);
            None
        }
        Msg::SetAuto3d(on) => {
            if let Some(snapshot) = &mut state.snapshot {
                snapshot.auto_3d = on;
            }
            Some(Action::SetSetting { key: "auto-3d", on })
        }
    }
}
