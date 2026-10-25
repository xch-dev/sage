use std::{
    future::Future,
    pin::Pin,
    sync::LazyLock,
    time::{Duration, Instant},
};

use chia_wallet_sdk::{chia::sha2::Sha256, prelude::*};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use mime_sniffer::MimeTypeSniffer;
use reqwest::{Client, Response, StatusCode, header::CONTENT_TYPE};
use tokio::task::spawn_blocking;
use tracing::debug;

use crate::UriError;

use super::{Thumbnail, thumbnail as make_thumbnail, uri_candidates};

#[derive(Debug, Clone)]
pub struct Data {
    pub blob: Vec<u8>,
    pub mime_type: String,
    pub hash: Bytes32,
    pub thumbnail: Option<Thumbnail>,
}

/// MintGarden archives NFT data, metadata, and license files by their SHA-256
/// hash, so this works even after every URI baked into the NFT has died.
const MINTGARDEN_ARCHIVE_URL: &str = "https://archive.mintgarden.io/content";

/// The time limit for downloading a single file from a single source. This
/// only covers the network transfer, so a slow thumbnail doesn't cause a file
/// that was already downloaded to be thrown away.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(20);

/// MintGarden thumbnails are optional, so they get less time than the file.
const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(10);

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        // Some gateways sit behind bot protection that challenges clients
        // pretending to be a browser, so identify honestly as Sage.
        .user_agent(concat!("Sage/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default()
});

/// Fetches an NFT URI, translating `ipfs://` and `ar://` URIs to HTTPS
/// gateways and falling back to other gateways when an IPFS or Arweave
/// gateway link fails. See [`uri_candidates`].
pub async fn fetch_uri(uri: String, testnet: bool) -> Result<Data, UriError> {
    fetch_uri_inner(uri, None, testnet).await
}

/// Like [`fetch_uri`], but if a gateway serves content that doesn't match the
/// expected hash, the remaining gateways are tried before giving up. If none
/// of them match, the first mismatching file is returned.
pub async fn fetch_uri_with_hash(
    uri: String,
    hash: Bytes32,
    testnet: bool,
) -> Result<Data, UriError> {
    fetch_uri_inner(uri, Some(hash), testnet).await
}

async fn fetch_uri_inner(
    uri: String,
    expected_hash: Option<Bytes32>,
    testnet: bool,
) -> Result<Data, UriError> {
    let mut mismatch = None;
    let mut last_error = None;

    for url in uri_candidates(&uri) {
        let (blob, mime_type) = match download(&url).await {
            Ok(download) => download,
            Err(error) => {
                debug!("Failed to fetch {url} for NFT URI {uri}: {error}");
                last_error = Some(error);
                continue;
            }
        };

        let hash = sha256(&blob);

        if expected_hash.is_none_or(|expected| expected == hash) {
            return Ok(process_data(&uri, blob, mime_type, hash, testnet).await);
        }

        debug!("Hash mismatch for {url} (NFT URI {uri}), found {hash}");

        if mismatch.is_none() {
            mismatch = Some((blob, mime_type, hash));
        }
    }

    if let Some((blob, mime_type, hash)) = mismatch {
        return Ok(process_data(&uri, blob, mime_type, hash, testnet).await);
    }

    Err(last_error.unwrap_or(UriError::NoUris))
}

/// Fetches a file from MintGarden's archive by its hash. Returns `None` if
/// MintGarden doesn't have it, or if what it returns doesn't match the hash.
pub async fn fetch_mintgarden_archive(
    hash: Bytes32,
    testnet: bool,
) -> Result<Option<Data>, UriError> {
    let url = format!("{MINTGARDEN_ARCHIVE_URL}/{hash}");

    let response = CLIENT.get(&url).timeout(DOWNLOAD_TIMEOUT).send().await?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let (blob, mime_type) = read_response(response).await?;

    let found = sha256(&blob);

    if found != hash {
        debug!("Hash mismatch from MintGarden archive, expected {hash} but found {found}");
        return Ok(None);
    }

    Ok(Some(
        process_data(&url, blob, mime_type, found, testnet).await,
    ))
}

async fn download(url: &str) -> Result<(Vec<u8>, String), UriError> {
    read_response(CLIENT.get(url).timeout(DOWNLOAD_TIMEOUT).send().await?).await
}

async fn read_response(response: Response) -> Result<(Vec<u8>, String), UriError> {
    // Error pages (such as a gateway's 404, 429, or bot challenge) aren't the
    // file, so treat them as a failure rather than as mismatched content.
    let response = response.error_for_status()?;

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

    // Object storage (including MintGarden's archive) often serves files as a
    // generic binary type, which isn't useful for displaying them.
    let mime_type = match mime_type {
        Some(mime_type) if !is_generic_mime_type(&mime_type) => mime_type,
        _ => sniff_mime_type(&blob),
    };

    Ok((blob, mime_type))
}

fn sniff_mime_type(blob: &[u8]) -> String {
    if let Some(mime_type) = blob.sniff_mime_type() {
        return mime_type.to_string();
    }

    // The sniffer doesn't detect JSON, which metadata files are.
    if serde_json::from_slice::<serde::de::IgnoredAny>(blob).is_ok() {
        return "application/json".to_string();
    }

    "image/png".to_string()
}

fn is_generic_mime_type(mime_type: &str) -> bool {
    let essence = mime_type.split(';').next().unwrap_or_default().trim();
    essence.eq_ignore_ascii_case("application/octet-stream")
        || essence.eq_ignore_ascii_case("binary/octet-stream")
}

fn sha256(blob: &[u8]) -> Bytes32 {
    let mut hasher = Sha256::new();
    hasher.update(blob);
    Bytes32::new(hasher.finalize())
}

async fn process_data(
    uri: &str,
    blob: Vec<u8>,
    mime_type: String,
    hash: Bytes32,
    testnet: bool,
) -> Data {
    let mut thumbnail = match mintgarden_thumbnail(hash, testnet).await {
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

    Data {
        blob,
        mime_type,
        hash,
        thumbnail,
    }
}

pub async fn fetch_uris_without_hash(uris: Vec<String>, testnet: bool) -> Result<Data, UriError> {
    let mut futures = FuturesUnordered::new();

    for uri in uris {
        futures.push(fetch_uri(uri, testnet));
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

/// Fetches a file with a known hash, using MintGarden's archive alongside the
/// NFT's own URIs. The first source that produces matching content wins.
pub async fn fetch_uris_with_hash(uris: Vec<String>, hash: Bytes32, testnet: bool) -> Option<Data> {
    let mut futures = FuturesUnordered::new();

    futures.push(Box::pin(async move {
        match fetch_mintgarden_archive(hash, testnet).await {
            Ok(data) => data,
            Err(error) => {
                debug!("Failed to fetch {hash} from MintGarden archive: {error}");
                None
            }
        }
    }) as Pin<Box<dyn Future<Output = Option<Data>> + Send>>);

    for uri in uris {
        futures.push(Box::pin(async move {
            match fetch_uri_with_hash(uri.clone(), hash, testnet).await {
                Ok(item) if item.hash == hash => Some(item),
                Ok(item) => {
                    debug!(
                        "Hash mismatch for NFT URI {uri}, expected {hash} but found {}",
                        item.hash
                    );
                    None
                }
                Err(error) => {
                    debug!("Failed to fetch NFT URI {uri}, expected hash {hash}: {error}");
                    None
                }
            }
        }));
    }

    while let Some(result) = futures.next().await {
        if let Some(item) = result {
            return Some(item);
        }
    }

    None
}

pub async fn mintgarden_thumbnail(
    data_hash: Bytes32,
    testnet: bool,
) -> Result<Option<Thumbnail>, UriError> {
    let url = format!(
        "https://assets.{}.mintgarden.io/thumbnails/{data_hash}_512.webp",
        if testnet { "testnet" } else { "mainnet" }
    );

    let response = CLIENT.get(&url).timeout(THUMBNAIL_TIMEOUT).send().await?;

    if response.status() != StatusCode::OK {
        return Ok(None);
    }

    let bytes = response.bytes().await?;

    Ok(make_thumbnail(&bytes, "image/webp")?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniff_mime_type() {
        assert_eq!(sniff_mime_type(b"\x89PNG\r\n\x1a\n\0\0\0\0"), "image/png");
        assert_eq!(
            sniff_mime_type(br#"{"format":"CHIP-0007"}"#),
            "application/json"
        );
        assert_eq!(sniff_mime_type(b"\0\x01\x02"), "image/png");
    }

    #[test]
    fn test_is_generic_mime_type() {
        assert!(is_generic_mime_type("application/octet-stream"));
        assert!(is_generic_mime_type("Binary/Octet-Stream; charset=binary"));
        assert!(!is_generic_mime_type("application/json"));
    }
}
