use std::io::{self, BufRead, Write};
use termwiz::color::ColorAttribute;
use termwiz::escape::parser::Parser;
use termwiz::escape::Action;
use termwiz::surface::Surface;
use thiserror::Error;
use tracing::trace;

use crate::constants::{MAX_HEIGHT, MAX_WIDTH};
use crate::pty_executor::dimension::Dimension;
use crate::pty_executor::PtyIO;
use crate::terminal_builder::action::process_action;

mod action;
mod utils;

#[derive(Debug, Error)]
pub enum TerminalBuilderError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub struct TerminalBuilder {
    pty_process: PtyIO,
    surface: Surface,
}

impl TerminalBuilder {
    pub fn run(
        pty_process: PtyIO,
        cols: &Dimension,
        rows: &Dimension,
    ) -> Result<Surface, TerminalBuilderError> {
        let mut terminal = Self {
            pty_process,
            surface: Surface::new(
                cols.to_u16(MAX_WIDTH).into(),
                rows.to_u16(MAX_HEIGHT).into(),
            ),
        };

        terminal.run_loop()?;
        match (rows, cols) {
            (Dimension::Auto, Dimension::Auto) => terminal.resize_surface(true, true),
            (Dimension::Auto, Dimension::Value(_)) => terminal.resize_surface(true, false),
            (Dimension::Value(_), Dimension::Auto) => terminal.resize_surface(false, true),
            (Dimension::Value(_), Dimension::Value(_)) => (),
        }

        Ok(terminal.surface.clone())
    }

    fn run_loop(&mut self) -> Result<Surface, TerminalBuilderError> {
        let reader = &mut self.pty_process.reader;
        let writer = &mut self.pty_process.writer;
        let surface = &mut self.surface;

        let mut parser = Parser::new();

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }

            let mut actions = Vec::new();
            parser.parse(buf, |action| action.append_to(&mut actions));

            for action in actions {
                if let Action::PrintString(s) = &action {
                    trace!("Currently parsing: {}", s);
                    io::stdout().flush()?;
                }
                let seq = process_action(surface, writer, &action);
                surface.flush_changes_older_than(seq);
            }

            let len = buf.len();
            reader.consume(len);
        }

        Ok(self.surface.clone())
    }

    pub fn resize_surface(&mut self, resize_cols: bool, resize_rows: bool) {
        let lines = self.surface.screen_lines();
        let (current_cols, current_rows) = self.surface.dimensions();

        let new_cols = if resize_cols {
            lines
                .iter()
                .map(|line| {
                    let mut last_idx = 0;
                    for cell in line.visible_cells() {
                        if !(cell.str().chars().all(char::is_whitespace)
                            && matches!(cell.attrs().background(), ColorAttribute::Default))
                        {
                            last_idx = cell.cell_index() + 1;
                        }
                    }
                    last_idx
                })
                .max()
                .unwrap_or(0)
        } else {
            current_cols
        };

        let new_rows = if resize_rows {
            lines
                .iter()
                .rposition(|line| !line.is_whitespace())
                .map(|idx| idx + 1)
                .unwrap_or(0)
        } else {
            current_rows
        };

        self.surface.resize(new_cols, new_rows);
    }
}
