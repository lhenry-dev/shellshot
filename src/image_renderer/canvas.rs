use crate::image_renderer::{render_size::calculate_char_size, ImageRendererError};
use ab_glyph::{FontArc, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Rect, Size, Transform};

#[derive(Debug)]
pub struct Canvas {
    pixmap: Pixmap,
    image_for_text: RgbaImage,
    font: FontArc,
    default_fg_color: Rgba<u8>,
    scale: PxScale,
    char_size: Size,
}

impl Canvas {
    pub fn new(
        width: u32,
        height: u32,
        font: FontArc,
        default_fg_color: Rgba<u8>,
        scale: PxScale,
    ) -> Result<Self, ImageRendererError> {
        let pixmap = Pixmap::new(width, height).ok_or(ImageRendererError::CanvasInitFailed)?;
        let image_for_text = RgbaImage::new(width, height);
        let char_size = calculate_char_size(&font, scale);

        Ok(Self {
            pixmap,
            image_for_text,
            font,
            default_fg_color,
            scale,
            char_size,
        })
    }

    pub fn fill(&mut self, color: Rgba<u8>) {
        self.pixmap
            .fill(Color::from_rgba8(color[0], color[1], color[2], color[3]));
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: Rgba<u8>) {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(color[0], color[1], color[2], color[3]));
            self.pixmap
                .fill_rect(rect, &paint, Transform::identity(), None);
        }
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
        color: Option<Rgba<u8>>,
        background: Option<Rgba<u8>>,
    ) {
        let fg = color.unwrap_or(self.default_fg_color);

        if let Some(bg_color) = background {
            self.fill_rect(
                x,
                y,
                text.chars().count() as u32 * self.char_width(),
                self.char_height(),
                bg_color,
            );
        }

        draw_text_mut(
            &mut self.image_for_text,
            fg,
            x,
            y,
            self.scale,
            &self.font,
            text,
        );
    }

    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    pub fn char_width(&self) -> u32 {
        self.char_size.width() as u32
    }

    pub fn char_height(&self) -> u32 {
        self.char_size.height() as u32
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
                    final_pixel[i] = (text_pixel[i] as f32 * alpha
                        + final_pixel[i] as f32 * (1.0 - alpha))
                        as u8;
                }
                final_pixel[3] = 255;
            }
        }

        Ok(final_image)
    }
}
