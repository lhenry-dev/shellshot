use ab_glyph::FontArc;
use image::Rgba;
use termwiz::{
    cell::{Cell, CellAttributes},
    color::ColorAttribute,
};

use crate::{image_renderer::ImageRendererError, window_decoration::Fonts};

pub static DEJA_VU_FONT_DATA: &[u8] = include_bytes!("../../assets/DejaVuSansMono.ttf");
pub static DEJA_VU_CODE_BOLD_FONT_DATA: &[u8] =
    include_bytes!("../../assets/DejaVuSansMono-Bold.ttf");
pub static DEJA_VU_CODE_BOLDITALIC_FONT_DATA: &[u8] =
    include_bytes!("../../assets/DejaVuSansMono-BoldOblique.ttf");
pub static DEJA_VU_CODE_ITALIC_FONT_DATA: &[u8] =
    include_bytes!("../../assets/DejaVuSansMono-Oblique.ttf");

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

pub fn default_font() -> Result<Fonts, ImageRendererError> {
    Ok(Fonts {
        regular: FontArc::try_from_slice(DEJA_VU_FONT_DATA)
            .map_err(|_| ImageRendererError::FontLoadError)?,
        bold: FontArc::try_from_slice(DEJA_VU_CODE_BOLD_FONT_DATA)
            .map_err(|_| ImageRendererError::FontLoadError)?,
        italic: FontArc::try_from_slice(DEJA_VU_CODE_ITALIC_FONT_DATA)
            .map_err(|_| ImageRendererError::FontLoadError)?,
        bold_italic: FontArc::try_from_slice(DEJA_VU_CODE_BOLDITALIC_FONT_DATA)
            .map_err(|_| ImageRendererError::FontLoadError)?,
    })
}

pub fn get_default_color_palette() -> [Rgba<u8>; 256] {
    const COLORS6: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

    std::array::from_fn(|idx| {
        let i = idx as u8;
        match i {
            0 => Rgba([0x28, 0x2c, 0x34, 0xff]),
            7 => Rgba([0xf5, 0xf1, 0xe5, 0xff]),
            8 => Rgba([0x80, 0x80, 0x80, 0xff]),
            15 => Rgba([0xff, 0xff, 0xff, 0xff]),
            1..=6 | 9..=14 => {
                let k = if i & 8 != 0 { 0xff } else { 0xC0 };
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
