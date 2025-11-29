use std::sync::OnceLock;

use ab_glyph::FontArc;
use image::Rgba;
use termwiz::{
    cell::{Cell, CellAttributes},
    color::ColorAttribute,
};

use crate::image_renderer::ImageRendererError;

pub static CASCADIA_CODE_FONT_DATA: &[u8] = include_bytes!("../../assets/CascadiaCode.ttf");
pub static CASCADIA_CODE_FONT: OnceLock<Result<FontArc, ImageRendererError>> = OnceLock::new();

pub fn default_build_command_line(command: &str) -> Vec<Cell> {
    let mut cells = Vec::with_capacity(2 + command.len());

    let mut prompt_attrs = CellAttributes::blank();
    prompt_attrs.set_foreground(ColorAttribute::PaletteIndex(10));

    cells.push(Cell::new('$', prompt_attrs.clone()));

    cells.push(Cell::new(' ', prompt_attrs));

    let default_attrs = CellAttributes::blank();

    for ch in command.chars() {
        cells.push(Cell::new(ch, default_attrs.clone()));
    }

    cells
}

pub fn default_font() -> Result<&'static FontArc, ImageRendererError> {
    CASCADIA_CODE_FONT
        .get_or_init(|| {
            FontArc::try_from_slice(CASCADIA_CODE_FONT_DATA)
                .map_err(|_| ImageRendererError::FontLoadError)
        })
        .as_ref()
        .map_err(|_| ImageRendererError::FontLoadError)
}

pub fn get_default_color_palette() -> [Rgba<u8>; 256] {
    std::array::from_fn(|i| {
        let i = i as u8;
        match i {
            0 => Rgba([0x00, 0x00, 0x00, 0xff]),
            7 => Rgba([0xc0, 0xc0, 0xc0, 0xff]),
            8 => Rgba([0x80, 0x80, 0x80, 0xff]),
            15 => Rgba([0xff, 0xff, 0xff, 0xff]),

            1..=6 | 9..=14 => {
                let k = if i & 8 != 0 { 0xff } else { 0x80 };
                let r = (i & 1) * k;
                let g = ((i >> 1) & 1) * k;
                let b = ((i >> 2) & 1) * k;
                Rgba([r, g, b, 0xff])
            }

            16..=231 => {
                let i = i - 16;
                let c: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];
                let r = c[(i / 36) as usize % 6];
                let g = c[(i / 6) as usize % 6];
                let b = c[(i % 6) as usize];
                Rgba([r, g, b, 0xff])
            }

            232..=255 => {
                let x = 8 + (i - 232) * 10;
                Rgba([x, x, x, 0xff])
            }
        }
    })
}
