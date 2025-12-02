use arboard::Clipboard;
use image::{EncodableLayout, ImageBuffer, ImageFormat, Rgba};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to decode image")]
    ImageDecode(#[from] image::ImageError),

    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),
}

/// Save the provided RGBA image buffer to the given filename path.
///
/// This will create parent directories if they do not exist. If the file
/// extension is "png" the image will be encoded as PNG, otherwise the raw
/// image bytes are written to the file.
///
/// # Errors
///
/// Returns an error if:
/// - I/O operations fail (e.g., directory creation or file writing)
/// - Image encoding fails
pub fn save_to_file(
    image_data: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    output: &str,
) -> Result<(), SaveError> {
    let path = Path::new(output);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if path.extension().and_then(|s| s.to_str()) == Some("png") {
        image_data.save_with_format(path, ImageFormat::Png)?;
    } else {
        std::fs::write(path, image_data.as_bytes())?;
    }

    Ok(())
}

/// Copy the provided RGBA image buffer to the system clipboard.
///
/// The image bytes are provided as raw RGBA samples to arboard.
///
/// # Errors
///
/// Returns an error if clipboard operations fail.
pub fn save_to_clipboard(image_data: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<(), SaveError> {
    let mut clipboard = Clipboard::new().map_err(SaveError::Clipboard)?;

    let flat_samples = image_data.as_flat_samples();

    clipboard
        .set_image(arboard::ImageData {
            width: image_data.width() as usize,
            height: image_data.height() as usize,
            bytes: std::borrow::Cow::Borrowed(flat_samples.as_slice()),
        })
        .map_err(SaveError::Clipboard)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn sample_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_pixel(2, 2, Rgba([255, 0, 0, 255]))
    }

    #[test]
    fn test_save_to_png_in_tempdir() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("image.png");
        let image = sample_image();

        let result = save_to_file(&image, file_path.to_str().unwrap());
        assert!(result.is_ok(), "Expected Ok, got {result:?}");

        assert!(file_path.exists());
        let bytes = fs::read(&file_path).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_save_to_file_creates_nested_dirs() {
        let tmp = tempdir().unwrap();
        let nested = tmp.path().join("nested/folder/image.png");
        let image = sample_image();

        let result = save_to_file(&image, nested.to_str().unwrap());
        assert!(result.is_ok());
        assert!(nested.exists());
    }

    // INCOMPATIBLE WITH CI ENVIRONMENT
    #[test]
    fn test_save_to_clipboard() {
        let image = sample_image();
        let result = save_to_clipboard(&image);

        if let Err(SaveError::Clipboard(err)) = &result {
            match err {
                arboard::Error::ClipboardNotSupported => return,
                arboard::Error::Unknown { .. } => return,
                _ => panic!("Unexpected clipboard error: {err:?}"),
            }
        }

        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling_for_invalid_path() {
        let tmp = tempdir().unwrap();
        let dir_as_file = tmp.path();
        let image = sample_image();

        let result = save_to_file(&image, dir_as_file.to_str().unwrap());
        assert!(result.is_err());
    }
}
