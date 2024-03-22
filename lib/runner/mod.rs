use std::ffi::OsStr;

use async_signal::{Signal, Signals};
use futures::StreamExt;
use tokio::{
    process::Command,
    task::{spawn, JoinHandle},
};

const EXIT_CODE_GOT_SIGNAL: i32 = 128;

fn spawn_signal_task() -> Result<JoinHandle<i32>, std::io::Error> {
    let mut signals = if cfg!(target_os = "windows") {
        Signals::new([Signal::Int])?
    } else {
        Signals::new([
            Signal::Int,  // Interrupt
            Signal::Term, // Terminate
            Signal::Quit, // Quit
        ])?
    };

    /*
        NOTE: If we got a signal, we'll return 128 + signal number as
        our exit code - this is not very obvious to an end user but will
        give us something to work and debug with in case there's an issue.

        For experienced users this may also make it more obvious that the
        program they were trying to run did not error - it was interrupted.
    */
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

    This command is interruptible by passing one of the following signals to Aftman:

    - SIGINT (Ctrl+C)
    - SIGTERM
    - SIGQUIT

    Note that on Windows, only SIGINT (Ctrl+C) is supported.

    # Errors

    - If signal listeners could not be created
    - If the given command could not be spawned
*/
pub async fn run<C, A, S>(command: C, args: A) -> Result<i32, std::io::Error>
where
    C: AsRef<OsStr>,
    A: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let signal_handle = spawn_signal_task()?;
    let signal_aborter = signal_handle.abort_handle();

    let mut child_handle = Command::new(command)
        // Important - we do not want to leave any zombie
        // processes behind if this async function is cancelled
        .kill_on_drop(true)
        .args(args)
        .spawn()?;

    let code = tokio::select! {
        // If the spawned process exits cleanly, we'll return its exit code.
        command_result = child_handle.wait() => {
            let code = command_result.ok().and_then(|s| s.code()).unwrap_or(1);
            signal_aborter.abort();
            code
        }
        // If the command was manually interrupted by a signal, we will
        // return a special exit code for the signal. More details above.
        task_result = signal_handle => {
            child_handle.kill().await.ok();
            task_result.unwrap_or(EXIT_CODE_GOT_SIGNAL)
        }
    };

    Ok(code)
}
