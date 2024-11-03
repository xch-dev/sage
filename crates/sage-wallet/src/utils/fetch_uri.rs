use std::time::Duration;

use chia::protocol::Bytes32;
use clvmr::sha2::Sha256;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use reqwest::header::CONTENT_TYPE;
use thiserror::Error;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct Data {
    pub blob: Vec<u8>,
    pub mime_type: String,
    pub hash: Bytes32,
}

#[derive(Debug, Error)]
pub enum UriError {
    #[error("Failed to fetch data for URI {0}: {1}")]
    Fetch(String, reqwest::Error),

    #[error("Timed out fetching data for URI {0}")]
    FetchTimeout(String),

    #[error("Missing or invalid content type for URI {0}")]
    InvalidContentType(String),

    #[error("Failed to stream response bytes for URI {0}: {1}")]
    Stream(String, reqwest::Error),

    #[error("Timed out streaming response bytes for URI {0}")]
    StreamTimeout(String),

    #[error("Mime type mismatch for URI {uri}, expected {expected} but found {found}")]
    MimeTypeMismatch {
        uri: String,
        expected: String,
        found: String,
    },

    #[error("Hash mismatch for URI {uri}, expected {expected} but found {found}")]
    HashMismatch {
        uri: String,
        expected: Bytes32,
        found: Bytes32,
    },

    #[error("No URIs provided")]
    NoUris,
}

pub async fn fetch_uri(
    uri: &str,
    request_timeout: Duration,
    stream_timeout: Duration,
) -> Result<Data, UriError> {
    let response = match timeout(request_timeout, reqwest::get(uri)).await {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            return Err(UriError::Fetch(uri.to_string(), error));
        }
        Err(_) => {
            return Err(UriError::FetchTimeout(uri.to_string()));
        }
    };

    let Some(mime_type) = response
        .headers()
        .get(CONTENT_TYPE)
        .cloned()
        .and_then(|value| value.to_str().map(ToString::to_string).ok())
    else {
        return Err(UriError::InvalidContentType(uri.to_string()));
    };

    let blob = match timeout(stream_timeout, response.bytes()).await {
        Ok(Ok(data)) => data.to_vec(),
        Ok(Err(error)) => {
            return Err(UriError::Stream(uri.to_string(), error));
        }
        Err(_) => {
            return Err(UriError::StreamTimeout(uri.to_string()));
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let hash = Bytes32::new(hasher.finalize());

    Ok(Data {
        blob,
        mime_type,
        hash,
    })
}

pub async fn fetch_uris(
    uris: Vec<String>,
    request_timeout: Duration,
    stream_timeout: Duration,
) -> Result<Data, UriError> {
    let mut futures = FuturesUnordered::new();

    for uri in uris {
        futures.push(async move {
            let result = fetch_uri(&uri, request_timeout, stream_timeout).await;
            (uri, result)
        });
    }

    let mut data = None;

    while let Some((uri, result)) = futures.next().await {
        let item = result?;

        let Some(data) = data.take() else {
            data = Some(item);
            continue;
        };

        if data.hash != item.hash {
            return Err(UriError::HashMismatch {
                uri,
                expected: data.hash,
                found: item.hash,
            });
        }

        if data.mime_type != item.mime_type {
            return Err(UriError::MimeTypeMismatch {
                uri,
                expected: data.mime_type,
                found: item.mime_type,
            });
        }

        assert_eq!(data.blob, item.blob);
    }

    data.ok_or(UriError::NoUris)
}
