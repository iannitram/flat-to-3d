//! Locked tests for E2c: `link::stop_3d` over E2b's `CommandRunner`. A fake runner answers
//! each command from a script and records what was run; nothing touches adb or WiVRn.

use flat3d_inject::api::LinkState;
use flat3d_inject::link::{
    start_3d, start_3d_commands, stop_3d, stop_3d_commands, CommandOutput, CommandRunner, LinkError,
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

fn ok() -> CommandOutput {
    CommandOutput {
        success: true,
        stdout: String::new(),
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
fn stop_runs_all_three_commands_in_order_and_is_stopped() {
    let mut f = fake(vec![ok(), ok(), ok()]);
    assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
    assert_eq!(f.ran, stop_3d_commands());
}

#[test]
fn an_unplugged_headset_still_stops_the_server() {
    let no_dev = "error: no devices/emulators found";
    let mut f = fake(vec![failed(no_dev), failed(no_dev), ok()]);
    assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
    assert_eq!(f.ran, stop_3d_commands());
}

#[test]
fn the_server_stop_failing_is_an_error_naming_it() {
    let mut f = fake(vec![
        ok(),
        ok(),
        failed("Failed to stop flat3d-wivrn.service: Access denied\n"),
    ]);
    let expected = LinkError {
        command: "systemctl --user stop flat3d-wivrn.service".into(),
        message: "Failed to stop flat3d-wivrn.service: Access denied".into(),
    };
    assert_eq!(stop_3d(&mut f), Err(expected));
    assert_eq!(f.ran.len(), 3);
}

#[test]
fn start_stop_start_reports_ready_stopped_ready() {
    let mut f = fake(vec![
        ok_with(DEVICE),
        ok(),
        ok(),
        ok(),
        ok(),
        ok(),
        ok(),
        ok_with(DEVICE),
        ok(),
        ok(),
        ok(),
    ]);
    assert_eq!(start_3d(&mut f), Ok(LinkState::Ready));
    assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
    assert_eq!(start_3d(&mut f), Ok(LinkState::Ready));
    let mut expected = start_3d_commands();
    expected.extend(stop_3d_commands());
    expected.extend(start_3d_commands());
    assert_eq!(f.ran, expected);
}

fn ok_with(stdout: &str) -> CommandOutput {
    CommandOutput {
        success: true,
        stdout: stdout.into(),
        stderr: String::new(),
    }
}

#[test]
fn the_code_map_says_link_rs_is_present() {
    let map = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/code-map.md");
    let text = std::fs::read_to_string(map).unwrap();
    assert!(text.contains("| `crates/engine/src/link.rs` | present |"));
}
