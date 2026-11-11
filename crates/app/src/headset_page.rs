use flat3d_inject::api::LinkSnapshot;

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
    SetTunnel(bool),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start3d,
    Stop3d,
    SetSetting { key: &'static str, on: bool },
    Refresh,
}
pub fn update(_state: &mut HeadsetState, _msg: Msg) -> Option<Action> {
    None
}
