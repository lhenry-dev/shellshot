use std::sync::OnceLock;

use ab_glyph::FontArc;
use image::Rgba;

use crate::constants::DEFAULT_BG_COLOR;
use crate::constants::DEFAULT_FG_COLOR;
use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::ImageRendererError;
use crate::screen_builder::Cell;
use crate::screen_builder::Size;
use crate::window_decoration::WindowMetrics;

use super::WindowDecoration;

#[derive(Debug)]
pub struct NoDecoration;

const BACKGROUND_COLOR: [u8; 4] = DEFAULT_BG_COLOR;
static CASCADIA_CODE_FONT_DATA: &[u8] = include_bytes!("../../assets/CascadiaCode.ttf");
static CASCADIA_CODE_FONT: OnceLock<Result<FontArc, ImageRendererError>> = OnceLock::new();

impl WindowDecoration for NoDecoration {
    fn build_command_line(&self, command: &str) -> Vec<Cell> {
        format!("$ {command}")
            .chars()
            .map(|ch| Cell {
                ch,
                fg: None,
                bg: None,
            })
            .collect()
    }

    fn compute_metrics(&self, char_size: Size) -> WindowMetrics {
        let padding = char_size.height;

        WindowMetrics {
            padding,
            border_width: 0,
            title_bar_height: 0,
        }
    }

    fn default_fg_color(&self) -> Rgba<u8> {
        Rgba(DEFAULT_FG_COLOR)
    }

    fn font(&self) -> Result<&FontArc, ImageRendererError> {
        CASCADIA_CODE_FONT
            .get_or_init(|| {
                FontArc::try_from_slice(CASCADIA_CODE_FONT_DATA)
                    .map_err(|_| ImageRendererError::FontLoadError)
            })
            .as_ref()
            .map_err(|_| ImageRendererError::FontLoadError)
    }

    fn draw_window(
        &self,
        canvas: &mut Canvas,
        _metrics: &WindowMetrics,
    ) -> Result<(), ImageRendererError> {
        canvas.fill(Rgba(BACKGROUND_COLOR));
        Ok(())
    }
}
