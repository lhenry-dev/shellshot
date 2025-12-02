use std::io::{self, BufRead};
use termwiz::color::ColorAttribute;
use termwiz::escape::parser::Parser;
use termwiz::surface::Surface;
use thiserror::Error;

use crate::constants::{SCREEN_MAX_HEIGHT, SCREEN_MAX_WIDTH};
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
                cols.to_u16(SCREEN_MAX_WIDTH).into(),
                rows.to_u16(SCREEN_MAX_HEIGHT).into(),
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

#[cfg(test)]
mod tests {
    use termwiz::surface::Change;

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

        // Utiliser des dimensions fixes
        let surface =
            TerminalBuilder::run(pty_process, &Dimension::Value(10), &Dimension::Value(5))
                .expect("TerminalBuilder should run");

        // Vérifie les dimensions
        let (cols, rows) = surface.dimensions();
        assert_eq!(cols, 10);
        assert_eq!(rows, 5);

        // Vérifie que du texte a été écrit
        let first_line: String = surface.screen_lines()[0]
            .visible_cells()
            .map(|c| c.str().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(first_line.contains("H") || first_line.contains("e"));
    }

    #[test]
    fn test_run_loop_empty_content() {
        let pty_process = create_mock_pty(b"");
        let mut builder = TerminalBuilder {
            pty_process,
            surface: Surface::new(5, 5),
        };

        let result = builder.run_loop();
        assert!(result.is_ok());
        let surface = result.unwrap();
        let (cols, rows) = surface.dimensions();
        assert_eq!(cols, 5);
        assert_eq!(rows, 5);
    }

    #[test]
    fn test_resize_surface_auto() {
        let mut surface = Surface::new(10, 5);
        surface.add_change(Change::Text("X".to_string()));
        let mut builder = TerminalBuilder {
            pty_process: create_mock_pty(b""),
            surface,
        };

        builder.resize_surface(true, true);
        let (new_cols, new_rows) = builder.surface.dimensions();
        assert!(new_cols > 0);
        assert!(new_rows > 0);
    }
}
