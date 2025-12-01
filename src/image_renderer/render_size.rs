use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use termwiz::{cell::Cell, surface::Surface};

use crate::window_decoration::WindowMetrics;

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub fn calculate_char_size(font: &FontArc, scale: PxScale) -> Size {
    let glyph_id = font.glyph_id('M');
    let scaled_font = font.as_scaled(scale);
    let char_width = scaled_font.h_advance(glyph_id).ceil() as u32;

    let scaled_font = font.as_scaled(scale);
    let char_height = (scaled_font.height() + scaled_font.line_gap()).ceil() as u32;

    Size {
        width: char_width,
        height: char_height,
    }
}

pub fn calculate_image_size(
    command_line: &[Cell],
    screen: &Surface,
    metrics: &WindowMetrics,
    char_size: Size,
) -> Size {
    let char_width = char_size.width;
    let char_height = char_size.height;
    let padding = 2 * metrics.padding;
    let border = 2 * metrics.border_width;

    let (screen_width, screen_height) = screen.dimensions();
    let mut content_width = screen_width as u32 * char_width + padding + border;
    let mut content_height =
        screen_height as u32 * char_height + padding + border + metrics.title_bar_height;

    let command_line_width: u32 = command_line
        .iter()
        .map(|cell| cell.str().chars().count() as u32)
        .sum::<u32>()
        * char_width
        + padding
        + border;
    content_width = content_width.max(command_line_width);
    content_height += char_height;

    Size {
        width: content_width,
        height: content_height,
    }
}
