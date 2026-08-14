//! Running shell commands on behalf of scripts, behind a trait so script
//! logic is unit tested against a fake instead of real processes.

use std::process::{Command, Output};
use thiserror::Error;

/// What a script asked to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellInvocation {
    /// A command line interpreted by `/bin/sh -c`.
    Command(String),
    /// A program run directly with an argument vector, no shell involved.
    Program { program: String, args: Vec<String> },
}

/// Something that went wrong running a shell command.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ShellError {
    /// The process could not be started at all.
    #[error("could not run command: {0}")]
    Spawn(String),
    /// The process ran and exited with a failure status.
    #[error("command failed with status {status}: {stderr_first_line}")]
    Failed {
        status: i32,
        stderr_first_line: String,
    },
    /// The process output was not valid UTF-8.
    #[error("command output was not valid UTF-8")]
    NonUtf8Output,
}

/// Runs shell commands for scripts. `Send + Sync` because scripts are
/// evaluated on background threads.
pub trait ShellRunner: Send + Sync + 'static {
    /// Runs `invocation` and returns its standard output.
    fn run(&self, invocation: &ShellInvocation) -> Result<String, ShellError>;
}

/// The real system shell.
pub struct SystemShell;

impl ShellRunner for SystemShell {
    fn run(&self, invocation: &ShellInvocation) -> Result<String, ShellError> {
        let output = command_for(invocation)
            .output()
            .map_err(|error| ShellError::Spawn(error.to_string()))?;
        stdout_of(output)
    }
}

fn command_for(invocation: &ShellInvocation) -> Command {
    match invocation {
        ShellInvocation::Command(line) => {
            let mut command = Command::new("/bin/sh");
            command.arg("-c").arg(line);
            command
        }
        ShellInvocation::Program { program, args } => {
            let mut command = Command::new(program);
            command.args(args);
            command
        }
    }
}

fn stdout_of(output: Output) -> Result<String, ShellError> {
    if !output.status.success() {
        return Err(ShellError::Failed {
            status: output.status.code().unwrap_or(-1),
            stderr_first_line: first_line(&String::from_utf8_lossy(&output.stderr)),
        });
    }
    String::from_utf8(output.stdout).map_err(|_| ShellError::NonUtf8Output)
}

/// The first non-empty line of `message`, or an empty string.
fn first_line(message: &str) -> String {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// A shell that returns canned output and records every invocation.
#[cfg(test)]
pub mod fake {
    use super::{ShellError, ShellInvocation, ShellRunner};
    use std::sync::Mutex;

    /// A [`ShellRunner`] for tests: canned output, recorded invocations.
    pub struct FakeShell {
        response: Result<String, ShellError>,
        invocations: Mutex<Vec<ShellInvocation>>,
    }

    impl FakeShell {
        /// A shell whose every command succeeds with `stdout`.
        pub fn outputting(stdout: &str) -> Self {
            Self {
                response: Ok(stdout.to_string()),
                invocations: Mutex::new(Vec::new()),
            }
        }

        /// A shell whose every command fails.
        pub fn failing() -> Self {
            Self {
                response: Err(ShellError::Failed {
                    status: 1,
                    stderr_first_line: "fake failure".to_string(),
                }),
                invocations: Mutex::new(Vec::new()),
            }
        }

        /// Every invocation run so far, in order.
        pub fn invocations(&self) -> Vec<ShellInvocation> {
            self.invocations.lock().expect("lock should work").clone()
        }
    }

    impl ShellRunner for FakeShell {
        fn run(&self, invocation: &ShellInvocation) -> Result<String, ShellError> {
            self.invocations
                .lock()
                .expect("lock should work")
                .push(invocation.clone());
            self.response.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_form_runs_through_the_shell() {
        let output = SystemShell
            .run(&ShellInvocation::Command("echo hello".to_string()))
            .expect("echo should succeed");

        assert_eq!(output, "hello\n");
    }

    #[test]
    fn program_form_runs_without_shell_interpretation() {
        let output = SystemShell
            .run(&ShellInvocation::Program {
                program: "echo".to_string(),
                args: vec!["$HOME".to_string()],
            })
            .expect("echo should succeed");

        assert_eq!(output, "$HOME\n");
    }

    #[test]
    fn a_failing_command_reports_its_status_and_stderr() {
        let error = SystemShell
            .run(&ShellInvocation::Command(
                "echo oops >&2; exit 3".to_string(),
            ))
            .expect_err("should fail");

        assert_eq!(
            error,
            ShellError::Failed {
                status: 3,
                stderr_first_line: "oops".to_string(),
            }
        );
    }

    #[test]
    fn a_missing_program_reports_a_spawn_error() {
        let error = SystemShell
            .run(&ShellInvocation::Program {
                program: "definitely-not-a-real-program".to_string(),
                args: Vec::new(),
            })
            .expect_err("should fail");

        assert!(matches!(error, ShellError::Spawn(_)));
    }
}
