use termwiz::{
    escape::ControlCode,
    surface::{Change, Position, SequenceNo, Surface, SEQ_ZERO},
};

use crate::terminal_builder::utils::tabulate;

pub fn process_control(surface: &mut Surface, control_code: ControlCode) -> SequenceNo {
    match control_code {
        ControlCode::LineFeed | ControlCode::VerticalTab | ControlCode::FormFeed => {
            // let (_, y) = surface.cursor_position();
            // ensure_height(surface, y + 1);
            surface.add_change("\r\n")
        }
        ControlCode::CarriageReturn => surface.add_change("\r"),
        ControlCode::HorizontalTab => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(tabulate(surface.cursor_position().0, 1)),
            y: Position::Relative(0),
        }),
        ControlCode::Backspace => {
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(-1),
                y: Position::Relative(0),
            });
            surface.add_change(" ");
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(-1),
                y: Position::Relative(0),
            })
        }
        ControlCode::NEL => surface.add_change("\r\n"),
        ControlCode::RI => {
            let (x, y) = surface.cursor_position();
            if y == 0 {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(x),
                    y: Position::Absolute(0),
                })
            } else {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(x),
                    y: Position::Relative(-1),
                })
            }
        }
        _ => SEQ_ZERO,
    }
}
