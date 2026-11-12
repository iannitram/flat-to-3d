//! Locked tests for E6b: AdbRunner rewrites `adb ...` commands.

use flat3d_inject::link::adb::AdbRunner;
use flat3d_inject::link::{CommandOutput, CommandRunner};

struct Fake {
    ran: Vec<Vec<String>>,
}
impl CommandRunner for Fake {
    fn run(&mut self, command: &[String]) -> CommandOutput {
        self.ran.push(command.to_vec());
        CommandOutput {
            success: true,
            stdout: "out".into(),
            stderr: String::new(),
        }
    }
}

fn c(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}
fn runner(serial: Option<&str>) -> AdbRunner<Fake> {
    AdbRunner::new(Fake { ran: vec![] }, "/h/bin/adb", serial)
}

#[test]
fn the_adb_word_is_swapped_and_the_serial_added() {
    let mut r = runner(Some("S1"));
    r.run(&c(&["adb", "reverse", "tcp:1", "tcp:2"]));
    assert_eq!(
        r.inner().ran,
        vec![c(&["/h/bin/adb", "-s", "S1", "reverse", "tcp:1", "tcp:2"])]
    );
}

#[test]
fn without_a_serial_there_is_no_dash_s() {
    let mut r = runner(None);
    r.run(&c(&["adb", "shell", "ls"]));
    assert_eq!(r.inner().ran, vec![c(&["/h/bin/adb", "shell", "ls"])]);
}

#[test]
fn server_commands_never_get_a_serial() {
    let mut r = runner(Some("S1"));
    r.run(&c(&["adb", "devices"]));
    r.run(&c(&["adb", "kill-server"]));
    r.run(&c(&["adb", "start-server"]));
    assert_eq!(
        r.inner().ran,
        vec![
            c(&["/h/bin/adb", "devices"]),
            c(&["/h/bin/adb", "kill-server"]),
            c(&["/h/bin/adb", "start-server"])
        ]
    );
}

#[test]
fn a_command_that_already_has_a_serial_is_not_doubled() {
    let mut r = runner(Some("S1"));
    r.run(&c(&["adb", "-s", "OTHER", "shell", "ls"]));
    assert_eq!(
        r.inner().ran,
        vec![c(&["/h/bin/adb", "-s", "OTHER", "shell", "ls"])]
    );
}

#[test]
fn other_commands_pass_through_unchanged() {
    let mut r = runner(Some("S1"));
    r.run(&c(&["systemctl", "--user", "stop", "flat3d-wivrn.service"]));
    r.run(&[]);
    assert_eq!(
        r.inner().ran,
        vec![
            c(&["systemctl", "--user", "stop", "flat3d-wivrn.service"]),
            vec![]
        ]
    );
}

#[test]
fn the_inner_result_is_returned() {
    let mut r = runner(None);
    let out = r.run(&c(&["adb", "devices"]));
    assert!(out.success);
    assert_eq!(out.stdout, "out");
}
