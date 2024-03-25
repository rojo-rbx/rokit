use std::{fmt::Write, time::Duration};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

const PROGRESS_BAR_CHARACTERS: &str = "█▉▊▋▌▍▎▏ ";
const PROGRESS_BAR_TICKERS: &str = "⠙⠹⠸⠼⠴⠦⠧⠇⠏ ";

const PROGRESS_TEMPLATE_DEFAULT: &str =
    "{spinner:.bold.cyan} {msg:11.bold.cyan} [{bar:32.bold}] {current_task:>2} / {total_tasks:2}";
const PROGRESS_TEMPLATE_FINISHED: &str = "{done:.bold.green} {msg}";

fn new_progress_style(num_tasks: usize, subtasks_per_task: usize) -> ProgressStyle {
    ProgressStyle::with_template(PROGRESS_TEMPLATE_DEFAULT)
        .unwrap()
        .with_key(
            "current_task",
            move |state: &ProgressState, writer: &mut dyn Write| {
                let current_pos = state.pos();
                let current_task = current_pos / subtasks_per_task as u64;
                writer.write_str(current_task.to_string().as_str()).unwrap()
            },
        )
        .with_key(
            "total_tasks",
            move |_: &ProgressState, writer: &mut dyn Write| {
                writer.write_str(num_tasks.to_string().as_str()).unwrap()
            },
        )
        .progress_chars(PROGRESS_BAR_CHARACTERS)
        .tick_chars(PROGRESS_BAR_TICKERS)
}

fn new_finishing_style() -> ProgressStyle {
    ProgressStyle::with_template(PROGRESS_TEMPLATE_FINISHED)
        .unwrap()
        .with_key("done", |_: &ProgressState, writer: &mut dyn Write| {
            writer.write_char('🚀').unwrap()
        })
        .progress_chars(PROGRESS_BAR_CHARACTERS)
        .tick_chars(PROGRESS_BAR_TICKERS)
}

pub fn new_progress_bar(
    message: impl Into<String>,
    num_tasks: usize,
    subtasks_per_task: usize,
) -> ProgressBar {
    let pb = ProgressBar::new_spinner()
        .with_style(new_progress_style(num_tasks, subtasks_per_task))
        .with_message(message.into());
    pb.enable_steady_tick(Duration::from_millis(50));
    pb.set_length((num_tasks * subtasks_per_task) as u64);
    pb.tick();
    pb
}

pub fn finish_progress_bar(pb: ProgressBar, final_message: impl Into<String>) {
    pb.set_style(new_finishing_style());
    pb.set_message(final_message.into());
    pb.finish();
}
