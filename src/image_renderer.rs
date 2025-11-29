use ab_glyph::PxScale;
use image::RgbaImage;
use termwiz::surface::Surface;
use thiserror::Error;
use unicode_width::UnicodeWidthChar;

use crate::constants::{FONT_SIZE, QUALITY_MULTIPLIER};
use crate::image_renderer::canvas::Canvas;
use crate::image_renderer::render_size::{calculate_char_size, calculate_image_size};
use crate::image_renderer::utils::resolve_rgba_with_palette;
use crate::window_decoration::{WindowDecoration, WindowMetrics};

pub mod canvas;
pub mod render_size;
mod utils;

#[derive(Debug, Error)]
pub enum ImageRendererError {
    #[error("Failed to load font")]
    FontLoadError,

    #[error("Numeric conversion failed: {0}")]
    Conversion(#[from] std::num::TryFromIntError),

    #[error("Failed to initialize canvas")]
    CanvasInitFailed,

    #[error("Failed to create final image from raw data")]
    ImageCreationFailed,
}

/// `ImageRenderer` is responsible for rendering a `ScreenBuilder` into an image
/// using the provided window decoration and rendering metrics.
#[derive(Debug)]
pub struct ImageRenderer {
    canvas: Canvas,
    metrics: WindowMetrics,
    window_decoration: Box<dyn WindowDecoration>,
}

impl ImageRenderer {
    /// Renders a `ScreenBuilder` into an `RgbaImage` using the provided window decoration.
    ///
    /// # Arguments
    ///
    /// * `screen` - The screen content to render.
    /// * `window_decoration` - A boxed `WindowDecoration` implementation to draw window chrome.
    ///
    /// # Returns
    ///
    /// A Result containing the rendered `RgbaImage` or an `ImageRendererError`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Font loading fails
    /// - Canvas initialization fails
    /// - Image creation fails
    pub fn render_image(
        command: &[String],
        screen: &Surface,
        window_decoration: Box<dyn WindowDecoration>,
    ) -> Result<RgbaImage, ImageRendererError> {
        let mut renderer = Self::create_renderer(screen, window_decoration)?;
        renderer.compose_image(command, screen)
    }

    fn create_renderer(
        screen: &Surface,
        window_decoration: Box<dyn WindowDecoration>,
    ) -> Result<Self, ImageRendererError> {
        let font = window_decoration.font()?;
        let default_fg_color = window_decoration.default_fg_color();

        let scale = PxScale::from((FONT_SIZE * QUALITY_MULTIPLIER) as f32);
        let char_size = calculate_char_size(font, scale);

        let metrics = window_decoration.compute_metrics(char_size);
        let image_size = calculate_image_size(screen, &metrics, char_size);
        let canvas = Canvas::new(
            image_size.width() as u32,
            image_size.height() as u32,
            font.clone(),
            default_fg_color,
            scale,
        )?;

        Ok(Self {
            canvas,
            metrics,
            window_decoration,
        })
    }

    fn compose_image(
        &mut self,
        command: &[String],
        screen: &Surface,
    ) -> Result<RgbaImage, ImageRendererError> {
        self.window_decoration
            .draw_window(&mut self.canvas, &self.metrics)?;

        self.draw_command_line(command)?;

        self.draw_terminal_content(screen)?;

        println!("Finalizing image rendering...");

        let final_image = self.canvas.to_final_image()?;

        Ok(final_image)
    }

    fn draw_command_line(&mut self, command: &[String]) -> Result<(), ImageRendererError> {
        let start_x = self.metrics.border_width + self.metrics.padding;
        let start_y =
            self.metrics.border_width + self.metrics.title_bar_height + self.metrics.padding;

        let color_palette = self.window_decoration.get_color_palette();

        let command_line = self
            .window_decoration
            .build_command_line(&command.join(" "));

        let y = i32::try_from(start_y)?;
        let mut x_offset = 0;
        for cell in command_line.iter() {
            let x = i32::try_from(start_x + x_offset)?;

            let text = cell.str();

            self.canvas.draw_text(
                text,
                x,
                y,
                resolve_rgba_with_palette(color_palette, cell.attrs().foreground()),
                resolve_rgba_with_palette(color_palette, cell.attrs().background()),
            );

            let text_width = text
                .chars()
                .map(|ch| ch.width().unwrap_or(0))
                .sum::<usize>();
            x_offset += self.canvas.char_width() * u32::try_from(text_width)?;
        }

        Ok(())
    }

    fn draw_terminal_content(&mut self, screen: &Surface) -> Result<(), ImageRendererError> {
        let start_x = self.metrics.border_width + self.metrics.padding;
        let start_y =
            self.metrics.border_width + self.metrics.title_bar_height + self.metrics.padding;

        let color_palette = self.window_decoration.get_color_palette();

        for (row_idx, line) in screen.screen_lines().iter().enumerate() {
            let row_idx = u32::try_from(row_idx + 1)?;
            let y = i32::try_from(start_y + row_idx * self.canvas.char_height())?;

            let mut x_offset = 0;
            for cell in line.visible_cells() {
                let x = i32::try_from(start_x + x_offset)?;

                let text = cell.str();

                self.canvas.draw_text(
                    text,
                    x,
                    y,
                    resolve_rgba_with_palette(color_palette, cell.attrs().foreground()),
                    resolve_rgba_with_palette(color_palette, cell.attrs().background()),
                );

                let text_width = text
                    .chars()
                    .map(|ch| ch.width().unwrap_or(0))
                    .sum::<usize>();
                x_offset += self.canvas.char_width() * u32::try_from(text_width)?;
            }
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_render_image_with_mock_screen() {
//         let window_decoration = crate::window_decoration::create_window_decoration(None);

//         let screen =
//             ScreenBuilder::from_output("test", "echo test", window_decoration.as_ref()).unwrap();

//         let result = ImageRenderer::render_image(&screen, window_decoration);

//         assert!(result.is_ok());
//         let image = result.unwrap();
//         assert!(image.width() > 0);
//         assert!(image.height() > 0);
//     }
// }
