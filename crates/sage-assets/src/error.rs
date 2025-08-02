use chia::protocol::Bytes32;
use thiserror::Error;

use crate::ThumbnailError;

#[derive(Debug, Error)]
pub enum UriError {
    #[error("Failed to fetch NFT data: {0}")]
    Fetch(#[from] reqwest::Error),

    #[error("Missing or invalid content type")]
    InvalidContentType,

    #[error("Mime type mismatch, expected {expected} but found {found}")]
    MimeTypeMismatch { expected: String, found: String },

    #[error("Hash mismatch, expected {expected} but found {found}")]
    HashMismatch { expected: Bytes32, found: Bytes32 },

    #[error("No URIs provided")]
    NoUris,

    #[error("Failed to create thumbnail: {0}")]
    Thumbnail(#[from] ThumbnailError),
}
