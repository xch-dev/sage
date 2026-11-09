use std::{
    pin::Pin,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use chia_wallet_sdk::{chia::sha2::Sha256, prelude::*};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use mime_sniffer::MimeTypeSniffer;
use reqwest::{Client, StatusCode, header::CONTENT_TYPE};
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

/// Somewhere a file can be downloaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// MintGarden's archive, which stores NFT data, metadata, and license
    /// files by their SHA-256 hash, so it works even after every URI baked
    /// into the NFT has died.
    Archive(Bytes32),
    Url(String),
}

/// How a set of sources should be raced against each other.
#[derive(Debug, Clone, Copy)]
pub struct RaceOptions {
    /// If set, only a download with this hash counts as a success.
    pub expected_hash: Option<Bytes32>,
    /// The next source is started as soon as one fails, or after this long if
    /// none of the sources in flight have started sending the file. That way a
    /// stalled source can't use up the whole deadline, but a large file that's
    /// downloading fine isn't downloaded from several places at once.
    pub hedge_delay: Duration,
    /// The time limit for downloading, across all sources.
    pub deadline: Duration,
}

/// The result of racing a set of sources.
#[derive(Debug, Default)]
pub struct RaceOutcome {
    /// The first download that matches the expected hash, or the first
    /// successful download if there isn't one.
    pub matched: Option<Download>,
    /// The first download that didn't match the expected hash.
    pub mismatch: Option<Download>,
    /// Whether a source failed in a way that might succeed if retried later,
    /// or the deadline was reached before every source had been tried.
    pub transient_failure: bool,
    /// The last error a source failed with.
    pub last_error: Option<UriError>,
}

const MINTGARDEN_ARCHIVE_URL: &str = "https://archive.mintgarden.io/content";

/// The time limit for a single request to a single source.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// The time limit for downloading a URI when minting, across all of the
/// gateways it may be tried through.
const URI_DEADLINE: Duration = Duration::from_secs(30);

/// MintGarden thumbnails are optional, so they get less time than the file.
const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(10);

/// See [`RaceOptions::hedge_delay`].
pub const HEDGE_DELAY: Duration = Duration::from_secs(2);

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        // Some gateways sit behind bot protection that challenges clients
        // pretending to be a browser, so identify honestly as Sage.
        .user_agent(concat!("Sage/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
});

/// The sources to try for an NFT file with a known hash, in order: MintGarden's
/// archive, then each URI as-is, then gateway fallbacks for them (see
/// [`uri_candidates`]). Candidates are interleaved so that every URI is tried
/// directly before any of them fall back to other gateways, and duplicates
/// (such as an `ipfs://` URI and a gateway link to the same CID) are removed.
pub fn nft_sources(uris: &[String], hash: Bytes32) -> Vec<Source> {
    let candidates: Vec<Vec<String>> = uris.iter().map(|uri| uri_candidates(uri, true)).collect();
    let rounds = candidates.iter().map(Vec::len).max().unwrap_or_default();

    let mut sources = vec![Source::Archive(hash)];

    for round in 0..rounds {
        for url in candidates.iter().filter_map(|urls| urls.get(round)) {
            let source = Source::Url(url.clone());

            if !sources.contains(&source) {
                sources.push(source);
            }
        }
    }

    sources
}

/// Races sources against each other in order (see [`RaceOptions`]), until one
/// of them produces a matching file or the deadline is reached.
pub async fn race_sources(sources: Vec<Source>, options: RaceOptions) -> RaceOutcome {
    let streaming = Arc::new(AtomicUsize::new(0));
    let mut pending = sources.into_iter();
    let mut in_flight: FuturesUnordered<SourceFuture> = FuturesUnordered::new();
    let mut outcome = RaceOutcome::default();

    let start = |source: Source| -> SourceFuture {
        let streaming = streaming.clone();
        Box::pin(async move {
            let result = download_source(&source, &streaming).await;
            (source, result)
        })
    };

    if let Some(source) = pending.next() {
        in_flight.push(start(source));
    }

    let race = async {
        loop {
            tokio::select! {
                result = in_flight.next() => {
                    let Some((source, result)) = result else {
                        return;
                    };

                    match result {
                        Ok(Some(download))
                            if options
                                .expected_hash
                                .is_none_or(|hash| hash == download.hash) =>
                        {
                            outcome.matched = Some(download);
                            return;
                        }
                        Ok(Some(download)) => {
                            debug!("Hash mismatch for {source:?}, found {}", download.hash);
                            outcome.mismatch.get_or_insert(download);
                        }
                        Ok(None) => {}
                        Err(error) => {
                            debug!("Failed to fetch {source:?}: {error}");
                            outcome.transient_failure |= error.is_transient();
                            outcome.last_error = Some(error);
                        }
                    }

                    if let Some(source) = pending.next() {
                        in_flight.push(start(source));
                    } else if in_flight.is_empty() {
                        return;
                    }
                }
                () = sleep(options.hedge_delay), if pending.len() > 0 => {
                    if streaming.load(Ordering::Relaxed) == 0
                        && let Some(source) = pending.next()
                    {
                        in_flight.push(start(source));
                    }
                }
            }
        }
    };

    if timeout(options.deadline, race).await.is_err() {
        outcome.transient_failure = true;
    }

    outcome
}

/// Fetches a file with a known hash within a deadline, racing MintGarden's
/// archive against the NFT's own URIs (see [`nft_sources`]). The deadline only
/// covers downloading, not making the thumbnail afterward.
pub async fn fetch_uris_with_hash(
    uris: Vec<String>,
    hash: Bytes32,
    testnet: bool,
    deadline: Duration,
) -> Option<Data> {
    let outcome = race_sources(
        nft_sources(&uris, hash),
        RaceOptions {
            expected_hash: Some(hash),
            hedge_delay: HEDGE_DELAY,
            deadline,
        },
    )
    .await;

    Some(outcome.matched?.into_data(testnet).await)
}

type SourceFuture =
    Pin<Box<dyn Future<Output = (Source, Result<Option<Download>, UriError>)> + Send>>;

/// Downloads a source. Returns `None` if the archive doesn't have the file.
async fn download_source(
    source: &Source,
    streaming: &AtomicUsize,
) -> Result<Option<Download>, UriError> {
    match source {
        Source::Archive(hash) => {
            let url = format!("{MINTGARDEN_ARCHIVE_URL}/{hash}");
            match download(&url, streaming).await {
                Err(UriError::Fetch(error)) if error.status() == Some(StatusCode::NOT_FOUND) => {
                    Ok(None)
                }
                // Anything else the archive serves (such as a bot challenge
                // page) isn't the file, and is likely to be temporary.
                Ok(download) if download.hash != *hash => Err(UriError::InvalidArchiveContent),
                result => result.map(Some),
            }
        }
        Source::Url(url) => download(url, streaming).await.map(Some),
    }
}

/// Counts a source as sending its file for as long as this is alive.
struct StreamingGuard<'a>(&'a AtomicUsize);

impl<'a> StreamingGuard<'a> {
    fn new(streaming: &'a AtomicUsize) -> Self {
        streaming.fetch_add(1, Ordering::Relaxed);
        Self(streaming)
    }
}

impl Drop for StreamingGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

async fn download(url: &str, streaming: &AtomicUsize) -> Result<Download, UriError> {
    let response = CLIENT.get(url).timeout(REQUEST_TIMEOUT).send().await?;

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

    let blob = {
        let _guard = StreamingGuard::new(streaming);
        response.bytes().await?.to_vec()
    };

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

/// The type and subtype of a MIME type, without any parameters.
fn mime_essence(mime_type: &str) -> &str {
    mime_type.split(';').next().unwrap_or_default().trim()
}

/// MintGarden only has thumbnails for media, so there's no point asking for
/// one for metadata or license files. This is deliberately narrow, since media
/// is sometimes served with a misleading type (such as SVG as `text/plain`).
fn has_mintgarden_thumbnail(mime_type: &str) -> bool {
    let essence = mime_essence(mime_type);
    !(essence.eq_ignore_ascii_case("application/json")
        || essence.eq_ignore_ascii_case("application/pdf"))
}

fn is_generic_mime_type(mime_type: &str) -> bool {
    let essence = mime_essence(mime_type);
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
        let sources = uri_candidates(uri, false)
            .into_iter()
            .map(Source::Url)
            .collect();

        futures.push(async move {
            let outcome = race_sources(
                sources,
                RaceOptions {
                    expected_hash: None,
                    hedge_delay: HEDGE_DELAY,
                    deadline: URI_DEADLINE,
                },
            )
            .await;

            outcome
                .matched
                .ok_or(outcome.last_error.unwrap_or(UriError::Timeout))
        });
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
    fn test_has_mintgarden_thumbnail() {
        assert!(has_mintgarden_thumbnail("image/png"));
        assert!(has_mintgarden_thumbnail("text/plain; charset=utf-8"));
        assert!(!has_mintgarden_thumbnail("application/json"));
        assert!(!has_mintgarden_thumbnail("Application/PDF"));
    }

    #[test]
    fn test_nft_sources() {
        let hash = Bytes32::default();
        let cid = "bafybeicsfvwqp25vgqoc5azurhtdklb7imhlxsjb4lyvkt2mqgx2jibmi4";
        let sources = nft_sources(
            &[
                format!("ipfs://{cid}"),
                format!("https://nftstorage.link/ipfs/{cid}"),
            ],
            hash,
        );

        let url = |url: String| Source::Url(url);

        assert_eq!(
            sources,
            vec![
                Source::Archive(hash),
                url(format!("https://ipfs.filebase.io/ipfs/{cid}")),
                url(format!("https://nftstorage.link/ipfs/{cid}")),
                url(format!("https://gateway.pinata.cloud/ipfs/{cid}")),
                url(format!("https://ipfs.io/ipfs/{cid}")),
            ]
        );
    }

    #[test]
    fn test_is_generic_mime_type() {
        assert!(is_generic_mime_type("application/octet-stream"));
        assert!(is_generic_mime_type("Binary/Octet-Stream; charset=binary"));
        assert!(!is_generic_mime_type("application/json"));
    }
}
