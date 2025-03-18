use std::time::{Duration, Instant};

use chia::protocol::Bytes32;
use chia::sha2::Sha256;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use mime_sniffer::MimeTypeSniffer;
use reqwest::{header::CONTENT_TYPE, StatusCode};
use thiserror::Error;
use tokio::task::spawn_blocking;
use tracing::debug;

use super::{thumbnail as make_thumbnail, Thumbnail, ThumbnailError};

#[derive(Debug, Clone)]
pub struct Data {
    pub blob: Vec<u8>,
    pub mime_type: String,
    pub hash: Bytes32,
    pub thumbnail: Option<Thumbnail>,
}

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

pub async fn fetch_uri(uri: String) -> Result<Data, UriError> {
    let response = reqwest::get(&uri).await?;

    let mime_type = match response.headers().get(CONTENT_TYPE) {
        Some(header) => Some(
            header
                .to_str()
                .map(ToString::to_string)
                .map_err(|_| UriError::InvalidContentType)?,
        ),
        None => None,
    };

    let blob = response.bytes().await?.to_vec();

    let mime_type = if let Some(mime_type) = mime_type {
        mime_type
    } else {
        blob.as_slice()
            .sniff_mime_type()
            .unwrap_or("image/png")
            .to_string()
    };

    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let hash = Bytes32::new(hasher.finalize());

    let mut thumbnail = match mintgarden_thumbnail(hash).await {
        Ok(thumbnail) => thumbnail,
        Err(error) => {
            debug!("Failed to fetch MintGarden thumbnail for {uri}: {error}");
            None
        }
    };

    if thumbnail.is_none() {
        let start = Instant::now();

        let blob_clone = blob.clone();
        let mime_type_clone = mime_type.clone();

        thumbnail =
            match spawn_blocking(move || make_thumbnail(&blob_clone, &mime_type_clone)).await {
                Ok(Ok(thumbnail)) => thumbnail,
                Ok(Err(error)) => {
                    debug!("No thumbnail created for {uri}: {error}");
                    None
                }
                Err(error) => {
                    debug!("Failed to create thumbnail for {uri}: {error}");
                    None
                }
            };

        let elapsed = start.elapsed();

        if elapsed > Duration::from_millis(50) {
            debug!("Thumbnail creation took {elapsed:?} for {uri}");
        }
    }

    Ok(Data {
        blob,
        mime_type,
        hash,
        thumbnail,
    })
}

pub async fn fetch_uris_without_hash(uris: Vec<String>) -> Result<Data, UriError> {
    let mut futures = FuturesUnordered::new();

    for uri in uris {
        futures.push(fetch_uri(uri));
    }

    let mut data = None;

    while let Some(result) = futures.next().await {
        let item = result?;

        let Some(old_data) = data.take() else {
            data = Some(item);
            continue;
        };

        if old_data.hash != item.hash {
            return Err(UriError::HashMismatch {
                expected: old_data.hash,
                found: item.hash,
            });
        }

        if old_data.mime_type != item.mime_type {
            return Err(UriError::MimeTypeMismatch {
                expected: old_data.mime_type,
                found: item.mime_type,
            });
        }

        data = Some(old_data);
    }

    data.ok_or(UriError::NoUris)
}

pub async fn fetch_uris_with_hash(uris: Vec<String>, hash: Bytes32) -> Option<Data> {
    let mut futures = FuturesUnordered::new();

    for uri in uris {
        futures.push(async move { (uri.clone(), fetch_uri(uri).await) });
    }

    while let Some((uri, result)) = futures.next().await {
        let Ok(item) = result else {
            debug!("Failed to fetch NFT URI {uri}, expected hash {hash}");
            continue;
        };

        if hash != item.hash {
            return None;
        }

        return Some(item);
    }

    None
}

pub async fn mintgarden_thumbnail(data_hash: Bytes32) -> Result<Option<Thumbnail>, UriError> {
    let url = format!("https://assets.mainnet.mintgarden.io/thumbnails/{data_hash}_512.webp");

    let response = reqwest::get(&url).await?;

    if response.status() != StatusCode::OK {
        return Ok(None);
    }

    let bytes = response.bytes().await?;

    Ok(make_thumbnail(&bytes, "image/webp")?)
}
