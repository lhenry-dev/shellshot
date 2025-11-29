use std::{io::Write, sync::Mutex};

use once_cell::sync::Lazy;
use termwiz::{
    cell::AttributeChange,
    escape::{
        csi::{Cursor, Sgr},
        OneBased, CSI,
    },
    surface::{Change, Position, SequenceNo, Surface, SEQ_ZERO},
};

use crate::terminal_builder::utils::{tabulate, tabulate_back};

static SAVED_POSITIONS: Lazy<Mutex<Vec<(usize, usize)>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn process_csi(
    surface: &mut Surface,
    writer: &mut Box<dyn Write + Send>,
    csi: CSI,
) -> SequenceNo {
    match csi {
        CSI::Sgr(sgr) => process_sgr(surface, sgr),
        CSI::Cursor(cursor) => process_cursor(surface, writer, cursor),
        CSI::Edit(_edit) => SEQ_ZERO,
        CSI::Mode(_mode) => SEQ_ZERO,
        _ => SEQ_ZERO,
    }
}

fn process_sgr(surface: &mut Surface, sgr: Sgr) -> SequenceNo {
    match sgr {
        Sgr::Reset => surface.add_change(Change::AllAttributes(Default::default())),
        Sgr::Intensity(intensity) => {
            surface.add_change(Change::Attribute(AttributeChange::Intensity(intensity)))
        }
        Sgr::Underline(underline) => {
            surface.add_change(Change::Attribute(AttributeChange::Underline(underline)))
        }
        Sgr::UnderlineColor(_) => SEQ_ZERO,
        Sgr::Blink(_) => SEQ_ZERO,
        Sgr::Inverse(inverse) => {
            surface.add_change(Change::Attribute(AttributeChange::Reverse(inverse)))
        }
        Sgr::Foreground(color) => {
            surface.add_change(Change::Attribute(AttributeChange::Foreground(color.into())))
        }
        Sgr::Background(color) => {
            surface.add_change(Change::Attribute(AttributeChange::Background(color.into())))
        }
        Sgr::Italic(italic) => {
            surface.add_change(Change::Attribute(AttributeChange::Italic(italic)))
        }
        Sgr::StrikeThrough(enabled) => {
            surface.add_change(Change::Attribute(AttributeChange::StrikeThrough(enabled)))
        }
        Sgr::Invisible(enabled) => {
            surface.add_change(Change::Attribute(AttributeChange::Invisible(enabled)))
        }
        _ => SEQ_ZERO,
    }
}

fn process_cursor(
    surface: &mut Surface,
    writer: &mut Box<dyn Write + Send>,
    cursor: Cursor,
) -> SequenceNo {
    match cursor {
        Cursor::BackwardTabulation(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(tabulate_back(surface.cursor_position().0, n as usize)),
            y: Position::Relative(0),
        }),
        Cursor::ForwardTabulation(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(tabulate(surface.cursor_position().0, n as usize)),
            y: Position::Relative(0),
        }),
        Cursor::CharacterAbsolute(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(n.as_zero_based() as usize),
            y: Position::Relative(0),
        }),
        Cursor::CharacterAndLinePosition { line, col } => {
            surface.add_change(Change::CursorPosition {
                x: Position::Absolute(col.as_zero_based() as usize),
                y: Position::Absolute(line.as_zero_based() as usize),
            })
        }
        Cursor::CharacterPositionForward(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(n as isize),
            y: Position::Relative(0),
        }),
        Cursor::CharacterPositionBackward(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(-(n as isize)),
            y: Position::Relative(0),
        }),
        Cursor::CharacterPositionAbsolute(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(n.as_zero_based() as usize),
            y: Position::Relative(0),
        }),
        Cursor::LinePositionForward(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(0),
            y: Position::Relative(n as isize),
        }),
        Cursor::LinePositionBackward(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(0),
            y: Position::Relative(-(n as isize)),
        }),
        Cursor::LinePositionAbsolute(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(n as usize),
        }),
        Cursor::Up(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(0),
            y: Position::Relative(-(n as isize)),
        }),
        Cursor::Down(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(0),
            y: Position::Relative(n as isize),
        }),
        Cursor::Right(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(n as isize),
            y: Position::Relative(0),
        }),
        Cursor::Left(n) => surface.add_change(Change::CursorPosition {
            x: Position::Relative(-(n as isize)),
            y: Position::Relative(0),
        }),
        Cursor::NextLine(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(n as isize),
        }),
        Cursor::PrecedingLine(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(-(n as isize)),
        }),
        Cursor::Position { line, col } => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(col.as_zero_based() as usize),
            y: Position::Absolute(line.as_zero_based() as usize),
        }),
        Cursor::SaveCursor => {
            let pos = surface.cursor_position();
            SAVED_POSITIONS.lock().unwrap().push(pos);
            SEQ_ZERO
        }
        Cursor::RestoreCursor => {
            if let Some((x, y)) = SAVED_POSITIONS.lock().unwrap().pop() {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(x),
                    y: Position::Absolute(y),
                });
            }
            SEQ_ZERO
        }
        Cursor::RequestActivePositionReport => {
            let cursor_position = surface.cursor_position();
            let col = OneBased::from_zero_based(cursor_position.0 as u32);
            let line = OneBased::from_zero_based(cursor_position.1 as u32);

            // let report = CSI::Cursor(Cursor::ActivePositionReport { line, col });
            let report = format!("\x1b[{};{}R", line, col);
            write!(writer, "{report}").ok();
            writer.flush().ok();
            println!("Reported cursor position: line {}, col {}", line, col);
            SEQ_ZERO
        }
        _ => SEQ_ZERO,
    }
}
