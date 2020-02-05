#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, missing_docs))]
#![deny(unused_must_use)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

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
