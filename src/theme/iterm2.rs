use image::Rgba;
use plist::from_bytes;
use serde::Deserialize;
use std::{fs, io::Read, path::Path};
use thiserror::Error;

use crate::theme::{Theme, build_256_palette};

#[derive(Debug, Error)]
pub enum ITermError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("PLIST parsing failed: {0}")]
    PlistError(#[from] plist::Error),
}

#[derive(Deserialize, Debug)]
struct Color {
    #[serde(rename = "Red Component")]
    red: f32,
    #[serde(rename = "Green Component")]
    green: f32,
    #[serde(rename = "Blue Component")]
    blue: f32,
}

impl From<Color> for Rgba<u8> {
    fn from(val: Color) -> Self {
        Self([
            (val.red.clamp(0.0, 1.0) * 255.0).round() as u8,
            (val.green.clamp(0.0, 1.0) * 255.0).round() as u8,
            (val.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
            255,
        ])
    }
}

#[derive(Deserialize, Debug)]
#[expect(dead_code)]
pub struct ITerm2 {
    #[serde(rename = "Ansi 0 Color")]
    ansi_0: Color,
    #[serde(rename = "Ansi 1 Color")]
    ansi_1: Color,
    #[serde(rename = "Ansi 2 Color")]
    ansi_2: Color,
    #[serde(rename = "Ansi 3 Color")]
    ansi_3: Color,
    #[serde(rename = "Ansi 4 Color")]
    ansi_4: Color,
    #[serde(rename = "Ansi 5 Color")]
    ansi_5: Color,
    #[serde(rename = "Ansi 6 Color")]
    ansi_6: Color,
    #[serde(rename = "Ansi 7 Color")]
    ansi_7: Color,
    #[serde(rename = "Ansi 8 Color")]
    ansi_8: Color,
    #[serde(rename = "Ansi 9 Color")]
    ansi_9: Color,
    #[serde(rename = "Ansi 10 Color")]
    ansi_10: Color,
    #[serde(rename = "Ansi 11 Color")]
    ansi_11: Color,
    #[serde(rename = "Ansi 12 Color")]
    ansi_12: Color,
    #[serde(rename = "Ansi 13 Color")]
    ansi_13: Color,
    #[serde(rename = "Ansi 14 Color")]
    ansi_14: Color,
    #[serde(rename = "Ansi 15 Color")]
    ansi_15: Color,
    #[serde(rename = "Background Color")]
    background: Color,
    #[serde(rename = "Bold Color")]
    #[allow(dead_code)]
    bold: Color,
    #[serde(rename = "Cursor Color")]
    cursor: Color,
    #[serde(rename = "Cursor Text Color")]
    cursor_text: Color,
    #[serde(rename = "Foreground Color")]
    foreground: Color,
    #[serde(rename = "Selected Text Color")]
    selected_text: Color,
    #[serde(rename = "Selection Color")]
    selection: Color,
}

impl ITerm2 {
    pub fn parse_str(s: &str) -> Result<Theme, ITermError> {
        let theme: Self = from_bytes(s.as_bytes())?;

        let ansi = [
            theme.ansi_0.into(),
            theme.ansi_1.into(),
            theme.ansi_2.into(),
            theme.ansi_3.into(),
            theme.ansi_4.into(),
            theme.ansi_5.into(),
            theme.ansi_6.into(),
            theme.ansi_7.into(),
            theme.ansi_8.into(),
            theme.ansi_9.into(),
            theme.ansi_10.into(),
            theme.ansi_11.into(),
            theme.ansi_12.into(),
            theme.ansi_13.into(),
            theme.ansi_14.into(),
            theme.ansi_15.into(),
        ];

        Ok(Theme {
            palette: build_256_palette(ansi),
            foreground_color: theme.foreground.into(),
            background_color: theme.background.into(),
        })
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Theme, ITermError> {
        let data = fs::read_to_string(path)?;

        Self::parse_str(&data)
    }

    pub fn load_reader<R: Read>(mut reader: R) -> Result<Theme, ITermError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Self::parse_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const VALID_PLIST: &str = include_str!("../../assets/tests/iterm_test.itermcolors");

    #[test]
    fn test_parse_valid_plist() {
        let theme = ITerm2::parse_str(VALID_PLIST).expect("Failed to parse valid plist");
        assert_eq!(theme.foreground_color, Rgba([230, 204, 179, 255]));
        assert_eq!(theme.background_color, Rgba([26, 51, 77, 255]));
        assert_eq!(theme.palette.len(), 256);
    }

    #[test]
    fn test_parse_invalid_plist() {
        let invalid_plist = "<plist>not valid</plist>";
        let err = ITerm2::parse_str(invalid_plist).unwrap_err();
        matches!(err, ITermError::PlistError(_));
    }

    #[test]
    fn test_color_conversion() {
        let color = Color {
            red: 0.5,
            green: 0.25,
            blue: 0.75,
        };
        let rgba: Rgba<u8> = color.into();
        assert_eq!(rgba, Rgba([128, 64, 191, 255]));
    }

    #[test]
    fn test_load_reader_valid() {
        let cursor = Cursor::new(VALID_PLIST);
        let theme = ITerm2::load_reader(cursor).expect("Failed to load reader");
        assert_eq!(theme.foreground_color, Rgba([230, 204, 179, 255]));
    }

    #[test]
    fn test_load_file_nonexistent() {
        let result = ITerm2::load_file("this_file_should_not_exist.plist");
        matches!(result.unwrap_err(), ITermError::Io(_));
    }
}
