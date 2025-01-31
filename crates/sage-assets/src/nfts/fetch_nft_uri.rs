use chia::protocol::Bytes32;
use clvmr::sha2::Sha256;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use mime_sniffer::MimeTypeSniffer;
use reqwest::header::CONTENT_TYPE;
use thiserror::Error;
use tracing::debug;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use percent_encoding::percent_decode;

const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024; // 10MB limit for text content

#[derive(Debug, Clone)]
pub struct Data {
    pub blob: Vec<u8>,
    pub mime_type: String,
    pub hash: Bytes32,
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

    #[error("Invalid data URI format")]
    InvalidDataUri,

    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error("Text content too large")]
    TextContentTooLarge,

    #[error("Invalid UTF-8 encoding")]
    InvalidUtf8Encoding,
}

fn validate_text_content(content: &[u8]) -> Result<(), UriError> {
    // Check size limit
    if content.len() > MAX_TEXT_SIZE {
        return Err(UriError::TextContentTooLarge);
    }

    // Validate UTF-8
    String::from_utf8(content.to_vec())
        .map_err(|_| UriError::InvalidUtf8Encoding)?;

    Ok(())
}

pub async fn fetch_uri(uri: String) -> Result<Data, UriError> {
    // Check if it's a data URI
    if uri.starts_with("data:") {
        return parse_data_uri(&uri);
    }

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

    // Validate text content if mime type is text/plain
    if mime_type == "text/plain" {
        validate_text_content(&blob)?;
    }

    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let hash = Bytes32::new(hasher.finalize());

    Ok(Data {
        blob,
        mime_type,
        hash,
    })
}

fn parse_data_uri(uri: &str) -> Result<Data, UriError> {
    // Remove "data:" prefix
    let content = uri.strip_prefix("data:").ok_or(UriError::InvalidDataUri)?;

    // Split into metadata and data parts
    let parts: Vec<&str> = content.split(',').collect();
    if parts.len() != 2 {
        return Err(UriError::InvalidDataUri);
    }

    let (metadata, data) = (parts[0], parts[1]);

    // Parse mime type and encoding
    let (mime_type, is_base64) = if metadata.ends_with(";base64") {
        (metadata[..metadata.len() - 7].to_string(), true)
    } else {
        (metadata.to_string(), false)
    };

    // Decode the data
    let blob = if is_base64 {
        BASE64.decode(data)?
    } else {
        // For non-base64, handle percent encoding
        percent_decode(data.as_bytes())
            .decode_utf8()
            .map_err(|_| UriError::InvalidDataUri)?
            .to_string()
            .as_bytes()
            .to_vec()
    };

    // Validate text content if mime type is text/plain
    if mime_type == "text/plain" {
        validate_text_content(&blob)?;
    }

    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let hash = Bytes32::new(hasher.finalize());

    Ok(Data {
        blob,
        mime_type,
        hash,
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
