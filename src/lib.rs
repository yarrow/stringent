//! This library provides the [`verify()`][verify] method, a convenient way to
//! return an error when a [`std::process::Command`][Command] process starts
//! successfully but exits abnormally.
//!
//! [Command]: https://doc.rust-lang.org/std/process/struct.Command.html
//! [status]: https://doc.rust-lang.org/std/process/struct.Command.html#method.status
//! [spawn]: https://doc.rust-lang.org/std/process/struct.Command.html#method.spawn
//! [output]: https://doc.rust-lang.org/std/process/struct.Command.html#method.output
//! [Child]: https://doc.rust-lang.org/std/process/struct.Child.html
//! [wait]: https://doc.rust-lang.org/std/process/struct.Child.html#method.wait
//! [try_wait]: https://doc.rust-lang.org/std/process/struct.Child.html#method.try_wait
//! [wait_with_output]: https://doc.rust-lang.org/std/process/struct.Child.html#method.wait_with_output
//! [ioError]: https://doc.rust-lang.org/std/io/type.Error.html
//! [ioResult]: https://doc.rust-lang.org/std/io/type.Result.html
//! [ExitStatus]: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
//! [Output]: https://doc.rust-lang.org/std/process/struct.Output.html
//! [verify]: trait.Verify.html#tymethod.verify
//! [CommandStatusError]: enum.CommandStatusError.html
//! [CommandError]: struct.CommandError.html
//!
//! # Examples
//!
//! In the simplest case we want to run a command without capturing either `stdout` or
//! `stderr`.
//!
//! ```no_run
//! use std::process::Command;
//! use stringent::{CommandStatusError, Verify};
//!
//! fn run_command() -> Result<(), CommandStatusError> {
//!     Command::new("cmd").status().verify()?;
//!     Ok(())
//! }
//! ```
//!
//! If we do capture `stdout` and `stderr`, then the error path as well as the happy path may
//! want to process the captured output.
//!
//! ```no_run
//! use std::process::Command;
//! use stringent::{CommandError, Std, Verify};
//!
//! fn run_commands() -> Result<(), CommandError> {
//!     let commands = vec!["cmd1", "cmd2", "cmd3"];
//!     for cmd in commands {
//!         match Command::new(cmd).output().verify() {
//!             Ok(output) => process(&output),
//!             Err(err) => {
//!                 match err.output {
//!                     Some(ref std) => log(&std.stderr),
//!                     _ => {}
//!                 }
//!                 return Err(err);
//!             }
//!         }
//!     }
//!     Ok(())
//! }
//!
//! # fn process(output: &std::process::Output) { }
//! # fn log(stderr: &Vec<u8>) { }
//!
//! ```
//!
//! ```no_run
//! use std::process::Command;
//! use stringent::{CommandError, Verify};
//!
//! fn run_commands() -> Result<(), CommandError> {
//!     Command::new("immediate").status().verify()?;
//!     let mut child = Command::new("run_in_parallel").spawn().verify()?;
//!     long_running_computation();
//!     let output = child.wait_with_output().verify()?;
//!     process(&output.stdout);
//!     Ok(())
//! }
//! # fn process(stdout: &Vec<u8>) { }
//! # fn long_running_computation() { }
//! ```
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
//! This crate adds a [`verify()`][verify] method to the `Result`s returned by
//! [`Commands`][Command]'s [`status()`][status], [`spawn()`][spawn] and [`output()`][output]
//! methods, and to [`Child`][Child]'s [`wait()`][wait], [`try_wait`][try_wait], and
//! [`wait_with_output`][wait_with_output] methods. The [`verify()`][verify] method turns
//! unsuccessful [`ExitStatus`][ExitStatus] values into errors, so the following will return
//! [`CommandStatusError`][CommandStatusError]s for commands that don't successfully complete:
//! * `cmd.status().verify()?`
//! * `child.wait().verify()?`
//! * `child.try_wait().verify()?`
//!
//! [`verify()`][verify] similarly turns an unsuccessful [`Output`][Output] into a
//! [`CommandError`][CommandError].  The `stdout` and `stderr` fields of the
//! [`Output`][Output] are saved in the corresponding fields of the
//! [`CommandError`][CommandError].
//! * `cmd.output().verify()?`
//! * `child.wait_with_output().verify()?`
//!
//! Without [`verify()`][verify] (but with [`CommandStatusError`][CommandStatusError]), we'd need to
//! write something like this:
//!
//! ```no_run
//! # use std::process::Command;
//! # use stringent::CommandStatusError;
//!
//! fn run_commands(first: &mut Command, second: &mut Command) -> Result<(), CommandStatusError> {
//!     let mut status = match first.status() {
//!         Ok(status) => status,
//!         Err(io_err) => return Err(CommandStatusError::SpawnFailed(io_err)),
//!     };
//!     if status.success() {
//!         status = match second.status() {
//!             Ok(status) => status,
//!             Err(io_err) => return Err(CommandStatusError::SpawnFailed(io_err)),
//!         }
//!     }
//!     if status.success() { return Ok(()) }
//!     match status.code() {
//!         Some(code) => Err(CommandStatusError::ExitCode(code)),
//!         None => {
//!             #[cfg(unix)]
//!                 use std::os::unix::process::ExitStatusExt;
//!                 let signal = status.signal();
//!             #[cfg(not(unix))]
//!                 let signal = None;
//!             Err(CommandStatusError::Signal(signal))
//!         }
//!     }
//! }
//! ```
//! The [`verify()`][verify] method also handles the result of the [`spawn`][spawn] method,
//! for convenience when writing functions that return a `Result<T, CommandStatusError>`:
//!
//! ```no_run
//! use std::process::Command;
//! use stringent::{CommandStatusError, Verify};
//!
//! fn run_commands() -> Result<(), CommandStatusError> {
//!     let mut child = Command::new("mycommand").spawn().verify()?;
//!     // do more things...
//!     child.wait().verify()?;
//!     Ok(())
//! }
//! ```

#![deny(warnings, unused, clippy::all, clippy::pedantic)]
#![deny(missing_copy_implementations, missing_debug_implementations)]
#![deny(missing_docs, missing_doc_code_examples)]
#![allow(clippy::missing_errors_doc)]

use std::error::Error;
use std::fmt;
use std::io;
use std::process::{Child, ExitStatus, Output};
use std::result::Result;

/// Adds error cases for commands that exit with error codes or that are killed
#[derive(Debug)]
pub enum CommandStatusError {
    /// Holds the [`io::Error`][ioError] when
    /// [`status()`](https://doc.rust-lang.org/std/process/struct.Command.html#method.status)
    /// or
    /// [`wait()`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait)
    /// return an error
    SpawnFailed(io::Error),
    /// Holds the exit code when a command terminates with an error
    ExitCode(i32),
    /// Holds the signal number when the command is killed by a signal (only on unix)
    Signal(Option<i32>),
}

impl fmt::Display for CommandStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CommandStatusError::*;
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

impl Error for CommandStatusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommandStatusError::SpawnFailed(io_err) => Some(io_err),
            _ => None,
        }
    }
}

/// Holds the captured `stdout` and `stderr` from a [`std::process::Output`][Output] value.
#[derive(Debug)]
pub struct Std {
    /// Saved `stdout` from the command.
    pub stdout: Vec<u8>,
    /// Saved `stderr` from the command.
    pub stderr: Vec<u8>,
}

/// Adds an `output` field `Option<Std>` field for saved `stdout` and `stderr` fields for saved
/// output to the [`CommandStatusError`](struct.CommandStatusError.html) `err` field. The `output`
/// field is `None` when
///
/// * `err` is `SpawnFailed`, or
/// * This `CommandError` was create from a `CommandStatusError` (i.e., from the result of
/// `status().verify()`, `wait().verify()`, or `try_wait().verify()`.
///
#[derive(Debug)]
pub struct CommandError {
    /// Match on the `err` field to distinguish between `SpawnFailed`, `ExitCode`, and `Signal`.
    pub err: CommandStatusError,
    /// Saved `stdout` from the command.
    pub output: Option<Std>,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.err.source()
    }
}

/// Adds the `verify()` method to the `io::Result` values returned by `Command::status`,
/// `Command::output`, `Child::wait`, `Child::try_wait`, and `Child::wait_with_output`.
pub trait Verify<T, E> {
    /// Changes a `Result<T, io::Error>` value to `Result<T, CmdErr>` value, where
    /// `CmdErr` is `CommandStatusError` when `T` is `ExitStatus` or `Option<ExitStatus>`,
    /// and `CommandError` when `T` is `Output`.
    ///
    fn verify(self) -> Result<T, E>;
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
    fn stringent_result(self) -> Result<Self, CommandStatusError> {
        use CommandStatusError::*;
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

impl Verify<ExitStatus, CommandStatusError> for Result<ExitStatus, io::Error> {
    fn verify(self) -> Result<ExitStatus, CommandStatusError> {
        match self {
            Err(io_err) => Err(CommandStatusError::SpawnFailed(io_err)),
            Ok(status) => status.stringent_result(),
        }
    }
}

impl Verify<Option<ExitStatus>, CommandStatusError> for Result<Option<ExitStatus>, io::Error> {
    fn verify(self) -> Result<Option<ExitStatus>, CommandStatusError> {
        match self {
            Err(io_err) => Err(CommandStatusError::SpawnFailed(io_err)),
            Ok(status) => status.stringent_result(),
        }
    }
}

impl Verify<Child, CommandStatusError> for Result<Child, io::Error> {
    fn verify(self) -> Result<Child, CommandStatusError> {
        match self {
            Err(io_err) => Err(CommandStatusError::SpawnFailed(io_err)),
            Ok(child) => Ok(child),
        }
    }
}

impl Verify<Output, CommandError> for Result<Output, io::Error> {
    fn verify(self) -> Result<Output, CommandError> {
        match self {
            Err(io_err) => Err(CommandError {
                err: CommandStatusError::SpawnFailed(io_err),
                output: None,
            }),
            Ok(output) => match output.status.stringent_result() {
                Err(err) => Err(CommandError {
                    err,
                    output: Some(Std {
                        stdout: output.stdout,
                        stderr: output.stderr,
                    }),
                }),
                Ok(_) => Ok(output),
            },
        }
    }
}

impl From<CommandStatusError> for CommandError {
    fn from(status_error: CommandStatusError) -> Self {
        Self {
            err: status_error,
            output: None,
        }
    }
}
