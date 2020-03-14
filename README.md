This library provides the [`verify()`][verify] method, a convenient way to
return an error when a [`std::process::Command`][Command] process starts
successfully but exits abnormally.

```rust
let status = Command(`badexit`).status;
assert!(status
```
given [`Command`][Command]('`cmd.status()` returns an [`ExitStatus`][ExitStatus] if `cmd` starts successfully but is killed by a signal or stops with an error code. But `cmd.status().verify()` turns those [`ExitStatus`][ExitStatus] values into errors.

You can also use [`verify()`][verify] in:

* `cmd.output().verify()`
* `child.wait().verify()`
* `child.try_wait().verify()`
* `child.wait_with_output().verify()`

[Command]: https://doc.rust-lang.org/std/process/struct.Command.html
[status]: https://doc.rust-lang.org/std/process/struct.Command.html#method.status
[spawn]: https://doc.rust-lang.org/std/process/struct.Command.html#method.spawn
[output]: https://doc.rust-lang.org/std/process/struct.Command.html#method.output
[Child]: https://doc.rust-lang.org/std/process/struct.Child.html
[wait]: https://doc.rust-lang.org/std/process/struct.Child.html#method.wait
[try_wait]: https://doc.rust-lang.org/std/process/struct.Child.html#method.try_wait
[wait_with_output]: https://doc.rust-lang.org/std/process/struct.Child.html#method.wait_with_output
[ioError]: https://doc.rust-lang.org/std/io/type.Error.html
[ioResult]: https://doc.rust-lang.org/std/io/type.Result.html
[ExitStatus]: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
[Output]: https://doc.rust-lang.org/std/process/struct.Output.html
[verify]: trait.Verify.html#tymethod.verify
[CommandStatusError]: enum.CommandStatusError.html
[CommandError]: struct.CommandError.html

# Example

```rust
use std::process::Command;
use stringent::{CommandError, Verify};

fn run_commands() -> Result<(), CommandError> {
    Command::new("immediate").status().verify()?;
    let mut child = Command::new("run_in_parallel").spawn().verify()?;
    long_running_computation();
    child.wait().verify()?;
    //let output = child.wait_with_output().verify()?;
    //process(&output.stdout);
    Ok(())
}
# fn long_running_computation() { }
```

The standard library's [`Command`][Command] module's [`io::Result`][ioResult]
values for command completion can sometimes feel like "the operation was a
success but the patient died". For instance, the [`status()`][status]
method (which runs a [`Command`][Command] in a new process) returns either
`Err(`[`io::Error`][ioError]`)` (when the process creation fails), or
`Ok(`[`ExitStatus`][ExitStatus]`)`; but then we need to check the returned
status to see if the command actually succeeded:

```no_run
# use std::process::Command;
# let mut cmd = Command::new("x");
let success = match cmd.status() {
    Err(_) => false,
    Ok(status) => status.success(),
};
```
In particular, we can't use `cmd.status()?` to pass errors back to our
caller, because that will ignore commands that exit with an error code or
that are killed by a signal.

This crate adds a [`verify()`][verify] method to the `Result`s returned by
[`Commands`][Command]'s [`status()`][status], [`spawn()`][spawn] and [`output()`][output]
methods, and to [`Child`][Child]'s [`wait()`][wait], [`try_wait`][try_wait], and
[`wait_with_output`][wait_with_output] methods. The [`verify()`][verify] method turns
unsuccessful [`ExitStatus`][ExitStatus] values into errors, so the following will return
[`CommandStatusError`][CommandStatusError]s for commands that don't successfully complete:
* `cmd.status().verify()?`
* `child.wait().verify()?`
* `child.try_wait().verify()?`

[`verify()`][verify] similarly turns an unsuccessful [`Output`][Output] into a
[`CommandError`][CommandError].  The `stdout` and `stderr` fields of the
[`Output`][Output] are saved in the corresponding fields of the
[`CommandError`][CommandError].
* `cmd.output().verify()?`
* `child.wait_with_output().verify()?`

Without [`verify()`][verify] (but with [`CommandStatusError`][CommandStatusError]), we'd need to
write something like this:

```no_run
# use std::process::Command;
# use stringent::CommandStatusError;

fn run_commands(first: &mut Command, second: &mut Command) -> Result<(), CommandStatusError> {
    let mut status = match first.status() {
        Ok(status) => status,
        Err(io_err) => return Err(CommandStatusError::SpawnFailed(io_err)),
    };
    if status.success() {
        status = match second.status() {
            Ok(status) => status,
            Err(io_err) => return Err(CommandStatusError::SpawnFailed(io_err)),
        }
    }
    if status.success() { return Ok(()) }
    match status.code() {
        Some(code) => Err(CommandStatusError::ExitCode(code)),
        None => {
            #[cfg(unix)]
                use std::os::unix::process::ExitStatusExt;
                let signal = status.signal();
            #[cfg(not(unix))]
                let signal = None;
            Err(CommandStatusError::Signal(signal))
        }
    }
}
```
The [`verify()`][verify] method also handles the result of the [`spawn`][spawn] method,
for convenience when writing functions that return a `Result<T, CommandStatusError>`:

```no_run
use std::process::Command;
use stringent::{CommandStatusError, Verify};

fn run_commands() -> Result<(), CommandStatusError> {
    let mut child = Command::new("mycommand").spawn().verify()?;
    // do more things...
    child.wait().verify()?;
    Ok(())
}
```
