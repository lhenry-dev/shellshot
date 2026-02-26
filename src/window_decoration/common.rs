use ab_glyph::FontArc;
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
