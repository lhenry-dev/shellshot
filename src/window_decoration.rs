use crate::{
    image_renderer::{canvas::Canvas, ImageRendererError},
    window_decoration::no_decoration::NoDecoration,
};

mod classic;
mod communs;
mod no_decoration;

use ab_glyph::FontArc;
use clap::ValueEnum;
pub use classic::Classic;
use image::Rgba;
use termwiz::cell::Cell;
use tiny_skia::Size;

/// Type of window decoration to apply around the rendered content
#[derive(Clone, Debug, ValueEnum)]
pub enum WindowDecorationType {
    /// Classic window decoration
    Classic,
}

#[derive(Clone, Debug)]
pub struct WindowMetrics {
    pub padding: u32,
    pub border_width: u32,
    pub title_bar_height: u32,
}

pub trait WindowDecoration: std::fmt::Debug {
    fn build_command_line(&self, command: &str) -> Vec<Cell>;

    fn compute_metrics(&self, char_size: Size) -> WindowMetrics;

    fn default_fg_color(&self) -> Rgba<u8>;

    fn get_color_palette(&self) -> [Rgba<u8>; 256];

    fn font(&self) -> Result<&FontArc, ImageRendererError>;

    fn draw_window(
        &self,
        canvas: &mut Canvas,
        metrics: &WindowMetrics,
    ) -> Result<(), ImageRendererError>;
}

pub fn create_window_decoration(
    decoration_type: Option<&WindowDecorationType>,
) -> Box<dyn WindowDecoration> {
    match decoration_type {
        Some(WindowDecorationType::Classic) => Box::new(Classic),
        None => Box::new(NoDecoration),
    }
}

#[cfg(test)]
mod tests {
    use ab_glyph::PxScale;

    use crate::image_renderer::render_size::calculate_char_size;

    use super::*;

    fn all_window_decorations() -> Vec<Option<WindowDecorationType>> {
        let mut types = WindowDecorationType::value_variants()
            .iter()
            .cloned()
            .map(Some)
            .collect::<Vec<_>>();
        types.push(None);
        types
    }

    #[test]
    fn test_all_window_decorations_command_line() {
        for decoration_type in all_window_decorations() {
            let window_decoration = create_window_decoration(decoration_type.as_ref());
            let command_line = window_decoration.build_command_line("echo test");

            assert!(
                !command_line.is_empty(),
                "Unexpected number of cells for {decoration_type:?}",
            );
        }
    }

    #[test]
    fn test_all_window_decorations_draw() {
        let canvas_width = 200;
        let canvas_height = 100;
        let scale = PxScale::from(1.0);

        for decoration_type in all_window_decorations() {
            let window_decoration = create_window_decoration(decoration_type.as_ref());

            let font = window_decoration.font().expect("Font should be available");

            let char_size = calculate_char_size(font, scale);
            let metrics = window_decoration.compute_metrics(char_size);

            let mut canvas = Canvas::new(canvas_width, canvas_height, font.clone(), scale)
                .expect("Failed to create Canvas");

            let result = window_decoration.draw_window(&mut canvas, &metrics);
            assert!(
                result.is_ok(),
                "draw_window failed for {decoration_type:?}: {result:?}",
            );
        }
    }
}
