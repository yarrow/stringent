#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, missing_docs))]
#![deny(unused_must_use)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::process::ExitStatus;
use std::result::Result;
use std::error::Error;
use std::fmt;
use std::io;

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
            SpawnFailed(_) => write!(f, "Spawn failed"),
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

pub trait Stringent<T, E> {
    fn stringent(self) -> Result<T, E>;
}

#[cfg(unix)]
fn signal_of(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

// This handles the case where `code()==None` and `signal()==None`,
// which as far as I know isn't possible.  But I don't know very far!
#[cfg(not(unix))]
fn signal_of(status: &ExitStatus) -> Option<i32> {
    None
}

impl Stringent<ExitStatus, CommandError> for Result<ExitStatus, io::Error> {
    fn stringent(self) -> Result<ExitStatus, CommandError> {
        use CommandError::*;
        match self {
            Err(io_err) => Err(SpawnFailed(io_err)),
            Ok(status) => {
                if status.success() {
                    Ok(status)
                } else {
                    match status.code() {
                        Some(code) => Err(ExitCode(code)),
                        None => Err(Signal(signal_of(&status))),
                    }
                }
            }
        }
    }
}
