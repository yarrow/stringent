#![cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use stringent::{CommandError, Stringent};

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
