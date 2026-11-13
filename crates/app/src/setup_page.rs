//! The Setup page checklist. docs/code-map.md, Parts, crates/app/src.

use flat3d_inject::api::{AdbSource, LinkSnapshot, LinkState};
use flat3d_inject::setup::Packages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub title: &'static str,
    pub done: bool,
    pub manual: bool,
    pub hint: &'static str,
}

pub fn steps(packages: Option<&Packages>, snapshot: Option<&LinkSnapshot>) -> Vec<Step> {
    let link = snapshot.map(|snapshot| snapshot.state);
    let adb_found = snapshot.is_some_and(|snapshot| snapshot.adb_source != AdbSource::Missing);
    let plugged_in = matches!(
        link,
        Some(LinkState::Unauthorised | LinkState::Starting | LinkState::Ready)
    );
    let debugging_allowed = matches!(link, Some(LinkState::Starting | LinkState::Ready));

    vec![
        Step {
            title: "adb found",
            done: adb_found,
            manual: false,
            hint: "Run flat3d-inject install.",
        },
        Step {
            title: "Headset plugged in",
            done: plugged_in,
            manual: false,
            hint: "Plug in the Quest with a USB cable.",
        },
        Step {
            title: "USB debugging allowed",
            done: debugging_allowed,
            manual: false,
            hint: "Put the headset on and allow USB debugging.",
        },
        Step {
            title: "WiVRn on the headset",
            done: packages.is_some_and(|packages| packages.wivrn),
            manual: false,
            hint: "Install WiVRn on the Quest.",
        },
        Step {
            title: "Seated boundary",
            done: false,
            manual: true,
            hint: "Set a seated boundary in the headset; this cannot be checked from the PC.",
        },
    ]
}

pub fn all_done(steps: &[Step]) -> bool {
    steps
        .iter()
        .filter(|step| !step.manual)
        .all(|step| step.done)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupKind {
    Packages,
    InstallApps,
    Doctor,
}

impl SetupKind {
    pub fn args(self) -> &'static [&'static str] {
        match self {
            Self::Packages => &["setup", "packages"],
            Self::InstallApps => &["setup", "install-apps"],
            Self::Doctor => &["doctor"],
        }
    }
}

pub fn adb_source_text(snapshot: Option<&LinkSnapshot>) -> &'static str {
    match snapshot.map(|snapshot| snapshot.adb_source) {
        Some(AdbSource::Running) => "Using the adb that is already running",
        Some(AdbSource::Bundled) => "Using the bundled adb",
        Some(AdbSource::Missing) => "No adb found",
        None => "Unknown until the link service runs",
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetupState {
    pub packages: Option<Packages>,
    pub snapshot: Option<LinkSnapshot>,
    pub busy: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Packages(Result<Packages, String>),
    Snapshot(Result<LinkSnapshot, String>),
    RunPressed(SetupKind),
    Done(Result<(), String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    RunSetup(SetupKind),
    Refresh,
}

pub fn update(state: &mut SetupState, msg: Msg) -> Option<Action> {
    match msg {
        Msg::Packages(Ok(packages)) => {
            state.packages = Some(packages);
            state.error = None;
            None
        }
        Msg::Packages(Err(error)) => {
            state.error = Some(error);
            None
        }
        Msg::Snapshot(Ok(snapshot)) => {
            state.snapshot = Some(snapshot);
            state.error = None;
            None
        }
        Msg::Snapshot(Err(error)) => {
            state.error = Some(error);
            None
        }
        Msg::RunPressed(kind) => {
            if state.busy {
                None
            } else {
                state.busy = true;
                Some(Action::RunSetup(kind))
            }
        }
        Msg::Done(Ok(())) => {
            state.busy = false;
            Some(Action::Refresh)
        }
        Msg::Done(Err(error)) => {
            state.busy = false;
            state.error = Some(error);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{adb_source_text, update, Action, Msg, SetupKind, SetupState};
    use flat3d_inject::api::{AdbSource, LinkSnapshot};
    use flat3d_inject::cli::{parse_any, AnyCommand, DoctorCommand, SetupCommand};
    use flat3d_inject::setup::Packages;

    #[test]
    fn run_pressed_once_while_busy() {
        let mut state = SetupState::default();
        assert_eq!(
            update(&mut state, Msg::RunPressed(SetupKind::Packages)),
            Some(Action::RunSetup(SetupKind::Packages))
        );
        assert!(state.busy);
        assert_eq!(update(&mut state, Msg::RunPressed(SetupKind::Doctor)), None);
        assert!(state.busy);
    }

    #[test]
    fn done_err_sets_error_and_frees() {
        let mut state = SetupState {
            busy: true,
            ..SetupState::default()
        };
        assert_eq!(
            update(&mut state, Msg::Done(Err("doctor failed".into()))),
            None
        );
        assert!(!state.busy);
        assert_eq!(state.error.as_deref(), Some("doctor failed"));
    }

    #[test]
    fn failed_packages_read_keeps_old() {
        let old = Packages { wivrn: true };
        let mut state = SetupState {
            packages: Some(old),
            ..SetupState::default()
        };
        update(&mut state, Msg::Packages(Err("read failed".into())));
        assert_eq!(state.packages, Some(old));
        assert_eq!(state.error.as_deref(), Some("read failed"));
    }

    #[test]
    fn kind_args_exact() {
        assert_eq!(SetupKind::Packages.args(), &["setup", "packages"]);
        assert_eq!(SetupKind::InstallApps.args(), &["setup", "install-apps"]);
        assert_eq!(SetupKind::Doctor.args(), &["doctor"]);
        assert!([
            SetupKind::Packages,
            SetupKind::InstallApps,
            SetupKind::Doctor
        ]
        .iter()
        .all(|kind| !kind.args().contains(&"--json")));
    }

    #[test]
    fn kind_args_parse() {
        for (kind, expected) in [
            (
                SetupKind::Packages,
                AnyCommand::Setup(SetupCommand::Packages),
            ),
            (
                SetupKind::InstallApps,
                AnyCommand::Setup(SetupCommand::InstallApps),
            ),
            (SetupKind::Doctor, AnyCommand::Doctor(DoctorCommand::Run)),
        ] {
            let mut argv: Vec<String> = kind.args().iter().map(|arg| (*arg).to_string()).collect();
            argv.push("--json".into());
            let parsed = parse_any(&argv).unwrap();
            assert!(parsed.json);
            assert!(matches!(
                (parsed.command, expected),
                (
                    AnyCommand::Setup(SetupCommand::Packages),
                    AnyCommand::Setup(SetupCommand::Packages)
                ) | (
                    AnyCommand::Setup(SetupCommand::InstallApps),
                    AnyCommand::Setup(SetupCommand::InstallApps)
                ) | (
                    AnyCommand::Doctor(DoctorCommand::Run),
                    AnyCommand::Doctor(DoctorCommand::Run)
                )
            ));
        }
    }

    #[test]
    fn adb_source_text_all_four() {
        let make = |source| {
            let mut snapshot = LinkSnapshot::stopped(1);
            snapshot.adb_source = source;
            snapshot
        };
        assert_eq!(
            adb_source_text(Some(&make(AdbSource::Running))),
            "Using the adb that is already running"
        );
        assert_eq!(
            adb_source_text(Some(&make(AdbSource::Bundled))),
            "Using the bundled adb"
        );
        assert_eq!(
            adb_source_text(Some(&make(AdbSource::Missing))),
            "No adb found"
        );
        assert_eq!(adb_source_text(None), "Unknown until the link service runs");
    }
}
