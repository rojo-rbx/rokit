use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

const PROGRESS_BAR_TEMPLATE: &str =
    "{spinner:.bold.cyan} {msg:>11.bold.cyan} [{bar:32}] {pos} / {len}";
const PROGRESS_BAR_CHARACTERS: &str = "=> ";
const PROGRESS_BAR_TICKERS: &str = "⠙⠹⠸⠼⠴⠦⠧⠇⠏ ";

fn new_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(PROGRESS_BAR_TEMPLATE)
        .unwrap()
        .progress_chars(PROGRESS_BAR_CHARACTERS)
        .tick_chars(PROGRESS_BAR_TICKERS)
}

pub fn new_progress_bar(message: impl Into<String>, length: usize) -> ProgressBar {
    let pb = ProgressBar::new_spinner()
        .with_style(new_progress_style())
        .with_message(message.into());
    pb.enable_steady_tick(Duration::from_millis(60));
    pb.set_length(length as u64);
    pb
}
