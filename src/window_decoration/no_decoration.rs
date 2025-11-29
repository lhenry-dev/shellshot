use ab_glyph::FontArc;
use image::Rgba;
use termwiz::cell::Cell;
use tiny_skia::Size;

use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::ImageRendererError;
use crate::window_decoration::communs::default_build_command_line;
use crate::window_decoration::communs::default_font;
use crate::window_decoration::communs::get_default_color_palette;
use crate::window_decoration::communs::DEFAULT_BG_COLOR;
use crate::window_decoration::communs::DEFAULT_FG_COLOR;
use crate::window_decoration::WindowMetrics;

use super::WindowDecoration;

#[derive(Debug)]
pub struct NoDecoration;

impl WindowDecoration for NoDecoration {
    fn build_command_line(&self, command: &str) -> Vec<Cell> {
        default_build_command_line(command)
    }

    fn compute_metrics(&self, char_size: Size) -> WindowMetrics {
        let padding = char_size.height() as u32;

        WindowMetrics {
            padding,
            border_width: 0,
            title_bar_height: 0,
        }
    }

    fn get_color_palette(&self) -> [Rgba<u8>; 256] {
        get_default_color_palette()
    }

    fn default_fg_color(&self) -> Rgba<u8> {
        Rgba(DEFAULT_FG_COLOR)
    }

    fn font(&self) -> Result<&FontArc, ImageRendererError> {
        default_font()
    }

    fn draw_window(
        &self,
        canvas: &mut Canvas,
        _metrics: &WindowMetrics,
    ) -> Result<(), ImageRendererError> {
        canvas.fill(Rgba(DEFAULT_BG_COLOR));
        Ok(())
    }
}
