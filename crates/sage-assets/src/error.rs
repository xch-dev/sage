use chia_wallet_sdk::prelude::*;
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

    #[error("Timed out fetching NFT data")]
    Timeout,

    #[error("Failed to create thumbnail: {0}")]
    Thumbnail(#[from] ThumbnailError),

    #[error("Invalid XCH/USD price response")]
    InvalidPriceResponse,
}

impl UriError {
    /// Whether the error is likely to go away if the request is retried later,
    /// such as a timeout, rate limit, or server error.
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Fetch(error) => {
                error.is_timeout()
                    || error.status().is_some_and(|status| {
                        status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
                    })
            }
            Self::Timeout => true,
            _ => false,
        }
    }
}
