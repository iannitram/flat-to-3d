//! Test helpers shared by the engine's integration tests.

use crate::exec::{CommandOutput, CommandRunner, RelayRunner};
use std::collections::VecDeque;

struct Reply {
    prefix: Vec<String>,
    output: CommandOutput,
}

/// A recording command runner with optional prefix-matched scripted replies.
pub struct FakeRunner {
    pub ran: Vec<Vec<String>>,
    replies: VecDeque<Reply>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self {
            ran: Vec::new(),
            replies: VecDeque::new(),
        }
    }

    pub fn reply(mut self, prefix: &[&str], out: CommandOutput) -> Self {
        self.replies.push_back(Reply {
            prefix: prefix.iter().map(|part| (*part).to_string()).collect(),
            output: out,
        });
        self
    }

    pub fn ok(self, prefix: &[&str], stdout: &str) -> Self {
        self.reply(
            prefix,
            CommandOutput {
                success: true,
                stdout: stdout.to_string(),
                stderr: String::new(),
            },
        )
    }

    pub fn fail(self, prefix: &[&str], stderr: &str) -> Self {
        self.reply(
            prefix,
            CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: stderr.to_string(),
            },
        )
    }

    fn take(&mut self, argv: &[String]) -> CommandOutput {
        if let Some(index) = self
            .replies
            .iter()
            .position(|reply| argv.starts_with(reply.prefix.as_slice()))
        {
            self.replies
                .remove(index)
                .expect("matched reply exists")
                .output
        } else {
            CommandOutput {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            }
        }
    }
}

impl Default for FakeRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for FakeRunner {
    fn run(&mut self, argv: &[String]) -> CommandOutput {
        self.ran.push(argv.to_vec());
        self.take(argv)
    }
}

impl RelayRunner for FakeRunner {
    fn run(&mut self, argv: &[String]) -> Result<String, String> {
        self.ran.push(argv.to_vec());
        let output = self.take(argv);
        if output.success {
            Ok(output.stdout)
        } else {
            Err(output.stderr)
        }
    }

    fn start_relay(&mut self, argv: &[String], name: &str) -> Result<(), String> {
        let mut words = vec!["relay".to_string()];
        words.extend_from_slice(argv);
        words.push(name.to_string());
        self.ran.push(words);
        Ok(())
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        self.ran.push(vec!["relay".into(), "stop".into()]);
        Ok(())
    }
}
