use termwiz::cell::Cell;

use crate::{
    image_renderer::{
        ImageRendererError,
        canvas::{Canvas, Corners},
        render_size::Size,
        utils::darken_color,
    },
    theme::Theme,
    window_decoration::{
        Fonts, WindowMetrics,
        common::{default_build_command_line, default_font},
    },
};

use super::WindowDecoration;

#[derive(Debug)]
pub struct Windows;

impl WindowDecoration for Windows {
    fn build_command_line(&self, command: &str) -> Vec<Cell> {
        default_build_command_line(command)
    }

    fn compute_metrics(&self, char_size: Size) -> WindowMetrics {
        let char_height = char_size.height;

        let padding = char_height;
        let border_width = 0;
        let title_bar_height = char_height + char_height / 2;

        WindowMetrics {
            padding,
            border_width,
            title_bar_height,
        }
    }

    fn font(&self) -> Result<Fonts, ImageRendererError> {
        default_font()
    }

    fn draw_window(
        &self,
        canvas: &mut Canvas,
        metrics: &WindowMetrics,
        theme: &Theme,
    ) -> Result<(), ImageRendererError> {
        draw_window_decorations(canvas, metrics, theme)
    }
}

fn draw_window_decorations(
    canvas: &mut Canvas,
    metrics: &WindowMetrics,
    theme: &Theme,
) -> Result<(), ImageRendererError> {
    let bg_color = theme.background_color;
    let title_bar_color = darken_color(bg_color, 0.2); // 20% darker than background

    canvas.fill_rounded_rect(
        i32::try_from(metrics.border_width)?,
        i32::try_from(metrics.border_width)?,
        canvas.width() - 2 * metrics.border_width,
        canvas.height() - 2 * metrics.border_width,
        bg_color,
        6.0,
        &Corners::ALL,
    );

    canvas.fill_rounded_rect(
        i32::try_from(metrics.border_width)?,
        i32::try_from(metrics.border_width)?,
        canvas.width() - 2 * metrics.border_width,
        metrics.title_bar_height,
        title_bar_color,
        6.0,
        &(Corners::TOP_LEFT | Corners::TOP_RIGHT),
    );

    draw_window_buttons(canvas, metrics, theme);

    Ok(())
}

fn draw_window_buttons(canvas: &mut Canvas, metrics: &WindowMetrics, theme: &Theme) {
    let btn_size = metrics.title_bar_height;
    let top = metrics.border_width;
    let spacing = btn_size + btn_size / 5;

    let right = canvas.width() - metrics.border_width;

    let close_x = right - btn_size;
    let max_x = right - spacing - btn_size;
    let min_x = right - 2 * spacing - btn_size;

    let pad = btn_size / 3;
    let thickness = (btn_size / 12).max(1);
    let icon_color = theme.foreground_color;

    // --- Close (X) ---
    canvas.draw_line(
        close_x + pad,
        top + pad,
        close_x + btn_size - pad,
        top + btn_size - pad,
        thickness,
        icon_color,
    );
    canvas.draw_line(
        close_x + btn_size - pad,
        top + pad,
        close_x + pad,
        top + btn_size - pad,
        thickness,
        icon_color,
    );

    // --- Maximize (□) ---
    canvas.draw_rect_outline(
        max_x + pad,
        top + pad,
        btn_size - pad * 2,
        btn_size - pad * 2,
        thickness,
        icon_color,
    );

    // --- Minimize (—) ---
    canvas.draw_line(
        min_x + pad,
        top + btn_size / 2,
        min_x + btn_size - pad,
        top + btn_size / 2,
        thickness,
        icon_color,
    );
}
