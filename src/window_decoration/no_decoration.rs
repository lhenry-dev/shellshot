use image::Rgba;
use termwiz::cell::Cell;

use crate::image_renderer::ImageRendererError;
use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::render_size::Size;
use crate::window_decoration::Fonts;
use crate::window_decoration::WindowMetrics;
use crate::window_decoration::common::default_build_command_line;
use crate::window_decoration::common::default_font;
use crate::window_decoration::common::get_default_color_palette;

use super::WindowDecoration;

#[derive(Debug)]
pub struct NoDecoration;

impl WindowDecoration for NoDecoration {
    fn build_command_line(&self, command: &str) -> Vec<Cell> {
        default_build_command_line(command)
    }

    fn compute_metrics(&self, char_size: Size) -> WindowMetrics {
        let padding = char_size.height;

        WindowMetrics {
            padding,
            border_width: 0,
            title_bar_height: 0,
        }
    }

    fn get_color_palette(&self) -> [Rgba<u8>; 256] {
        get_default_color_palette()
    }

    fn font(&self) -> Result<Fonts, ImageRendererError> {
        default_font()
    }

    fn draw_window(
        &self,
        canvas: &mut Canvas,
        _metrics: &WindowMetrics,
    ) -> Result<(), ImageRendererError> {
        let bg_color = self.get_color_palette()[0];
        canvas.fill(bg_color);
        Ok(())
    }
}
