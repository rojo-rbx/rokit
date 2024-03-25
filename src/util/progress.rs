use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

const PROGRESS_BAR_TEMPLATE: &str = "{msg} [{bar:32.cyan/blue}] {pos} / {len}";
const PROGRESS_BAR_CHARACTERS: &str = "▪▸-";

pub fn new_progress_bar(message: impl Into<String>, length: usize) -> ProgressBar {
    let pb = ProgressBar::new(length as u64)
        .with_message(message.into())
        .with_style(
            ProgressStyle::with_template(PROGRESS_BAR_TEMPLATE)
                .unwrap()
                .progress_chars(PROGRESS_BAR_CHARACTERS),
        );
    pb.enable_steady_tick(Duration::from_millis(10));
    pb
}
