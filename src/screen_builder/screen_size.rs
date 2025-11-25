use unicode_width::UnicodeWidthChar;

use crate::screen_builder::{Cell, ScreenBuilderError};

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub fn calculate_screen_size(screen: &[Vec<Cell>]) -> Result<Size, ScreenBuilderError> {
    let width = screen
        .iter()
        .map(|line| {
            line.iter()
                .map(|cell| cell.ch.width().unwrap_or(0))
                .sum::<usize>()
        })
        .filter_map(|sum| u32::try_from(sum).ok())
        .max()
        .ok_or(ScreenBuilderError::NoWidth)?;

    let height = u32::try_from(screen.len())?;

    Ok(Size { width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cell(ch: char) -> Cell {
        Cell {
            ch,
            fg: None,
            bg: None,
        }
    }

    #[test]
    fn test_calculate_screen_dimensions() {
        let screen = vec![
            vec![make_cell('a'), make_cell('b'), make_cell('c')],
            vec![make_cell('d'), make_cell('e')],
        ];

        let dims = calculate_screen_size(&screen).unwrap();

        assert_eq!(dims.width, 3);
        assert_eq!(dims.height, 2);
    }

    #[test]
    fn test_empty_screen() {
        let screen: Vec<Vec<Cell>> = vec![];

        let result = calculate_screen_size(&screen);

        assert!(result.is_err(), "Expected an error for empty screen");
    }
}
