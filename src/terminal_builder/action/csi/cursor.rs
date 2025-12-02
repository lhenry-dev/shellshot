use std::{
    io::{self},
    sync::Mutex,
};

use termwiz::{
    escape::{csi::Cursor, OneBased, CSI},
    surface::{Change, Position, SequenceNo, Surface, SEQ_ZERO},
};
use tracing::warn;

use crate::terminal_builder::utils::{tabulate, tabulate_back};

static SAVED_POSITIONS: std::sync::LazyLock<Mutex<Vec<(usize, usize)>>> =
    std::sync::LazyLock::new(|| Mutex::new(Vec::new()));

pub fn process_cursor(
    surface: &mut Surface,
    writer: &mut dyn io::Write,
    cursor: &Cursor,
) -> SequenceNo {
    match cursor {
        Cursor::BackwardTabulation(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(tabulate_back(surface.cursor_position().0, *n as usize)),
            y: Position::Relative(0),
        }),
        Cursor::ForwardTabulation(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(tabulate(surface.cursor_position().0, *n as usize)),
            y: Position::Relative(0),
        }),
        Cursor::CharacterAbsolute(n) | Cursor::CharacterPositionAbsolute(n) => {
            surface.add_change(Change::CursorPosition {
                x: Position::Absolute(n.as_zero_based() as usize),
                y: Position::Relative(0),
            })
        }
        Cursor::CharacterAndLinePosition { line, col } => {
            surface.add_change(Change::CursorPosition {
                x: Position::Absolute(col.as_zero_based() as usize),
                y: Position::Absolute(line.as_zero_based() as usize),
            })
        }
        Cursor::CharacterPositionForward(n) | Cursor::Right(n) => {
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(*n as isize),
                y: Position::Relative(0),
            })
        }
        Cursor::CharacterPositionBackward(n) | Cursor::Left(n) => {
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(-(*n as isize)),
                y: Position::Relative(0),
            })
        }
        Cursor::LinePositionForward(n) | Cursor::Down(n) => {
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(0),
                y: Position::Relative(*n as isize),
            })
        }
        Cursor::LinePositionBackward(n) | Cursor::Up(n) => {
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(0),
                y: Position::Relative(-(*n as isize)),
            })
        }
        Cursor::LinePositionAbsolute(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(*n as usize),
        }),
        Cursor::NextLine(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(*n as isize),
        }),
        Cursor::PrecedingLine(n) => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Relative(-(*n as isize)),
        }),
        Cursor::Position { line, col } => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(col.as_zero_based() as usize),
            y: Position::Absolute(line.as_zero_based() as usize),
        }),
        Cursor::SaveCursor => {
            match SAVED_POSITIONS.lock() {
                Ok(mut positions) => positions.push(surface.cursor_position()),
                Err(poisoned) => {
                    warn!("Mutex poisoned when saving cursor position, using inner value");
                    poisoned.into_inner().push(surface.cursor_position());
                }
            }
            SEQ_ZERO
        }
        Cursor::RestoreCursor => {
            match SAVED_POSITIONS.lock() {
                Ok(mut positions) => {
                    if let Some((x, y)) = positions.pop() {
                        return surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(x),
                            y: Position::Absolute(y),
                        });
                    }
                }
                Err(poisoned) => {
                    warn!("Mutex poisoned when restoring cursor position, using inner value");
                    let value = poisoned.into_inner().pop();
                    if let Some((x, y)) = value {
                        return surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(x),
                            y: Position::Absolute(y),
                        });
                    }
                }
            }
            SEQ_ZERO
        }
        Cursor::RequestActivePositionReport => {
            let cursor_position = surface.cursor_position();
            let col = OneBased::from_zero_based(cursor_position.0 as u32);
            let line = OneBased::from_zero_based(cursor_position.1 as u32);

            let report = CSI::Cursor(Cursor::ActivePositionReport { line, col });
            write!(writer, "{report}").ok();
            writer.flush().ok();
            SEQ_ZERO
        }
        Cursor::TabulationClear(_)
        | Cursor::ActivePositionReport { .. }
        | Cursor::TabulationControl(_)
        | Cursor::LineTabulation(_)
        | Cursor::SetTopAndBottomMargins { .. }
        | Cursor::SetLeftAndRightMargins { .. }
        | Cursor::CursorStyle(_) => SEQ_ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::escape::csi::Cursor as CsiCursor;
    use termwiz::escape::OneBased;
    use termwiz::surface::{Change, Position, SequenceNo};

    fn make_surface() -> Surface {
        Surface::new(80, 24)
    }

    fn apply_cursor(surface: &mut Surface, cursor: &CsiCursor) -> SequenceNo {
        let mut writer = std::io::sink();
        process_cursor(surface, &mut writer, cursor)
    }

    #[test]
    fn cursor_right_moves_relative_x() {
        let mut s = make_surface();
        let before = s.cursor_position();

        let seq = apply_cursor(&mut s, &CsiCursor::Right(5));
        let after = s.cursor_position();

        assert_eq!(after.1, before.1);
        assert_eq!(after.0, before.0 + 5);
        let _ = seq;
    }

    #[test]
    fn cursor_left_moves_relative_x() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(10),
            y: Position::Absolute(0),
        });
        let before = s.cursor_position();

        apply_cursor(&mut s, &CsiCursor::Left(3));
        let after = s.cursor_position();

        assert_eq!(after.1, before.1);
        assert_eq!(after.0, before.0 - 3);
    }

    #[test]
    fn cursor_up_and_down_move_relative_y() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(2),
            y: Position::Absolute(5),
        });
        let before = s.cursor_position();

        apply_cursor(&mut s, &CsiCursor::Up(2));
        let mid = s.cursor_position();
        assert_eq!(mid.0, before.0);
        assert_eq!(mid.1, before.1 - 2);

        apply_cursor(&mut s, &CsiCursor::Down(4));
        let after = s.cursor_position();
        assert_eq!(after.0, before.0);
        assert_eq!(after.1, mid.1 + 4);
    }

    #[test]
    fn character_absolute_and_forward_backward() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(1),
            y: Position::Absolute(1),
        });

        apply_cursor(
            &mut s,
            &CsiCursor::CharacterAbsolute(OneBased::from_zero_based(12)),
        );
        assert_eq!(s.cursor_position().0, 12);

        apply_cursor(&mut s, &CsiCursor::CharacterPositionForward(3));
        assert_eq!(s.cursor_position().0, 15);

        apply_cursor(&mut s, &CsiCursor::CharacterPositionBackward(2));
        assert_eq!(s.cursor_position().0, 13);
    }

    #[test]
    fn character_and_line_position_sets_both() {
        let mut s = make_surface();

        let line = OneBased::from_zero_based(4);
        let col = OneBased::from_zero_based(7);

        apply_cursor(&mut s, &CsiCursor::CharacterAndLinePosition { line, col });

        assert_eq!(s.cursor_position().0, 7);
        assert_eq!(s.cursor_position().1, 4);
    }

    #[test]
    fn position_variant_sets_abs_position() {
        let mut s = make_surface();
        let line = OneBased::from_zero_based(2);
        let col = OneBased::from_zero_based(3);

        apply_cursor(&mut s, &CsiCursor::Position { line, col });

        assert_eq!(s.cursor_position(), (3, 2));
    }

    #[test]
    fn next_and_preceding_line_affect_y_and_reset_x() {
        let mut s = make_surface();

        s.add_change(Change::CursorPosition {
            x: Position::Absolute(10),
            y: Position::Absolute(2),
        });

        apply_cursor(&mut s, &CsiCursor::NextLine(1));
        let (x1, y1) = s.cursor_position();
        assert_eq!(x1, 0);
        assert_eq!(y1, 3);

        apply_cursor(&mut s, &CsiCursor::PrecedingLine(2));
        let (x2, y2) = s.cursor_position();
        assert_eq!(x2, 0);
        assert_eq!(y2, 1);
    }

    #[test]
    fn character_absolute_and_line_absolute_variants() {
        let mut s = make_surface();

        apply_cursor(
            &mut s,
            &CsiCursor::CharacterAndLinePosition {
                line: OneBased::from_zero_based(10),
                col: OneBased::from_zero_based(20),
            },
        );

        let (x, y) = s.cursor_position();
        assert_eq!(x, 20);
        assert_eq!(y, 10);
    }

    #[test]
    fn save_and_restore_cursor_position_stack_behaviour() {
        let mut s = make_surface();

        s.add_change(Change::CursorPosition {
            x: Position::Absolute(3),
            y: Position::Absolute(4),
        });

        apply_cursor(&mut s, &CsiCursor::SaveCursor);
        apply_cursor(&mut s, &CsiCursor::Right(4));
        apply_cursor(&mut s, &CsiCursor::SaveCursor);
        apply_cursor(&mut s, &CsiCursor::Down(2));

        apply_cursor(&mut s, &CsiCursor::RestoreCursor);
        let (x1, y1) = s.cursor_position();
        assert!(x1 > 0);
        assert!(y1 > 0);

        apply_cursor(&mut s, &CsiCursor::RestoreCursor);
        let (x2, y2) = s.cursor_position();
        assert!(x2 > 0);
        assert!(y2 > 0);
    }

    #[test]
    fn request_active_position_report_writes_report_to_writer() {
        let mut s = make_surface();

        s.add_change(Change::CursorPosition {
            x: Position::Absolute(7),
            y: Position::Absolute(12),
        });

        let mut buf = Vec::new();
        let csr = CsiCursor::RequestActivePositionReport;
        let seq = process_cursor(&mut s, &mut buf, &csr);

        assert!(!buf.is_empty());
        let _ = seq;
    }

    #[test]
    fn forward_and_backward_tabulation() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(2),
            y: Position::Absolute(0),
        });

        apply_cursor(&mut s, &CsiCursor::ForwardTabulation(1));
        let (x_after, _) = s.cursor_position();
        assert!(x_after > 0);

        apply_cursor(&mut s, &CsiCursor::BackwardTabulation(1));
        let (x_after2, _) = s.cursor_position();
        assert!(x_after2 > 0);
    }

    #[test]
    fn test_relative_movements() {
        let mut s = make_surface();
        let initial = s.cursor_position();

        apply_cursor(&mut s, &CsiCursor::Right(5));
        let (x, y) = s.cursor_position();
        assert_eq!(y, initial.1);
        assert_eq!(x, initial.0 + 5);

        apply_cursor(&mut s, &CsiCursor::Left(3));
        let (x2, _) = s.cursor_position();
        assert_eq!(x2, x - 3);

        apply_cursor(&mut s, &CsiCursor::Down(4));
        let (_, y2) = s.cursor_position();
        assert_eq!(y2, initial.1 + 4);

        apply_cursor(&mut s, &CsiCursor::Up(2));
        let (_, y3) = s.cursor_position();
        assert_eq!(y3, y2 - 2);
    }

    #[test]
    fn test_absolute_movements() {
        let mut s = make_surface();

        apply_cursor(
            &mut s,
            &CsiCursor::CharacterAbsolute(OneBased::from_zero_based(10)),
        );
        assert_eq!(s.cursor_position().0, 10);

        apply_cursor(&mut s, &CsiCursor::LinePositionAbsolute(5));
        assert_eq!(s.cursor_position().1, 5);

        apply_cursor(
            &mut s,
            &CsiCursor::CharacterAndLinePosition {
                line: OneBased::from_zero_based(3),
                col: OneBased::from_zero_based(7),
            },
        );
        let (x, y) = s.cursor_position();
        assert_eq!(x, 7);
        assert_eq!(y, 3);

        apply_cursor(
            &mut s,
            &CsiCursor::Position {
                line: OneBased::from_zero_based(2),
                col: OneBased::from_zero_based(4),
            },
        );
        let (x, y) = s.cursor_position();
        assert_eq!((x, y), (4, 2));
    }

    #[test]
    fn test_forward_and_backward_tabulation() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(2),
            y: Position::Absolute(0),
        });

        apply_cursor(&mut s, &CsiCursor::ForwardTabulation(1));
        let (x1, _) = s.cursor_position();
        assert!(x1 > 2);

        apply_cursor(&mut s, &CsiCursor::BackwardTabulation(1));
        let (x2, _) = s.cursor_position();
        assert!(x2 <= x1);
    }

    #[test]
    fn test_next_and_preceding_line() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(5),
            y: Position::Absolute(5),
        });

        apply_cursor(&mut s, &CsiCursor::NextLine(2));
        let (x, y) = s.cursor_position();
        assert_eq!(x, 0);
        assert_eq!(y, 7);

        apply_cursor(&mut s, &CsiCursor::PrecedingLine(3));
        let (x, y) = s.cursor_position();
        assert_eq!(x, 0);
        assert_eq!(y, 4);
    }

    #[test]
    fn test_save_and_restore_cursor_stack() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(2),
            y: Position::Absolute(3),
        });

        apply_cursor(&mut s, &CsiCursor::SaveCursor);
        apply_cursor(&mut s, &CsiCursor::Right(4));
        apply_cursor(&mut s, &CsiCursor::SaveCursor);
        apply_cursor(&mut s, &CsiCursor::Down(2));

        apply_cursor(&mut s, &CsiCursor::RestoreCursor);
        let (x1, y1) = s.cursor_position();
        assert_eq!((x1, y1), (6, 3));

        apply_cursor(&mut s, &CsiCursor::RestoreCursor);
        let (x2, y2) = s.cursor_position();
        assert_eq!((x2, y2), (2, 3));
    }

    #[test]
    fn test_request_active_position_report_writes_to_writer() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(7),
            y: Position::Absolute(8),
        });

        let mut buf = Vec::new();
        let seq = process_cursor(&mut s, &mut buf, &CsiCursor::RequestActivePositionReport);
        assert!(!buf.is_empty());
        let _ = seq;
    }

    #[test]
    fn character_position_absolute_moves_x_only() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(2),
            y: Position::Absolute(5),
        });

        apply_cursor(
            &mut s,
            &CsiCursor::CharacterPositionAbsolute(OneBased::from_zero_based(10)),
        );
        let (x, y) = s.cursor_position();
        assert_eq!(x, 10);
        assert_eq!(y, 5);
    }

    #[test]
    fn line_position_forward_moves_y_only() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(3),
            y: Position::Absolute(4),
        });

        apply_cursor(&mut s, &CsiCursor::LinePositionForward(3));
        let (x, y) = s.cursor_position();
        assert_eq!(x, 3);
        assert_eq!(y, 7);
    }

    #[test]
    fn line_position_backward_moves_y_only_and_clamped() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(3),
            y: Position::Absolute(2),
        });

        apply_cursor(&mut s, &CsiCursor::LinePositionBackward(1));
        let (x, y) = s.cursor_position();
        assert_eq!(x, 3);
        assert_eq!(y, 1);

        apply_cursor(&mut s, &CsiCursor::LinePositionBackward(5));
        let (x2, y2) = s.cursor_position();
        assert_eq!(x2, 3);
        assert_eq!(y2, 0);
    }
}
