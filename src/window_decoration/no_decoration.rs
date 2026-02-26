use termwiz::cell::Cell;

use crate::image_renderer::ImageRendererError;
use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::render_size::Size;
use crate::theme::Theme;
use crate::window_decoration::Fonts;
use crate::window_decoration::WindowMetrics;
use crate::window_decoration::common::default_build_command_line;
use crate::window_decoration::common::default_font;

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

    fn font(&self) -> Result<Fonts, ImageRendererError> {
        default_font()
    }

    fn draw_window(
        &self,
        canvas: &mut Canvas,
        _metrics: &WindowMetrics,
        theme: &Theme,
    ) -> Result<(), ImageRendererError> {
        canvas.fill(theme.background_color);
        Ok(())
    }
}
