use crate::{
    image_renderer::{
        render_size::{calculate_char_size, Size},
        utils::resolve_rgba_with_palette,
        ImageRendererError,
    },
    window_decoration::Fonts,
};
use ab_glyph::PxScale;
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use termwiz::cell::{CellAttributes, Intensity, Underline};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};
use tracing::warn;

bitflags::bitflags! {
    pub struct Corners: u8 {
        const TOP_LEFT     = 0b0001;
        const TOP_RIGHT    = 0b0010;
        const BOTTOM_RIGHT = 0b0100;
        const BOTTOM_LEFT  = 0b1000;
        const ALL = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits() | Self::BOTTOM_RIGHT.bits() | Self::BOTTOM_LEFT.bits();
    }
}

#[derive(Debug)]
pub struct Canvas {
    pixmap: Pixmap,
    image_for_text: RgbaImage,
    font: Fonts,
    scale: PxScale,
    char_size: Size,
}

impl Canvas {
    pub fn new(
        width: u32,
        height: u32,
        font: Fonts,
        scale: PxScale,
    ) -> Result<Self, ImageRendererError> {
        let pixmap = Pixmap::new(width, height).ok_or(ImageRendererError::CanvasInitFailed)?;
        let image_for_text = RgbaImage::new(width, height);
        let char_size = calculate_char_size(&font.regular, scale);

        Ok(Self {
            pixmap,
            image_for_text,
            font,
            scale,
            char_size,
        })
    }

    pub fn fill(&mut self, color: Rgba<u8>) {
        self.pixmap
            .fill(Color::from_rgba8(color[0], color[1], color[2], color[3]));
    }

    pub fn fill_rounded(&mut self, color: Rgba<u8>, radius: f32, corners: &Corners) {
        self.fill_rounded_rect(
            0,
            0,
            self.pixmap.width(),
            self.pixmap.height(),
            color,
            radius,
            corners,
        );
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: Rgba<u8>) {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(color[0], color[1], color[2], color[3]));
            self.pixmap
                .fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_rounded_rect(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Rgba<u8>,
        radius: f32,
        corners: &Corners,
    ) {
        let x = x as f32;
        let y = y as f32;
        let width = width as f32;
        let height = height as f32;

        let mut pb = PathBuilder::new();

        if corners.contains(Corners::TOP_LEFT) {
            pb.move_to(x + radius, y);
        } else {
            pb.move_to(x, y);
        }

        if corners.contains(Corners::TOP_RIGHT) {
            pb.line_to(x + width - radius, y);
            pb.quad_to(x + width, y, x + width, y + radius);
        } else {
            pb.line_to(x + width, y);
        }

        if corners.contains(Corners::BOTTOM_RIGHT) {
            pb.line_to(x + width, y + height - radius);
            pb.quad_to(x + width, y + height, x + width - radius, y + height);
        } else {
            pb.line_to(x + width, y + height);
        }

        if corners.contains(Corners::BOTTOM_LEFT) {
            pb.line_to(x + radius, y + height);
            pb.quad_to(x, y + height, x, y + height - radius);
        } else {
            pb.line_to(x, y + height);
        }

        if corners.contains(Corners::TOP_LEFT) {
            pb.line_to(x, y + radius);
            pb.quad_to(x, y, x + radius, y);
        } else {
            pb.line_to(x, y);
        }

        let Some(path) = pb.finish() else {
            warn!("Failed to build rounded rect path");
            return;
        };

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color[0], color[1], color[2], color[3]));

        self.pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: i32, color: Rgba<u8>) {
        if let Some(path) = PathBuilder::from_circle(x as f32, y as f32, radius as f32) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(color[0], color[1], color[2], color[3]));

            self.pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color_palette: &[Rgba<u8>; 256],
        attributes: &CellAttributes,
    ) {
        let foreground_color = if attributes.reverse() {
            color_palette[0]
        } else {
            resolve_rgba_with_palette(color_palette, attributes.foreground())
                .unwrap_or(color_palette[7])
        };

        let is_bold = matches!(attributes.intensity(), Intensity::Bold);

        let font = if is_bold && attributes.italic() {
            &self.font.bold_italic
        } else if is_bold {
            &self.font.bold
        } else if attributes.italic() {
            &self.font.italic
        } else {
            &self.font.regular
        };

        draw_text_mut(
            &mut self.image_for_text,
            foreground_color,
            x,
            y,
            self.scale,
            &font,
            text,
        );

        self.draw_cell_attributes(text, x, y, color_palette, attributes);
    }

    pub fn draw_cell_attributes(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color_palette: &[Rgba<u8>; 256],
        attributes: &CellAttributes,
    ) {
        let bg_color = if attributes.reverse() {
            resolve_rgba_with_palette(color_palette, attributes.foreground())
                .or(Some(color_palette[7]))
        } else {
            resolve_rgba_with_palette(color_palette, attributes.background())
        };
        if let Some(bg_color) = bg_color {
            self.fill_rect(
                x,
                y,
                text.chars().count() as u32 * self.char_width(),
                self.char_height(),
                bg_color,
            );
        }

        if attributes.underline() != Underline::None {
            let underline_color =
                resolve_rgba_with_palette(color_palette, attributes.underline_color())
                    .unwrap_or(color_palette[7]);
            self.fill_rect(
                x,
                y + self.char_height() as i32,
                text.chars().count() as u32 * self.char_width(),
                1,
                underline_color,
            );
        }

        if attributes.strikethrough() {
            self.fill_rect(
                x,
                y + (self.char_height() as i32 / 2),
                text.chars().count() as u32 * self.char_width(),
                1,
                color_palette[7],
            );
        }
    }

    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    pub fn char_width(&self) -> u32 {
        self.char_size.width
    }

    pub fn char_height(&self) -> u32 {
        self.char_size.height
    }

    pub fn to_final_image(&self) -> Result<RgbaImage, ImageRendererError> {
        let mut final_image = RgbaImage::from_raw(
            self.pixmap.width(),
            self.pixmap.height(),
            self.pixmap.data().to_vec(),
        )
        .ok_or(ImageRendererError::ImageCreationFailed)?;

        for (final_pixel, text_pixel) in final_image.pixels_mut().zip(self.image_for_text.pixels())
        {
            let alpha = text_pixel[3] as f32 / 255.0;
            if alpha > 0.0 {
                for i in 0..3 {
                    final_pixel[i] = (text_pixel[i] as f32)
                        .mul_add(alpha, final_pixel[i] as f32 * (1.0 - alpha))
                        as u8;
                }
                final_pixel[3] = 255;
            }
        }

        Ok(final_image)
    }
}

#[cfg(test)]
mod tests {
    use crate::window_decoration::common::{default_font, get_default_color_palette};

    use super::*;
    use image::Rgba;

    fn make_font() -> Fonts {
        default_font().unwrap().clone()
    }

    #[test]
    fn canvas_creation() {
        let font = make_font();
        let c = Canvas::new(100, 50, font, 16.0.into());
        assert!(c.is_ok());
        let c = c.unwrap();
        assert_eq!(c.width(), 100);
        assert_eq!(c.height(), 50);
    }

    #[test]
    fn fill_and_fill_rect() {
        let font = make_font();
        let mut c = Canvas::new(50, 30, font, 12.0.into()).unwrap();
        c.fill(Rgba([255, 0, 0, 255]));
        c.fill_rect(5, 5, 10, 10, Rgba([0, 255, 0, 255]));
        c.fill_rounded(Rgba([0, 0, 255, 255]), 5.0, &Corners::ALL);
    }

    #[test]
    fn draw_shapes_and_text() {
        let font = make_font();
        let mut c = Canvas::new(60, 40, font, 12.0.into()).unwrap();
        c.draw_circle(20, 20, 10, Rgba([255, 255, 0, 255]));
        c.draw_text(
            "Hello",
            5,
            5,
            &get_default_color_palette(),
            &CellAttributes::default(),
        );
    }

    #[test]
    fn final_image_has_correct_dimensions() {
        let font = make_font();
        let mut c = Canvas::new(80, 60, font, 14.0.into()).unwrap();
        c.fill(Rgba([100, 100, 100, 255]));
        let img = c.to_final_image().unwrap();
        assert_eq!(img.width(), 80);
        assert_eq!(img.height(), 60);
    }
}
