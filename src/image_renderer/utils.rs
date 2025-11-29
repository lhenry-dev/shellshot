use image::Rgba;
use termwiz::color::ColorAttribute;

pub fn resolve_rgba_with_palette(
    color_palette: [Rgba<u8>; 256],
    attr: ColorAttribute,
) -> Option<Rgba<u8>> {
    match attr {
        ColorAttribute::Default => None,

        ColorAttribute::PaletteIndex(idx) => {
            let index = idx as usize % color_palette.len();
            Some(color_palette[index])
        }

        ColorAttribute::TrueColorWithDefaultFallback(c)
        | ColorAttribute::TrueColorWithPaletteFallback(c, _) => Some(Rgba([
            (c.0 * 255.0).round() as u8,
            (c.1 * 255.0).round() as u8,
            (c.2 * 255.0).round() as u8,
            (c.3 * 255.0).round() as u8,
        ])),
    }
}
