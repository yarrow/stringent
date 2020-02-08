//! Return an error when a [`std::process::Command`][Command] method
//! successfully starts a process, but the process itself isn't successful.
//!
//! [Command]: https://doc.rust-lang.org/std/process/struct.Command.html
//! [status]: https://doc.rust-lang.org/std/process/struct.Command.html#method.status
//! [output]: https://doc.rust-lang.org/std/process/struct.Command.html#method.output
//! [Child]: https://doc.rust-lang.org/std/process/struct.Child.html
//! [wait]: https://doc.rust-lang.org/std/process/struct.Child.html#method.wait
//! [try_wait]: https://doc.rust-lang.org/std/process/struct.Child.html#method.try_wait
//! [wait_with_output]: https://doc.rust-lang.org/std/process/struct.Child.html#method.wait_with_output
//! [ioError]: https://doc.rust-lang.org/std/io/type.Error.html
//! [ioResult]: https://doc.rust-lang.org/std/io/type.Result.html
//! [ExitStatus]: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
//! [Output]: https://doc.rust-lang.org/std/process/struct.Output.html
//! [stringent]: stringent/trait.Stringent.html#tymethod.stringent
//! [CommandError]: stringent/enum.CommandError.html
//! [CommandErrorWithOutput]: stringent/enum.CommandErrorWithOutput.html
//!
//! The standard library's [`Command`][Command] module's [`io::Result`][ioResult]
//! values for command completion can sometimes feel like "the operation was a
//! success but the patient died". For instance, the [`status()`][status]
//! method (which runs a [`Command`][Command] in a new process) returns either
//! `Err(`[`io::Error`][ioError]`)` (when the process creation fails), or
//! `Ok(`[`ExitStatus`][ExitStatus]`)`; but then we need to check the returned
//! status to see if the command actually succeeded:
//!
//! ```no_run
//! # use std::process::Command;
//! # let mut cmd = Command::new("x");
//! let success = match cmd.status() {
//!     Err(_) => false,
//!     Ok(status) => status.success(),
//! };
//! ```
//! In particular, we can't use `cmd.status()?` to pass errors back to our
//! caller, because that will ignore commands that exit with an error code or
//! that are killed by a signal.
//!
//! This crate adds a [`stringent()`][stringent] method to the `Result`s returned by
//! [`Commands`][Command]'s [`status()`][status] and [`output()`][output] methods, and to
//! [`Child`][Child]'s [`wait()`][wait], [`try_wait`][try_wait], and
//! [`wait_with_output`][wait_with_output] methods. The [`stringent()`][stringent] method turns
//! unsuccessful [`ExitStatus`][ExitStatus] values into errors, so the following will return
//! [`CommandError`][CommandError]s for commands that don't successfully complete:
//! * `cmd.status().stringent()?`
//! * `child.wait().stringent()?`
//! * `child.try_wait().stringent()?`
//!
//! Similarly, [`stringent()`][stringent] turns an unsuccessful [`Output`][Output] into a
//! [`CommandErrorWithOutput`][CommandErrorWithOutput].  The `stdout` and `stderr` fields of the
//! [`Output`][Output] are saved in the corresponding fields of the
//! [`CommandErrorWithOutput`][CommandErrorWithOutput].
//! * `cmd.output().stringent()?`
//! * `child.wait_with_output().stringent()?`
//!
//!
//! # Example
//!
//! ```no_run
//! use std::process::Command;
//! use stringent::{CommandError, Stringent};
//!
//! fn run_commands(first: &mut Command, second: &mut Command) -> Result<(), CommandError> {
//!     first.status().stringent()?;
//!     second.status().stringent()?;
//!     Ok(())
//! }
//! ```
//!
//! Without [`stringent()`][stringent] (but with [`CommandError`][CommandError]), we'd need to
//! write something like this:
//!
//! ```no_run
//! # use std::process::Command;
//! # use stringent::CommandError;
//!
//! fn run_commands(first: &mut Command, second: &mut Command) -> Result<(), CommandError> {
//!     let mut status = first.status()?;
//!     if status.success() {
//!         status = second.status()?;
//!     }
//!     if status.success() { return Ok(()) }
//!     match status.code() {
//!         Some(code) => Err(CommandError::ExitCode(code)),
//!         None => {
//!             #[cfg(unix)]
//!                 use std::os::unix::process::ExitStatusExt;
//!                 let signal = status.signal();
//!             #[cfg(not(unix))]
//!                 let signal = None;
//!             Err(CommandError::Signal(signal))
//!         }
//!     }
//! }
//! ```

#![cfg_attr(debug_assertions, allow(unused))]
#![cfg_attr(
    not(debug_assertions),
    deny(unused, missing_docs, missing_debug_implementations)
)]
#![deny(missing_copy_implementations, missing_debug_implementations)]
#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
use std::error::Error;
use std::fmt;
use std::io;
use std::process::{ExitStatus, Output};
use std::result::Result;

/// Adds error cases for commands that exit with error codes or that are killed
#[derive(Debug)]
pub enum CommandError {
    /// Holds the [`io::Error`][ioError] when
    /// [`status()`](https://doc.rust-lang.org/std/process/struct.Command.html#method.status)
    /// or
    /// [`wait()`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait)
    /// return an error
    SpawnFailed(io::Error),
    /// Holds the exit code when a command terminates with an error
    ExitCode(i32),
    /// Holds the signal number when a (unix) process is killed by a signal
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

impl From<io::Error> for CommandError {
    fn from(io: io::Error) -> CommandError {
        CommandError::SpawnFailed(io)
    }
}

/// Includes `stdout` and `stderr` fields for saved output as well as a [`CommandError`](struct.CommandErrorWithOutput.html) field.
///
/// Note that the `stdout` and `stderr` fields are always present, even when `err` is
/// `SpawnFailed`. In that case they are zero-length and no allocation was done.
#[derive(Debug)]
pub struct CommandErrorWithOutput {
    /// Match on the `err` field to distinguish between `SpawnFailed`, `ExitCode`, and `Signal`.
    pub err: CommandError,
    /// Saved `stdout` from the command.
    pub stdout: Vec<u8>,
    /// Saved `stderr` from the command.
    pub stderr: Vec<u8>,
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

impl From<io::Error> for CommandErrorWithOutput {
    fn from(io: io::Error) -> CommandErrorWithOutput {
        CommandErrorWithOutput {
            err: CommandError::SpawnFailed(io),
            stdout: Vec::with_capacity(0),
            stderr: Vec::with_capacity(0),
        }
    }
}

/// Adds the `stringent()` method to the `io::Result` values returned by `Command::status`,
/// `Command::output`, `Child::wait`, `Child::try_wait`, and `Child::wait_with_output`.
pub trait Stringent<T, E> {
    /// Changes a `Result<T, io::Error>` value to `Result<T, CmdErr>` value, where
    /// `CmdErr` is `CommandError` when `T` is `ExitStatus` or `Option<ExitStatus>`,
    /// and `CommandErrorWithOutput` when `T` is `Output`.
    ///
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
            Err(io_err) => Err(CommandError::from(io_err)),
            Ok(status) => status.stringent_result(),
        }
    }
}

impl Stringent<Option<ExitStatus>, CommandError> for Result<Option<ExitStatus>, io::Error> {
    fn stringent(self) -> Result<Option<ExitStatus>, CommandError> {
        match self {
            Err(io_err) => Err(CommandError::from(io_err)),
            Ok(status) => status.stringent_result(),
        }
    }
}

impl Stringent<Output, CommandErrorWithOutput> for Result<Output, io::Error> {
    fn stringent(self) -> Result<Output, CommandErrorWithOutput> {
        match self {
            Err(io_err) => Err(CommandErrorWithOutput::from(io_err)),
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
