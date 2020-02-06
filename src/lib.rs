//! Return an error when the `std::process::Command` operation was a success,
//! but the process that ran died.
//!
//! This module helps with a papercut in `std::process::Command` and the
//! related `Child` module. When `Command` is unable to spawn a process, it
//! returns an `io::Error`.  But when a process runs but exits with an error
//! code, or is killed by a signal, `Command` returns successfully, albeit with
//! an `ExitStatus` that indicates the problem.
//!
//! For a simple application, you might want to print a message on an
//! io::Error,
//! `use Stringent` to add a `stringent()` method to `io::Result<ExitStatus>`,
//! `io::Result<Option<ExitStatus>>`, and `io::Result<Output>' to turn those
//! `Results` into `Results` with `CommandError`

#![cfg_attr(debug_assertions, allow(unused))]
#![cfg_attr(
    not(debug_assertions),
    deny(unused, missing_docs, missing_debug_implementations)
)]
#![deny(missing_copy_implementations, missing_debug_implementations)]
#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::error::Error;
use std::fmt;
use std::io;
use std::process::{ExitStatus, Output};
use std::result::Result;

#[derive(Debug)]
pub enum CommandError {
    SpawnFailed(io::Error),
    ExitCode(i32),
    Signal(Option<i32>),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CommandError::*;
        match self {
            SpawnFailed(io) => write!(f, "Spawn failed: {}", io),
            ExitCode(code) => write!(f, "Exit code {}", code),
            Signal(signal) => match signal {
                Some(sig) => write!(f, "Terminated by signal {}", sig),
                None => write!(f, "Terminated"),
            },
        }
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommandError::SpawnFailed(io_err) => Some(io_err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CommandErrorWithOutput {
    err: CommandError,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl fmt::Display for CommandErrorWithOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl Error for CommandErrorWithOutput {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.err.source()
    }
}

pub trait Stringent<T, E> {
    fn stringent(self) -> Result<T, E>;
}

#[cfg(unix)]
fn signal_of(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

// I don't think this can ever get called — it would mean that `code()`
// on Windows returned None, which as far as I know isn't possible.
// But I don't know very far!
#[cfg(not(unix))]
fn signal_of(status: ExitStatus) -> Option<i32> {
    None
}

trait StringentResult
where
    Self: Copy,
{
    fn option_status(self) -> Option<ExitStatus>;
    fn stringent_result(self) -> Result<Self, CommandError> {
        use CommandError::*;
        match self.option_status() {
            None => Ok(self),
            Some(status) if status.success() => Ok(self),
            Some(status) => match status.code() {
                Some(code) => Err(ExitCode(code)),
                None => Err(Signal(signal_of(status))),
            },
        }
    }
}

impl StringentResult for ExitStatus {
    fn option_status(self) -> Option<ExitStatus> {
        Some(self)
    }
}

impl StringentResult for Option<ExitStatus> {
    fn option_status(self) -> Option<ExitStatus> {
        self
    }
}

impl Stringent<ExitStatus, CommandError> for Result<ExitStatus, io::Error> {
    fn stringent(self) -> Result<ExitStatus, CommandError> {
        match self {
            Err(io_err) => Err(CommandError::SpawnFailed(io_err)),
            Ok(status) => status.stringent_result(),
        }
    }
}

impl Stringent<Option<ExitStatus>, CommandError> for Result<Option<ExitStatus>, io::Error> {
    fn stringent(self) -> Result<Option<ExitStatus>, CommandError> {
        match self {
            Err(io_err) => Err(CommandError::SpawnFailed(io_err)),
            Ok(status) => status.stringent_result(),
        }
    }
}

impl Stringent<Output, CommandErrorWithOutput> for Result<Output, io::Error> {
    fn stringent(self) -> Result<Output, CommandErrorWithOutput> {
        match self {
            Err(io_err) => Err(CommandErrorWithOutput {
                err: CommandError::SpawnFailed(io_err),
                stdout: vec![],
                stderr: vec![],
            }),
            Ok(output) => match output.status.stringent_result() {
                Err(err) => Err(CommandErrorWithOutput {
                    err,
                    stdout: output.stdout,
                    stderr: output.stderr,
                }),
                Ok(_) => Ok(output),
            },
        }
    }
}
