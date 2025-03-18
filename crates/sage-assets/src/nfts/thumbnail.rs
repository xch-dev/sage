use std::io::{self, Cursor};

use image::{DynamicImage, ImageFormat, ImageReader};
use thiserror::Error;
use webp::Decoder;

#[derive(Debug, Error)]
pub enum ThumbnailError {
    #[error("Failed to load image: {0}")]
    Load(#[from] image::ImageError),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to decode webp image")]
    Webp,
}

#[derive(Debug, Clone)]
pub struct Thumbnail {
    pub icon: Vec<u8>,
    pub thumbnail: Vec<u8>,
}

pub fn thumbnail(bytes: &[u8], mime: &str) -> Result<Option<Thumbnail>, ThumbnailError> {
    let Some(image) = load_image(bytes, mime)? else {
        return Ok(None);
    };

    let icon = image.thumbnail(64, 64);
    let thumbnail = image.thumbnail(256, 256);

    let mut icon_bytes = Vec::new();
    let mut thumbnail_bytes = Vec::new();

    let mut icon_writer = Cursor::new(&mut icon_bytes);
    let mut thumbnail_writer = Cursor::new(&mut thumbnail_bytes);

    icon.write_to(&mut icon_writer, ImageFormat::Png)?;
    thumbnail.write_to(&mut thumbnail_writer, ImageFormat::Png)?;

    Ok(Some(Thumbnail {
        icon: icon_bytes,
        thumbnail: thumbnail_bytes,
    }))
}

fn load_image(bytes: &[u8], mime: &str) -> Result<Option<DynamicImage>, ThumbnailError> {
    if mime == "image/webp" {
        return Ok(Some(
            Decoder::new(bytes)
                .decode()
                .ok_or(ThumbnailError::Webp)?
                .to_image(),
        ));
    }

    let Some(format) = mime_to_image_format(mime) else {
        return Ok(None);
    };

    Ok(Some(
        ImageReader::with_format(Cursor::new(bytes), format).decode()?,
    ))
}

fn mime_to_image_format(mime: &str) -> Option<ImageFormat> {
    match mime {
        "image/png" => Some(ImageFormat::Png),
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/bmp" => Some(ImageFormat::Bmp),
        "image/gif" => Some(ImageFormat::Gif),
        _ => None,
    }
}
