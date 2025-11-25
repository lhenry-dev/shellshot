use std::sync::OnceLock;

use ab_glyph::FontArc;
use image::Rgba;

use crate::constants::DEFAULT_BG_COLOR;
use crate::constants::DEFAULT_FG_COLOR;
use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::ImageRendererError;
use crate::screen_builder::ansi::GREEN;
use crate::screen_builder::ansi::RED;
use crate::screen_builder::ansi::YELLOW;
use crate::screen_builder::Cell;
use crate::screen_builder::Size;
use crate::window_decoration::WindowMetrics;

use super::WindowDecoration;

#[derive(Debug)]
pub struct Classic;

const BORDER_COLOR: [u8; 4] = [128, 128, 128, 255];
const BACKGROUND_COLOR: [u8; 4] = DEFAULT_BG_COLOR;

static CASCADIA_CODE_FONT_DATA: &[u8] = include_bytes!("../../assets/CascadiaCode.ttf");
static CASCADIA_CODE_FONT: OnceLock<Result<FontArc, ImageRendererError>> = OnceLock::new();

impl WindowDecoration for Classic {
    fn build_command_line(&self, command: &str) -> Vec<Cell> {
        let s = format!("$ {command}");

        let mut chars = s.chars();
        let mut cells = Vec::with_capacity(s.len());

        if let Some(first) = chars.next() {
            cells.push(Cell {
                ch: first,
                fg: Some(GREEN),
                bg: None,
            });
        }

        cells.extend(chars.map(|ch| Cell {
            ch,
            fg: None,
            bg: None,
        }));

        cells
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
        metrics: &WindowMetrics,
    ) -> Result<(), ImageRendererError> {
        draw_window_decorations(canvas, metrics)
    }
}

fn draw_window_decorations(
    canvas: &mut Canvas,
    metrics: &WindowMetrics,
) -> Result<(), ImageRendererError> {
    canvas.fill(Rgba(BORDER_COLOR));

    canvas.fill_rect(
        i32::try_from(metrics.border_width)?,
        i32::try_from(metrics.border_width)?,
        canvas.width() - 2 * metrics.border_width,
        canvas.height() - 2 * metrics.border_width,
        Rgba(BACKGROUND_COLOR),
    );

    canvas.fill_rect(
        i32::try_from(metrics.border_width)?,
        i32::try_from(metrics.border_width)?,
        canvas.width() - 2 * metrics.border_width,
        metrics.title_bar_height,
        Rgba([30, 34, 42, 255]),
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

    canvas.draw_circle(right - spacing, btn_y, radius, GREEN);

    canvas.draw_circle(right - 2 * spacing, btn_y, radius, YELLOW);

    canvas.draw_circle(right - 3 * spacing, btn_y, radius, RED);

    Ok(())
}
