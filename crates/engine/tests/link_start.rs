//! Locked tests for E2b: `link::start_3d` over a `CommandRunner`. A fake runner answers
//! each command from a script and records what was run; nothing touches adb or WiVRn.

use flat3d_inject::api::{ApiError, LinkState};
use flat3d_inject::link::{
    start_3d, start_3d_commands, CommandOutput, CommandRunner, LinkError, SystemRunner,
};

const DEVICE: &str = "List of devices attached\n1WMHH0A1B2C3D4\tdevice\n\n";

struct Fake {
    answers: Vec<CommandOutput>,
    ran: Vec<Vec<String>>,
}

impl CommandRunner for Fake {
    fn run(&mut self, command: &[String]) -> CommandOutput {
        self.ran.push(command.to_vec());
        self.answers.remove(0)
    }
}

fn ok(stdout: &str) -> CommandOutput {
    CommandOutput {
        success: true,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

fn failed(stderr: &str) -> CommandOutput {
    CommandOutput {
        success: false,
        stdout: String::new(),
        stderr: stderr.into(),
    }
}

fn fake(answers: Vec<CommandOutput>) -> Fake {
    Fake {
        answers,
        ran: Vec::new(),
    }
}

#[test]
fn a_ready_headset_runs_all_four_commands_in_order_and_is_ready() {
    let mut f = fake(vec![ok(DEVICE), ok(""), ok(""), ok("Starting: Intent")]);
    assert_eq!(start_3d(&mut f), Ok(LinkState::Ready));
    assert_eq!(f.ran, start_3d_commands());
}

#[test]
fn no_headset_stops_after_adb_devices() {
    let mut f = fake(vec![ok("List of devices attached\n\n")]);
    assert_eq!(start_3d(&mut f), Ok(LinkState::NoHeadset));
    assert_eq!(f.ran, start_3d_commands()[..1].to_vec());
}

#[test]
fn an_unauthorised_headset_stops_after_adb_devices() {
    let mut f = fake(vec![ok("List of devices attached\nX\tunauthorized\n")]);
    assert_eq!(start_3d(&mut f), Ok(LinkState::Unauthorised));
    assert_eq!(f.ran.len(), 1);
}

#[test]
fn adb_itself_failing_is_an_error_naming_adb_devices() {
    let mut f = fake(vec![failed("adb: not found\n")]);
    let expected = LinkError {
        command: "adb devices".into(),
        message: "adb: not found".into(),
    };
    assert_eq!(start_3d(&mut f), Err(expected));
}

#[test]
fn a_failing_step_stops_there_and_names_the_command_with_trimmed_stderr() {
    let mut f = fake(vec![
        ok(DEVICE),
        ok(""),
        failed("  error: no devices/emulators found\n"),
    ]);
    let expected = LinkError {
        command: "adb reverse tcp:9757 tcp:9757".into(),
        message: "error: no devices/emulators found".into(),
    };
    assert_eq!(start_3d(&mut f), Err(expected));
    assert_eq!(f.ran, start_3d_commands()[..3].to_vec());
}

#[test]
fn the_wivrn_server_failing_stops_before_adb_reverse() {
    let mut f = fake(vec![
        ok(DEVICE),
        failed("Unit flat3d-wivrn.service already exists."),
    ]);
    let err = start_3d(&mut f).unwrap_err();
    assert!(err.command.starts_with("systemd-run --user"), "{err:?}");
    assert_eq!(f.ran.len(), 2);
}

#[test]
fn a_link_error_becomes_a_command_failed_api_error() {
    let e = LinkError {
        command: "adb devices".into(),
        message: "boom".into(),
    };
    assert_eq!(
        ApiError::from(e),
        ApiError::new("command-failed", "adb devices: boom")
    );
}

#[test]
fn the_system_runner_reports_success_and_stdout() {
    let out = SystemRunner.run(&["sh".into(), "-c".into(), "echo hi; echo err >&2".into()]);
    assert_eq!(
        out,
        CommandOutput {
            success: true,
            stdout: "hi\n".into(),
            stderr: "err\n".into()
        }
    );
}

#[test]
fn the_system_runner_reports_a_non_zero_exit_as_failure() {
    assert!(!SystemRunner.run(&["false".into()]).success);
}

#[test]
fn the_system_runner_reports_a_missing_program_as_failure_naming_it() {
    let out = SystemRunner.run(&["flat3d-no-such-program".into()]);
    assert!(!out.success);
    assert!(out.stderr.contains("flat3d-no-such-program"), "{out:?}");
}

#[test]
fn the_system_runner_reports_an_empty_command_as_failure() {
    assert!(!SystemRunner.run(&[]).success);
}
