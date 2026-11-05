//! Additional review coverage for Stop 3D's error precedence.

use flat3d_inject::api::{ApiError, LinkState};
use flat3d_inject::link::{stop_3d, stop_3d_commands, CommandOutput, CommandRunner, LinkError};

struct Fake {
    answers: std::vec::IntoIter<CommandOutput>,
    ran: Vec<Vec<String>>,
}

impl CommandRunner for Fake {
    fn run(&mut self, command: &[String]) -> CommandOutput {
        self.ran.push(command.to_vec());
        self.answers.next().expect("unexpected extra command")
    }
}

#[test]
fn only_the_server_result_matters_for_every_combination_of_stop_failures() {
    for app_ok in [false, true] {
        for reverse_ok in [false, true] {
            for server_ok in [false, true] {
                let answers = [app_ok, reverse_ok, server_ok]
                    .into_iter()
                    .zip(["app error", "reverse error", " \tserver error\r\n "])
                    .map(|(success, stderr)| CommandOutput {
                        success,
                        stdout: "stdout must not become the error message".into(),
                        stderr: stderr.into(),
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
                let mut runner = Fake {
                    answers,
                    ran: Vec::new(),
                };
                let result = stop_3d(&mut runner);
                assert_eq!(runner.ran, stop_3d_commands());
                assert!(runner.answers.next().is_none());
                if server_ok {
                    assert_eq!(result, Ok(LinkState::Stopped));
                } else {
                    let expected = LinkError {
                        command: "systemctl --user stop flat3d-wivrn.service".into(),
                        message: "server error".into(),
                    };
                    assert_eq!(result, Err(expected.clone()));
                    assert_eq!(
                        ApiError::from(result.unwrap_err()),
                        ApiError::new(
                            "command-failed",
                            "systemctl --user stop flat3d-wivrn.service: server error",
                        )
                    );
                }
            }
        }
    }
}
