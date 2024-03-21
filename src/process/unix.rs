//! On Unix, we use tokio to spawn processes so that we can listen for signals
//! and wait for process completion at the same time.

use std::path::Path;
use std::thread;

use anyhow::Context;
use signal_hook::consts::signal::{SIGABRT, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;
use tokio::process::Command;
use tokio::sync::oneshot;

pub fn run(exe_path: &Path, args: Vec<String>) -> anyhow::Result<i32> {
    let (kill_tx, kill_rx) = oneshot::channel();

    // Spawn a thread dedicated to listening for signals and relaying them to
    // our async runtime.
    let (signal_thread, signal_handle) = {
        let mut signals = Signals::new([SIGABRT, SIGINT, SIGQUIT, SIGTERM]).unwrap();
        let signal_handle = signals.handle();

        let thread = thread::spawn(move || {
            if let Some(signal) = signals.into_iter().next() {
                kill_tx.send(signal).ok();
            }
        });

        (thread, signal_handle)
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .context("could not create tokio runtime")?;

    let _guard = runtime.enter();

    let mut child = Command::new(exe_path)
        .args(args)
        .spawn()
        .with_context(|| format!("could not spawn {}", exe_path.display()))?;

    let code = runtime.block_on(async move {
        tokio::select! {
            // If the child exits cleanly, we can return its exit code directly.
            // I wish everything were this tidy.
            status = child.wait() => {
                let code = status.ok().and_then(|s| s.code()).unwrap_or(1);
                signal_handle.close();
                signal_thread.join().unwrap();

                code
            }

            // If we received a signal while the process was running, murder it
            // and exit immediately with the correct error code.
            code = kill_rx => {
                child.kill().await.ok();
                signal_handle.close();
                signal_thread.join().unwrap();
                std::process::exit(128 + code.unwrap_or(0));
            }
        }
    });

    Ok(code)
}
