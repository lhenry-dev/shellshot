use image::Rgba;
use termwiz::color::ColorAttribute;

pub fn resolve_rgba_with_palette(
    color_palette: &[Rgba<u8>; 256],
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use termwiz::color::ColorAttribute;

    fn sample_palette() -> [Rgba<u8>; 256] {
        let mut palette = [Rgba([0, 0, 0, 255]); 256];
        for (i, color) in palette.iter_mut().enumerate() {
            *color = Rgba([i as u8, (255 - i) as u8, i as u8 / 2, 255]);
        }
        palette
    }

    #[test]
    fn test_default_returns_none() {
        let palette = sample_palette();
        let result = resolve_rgba_with_palette(&palette, ColorAttribute::Default);
        assert!(result.is_none());
    }

    #[test]
    fn test_truecolor_with_default_fallback() {
        let palette = sample_palette();

        let c: (f32, f32, f32, f32) = (0.5, 0.25, 1.0, 0.75);
        let result = resolve_rgba_with_palette(
            &palette,
            ColorAttribute::TrueColorWithDefaultFallback(c.into()),
        );

        assert_eq!(
            result,
            Some(Rgba([
                (c.0 * 255.0_f32).round() as u8,
                (c.1 * 255.0_f32).round() as u8,
                (c.2 * 255.0_f32).round() as u8,
                (c.3 * 255.0_f32).round() as u8,
            ]))
        );
    }

    #[test]
    fn test_truecolor_with_palette_fallback() {
        let palette = sample_palette();

        let c: (f32, f32, f32, f32) = (0.1, 0.2, 0.3, 0.4);
        let fallback_index = 42;
        let result = resolve_rgba_with_palette(
            &palette,
            ColorAttribute::TrueColorWithPaletteFallback(c.into(), fallback_index),
        );

        assert_eq!(
            result,
            Some(Rgba([
                (c.0 * 255.0_f32).round() as u8,
                (c.1 * 255.0_f32).round() as u8,
                (c.2 * 255.0_f32).round() as u8,
                (c.3 * 255.0_f32).round() as u8,
            ]))
        );
    }

    #[test]
    fn test_palette_index_wrap_around() {
        let palette = sample_palette();

        for idx in 256..260 {
            let expected = palette[idx % 256];
            let result =
                resolve_rgba_with_palette(&palette, ColorAttribute::PaletteIndex(idx as u8));
            assert_eq!(result, Some(expected));
        }
    }

    #[test]
    fn test_truecolor_edge_cases() {
        let palette = sample_palette();

        let c: (f32, f32, f32, f32) = (0.0, 1.0, 0.0, 1.0);
        let result = resolve_rgba_with_palette(
            &palette,
            ColorAttribute::TrueColorWithDefaultFallback(c.into()),
        );
        assert_eq!(result, Some(Rgba([0, 255, 0, 255])));
    }
}
