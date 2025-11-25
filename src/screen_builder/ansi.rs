use ansi_parser::AnsiSequence;
use image::Rgba;

use crate::screen_builder::{ensure_screen_line_exists, Cell, ScreenBuilderError};

// ANSI standard
pub const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
pub const RED: Rgba<u8> = Rgba([255, 95, 87, 255]);
pub const GREEN: Rgba<u8> = Rgba([52, 199, 89, 255]);
pub const YELLOW: Rgba<u8> = Rgba([255, 189, 45, 255]);
pub const BLUE: Rgba<u8> = Rgba([88, 153, 255, 255]);
pub const MAGENTA: Rgba<u8> = Rgba([255, 121, 198, 255]);
pub const CYAN: Rgba<u8> = Rgba([0, 255, 255, 255]);
pub const WHITE: Rgba<u8> = Rgba([238, 232, 213, 255]);

// ANSI bright / bold
pub const BRIGHT_BLACK: Rgba<u8> = Rgba([85, 85, 85, 255]);
pub const BRIGHT_RED: Rgba<u8> = Rgba([255, 120, 115, 255]);
pub const BRIGHT_GREEN: Rgba<u8> = Rgba([80, 255, 140, 255]);
pub const BRIGHT_YELLOW: Rgba<u8> = Rgba([255, 220, 100, 255]);
pub const BRIGHT_BLUE: Rgba<u8> = Rgba([120, 190, 255, 255]);
pub const BRIGHT_MAGENTA: Rgba<u8> = Rgba([255, 150, 220, 255]);
pub const BRIGHT_CYAN: Rgba<u8> = Rgba([120, 255, 255, 255]);
pub const BRIGHT_WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

pub fn process_ansi_sequence(
    seq: &AnsiSequence,
    screen: &mut Vec<Vec<Cell>>,
    cursor_x: &mut usize,
    cursor_y: &mut usize,
    fg: &mut Option<Rgba<u8>>,
    bg: &mut Option<Rgba<u8>>,
) -> Result<(), ScreenBuilderError> {
    match seq {
        AnsiSequence::CursorPos(row, col) => {
            *cursor_y = row.saturating_sub(1).try_into()?;
            *cursor_x = col.saturating_sub(1).try_into()?;
            ensure_screen_line_exists(screen, *cursor_y);
        }
        AnsiSequence::CursorForward(n) => {
            *cursor_x += usize::try_from(*n)?;
            ensure_screen_line_exists(screen, *cursor_y);
        }
        AnsiSequence::CursorBackward(n) => {
            *cursor_x = cursor_x.saturating_sub((*n).try_into()?);
        }
        AnsiSequence::CursorDown(n) => {
            *cursor_y += usize::try_from(*n)?;
            ensure_screen_line_exists(screen, *cursor_y);
        }
        AnsiSequence::CursorUp(n) => {
            *cursor_y = cursor_y.saturating_sub((*n).try_into()?);
        }
        AnsiSequence::EraseLine => {
            if (*cursor_y) < screen.len() {
                screen[*cursor_y].clear();
            }
        }
        AnsiSequence::SetGraphicsMode(params) => {
            apply_sgr(params, fg, bg);
        }
        _ => {}
    }

    Ok(())
}

pub fn apply_sgr(params: &[u8], fg: &mut Option<Rgba<u8>>, bg: &mut Option<Rgba<u8>>) {
    let mut bold = false;

    for &code in params {
        match code {
            0 => {
                *fg = None;
                *bg = None;
                bold = false;
            }
            1 => bold = true, // bold / bright
            39 => *fg = None,
            49 => *bg = None,

            // foreground standard
            30 => *fg = Some(if bold { BRIGHT_BLACK } else { BLACK }),
            31 => *fg = Some(if bold { BRIGHT_RED } else { RED }),
            32 => *fg = Some(if bold { BRIGHT_GREEN } else { GREEN }),
            33 => *fg = Some(if bold { BRIGHT_YELLOW } else { YELLOW }),
            34 => *fg = Some(if bold { BRIGHT_BLUE } else { BLUE }),
            35 => *fg = Some(if bold { BRIGHT_MAGENTA } else { MAGENTA }),
            36 => *fg = Some(if bold { BRIGHT_CYAN } else { CYAN }),
            37 => *fg = Some(if bold { BRIGHT_WHITE } else { WHITE }),

            // background standard (bold usually ignored)
            40 => *bg = Some(BLACK),
            41 => *bg = Some(RED),
            42 => *bg = Some(GREEN),
            43 => *bg = Some(YELLOW),
            44 => *bg = Some(BLUE),
            45 => *bg = Some(MAGENTA),
            46 => *bg = Some(CYAN),
            47 => *bg = Some(WHITE),

            // bright foreground codes (90–97)
            90 => *fg = Some(BRIGHT_BLACK),
            91 => *fg = Some(BRIGHT_RED),
            92 => *fg = Some(BRIGHT_GREEN),
            93 => *fg = Some(BRIGHT_YELLOW),
            94 => *fg = Some(BRIGHT_BLUE),
            95 => *fg = Some(BRIGHT_MAGENTA),
            96 => *fg = Some(BRIGHT_CYAN),
            97 => *fg = Some(BRIGHT_WHITE),

            // bright background codes (100–107)
            100 => *bg = Some(BRIGHT_BLACK),
            101 => *bg = Some(BRIGHT_RED),
            102 => *bg = Some(BRIGHT_GREEN),
            103 => *bg = Some(BRIGHT_YELLOW),
            104 => *bg = Some(BRIGHT_BLUE),
            105 => *bg = Some(BRIGHT_MAGENTA),
            106 => *bg = Some(BRIGHT_CYAN),
            107 => *bg = Some(BRIGHT_WHITE),

            _ => {}
        }
    }
}

#[cfg(test)]
mod ansi_tests {
    use ansi_parser::AnsiSequence;
    use heapless::Vec;
    use image::Rgba;

    use crate::screen_builder::{
        ansi::{process_ansi_sequence, BLUE, RED},
        Cell,
    };

    #[test]
    fn test_cursor_position_and_color() {
        let mut screen = vec![vec![]];
        let mut cursor_x = 0;
        let mut cursor_y = 0;
        let mut fg: Option<Rgba<u8>> = None;
        let mut bg: Option<Rgba<u8>> = None;

        let seq = AnsiSequence::CursorPos(2, 3);
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert_eq!(cursor_x, 2);
        assert_eq!(cursor_y, 1);
        assert!(screen.len() > 1);

        let graphic_mode = Vec::from_slice(&[31]).unwrap();
        let seq = AnsiSequence::SetGraphicsMode(graphic_mode);
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert_eq!(fg, Some(RED));

        let graphic_mode = Vec::from_slice(&[44]).unwrap();
        let seq = AnsiSequence::SetGraphicsMode(graphic_mode);
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert_eq!(bg, Some(BLUE));

        let graphic_mode = Vec::from_slice(&[0]).unwrap();
        let seq = AnsiSequence::SetGraphicsMode(graphic_mode);
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert_eq!(fg, None);
        assert_eq!(bg, None);
    }

    #[test]
    fn test_cursor_movements_and_erase_line() {
        let mut screen = vec![vec![
            Cell {
                ch: 'A',
                fg: None,
                bg: None,
            },
            Cell {
                ch: 'B',
                fg: None,
                bg: None,
            },
        ]];
        let mut cursor_x = 1;
        let mut cursor_y = 0;
        let mut fg = None;
        let mut bg = None;

        let seq = AnsiSequence::CursorForward(1);
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert_eq!(cursor_x, 2);

        let seq = AnsiSequence::CursorBackward(2);
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert_eq!(cursor_x, 0);

        let seq = AnsiSequence::EraseLine;
        process_ansi_sequence(
            &seq,
            &mut screen,
            &mut cursor_x,
            &mut cursor_y,
            &mut fg,
            &mut bg,
        )
        .unwrap();
        assert!(screen[0].is_empty());
    }
}
