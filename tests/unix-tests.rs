#![deny(warnings, unused, clippy::all, clippy::pedantic)]

#![cfg(unix)]
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

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
        let result = Ok(status).verify();
        match result {
            Ok(_) => {}
            _ => panic!("success wasn't successful"),
        }
    }
}

#[test]
fn test_panic() {
    if let Some(status) = panicked() {
        let result = Ok(status).verify();
        match result {
            Err(CommandError::ExitCode(PANIC)) => {}
            _ => panic!("Expected panic status to report a bad exit code"),
        }
    }
}

#[test]
fn test_killed_by_signal() {
    if let Some(status) = killed_by_signal() {
        let result = Ok(status).verify();
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

use std::process::{Command, Stdio};
use std::path::Path;

#[test] fn nonexistent_command() {
    let cmd = "/nonexistent_command";
    if ! Path::new(cmd).is_file() {
        match Command::new(cmd).status().verify() {
            Ok(_) => panic!("{} should not have succeeded", cmd),
            Err(CommandError::SpawnFailed(_)) => {},
            Err(e) => panic!("Unexpected error ({}) in executing {}", e, cmd),
        }
    }
}

#[test] fn bad_arguments() {
    let cmd = "/bin/sleep";
    if Path::new(cmd).is_file() {
        match Command::new(cmd).stderr(Stdio::null()).status().verify() {
            Ok(_) => panic!("{} should not have succeeded", cmd),
            Err(CommandError::ExitCode(_)) => {}, // Sleep with no arguments should complain
            Err(e) => panic!("Unexpected error ({}) in executing {}", e, cmd),
        }
    }
}

#[test] fn bad_arguments_with_output() {
    let cmd = "/bin/sleep";
    if Path::new(cmd).is_file() {
        match Command::new(cmd).output().verify() {
            Ok(_) => panic!("{} should not have succeeded", cmd),
            Err(output) => match output.err {
                CommandError::ExitCode(_) => {
                    if output.stderr.len() == 0 {
                        panic!("Expected to capture {}'s stderr", cmd);
                    }
                },
                _ => panic!("Unexpected error ({}) in executing {}", output.err, cmd),
            }
        }
    }
}

#[test] fn killed() {
    let cmd = "/bin/sleep";
    if Path::new(cmd).is_file() {
        let mut child = Command::new(cmd).arg("3").spawn().verify().expect(cmd);
        let result = match child.try_wait().verify() {
            Err(e) => panic!("Unexpected error {}", e),
            Ok(Some(_)) => panic!("Command terminated too quickly"),
            Ok(None) => {
                child.kill().expect("couldn't kill child process");
                child.wait().verify()
            }
        };
        match result {
            Ok(_) => panic!("Killed command {} should not have succeeded", cmd),
            Err(CommandError::Signal(_)) => {},
            Err(e) => panic!("Unexpected error ({}) in executing {}", e, cmd),
        }
    }
}

#[test] fn killed_with_output() {
    let cmd = "/bin/sleep";
    if Path::new(cmd).is_file() {
        let mut child = Command::new(cmd).arg("3").spawn().verify().expect(cmd);
        child.kill().expect("couldn't kill child process");
        match child.wait_with_output().verify() {
            Ok(_) => panic!("Killed command {} should not have succeeded", cmd),
            Err(output) => match output.err {
                CommandError::Signal(_) => { },
                _ => panic!("Unexpected error ({}) in executing {}", output.err, cmd),
            }
        };
    }
}
