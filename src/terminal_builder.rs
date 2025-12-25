use indicatif::style::TemplateError;
use std::io::{self, BufRead};
use termwiz::color::ColorAttribute;
use termwiz::escape::parser::Parser;
use termwiz::surface::Surface;
use thiserror::Error;

use crate::constants::{SCREEN_MAX_HEIGHT, SCREEN_MAX_WIDTH};
use crate::pty_executor::dimension::Dimension;
use crate::pty_executor::PtyIO;
use crate::terminal_builder::action::process_action;
use crate::terminal_builder::progress_bar::TerminalBuilderProgressBar;

mod action;
mod progress_bar;
mod utils;

#[derive(Debug, Error)]
pub enum TerminalBuilderError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Progress bar template error: {0}")]
    ProgressTemplateError(#[from] TemplateError),
}

pub struct TerminalBuilder {
    pty_process: PtyIO,
    surface: Surface,
    quiet: bool,
}

impl TerminalBuilder {
    pub fn run(
        pty_process: PtyIO,
        cols: &Dimension,
        rows: &Dimension,
        quiet: bool,
    ) -> Result<Surface, TerminalBuilderError> {
        let mut terminal = Self {
            pty_process,
            surface: Surface::new(
                cols.to_u16(SCREEN_MAX_WIDTH).into(),
                rows.to_u16(SCREEN_MAX_HEIGHT).into(),
            ),
            quiet,
        };

        terminal.run_loop()?;
        match (cols, rows) {
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

        let pb = TerminalBuilderProgressBar::new(self.quiet)?;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }

            let mut actions = Vec::new();
            parser.parse(buf, |action| action.append_to(&mut actions));

            for action in actions {
                pb.update_progress(&action);

                let seq = process_action(surface, writer, &action);
                surface.flush_changes_older_than(seq);
            }

            let len = buf.len();
            reader.consume(len);
        }

        pb.finish();

        Ok(self.surface.clone())
    }

    pub fn resize_surface(&mut self, resize_cols: bool, resize_rows: bool) {
        let lines = self.surface.screen_lines();
        let (current_cols, current_rows) = self.surface.dimensions();

        let mut max_col = 0;
        let mut max_row = 0;

        for (row_idx, line) in lines.iter().enumerate() {
            let mut last_idx = 0;
            for cell in line.visible_cells() {
                let is_non_empty = !cell.str().chars().all(char::is_whitespace)
                    || !matches!(cell.attrs().background(), ColorAttribute::Default);
                if is_non_empty {
                    last_idx = cell.cell_index() + 1;
                }
            }

            if resize_cols {
                max_col = max_col.max(last_idx);
            }

            if resize_rows && last_idx > 0 {
                max_row = row_idx + 1;
            }
        }

        let new_cols = if resize_cols { max_col } else { current_cols };
        let new_rows = if resize_rows { max_row } else { current_rows };

        self.surface.resize(new_cols, new_rows);
    }
}

#[cfg(test)]
mod tests {
    use termwiz::{
        cell::AttributeChange,
        surface::{Change, Position},
    };

    use super::*;
    use crate::pty_executor::writer::DetachableWriter;
    use crate::pty_executor::PtyIO;
    use std::io::{self, BufReader, Cursor};

    fn create_mock_pty(content: &[u8]) -> PtyIO {
        let cursor: Box<dyn io::Read + Send> = Box::new(Cursor::new(content.to_vec()));
        let reader = BufReader::new(cursor);
        let writer = DetachableWriter::new(Box::new(io::sink()));
        PtyIO { reader, writer }
    }

    #[test]
    fn test_terminal_builder_run_simple_text() {
        let content = b"Hello, Terminal!";
        let pty_process = create_mock_pty(content);

        let surface = TerminalBuilder::run(
            pty_process,
            &Dimension::Value(10),
            &Dimension::Value(5),
            true,
        )
        .expect("TerminalBuilder should run");

        let (cols, rows) = surface.dimensions();
        assert_eq!(cols, 10);
        assert_eq!(rows, 5);

        let first_line: String = surface.screen_lines()[0]
            .visible_cells()
            .map(|c| c.str().to_string())
            .collect::<String>();
        assert!(first_line.contains('H') || first_line.contains('e'));
    }

    #[test]
    fn test_run_loop_empty_content() {
        let pty_process = create_mock_pty(b"");
        let mut builder = TerminalBuilder {
            pty_process,
            surface: Surface::new(5, 5),
            quiet: true,
        };

        let result = builder.run_loop();
        assert!(result.is_ok());
        let surface = result.unwrap();
        let (cols, rows) = surface.dimensions();
        assert_eq!(cols, 5);
        assert_eq!(rows, 5);
    }

    #[test]
    fn test_resize_surface() {
        let mut surface = Surface::new(10, 5);

        surface.add_change(Change::Text("ABC    ".to_string()));
        surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1),
        });

        surface.add_change(Change::Text("       ".to_string()));
        surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1),
        });

        let mut builder = TerminalBuilder {
            pty_process: create_mock_pty(b""),
            surface,
            quiet: true,
        };

        builder.resize_surface(true, true);

        let (new_cols, new_rows) = builder.surface.dimensions();

        assert_eq!(new_cols, 3, "Expected 3 columns");
        assert_eq!(new_rows, 1, "Expected 1 rows");
    }

    #[test]
    fn test_resize_surface_with_colored_background_exact() {
        let mut surface = Surface::new(10, 5);

        surface.add_change(Change::Attribute(AttributeChange::Background(
            ColorAttribute::PaletteIndex(7),
        )));
        surface.add_change(Change::Text("   ".to_string()));
        surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1),
        });

        surface.add_change(Change::Text("X".to_string()));
        surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1),
        });

        surface.add_change(Change::Text("  ".to_string()));
        surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(1),
        });

        surface.add_change(Change::Attribute(AttributeChange::Background(
            ColorAttribute::Default,
        )));
        surface.add_change(Change::Text("   ".to_string()));

        let mut builder = TerminalBuilder {
            pty_process: create_mock_pty(b""),
            surface,
            quiet: true,
        };

        builder.resize_surface(true, true);

        let (new_cols, new_rows) = builder.surface.dimensions();

        assert_eq!(new_cols, 3, "Expected 3 columns");
        assert_eq!(new_rows, 3, "Expected 3 rows");
    }
}
