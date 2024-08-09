use std::ffi::OsStr;
use std::io::Result as IoResult;

#[cfg(windows)]
use command_group::AsyncCommandGroup;

use async_signal::{Signal, Signals};
use futures::StreamExt;
use tokio::{
    process::Command,
    task::{spawn, JoinHandle},
};

/*
    If we got a signal, we'll return 128 + signal number as our exit code.

    When debugging, and for experienced users, this may
    make it slightly more obvious that the program they were
    running didn't error - it was interrupted by a signal.
*/
const EXIT_CODE_GOT_SIGNAL: i32 = 128;

fn spawn_signal_listener_task() -> IoResult<JoinHandle<i32>> {
    let mut signals = if cfg!(target_os = "windows") {
        Signals::new([Signal::Int])?
    } else {
        Signals::new([
            Signal::Int,  // Interrupt
            Signal::Term, // Terminate
            Signal::Quit, // Quit
        ])?
    };

    let task = spawn(async move {
        while let Some(result) = signals.next().await {
            match result {
                Ok(sig) => return EXIT_CODE_GOT_SIGNAL + (sig as i32),
                Err(err) => tracing::error!("Failed to listen for signal: {err}"),
            }
        }

        EXIT_CODE_GOT_SIGNAL
    });

    Ok(task)
}

/**
    Runs the given command with the given arguments and returns its exit code.

    This command is interruptible by passing one of the following signals to Rokit:

    - SIGINT (Ctrl+C)
    - SIGTERM
    - SIGQUIT

    Note that on Windows, only SIGINT (Ctrl+C) is supported, but
    the process may also be reaped as part of the current job group.

    # Errors

    - If signal listeners could not be created
    - If the given command could not be spawned
*/
pub async fn run_interruptible<C, A, S>(command: C, args: A) -> IoResult<i32>
where
    C: AsRef<OsStr>,
    A: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let signal_handle = spawn_signal_listener_task()?;
    let signal_aborter = signal_handle.abort_handle();

    /*
        Important - we do not want to leave any zombie
        processes behind if this async function is cancelled.

        Note that since we also want to spawn the child process as part
        of the current process group, we have to use the builder API from
        `command-group` to spawn the child process, or this won't work.

        The newer `process-wrap` crate claims to also support this behavior
        for inheriting process group but it doesn't seem to work as expected.
    */
    let mut command = Command::new(command);
    let mut child = {
        #[cfg(unix)]
        {
            command.args(args).kill_on_drop(true).spawn()?
        }
        #[cfg(windows)]
        {
            command.args(args).group().kill_on_drop(true).spawn()?
        }
    };

    let code = tokio::select! {
        // If the spawned process exits cleanly, we'll return its exit code,
        // which may or may not exist. Interpret a non-existent code as 1.
        command_result = child.wait() => {
            let code = command_result.ok().and_then(|s| s.code()).unwrap_or(1);
            signal_aborter.abort();
            code
        }
        // If the command was manually interrupted by a signal, we will
        // return a special exit code for the signal. More details above.
        task_result = signal_handle => {
            child.kill().await.ok();
            task_result.unwrap_or(EXIT_CODE_GOT_SIGNAL)
        }
    };

    Ok(code)
}
