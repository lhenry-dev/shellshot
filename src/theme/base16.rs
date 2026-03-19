use image::Rgba;
use serde::Deserialize;
use thiserror::Error;

use crate::theme::{Theme, build_256_palette};

#[derive(Debug, Error)]
pub enum Base16Error {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML deserialization failed: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Hex string has invalid length: {0}")]
    InvalidLength(String),
    #[error("Failed to parse hex string `{hex}`: {source}")]
    ParseError {
        hex: String,
        #[source]
        source: std::num::ParseIntError,
    },
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case, dead_code)]
pub struct Base16 {
    scheme: String,
    author: String,
    base00: String,
    base01: String,
    base02: String,
    base03: String,
    base04: String,
    base05: String,
    base06: String,
    base07: String,
    base08: String,
    base09: String,
    base0A: String,
    base0B: String,
    base0C: String,
    base0D: String,
    base0E: String,
    base0F: String,
}

impl Base16 {
    pub fn load_bytes(bytes: &[u8]) -> Result<Theme, Base16Error> {
        let theme: Self = serde_yaml::from_slice(bytes)?;

        let ansi: [Rgba<u8>; 16] = [
            hex_to_rgba(&theme.base00)?, // 0
            hex_to_rgba(&theme.base08)?, // 1
            hex_to_rgba(&theme.base0B)?, // 2
            hex_to_rgba(&theme.base0A)?, // 3
            hex_to_rgba(&theme.base0D)?, // 4
            hex_to_rgba(&theme.base0E)?, // 5
            hex_to_rgba(&theme.base0C)?, // 6
            hex_to_rgba(&theme.base05)?, // 7
            hex_to_rgba(&theme.base03)?, // 8
            hex_to_rgba(&theme.base08)?, // 9
            hex_to_rgba(&theme.base0B)?, // 10
            hex_to_rgba(&theme.base0A)?, // 11
            hex_to_rgba(&theme.base0D)?, // 12
            hex_to_rgba(&theme.base0E)?, // 13
            hex_to_rgba(&theme.base0C)?, // 14
            hex_to_rgba(&theme.base07)?, // 15
        ];

        Ok(Theme {
            palette: build_256_palette(ansi),
            foreground_color: hex_to_rgba(&theme.base05)?,
            background_color: hex_to_rgba(&theme.base00)?,
        })
    }
}

pub fn hex_to_rgba(hex: &str) -> Result<Rgba<u8>, Base16Error> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        6 => Ok(Rgba([
            parse_hex_byte(&hex[0..2])?,
            parse_hex_byte(&hex[2..4])?,
            parse_hex_byte(&hex[4..6])?,
            255,
        ])),
        8 => Ok(Rgba([
            parse_hex_byte(&hex[0..2])?,
            parse_hex_byte(&hex[2..4])?,
            parse_hex_byte(&hex[4..6])?,
            parse_hex_byte(&hex[6..8])?,
        ])),
        _ => Err(Base16Error::InvalidLength(hex.to_string())),
    }
}

fn parse_hex_byte(hex: &str) -> Result<u8, Base16Error> {
    u8::from_str_radix(hex, 16).map_err(|e| Base16Error::ParseError {
        hex: hex.to_string(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_YAML: &[u8] = include_bytes!("../../assets/tests/base16_test.yaml");

    #[test]
    fn test_parse_valid_yaml() {
        let theme = Base16::load_bytes(VALID_YAML).expect("Failed to parse valid YAML");
        assert_eq!(theme.foreground_color, hex_to_rgba("#c5c8c6").unwrap());
        assert_eq!(theme.background_color, hex_to_rgba("#1d1f21").unwrap());
        assert_eq!(theme.palette.len(), 256);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid_yaml = "not: valid: yaml".as_bytes();
        let err = Base16::load_bytes(invalid_yaml).unwrap_err();
        matches!(err, Base16Error::YamlError(_));
    }

    #[test]
    fn test_hex_to_rgba_6_digits() {
        let color = hex_to_rgba("#112233").unwrap();
        assert_eq!(color, Rgba([0x11, 0x22, 0x33, 255]));
    }

    #[test]
    fn test_hex_to_rgba_8_digits() {
        let color = hex_to_rgba("#11223344").unwrap();
        assert_eq!(color, Rgba([0x11, 0x22, 0x33, 0x44]));
    }

    #[test]
    fn test_hex_to_rgba_invalid_length() {
        let err = hex_to_rgba("#12345").unwrap_err();
        matches!(err, Base16Error::InvalidLength(_));
    }

    #[test]
    fn test_hex_to_rgba_invalid_hex() {
        let err = hex_to_rgba("#zz2233").unwrap_err();
        matches!(err, Base16Error::ParseError { .. });
    }
}
