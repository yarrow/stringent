# stringent

Return an error when a [`std::process::Command`][Command] method
successfully starts a process, but the process itself isn't successful.

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

The standard library's [`Command`][Command] module's [`io::Result`][ioResult]
values for command completion can sometimes feel like "the operation was a
success but the patient died". For instance, the [`status()`][status]
method (which runs a [`Command`][Command] in a new process) returns either
`Err(`[`io::Error`][ioError]`)` (when the process creation fails), or
`Ok(`[`ExitStatus`][ExitStatus]`)`; but then we need to check the returned
status to see if the command actually succeeded:

```rust
let success = match cmd.status() {
    Err(_) => false,
    Ok(status) => status.success(),
};
```
In particular, we can't use `cmd.status()?` to pass errors back to our
caller, because that will ignore commands that exit with an error code or
that are killed by a signal.

This crate adds a `stringent` method to the `Result`s returned by
[`Commands`][Command]'s [`status()`][status], [`spawn()`][spawn] and [`output()`][output]
methods, and to [`Child`][Child]'s [`wait()`][wait], [`try_wait`][try_wait], and
[`wait_with_output`][wait_with_output] methods. The `stringent` method turns
unsuccessful [`ExitStatus`][ExitStatus] values into errors, so the following will return
`CommandError`s for commands that don't successfully complete:
* `cmd.status().stringent()?`
* `child.wait().stringent()?`
* `child.try_wait().stringent()?`

`stringent` similarly turns an unsuccessful [`Output`][Output] into a
`CommandErrorWithOutput`.  The `stdout` and `stderr` fields of the
[`Output`][Output] are saved in the corresponding fields of the
`CommandErrorWithOutput`.
* `cmd.output().stringent()?`
* `child.wait_with_output().stringent()?`

## Example

```rust
use std::process::Command;
use stringent::{CommandError, Stringent};

fn run_commands(first: &mut Command, second: &mut Command) -> Result<(), CommandError> {
    first.status().stringent()?;
    second.status().stringent()?;
    Ok(())
}
```

Without `stringent` (but with `CommandError`), we'd need to write something like this:

```rust

fn run_commands(first: &mut Command, second: &mut Command) -> Result<(), CommandError> {
    let mut status = match first.status() {
        Ok(status) => status,
        Err(io_err) => return Err(CommandError::SpawnFailed(io_err)),
    };
    if status.success() {
        status = match second.status() {
            Ok(status) => status,
            Err(io_err) => return Err(CommandError::SpawnFailed(io_err)),
        }
    }
    if status.success() { return Ok(()) }
    match status.code() {
        Some(code) => Err(CommandError::ExitCode(code)),
        None => {
            #[cfg(unix)]
                use std::os::unix::process::ExitStatusExt;
                let signal = status.signal();
            #[cfg(not(unix))]
                let signal = None;
            Err(CommandError::Signal(signal))
        }
    }
}
```
The `stringent` method also handles the result of the [`spawn`][spawn] method,
for convenience when writing functions that return a `Result<T, CommandError>`:

```rust
use std::process::Command;
use stringent::{CommandError, Stringent};

fn run_commands() -> Result<(), CommandError> {
    let mut child = Command::new("mycommand").spawn().stringent()?;
    // do more things...
    child.wait().stringent()?;
    Ok(())
}
```
