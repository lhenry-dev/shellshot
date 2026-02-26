use std::{io::Cursor, path::Path};

use image::Rgba;
use reqwest::Url;
use thiserror::Error;

use crate::theme::{
    base16::{Base16, Base16Error},
    iterm2::{ITerm2, ITermError},
};
use reqwest::blocking::get;

mod base16;
mod iterm2;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Unsupported theme extension: {0}")]
    UnsupportedExtension(String),
    #[error("Could not determine file format")]
    UnknownFormat,
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Failed to read response bytes: {0}")]
    Bytes(#[from] std::io::Error),
    #[error(transparent)]
    Base16(#[from] Base16Error),
    #[error(transparent)]
    ITerm(#[from] ITermError),
}

#[derive(Debug, Clone)]
pub struct Theme {
    /// Full ANSI palette (0–255)
    pub palette: [Rgba<u8>; 256],
    /// Default text color
    pub foreground_color: Rgba<u8>,
    /// Default background color
    pub background_color: Rgba<u8>,
}

impl Default for Theme {
    fn default() -> Self {
        let ansi: [Rgba<u8>; 16] = [
            // 0  : dark background
            Rgba([0x28, 0x2c, 0x34, 0xff]),
            // 1–6 : standard ANSI colors
            Rgba([0xc0, 0x00, 0x00, 0xff]),
            Rgba([0x00, 0xc0, 0x00, 0xff]),
            Rgba([0xc0, 0xc0, 0x00, 0xff]),
            Rgba([0x00, 0x00, 0xc0, 0xff]),
            Rgba([0xc0, 0x00, 0xc0, 0xff]),
            Rgba([0x00, 0xc0, 0xc0, 0xff]),
            // 7  : light foreground
            Rgba([0xf5, 0xf1, 0xe5, 0xff]),
            // 8  : bright black
            Rgba([0x80, 0x80, 0x80, 0xff]),
            // 9–14 : bright standard ANSI colors
            Rgba([0xff, 0x00, 0x00, 0xff]),
            Rgba([0x00, 0xff, 0x00, 0xff]),
            Rgba([0xff, 0xff, 0x00, 0xff]),
            Rgba([0x00, 0x00, 0xff, 0xff]),
            Rgba([0xff, 0x00, 0xff, 0xff]),
            Rgba([0x00, 0xff, 0xff, 0xff]),
            // 15 : bright white
            Rgba([0xff, 0xff, 0xff, 0xff]),
        ];

        let palette = build_256_palette(ansi);

        Self {
            palette,
            foreground_color: palette[7], // light foreground
            background_color: palette[0], // dark background
        }
    }
}

impl Theme {
    pub fn load<S: AsRef<str>>(source: S) -> Result<Self, ThemeError> {
        let source = source.as_ref();

        if let Ok(url) = Url::parse(source) {
            if url.scheme() == "http" || url.scheme() == "https" {
                return Self::load_from_url(source);
            }
        }

        Self::load_from_path(source)
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ThemeError> {
        let path = path.as_ref();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ThemeError::UnknownFormat)?;

        match extension.to_ascii_lowercase().as_str() {
            "yaml" | "yml" => Ok(Base16::load_file(path)?),
            "itermcolors" => Ok(ITerm2::load_file(path)?),
            _ => Err(ThemeError::UnsupportedExtension(extension.into())),
        }
    }

    pub fn load_from_url(url: &str) -> Result<Self, ThemeError> {
        let resp = get(url)?;
        let bytes = resp.bytes()?;

        let extension = Path::new(url)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ThemeError::UnknownFormat)?;

        match extension.to_ascii_lowercase().as_str() {
            "yaml" | "yml" => {
                let cursor = Cursor::new(bytes);
                Ok(Base16::load_reader(cursor)?)
            }
            "itermcolors" => {
                let cursor = Cursor::new(bytes);
                Ok(ITerm2::load_reader(cursor)?)
            }
            _ => Err(ThemeError::UnsupportedExtension(extension.into())),
        }
    }
}

/// Generates a full ANSI 256-color palette compatible with xterm.
///
/// Layout:
/// - 0–15   : provided ANSI base colors
/// - 16–231 : 6×6×6 RGB color cube
/// - 232–255: grayscale ramp
pub fn build_256_palette(base16: [Rgba<u8>; 16]) -> [Rgba<u8>; 256] {
    // Official xterm 6-level RGB values
    const COLORS6: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

    std::array::from_fn(|idx| {
        match idx {
            // 0–15 : ANSI colors provided by the theme
            0..=15 => base16[idx],

            // 16–231 : 6×6×6 color cube (216 colors)
            16..=231 => {
                let i = idx - 16;

                let r = COLORS6[(i / 36) % 6];
                let g = COLORS6[(i / 6) % 6];
                let b = COLORS6[i % 6];

                Rgba([r, g, b, 255])
            }

            // 232–255 : grayscale ramp (24 levels)
            232..=255 => {
                let level = 8 + ((idx - 232) as u8) * 10;
                Rgba([level, level, level, 255])
            }

            _ => unreachable!(),
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::theme::base16::hex_to_rgba;

    use super::*;
    use image::Rgba;
    use std::io::Cursor;

    const VALID_YAML: &str = include_str!("../assets/tests/base16_test.yaml");
    const VALID_PLIST: &str = include_str!("../assets/tests/iterm_test.itermcolors");

    #[test]
    fn test_default_theme_palette() {
        let theme = Theme::default();
        assert_eq!(theme.palette.len(), 256);
        assert_eq!(theme.foreground_color, theme.palette[7]);
        assert_eq!(theme.background_color, theme.palette[0]);
    }

    #[test]
    fn test_build_256_palette_correctness() {
        let base16 = [Rgba([0, 0, 0, 255]); 16];
        let palette = build_256_palette(base16);
        assert_eq!(palette.len(), 256);
        for i in 0..16 {
            assert_eq!(palette[i], base16[i]);
        }

        palette[232..].iter().enumerate().for_each(|(idx, color)| {
            let val = 8 + (idx as u8) * 10;
            assert_eq!(*color, Rgba([val, val, val, 255]));
        });
    }

    #[test]
    fn test_load_from_path_base16() {
        let cursor = Cursor::new(VALID_YAML);
        let theme = Base16::load_reader(cursor).expect("Base16 load_reader failed");
        assert_eq!(theme.foreground_color, hex_to_rgba("#c5c8c6").unwrap());
    }

    #[test]
    fn test_load_from_path_iterm() {
        let cursor = Cursor::new(VALID_PLIST);
        let theme = ITerm2::load_reader(cursor).expect("ITerm2 load_reader failed");
        assert_eq!(theme.foreground_color, Rgba([230, 204, 179, 255]));
        assert_eq!(theme.background_color, Rgba([26, 51, 77, 255]));
    }

    #[test]
    fn test_theme_error_unsupported_extension() {
        let path = Path::new("theme.txt");
        let err = Theme::load_from_path(path).unwrap_err();
        matches!(err, ThemeError::UnsupportedExtension(_));
    }

    #[test]
    fn test_theme_error_unknown_format() {
        let path = Path::new("theme");
        let err = Theme::load_from_path(path).unwrap_err();
        matches!(err, ThemeError::UnknownFormat);
    }

    #[test]
    fn test_load_from_url_invalid() {
        let url = "ht!tp://invalid";
        let err = Theme::load(url).unwrap_err();
        matches!(err, ThemeError::Http(_));
    }
}
