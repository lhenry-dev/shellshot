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

    let default_attrs = CellAttributes::blank();

    cells.push(Cell::new('$', prompt_attrs));

    cells.push(Cell::new(' ', default_attrs.clone()));

    cells.extend(
        command
            .chars()
            .map(|ch| Cell::new(ch, default_attrs.clone())),
    );

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
    const COLORS6: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

    std::array::from_fn(|idx| {
        let i = idx as u8;
        match i {
            0 => Rgba([0x28, 0x2c, 0x34, 0xff]),
            7 => Rgba([0xee, 0xe8, 0xd5, 0xff]),
            8 => Rgba([0x80, 0x80, 0x80, 0xff]),
            15 => Rgba([0xff, 0xff, 0xff, 0xff]),
            1..=6 | 9..=14 => {
                let k = if i & 8 != 0 { 0xff } else { 0x80 };
                let r = if (i & 1) != 0 { k } else { 0 };
                let g = if (i >> 1 & 1) != 0 { k } else { 0 };
                let b = if (i >> 2 & 1) != 0 { k } else { 0 };
                Rgba([r, g, b, 0xff])
            }
            16..=231 => {
                let i = i - 16;
                let r = COLORS6[(i / 36) as usize % 6];
                let g = COLORS6[(i / 6) as usize % 6];
                let b = COLORS6[(i % 6) as usize];
                Rgba([r, g, b, 0xff])
            }
            232..=255 => {
                let x = 8 + (i - 232) * 10;
                Rgba([x, x, x, 0xff])
            }
        }
    })
}
