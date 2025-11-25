use image::Rgba;
use unicode_width::UnicodeWidthChar;

use crate::{constants::MAX_ROWS, screen_builder::Cell};

pub fn process_text_block(
    text: &str,
    screen: &mut Vec<Vec<Cell>>,
    cursor_x: &mut usize,
    cursor_y: &mut usize,
    fg: Option<Rgba<u8>>,
    bg: Option<Rgba<u8>>,
) {
    for ch in text.chars() {
        if ch == '\n' {
            *cursor_y += 1;
            *cursor_x = 0;
            ensure_screen_line_exists(screen, *cursor_y);
        } else if ch == '\r' {
            *cursor_x = 0;
        } else {
            let width = ch.width().unwrap_or(0);
            ensure_screen_line_exists(screen, *cursor_y);

            for _ in 0..width {
                if *cursor_x >= screen[*cursor_y].len() {
                    screen[*cursor_y].push(Cell { ch, fg, bg });
                } else {
                    screen[*cursor_y][*cursor_x] = Cell { ch, fg, bg };
                }
                *cursor_x += 1;
            }
        }
    }
}

pub fn ensure_screen_line_exists(screen: &mut Vec<Vec<Cell>>, line: usize) {
    while screen.len() <= line {
        screen.push(vec![]);
    }
}

pub fn enforce_max_rows(screen: &mut Vec<Vec<Cell>>, cursor_y: &mut usize) {
    while screen.len() > MAX_ROWS {
        screen.remove(0);
        *cursor_y = cursor_y.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_text_block_single_line() {
        let mut screen = vec![Vec::new()];
        let mut x = 0;
        let mut y = 0;

        process_text_block("abc", &mut screen, &mut x, &mut y, None, None);

        assert_eq!(y, 0);
        assert_eq!(x, 3);
        assert_eq!(screen.len(), 1);
        let text: String = screen[0].iter().map(|c| c.ch).collect();
        assert_eq!(text, "abc");
    }

    #[test]
    fn test_process_text_block_multiple_lines() {
        let mut screen = vec![Vec::new()];
        let mut x = 0;
        let mut y = 0;

        process_text_block("ab\ncd", &mut screen, &mut x, &mut y, None, None);

        assert_eq!(y, 1);
        assert_eq!(x, 2);
        assert_eq!(screen.len(), 2);
        let line1: String = screen[0].iter().map(|c| c.ch).collect();
        let line2: String = screen[1].iter().map(|c| c.ch).collect();
        assert_eq!(line1, "ab");
        assert_eq!(line2, "cd");
    }

    #[test]
    fn test_process_text_block_append_on_same_line() {
        let mut screen = vec![vec![Cell {
            ch: 'x',
            fg: None,
            bg: None,
        }]];
        let mut x = 1;
        let mut y = 0;

        process_text_block("yz", &mut screen, &mut x, &mut y, None, None);

        let text: String = screen[0].iter().map(|c| c.ch).collect();
        assert_eq!(text, "xyz");
        assert_eq!(x, 3);
        assert_eq!(y, 0);
    }

    #[test]
    fn test_process_text_block_carriage_return_overwrite() {
        let mut screen = vec![Vec::new()];
        let mut x = 0;
        let mut y = 0;

        process_text_block("Hello\rWorld", &mut screen, &mut x, &mut y, None, None);

        let text: String = screen[0].iter().map(|c| c.ch).collect();

        assert_eq!(text, "World");
        assert_eq!(x, 5);
        assert_eq!(y, 0);
    }

    #[test]
    fn test_process_text_block_carriage_return_partial_overwrite() {
        let mut screen = vec![Vec::new()];
        let mut x = 0;
        let mut y = 0;

        process_text_block("ABCDE\rXY", &mut screen, &mut x, &mut y, None, None);

        let text: String = screen[0].iter().map(|c| c.ch).collect();

        assert_eq!(text, "XYCDE");
        assert_eq!(x, 2);
        assert_eq!(y, 0);
    }

    #[test]
    fn test_process_text_block_carriage_return_after_newline() {
        let mut screen = vec![Vec::new()];
        let mut x = 0;
        let mut y = 0;

        process_text_block("abc\ndef\rXY", &mut screen, &mut x, &mut y, None, None);

        let line1: String = screen[0].iter().map(|c| c.ch).collect();
        let line2: String = screen[1].iter().map(|c| c.ch).collect();

        assert_eq!(line1, "abc");
        assert_eq!(line2, "XYf");
        assert_eq!(x, 2);
        assert_eq!(y, 1);
    }
}
