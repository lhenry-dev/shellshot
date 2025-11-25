use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::{
    constants::MIN_WIDTH_CHARS,
    screen_builder::{ScreenBuilder, Size},
    window_decoration::WindowMetrics,
};

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
    screen: &ScreenBuilder,
    metrics: &WindowMetrics,
    char_size: Size,
) -> Size {
    let padding = 2 * metrics.padding;
    let char_width = char_size.width;
    let char_height = char_size.height;

    let mut content_width = screen.dimensions.width * char_width + padding;
    let mut content_height = screen.dimensions.height * char_height + padding;

    content_width += 2 * metrics.border_width;
    content_height += 2 * metrics.border_width + metrics.title_bar_height;

    let width = content_width.max(MIN_WIDTH_CHARS * char_width);
    let height = content_height.max(1);

    Size { width, height }
}
