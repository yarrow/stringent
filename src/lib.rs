#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, missing_docs))]
#![deny(unused_must_use)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::error::Error;
use std::fmt;
use std::io;
use std::process::ExitStatus;
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    const XCPU: i32 = 24;
    const PANIC: i32 = 101;

    /// The only way to construct an arbitrary signal is to use `ExitStatusExt::from_raw`,
    /// and which `i32` values correspond to which values isn't guaranteed by any standard.
    /// So we check our example `ExitStatus` values and return `None` if they're not what
    /// we expect.
    fn successful() -> Option<ExitStatus> {
        let status: ExitStatus = ExitStatusExt::from_raw(0);
        if status.success() {
            Some(status)
        } else {
            None
        }
    }
    fn panicked() -> Option<ExitStatus> {
        let status: ExitStatus = ExitStatusExt::from_raw(PANIC.rotate_left(8));
        if status.success() {
            return None;
        }
        match status.code() {
            Some(PANIC) => return Some(status),
            _ => return None,
        }
    }
    fn killed_by_signal() -> Option<ExitStatus> {
        let status: ExitStatus = ExitStatusExt::from_raw(XCPU);
        if status.success() {
            return None;
        }
        if status.code().is_some() {
            return None;
        }
        if status.signal() == Some(XCPU) {
            return Some(status);
        }
        None
    }

    #[test]
    fn test_success() {
        if let Some(status) = successful() {
            let result = Ok(status).stringent();
            match result {
                Ok(_) => {}
                _ => panic!("success wasn't successful"),
            }
        }
    }

    #[test]
    fn test_panic() {
        if let Some(status) = panicked() {
            let result = Ok(status).stringent();
            match result {
                Err(CommandError::ExitCode(PANIC)) => {}
                _ => panic!("Expected panic status to report a bad exit code"),
            }
        }
    }

    #[test]
    fn test_killed_by_signal() {
        if let Some(status) = killed_by_signal() {
            let result = Ok(status).stringent();
            match result {
                Err(CommandError::Signal(Some(XCPU))) => {}
                _ => panic!("Expected killed_by_signal to report XCPU"),
            }
        }
    }

    #[test]
    fn show_exit_status_examples() {
        println!("OK: {:#?}", successful());
        println!("24: {:#?}", killed_by_signal());
        println!(
            "101={}? {:#?}",
            panicked().unwrap().code().unwrap(),
            panicked()
        );
    }
}
