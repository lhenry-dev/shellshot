use ansi_parser::{AnsiParser, Output};
use image::Rgba;
use thiserror::Error;

use crate::{screen_builder::ansi::process_ansi_sequence, window_decoration::WindowDecoration};

pub mod ansi;
pub mod helpers;
pub mod screen_size;

pub use helpers::{enforce_max_rows, ensure_screen_line_exists};
pub use screen_size::{calculate_screen_size, Size};

#[derive(Debug, Error)]
pub enum ScreenBuilderError {
    #[error("Failed to determine screen width (no valid values found)")]
    NoWidth,

    #[error("Numeric conversion failed: {0}")]
    Conversion(#[from] std::num::TryFromIntError),
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Rgba<u8>>,
    pub bg: Option<Rgba<u8>>,
}

#[derive(Debug)]
pub struct ScreenBuilder {
    pub cells: Vec<Vec<Cell>>,
    pub dimensions: Size,
}

impl ScreenBuilder {
    pub fn from_output(
        output: &str,
        command: &str,
        window_decoration: &dyn WindowDecoration,
    ) -> Result<Self, ScreenBuilderError> {
        let mut fg = None;
        let mut bg = None;
        let mut cursor_x = 0;
        let mut cursor_y = 1;
        let mut screen: Vec<Vec<Cell>> = vec![vec![]];

        let cmd_line = window_decoration.build_command_line(command);
        screen[0] = cmd_line;

        let unescaped = output
            .replace("\\x1b", "\x1b")
            .replace("\\n", "\n")
            .replace("\\r", "\r");

        let parsed_output: Vec<Output> = unescaped.ansi_parse().collect();

        for item in &parsed_output {
            match item {
                Output::TextBlock(text) => {
                    helpers::process_text_block(
                        text,
                        &mut screen,
                        &mut cursor_x,
                        &mut cursor_y,
                        fg,
                        bg,
                    );
                    enforce_max_rows(&mut screen, &mut cursor_y);
                }
                Output::Escape(seq) => {
                    process_ansi_sequence(
                        seq,
                        &mut screen,
                        &mut cursor_x,
                        &mut cursor_y,
                        &mut fg,
                        &mut bg,
                    )?;
                }
            }
        }

        if let Some(last_line) = screen.last() {
            if last_line.is_empty() {
                screen.pop();
            }
        }

        let dimensions = calculate_screen_size(&screen)?;

        Ok(Self {
            cells: screen,
            dimensions,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        screen_builder::ansi::{BLUE, GREEN, RED},
        window_decoration::create_window_decoration,
    };

    use super::*;

    #[test]
    fn test_cursor_forward_and_backward() {
        let output = "ABCD\x1B[2Dxy";
        let command = "";
        let window_decoration = create_window_decoration(None);
        let screen =
            ScreenBuilder::from_output(output, command, window_decoration.as_ref()).unwrap();
        let line: String = screen.cells[1].iter().map(|c| c.ch).collect();
        assert_eq!(line, "ABxy");
    }

    #[test]
    fn test_cursor_up_and_down() {
        let output = "line1\nline2\nline3\x1B[2AXX";
        let command = "";
        let window_decoration = create_window_decoration(None);
        let screen =
            ScreenBuilder::from_output(output, command, window_decoration.as_ref()).unwrap();
        let lines: Vec<String> = screen
            .cells
            .iter()
            .map(|l| l.iter().map(|c| c.ch).collect())
            .collect();
        assert_eq!(lines[1], "line1XX");
        assert_eq!(lines[2], "line2");
        assert_eq!(lines[3], "line3");
    }

    #[test]
    fn test_erase_line_and_color() {
        let output = "\x1B[31mRED\x1B[KNEW";
        let command = "";
        let window_decoration = create_window_decoration(None);
        let screen =
            ScreenBuilder::from_output(output, command, window_decoration.as_ref()).unwrap();
        let line: String = screen.cells[1].iter().map(|c| c.ch).collect();
        assert_eq!(line, "NEW");
        let reds = screen.cells[1].iter().filter(|c| c.fg == Some(RED));
        assert_eq!(reds.count(), 3);
    }

    #[test]
    fn test_mixed_movements_and_colors() {
        let output = "\x1B[32mA\x1B[31mB\x1B[1D\x1B[34mC";
        let command = "";
        let window_decoration = create_window_decoration(None);
        let screen =
            ScreenBuilder::from_output(output, command, window_decoration.as_ref()).unwrap();
        let line: String = screen.cells[1].iter().map(|c| c.ch).collect();
        assert_eq!(line, "AC");
        assert_eq!(screen.cells[1][0].fg, Some(GREEN));
        assert_eq!(screen.cells[1][1].fg, Some(BLUE));
    }

    #[test]
    fn test_command_line_with_tricky_output() {
        let output = "output\x1B[2Dxy";
        let command = "";
        let window_decoration = create_window_decoration(None);
        let screen =
            ScreenBuilder::from_output(output, command, window_decoration.as_ref()).unwrap();
        let out: String = screen.cells[1].iter().map(|c| c.ch).collect();
        assert_eq!(out, "outpxy");
    }
}
