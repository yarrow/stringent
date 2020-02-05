#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, missing_docs))]
#![deny(unused_must_use)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::io;

#[derive(Debug)]
pub enum CommandError {
    SpawnFailed(io::Error),
    ExitCode(i32),
    Signal(Option<i32>),
}
