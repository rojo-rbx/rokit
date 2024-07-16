use std::process::exit;

use clap::Parser;
use tracing::error;

mod cli;
mod runner;
mod util;

use self::cli::Cli;
use self::runner::Runner;

#[tokio::main]
async fn main() {
    /*
        Rokit has two modes of operation, depending on if
        it is currently wrapping a tool executable or not:

        - If it is wrapping a tool executable, it will
          run that executable and pipe its output back
        - If it is not wrapping a tool executable, it will
          run a CLI interface for managing / installing tools
    */
    let runner = Runner::new();
    let result = if runner.should_run() {
        runner.run().await
    } else {
        Cli::parse().run().await
    };

    /*
        NOTE: We use tracing for errors here for consistent
        output formatting between returned errors, and errors
        that may be logged while a wrapped executable is running.

        For more information about how tracing is set up, check the
        respective `run` methods for the `Cli` and `Runner` structs.
    */
    if let Err(e) = result {
        error!("{e:?}");
        exit(1);
    }
}
