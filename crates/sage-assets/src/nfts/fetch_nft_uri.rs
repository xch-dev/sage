use std::{
    pin::Pin,
    sync::LazyLock,
    time::{Duration, Instant},
};

use chia_wallet_sdk::{chia::sha2::Sha256, prelude::*};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use mime_sniffer::MimeTypeSniffer;
use reqwest::{Client, Response, StatusCode, header::CONTENT_TYPE};
use tokio::{
    task::spawn_blocking,
    time::{sleep, timeout},
};
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

/// A downloaded file, before any thumbnail has been made for it. Thumbnails
/// are comparatively expensive, so they're only made for the file that's
/// actually kept, and outside of any download deadline.
#[derive(Debug, Clone)]
pub struct Download {
    pub blob: Vec<u8>,
    pub mime_type: String,
    pub hash: Bytes32,
}

/// MintGarden archives NFT data, metadata, and license files by their SHA-256
/// hash, so this works even after every URI baked into the NFT has died.
const MINTGARDEN_ARCHIVE_URL: &str = "https://archive.mintgarden.io/content";

/// The time limit for a single request to a single source.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// The time limit for downloading a URI, across all of the gateways it may
/// be tried through.
const URI_DEADLINE: Duration = Duration::from_secs(30);

/// MintGarden thumbnails are optional, so they get less time than the file.
const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(10);

/// When racing sources against each other, the next one is started as soon as
/// one fails, or after this long if the ones in flight are slow. That way a
/// single stalled source can't use up the whole deadline, without every
/// source being downloaded at once.
const HEDGE_DELAY: Duration = Duration::from_secs(2);

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        // Some gateways sit behind bot protection that challenges clients
        // pretending to be a browser, so identify honestly as Sage.
        .user_agent(concat!("Sage/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
});

/// Downloads a URI, trying each of its candidates (see [`uri_candidates`]) in
/// turn within [`URI_DEADLINE`].
///
/// If an expected hash is given, a candidate that serves mismatching content
/// is skipped in favor of the next one. If none of them match, the first
/// mismatching download is returned.
pub async fn download_uri(
    uri: &str,
    expected_hash: Option<Bytes32>,
    include_fallbacks: bool,
) -> Result<Download, UriError> {
    let mut mismatch = None;
    let mut last_error = None;

    let search = async {
        for url in uri_candidates(uri, include_fallbacks) {
            match download(&url).await {
                Ok(download) if expected_hash.is_none_or(|hash| hash == download.hash) => {
                    return Some(download);
                }
                Ok(download) => {
                    debug!(
                        "Hash mismatch for {url} (NFT URI {uri}), found {}",
                        download.hash
                    );
                    mismatch.get_or_insert(download);
                }
                Err(error) => {
                    debug!("Failed to fetch {url} for NFT URI {uri}: {error}");
                    last_error = Some(error);
                }
            }
        }

        None
    };

    if let Ok(Some(download)) = timeout(URI_DEADLINE, search).await {
        return Ok(download);
    }

    if let Some(download) = mismatch {
        return Ok(download);
    }

    Err(last_error.unwrap_or(UriError::Timeout))
}

/// Downloads a file from MintGarden's archive by its hash. Returns `None` if
/// MintGarden doesn't have it, or if what it returns doesn't match the hash.
pub async fn download_mintgarden_archive(hash: Bytes32) -> Result<Option<Download>, UriError> {
    let url = format!("{MINTGARDEN_ARCHIVE_URL}/{hash}");

    let response = CLIENT.get(&url).timeout(REQUEST_TIMEOUT).send().await?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let download = read_response(response).await?;

    if download.hash != hash {
        debug!(
            "Hash mismatch from MintGarden archive, expected {hash} but found {}",
            download.hash
        );
        return Ok(None);
    }

    Ok(Some(download))
}

/// Fetches a file with a known hash within a deadline, racing MintGarden's
/// archive against the NFT's own URIs (and gateway fallbacks for them). The
/// first source that produces matching content wins. The deadline only
/// covers downloading, not making the thumbnail afterward.
pub async fn fetch_uris_with_hash(
    uris: Vec<String>,
    hash: Bytes32,
    testnet: bool,
    deadline: Duration,
) -> Option<Data> {
    // Interleave the candidates so that each URI is tried as-is before any of
    // them fall back to other gateways.
    let candidates: Vec<Vec<String>> = uris.iter().map(|uri| uri_candidates(uri, true)).collect();
    let rounds = candidates.iter().map(Vec::len).max().unwrap_or_default();

    let mut urls = Vec::new();

    for round in 0..rounds {
        for url in candidates.iter().filter_map(|urls| urls.get(round)) {
            if !urls.contains(url) {
                urls.push(url.clone());
            }
        }
    }

    let mut urls = urls.into_iter();
    let mut in_flight: FuturesUnordered<SourceFuture> = FuturesUnordered::new();

    in_flight.push(Box::pin(async move {
        match download_mintgarden_archive(hash).await {
            Ok(download) => download,
            Err(error) => {
                debug!("Failed to fetch {hash} from MintGarden archive: {error}");
                None
            }
        }
    }));

    if let Some(url) = urls.next() {
        in_flight.push(Box::pin(download_matching(url, hash)));
    }

    // The next source is started either when one fails, or when the ones in
    // flight are taking a while.
    let race = async {
        loop {
            tokio::select! {
                result = in_flight.next() => match result {
                    Some(Some(download)) => return Some(download),
                    Some(None) => {
                        if let Some(url) = urls.next() {
                            in_flight.push(Box::pin(download_matching(url, hash)));
                        } else if in_flight.is_empty() {
                            return None;
                        }
                    }
                    None => return None,
                },
                () = sleep(HEDGE_DELAY), if urls.len() > 0 => {
                    if let Some(url) = urls.next() {
                        in_flight.push(Box::pin(download_matching(url, hash)));
                    }
                }
            }
        }
    };

    let download = timeout(deadline, race).await.ok().flatten();

    // Stop the downloads that lost the race before making the thumbnail.
    drop(in_flight);

    let download = download?;

    Some(download.into_data(testnet).await)
}

type SourceFuture = Pin<Box<dyn Future<Output = Option<Download>> + Send>>;

async fn download_matching(url: String, hash: Bytes32) -> Option<Download> {
    match download(&url).await {
        Ok(download) if download.hash == hash => Some(download),
        Ok(download) => {
            debug!(
                "Hash mismatch for {url}, expected {hash} but found {}",
                download.hash
            );
            None
        }
        Err(error) => {
            debug!("Failed to fetch {url}, expected hash {hash}: {error}");
            None
        }
    }
}

async fn download(url: &str) -> Result<Download, UriError> {
    read_response(CLIENT.get(url).timeout(REQUEST_TIMEOUT).send().await?).await
}

async fn read_response(response: Response) -> Result<Download, UriError> {
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

    // Object storage often serves files as a generic binary type, which isn't
    // useful for displaying them, so try to detect the real type. If that
    // fails, the generic type is kept since it's at least accurate.
    let mime_type = match mime_type {
        Some(mime_type) if !is_generic_mime_type(&mime_type) => mime_type,
        Some(mime_type) => sniff_mime_type(&blob).unwrap_or(mime_type),
        None => sniff_mime_type(&blob).unwrap_or_else(|| "image/png".to_string()),
    };

    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let hash = Bytes32::new(hasher.finalize());

    Ok(Download {
        blob,
        mime_type,
        hash,
    })
}

fn sniff_mime_type(blob: &[u8]) -> Option<String> {
    if let Some(mime_type) = blob.sniff_mime_type() {
        return Some(mime_type.to_string());
    }

    // The sniffer doesn't detect JSON, which metadata files are.
    if serde_json::from_slice::<serde::de::IgnoredAny>(blob).is_ok() {
        return Some("application/json".to_string());
    }

    None
}

/// MintGarden only has thumbnails for media, so there's no point asking for
/// one for metadata or license files.
fn has_mintgarden_thumbnail(mime_type: &str) -> bool {
    let essence = mime_type.split(';').next().unwrap_or_default().trim();

    !(essence.eq_ignore_ascii_case("application/json")
        || essence.eq_ignore_ascii_case("application/pdf")
        || essence
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("text/")))
}

fn is_generic_mime_type(mime_type: &str) -> bool {
    let essence = mime_type.split(';').next().unwrap_or_default().trim();
    essence.eq_ignore_ascii_case("application/octet-stream")
        || essence.eq_ignore_ascii_case("binary/octet-stream")
}

impl Download {
    /// Makes a thumbnail for the file, preferring MintGarden's if it has one.
    pub async fn into_data(self, testnet: bool) -> Data {
        let Self {
            blob,
            mime_type,
            hash,
        } = self;

        let mut thumbnail = if has_mintgarden_thumbnail(&mime_type) {
            match mintgarden_thumbnail(hash, testnet).await {
                Ok(thumbnail) => thumbnail,
                Err(error) => {
                    debug!("Failed to fetch MintGarden thumbnail for {hash}: {error}");
                    None
                }
            }
        } else {
            None
        };

        if thumbnail.is_none() {
            let start = Instant::now();

            let blob_clone = blob.clone();
            let mime_type_clone = mime_type.clone();

            thumbnail =
                match spawn_blocking(move || make_thumbnail(&blob_clone, &mime_type_clone)).await {
                    Ok(Ok(thumbnail)) => thumbnail,
                    Ok(Err(error)) => {
                        debug!("No thumbnail created for {hash}: {error}");
                        None
                    }
                    Err(error) => {
                        debug!("Failed to create thumbnail for {hash}: {error}");
                        None
                    }
                };

            let elapsed = start.elapsed();

            if elapsed > Duration::from_millis(50) {
                debug!("Thumbnail creation took {elapsed:?} for {hash}");
            }
        }

        Data {
            blob,
            mime_type,
            hash,
            thumbnail,
        }
    }
}

/// Fetches an NFT's URIs when their hash isn't known yet, such as when
/// minting, and makes sure they all serve the same file.
///
/// `ipfs://` and `ar://` URIs are translated to HTTPS gateways, but gateway
/// links aren't retried through other gateways, since there's no hash to check
/// the result against and the fallback would hide that the URI is broken.
pub async fn fetch_uris_without_hash(uris: Vec<String>, testnet: bool) -> Result<Data, UriError> {
    let mut futures = FuturesUnordered::new();

    for uri in &uris {
        futures.push(download_uri(uri, None, false));
    }

    let mut first: Option<Download> = None;

    while let Some(result) = futures.next().await {
        let item = result?;

        let Some(first) = &first else {
            first = Some(item);
            continue;
        };

        if first.hash != item.hash {
            return Err(UriError::HashMismatch {
                expected: first.hash,
                found: item.hash,
            });
        }

        if first.mime_type != item.mime_type {
            return Err(UriError::MimeTypeMismatch {
                expected: first.mime_type.clone(),
                found: item.mime_type,
            });
        }
    }

    drop(futures);

    Ok(first.ok_or(UriError::NoUris)?.into_data(testnet).await)
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
        assert_eq!(
            sniff_mime_type(b"\x89PNG\r\n\x1a\n\0\0\0\0").as_deref(),
            Some("image/png")
        );
        assert_eq!(
            sniff_mime_type(br#"{"format":"CHIP-0007"}"#).as_deref(),
            Some("application/json")
        );
        assert_eq!(sniff_mime_type(b"\0\x01\x02"), None);
    }

    #[test]
    fn test_is_generic_mime_type() {
        assert!(is_generic_mime_type("application/octet-stream"));
        assert!(is_generic_mime_type("Binary/Octet-Stream; charset=binary"));
        assert!(!is_generic_mime_type("application/json"));
    }
}
