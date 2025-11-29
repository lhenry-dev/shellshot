use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use termwiz::surface::Surface;
use tiny_skia::Size;

use crate::{constants::MIN_WIDTH_CHARS, window_decoration::WindowMetrics};

pub fn calculate_char_size(font: &FontArc, scale: PxScale) -> Size {
    let glyph_id = font.glyph_id('M');
    let scaled_font = font.as_scaled(scale);
    let char_width = scaled_font.h_advance(glyph_id).ceil();

    let scaled_font = font.as_scaled(scale);
    let char_height = (scaled_font.height() + scaled_font.line_gap()).ceil();

    Size::from_wh(char_width, char_height).unwrap()
}

pub fn calculate_image_size(screen: &Surface, metrics: &WindowMetrics, char_size: Size) -> Size {
    let padding = 2 * metrics.padding;
    let char_width = char_size.width() as u32;
    let char_height = char_size.height() as u32;

    let mut content_width = screen.dimensions().0 as u32 * char_width + padding;
    let mut content_height = screen.dimensions().1 as u32 * char_height + padding;

    content_width += 2 * metrics.border_width;
    content_height += 2 * metrics.border_width + metrics.title_bar_height;

    let width = content_width.max(MIN_WIDTH_CHARS * char_width);
    let height = content_height.max(1);

    Size::from_wh(width as f32, height as f32).unwrap()
}
