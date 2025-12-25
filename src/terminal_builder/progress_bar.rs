use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use termwiz::escape::Action;

use crate::terminal_builder::TerminalBuilderError;

pub struct TerminalBuilderProgressBar {
    pub progress_bar: Option<ProgressBar>,
}

impl TerminalBuilderProgressBar {
    pub fn new(quiet: bool) -> Result<Self, TerminalBuilderError> {
        let progress_bar = match quiet {
            true => None,
            false => {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner().template("{spinner} Parsing: {msg}")?,
                );
                pb.enable_steady_tick(Duration::from_millis(80));
                Some(pb)
            }
        };
        Ok(Self { progress_bar })
    }

    pub fn update_progress(&self, action: &Action) {
        let Some(pb) = &self.progress_bar else {
            return;
        };

        match action {
            Action::PrintString(_) | Action::Print(_) => {
                pb.set_message(action.to_string());
            }
            _ => {}
        }
    }

    pub fn finish(&self) {
        let Some(pb) = &self.progress_bar else {
            return;
        };

        pb.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::escape::{Action, ControlCode};

    #[test]
    fn new_quiet_true_creates_no_progress_bar() {
        let pb = TerminalBuilderProgressBar::new(true).unwrap();
        assert!(pb.progress_bar.is_none());
    }

    #[test]
    fn new_quiet_false_creates_progress_bar() {
        let pb = TerminalBuilderProgressBar::new(false).unwrap();
        assert!(pb.progress_bar.is_some());
    }

    #[test]
    fn update_progress_does_nothing_when_quiet() {
        let pb = TerminalBuilderProgressBar::new(true).unwrap();

        pb.update_progress(&Action::PrintString("Test Progress Bar".into()));
    }

    #[test]
    fn update_progress_updates_message_for_print_actions() {
        let pb = TerminalBuilderProgressBar::new(false).unwrap();

        let action = Action::PrintString("Test Progress Bar".into());
        pb.update_progress(&action);
    }

    #[test]
    fn update_progress_ignores_non_print_actions() {
        let pb = TerminalBuilderProgressBar::new(false).unwrap();

        let action = Action::Control(ControlCode::LineFeed);
        pb.update_progress(&action);
    }

    #[test]
    fn finish_does_nothing_when_quiet() {
        let pb = TerminalBuilderProgressBar::new(true).unwrap();
        pb.finish();
    }

    #[test]
    fn finish_clears_progress_bar() {
        let pb = TerminalBuilderProgressBar::new(false).unwrap();
        pb.finish();
    }
}
