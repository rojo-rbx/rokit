use std::{fmt::Write, time::Duration};

use console::style;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

const PROGRESS_BAR_CHARACTERS: &str = "█▉▊▋▌▍▎▏ ";
const PROGRESS_BAR_TICKERS: &str = "⠙⠹⠸⠼⠴⠦⠧⠇⠏ ";

const PROGRESS_TEMPLATE_DEFAULT: &str =
    "{spinner:.bold.cyan} {msg:11.bold.cyan} [{bar:32.bold}] {current_task:>2} / {total_tasks:2}";

/**
    A styled progress bar for the Rokit CLI.

    Tracks subtasks (partial tasks) and a more granular progress
    bar while still only displaying the main task count to the user.
*/
pub struct CliProgressTracker {
    inner: ProgressBar,
    num_subtasks: Option<usize>,
}

impl CliProgressTracker {
    /**
        Create a new progress tracker with a message and a number of tasks.

        Also has subtasks per each task, making the progress bar display more granular.
    */
    pub fn new_with_message_and_subtasks(
        message: impl Into<String>,
        num_tasks: usize,
        subtasks_per_task: usize,
    ) -> Self {
        Self {
            inner: new_progress_bar(message, num_tasks, subtasks_per_task),
            num_subtasks: Some(subtasks_per_task),
        }
    }

    /**
        Create a new progress tracker with a message and a number of tasks.

        Does not have any subtasks.
    */
    pub fn new_with_message(message: impl Into<String>, num_tasks: usize) -> Self {
        Self {
            inner: new_progress_bar(message, num_tasks, 1),
            num_subtasks: None,
        }
    }

    /**
        Increments the main task count.
    */
    pub fn task_completed(&self) {
        match self.num_subtasks {
            Some(n) => self.inner.inc(n as u64),
            None => self.inner.inc(1),
        }
    }

    /**
        Increments the subtask count.

        Note that this *must* be called for the exact amount of subtasks
        per task, otherwise the progress bar displayed will be incorrect.
    */
    pub fn subtask_completed(&self) {
        assert!(
            self.num_subtasks.is_some(),
            "subtask_completed called without subtasks"
        );
        self.inner.inc(1);
    }

    /**
        Returns a formatted string of the elapsed time.

        This is formatted as `(took x.yz)` and is meant to be
        displayed at the end of the final progress tracker message.
    */
    pub fn formatted_elapsed(&self) -> String {
        style(format!("(took {:.2?})", self.inner.elapsed()))
            .dim()
            .to_string()
    }

    /**
        Updates the message in front of the progress bar.
    */
    pub fn update_message(&self, message: impl Into<String>) {
        self.inner.set_message(message.into());
    }

    /**
        Prints a message above the current progress bar.
    */
    #[allow(dead_code)]
    pub fn print_message(&self, message: impl Into<String>) {
        self.inner.println(message.into());
    }

    /**
        Finishes the progress tracker with a final message.

        This will clear the progress bar and display the final message given.
    */
    pub fn finish_with_message(&self, final_message: impl Into<String>) {
        self.finish_with_emoji_and_message("🚀", final_message);
    }

    /**
        Finishes the progress tracker with a final message and a custom emoji prefix.

        This will clear the progress bar and display the final message given.
    */
    pub fn finish_with_emoji_and_message(&self, emoji: &str, final_message: impl Into<String>) {
        self.inner.println(format!(
            "{} {}",
            style(emoji).bold().green(),
            final_message.into()
        ));
        self.inner.finish_and_clear();
    }
}

fn new_progress_style(num_tasks: usize, subtasks_per_task: usize) -> ProgressStyle {
    ProgressStyle::with_template(PROGRESS_TEMPLATE_DEFAULT)
        .unwrap()
        .with_key(
            "current_task",
            move |state: &ProgressState, writer: &mut dyn Write| {
                let current_pos = state.pos();
                let current_task = current_pos / subtasks_per_task as u64;
                writer.write_str(current_task.to_string().as_str()).unwrap();
            },
        )
        .with_key(
            "total_tasks",
            move |_: &ProgressState, writer: &mut dyn Write| {
                writer.write_str(num_tasks.to_string().as_str()).unwrap();
            },
        )
        .progress_chars(PROGRESS_BAR_CHARACTERS)
        .tick_chars(PROGRESS_BAR_TICKERS)
}

fn new_progress_bar(
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
