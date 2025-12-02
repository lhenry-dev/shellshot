use ab_glyph::FontArc;
use image::Rgba;
use termwiz::cell::Cell;

use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::canvas::Corners;
use crate::image_renderer::render_size::Size;
use crate::image_renderer::ImageRendererError;
use crate::window_decoration::common::default_build_command_line;
use crate::window_decoration::common::default_font;
use crate::window_decoration::common::get_default_color_palette;
use crate::window_decoration::common::DEFAULT_BG_COLOR;
use crate::window_decoration::common::DEFAULT_FG_COLOR;
use crate::window_decoration::WindowMetrics;

use super::WindowDecoration;

#[derive(Debug)]
pub struct Classic;

const GREEN: Rgba<u8> = Rgba([52, 199, 89, 255]);
const YELLOW: Rgba<u8> = Rgba([255, 189, 45, 255]);
const RED: Rgba<u8> = Rgba([255, 95, 87, 255]);

const BORDER_COLOR: [u8; 4] = [128, 128, 128, 255];

impl WindowDecoration for Classic {
    fn build_command_line(&self, command: &str) -> Vec<Cell> {
        default_build_command_line(command)
    }

    fn compute_metrics(&self, char_size: Size) -> WindowMetrics {
        let char_height = char_size.height;

        let padding = char_height;
        let border_width = 0;
        let title_bar_height = char_height;

        WindowMetrics {
            padding,
            border_width,
            title_bar_height,
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
        metrics: &WindowMetrics,
    ) -> Result<(), ImageRendererError> {
        draw_window_decorations(canvas, metrics)
    }
}

fn draw_window_decorations(
    canvas: &mut Canvas,
    metrics: &WindowMetrics,
) -> Result<(), ImageRendererError> {
    canvas.fill_rounded(
        Rgba(BORDER_COLOR),
        metrics.title_bar_height as f32 / 4.0,
        &Corners::ALL,
    );

    canvas.fill_rounded_rect(
        i32::try_from(metrics.border_width)?,
        i32::try_from(metrics.border_width)?,
        canvas.width() - 2 * metrics.border_width,
        canvas.height() - 2 * metrics.border_width,
        Rgba(DEFAULT_BG_COLOR),
        metrics.title_bar_height as f32 / 4.0,
        &Corners::ALL,
    );

    canvas.fill_rounded_rect(
        i32::try_from(metrics.border_width)?,
        i32::try_from(metrics.border_width)?,
        canvas.width() - 2 * metrics.border_width,
        metrics.title_bar_height,
        Rgba([30, 34, 42, 255]),
        metrics.title_bar_height as f32 / 4.0,
        &(Corners::TOP_LEFT | Corners::TOP_RIGHT),
    );

    draw_window_buttons(canvas, metrics)
}

fn draw_window_buttons(
    canvas: &mut Canvas,
    metrics: &WindowMetrics,
) -> Result<(), ImageRendererError> {
    let btn_y = i32::try_from(metrics.border_width + (metrics.title_bar_height / 2))?;
    let radius = i32::try_from(metrics.title_bar_height / 4)?;
    let spacing = radius * 3;

    let right = i32::try_from(canvas.width() - metrics.border_width)?;

    canvas.draw_circle(right - spacing, btn_y, radius, RED);

    canvas.draw_circle(right - 2 * spacing, btn_y, radius, YELLOW);

    canvas.draw_circle(right - 3 * spacing, btn_y, radius, GREEN);

    Ok(())
}
