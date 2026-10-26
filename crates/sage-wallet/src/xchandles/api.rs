use std::{future::Future, time::Duration};

use chia_wallet_sdk::prelude::*;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;

use super::Handle;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub enum XchandlesApiError {
    #[error("Handle not found")]
    HandleNotFound,

    #[error("Handle has expired")]
    HandleExpired,

    #[error("The XCHandles index is stale")]
    IndexStale,

    #[error("XCHandles API returned {status}: {code} ({message})")]
    Status {
        status: u16,
        code: String,
        message: String,
    },

    #[error("Invalid XCHandles API URL: {0}")]
    InvalidUrl(String),

    #[error("XCHandles API request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid XCHandles API response: {0}")]
    InvalidResponse(#[from] serde_json::Error),
}

impl XchandlesApiError {
    /// Whether the API failed to give an answer about the handle, rather than answering
    /// that it isn't registered. This includes rate limiting, server errors, responses that
    /// can't be parsed (such as a captive portal page), and a misconfigured API URL, none of
    /// which say anything about the handle itself.
    pub fn is_unavailable(&self) -> bool {
        !self.is_not_registered()
    }

    /// Whether the API answered that the handle has no live registration.
    pub fn is_not_registered(&self) -> bool {
        matches!(self, Self::HandleNotFound | Self::HandleExpired)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HandleResponse {
    pub registry_launcher_id: Bytes32,
    pub handle: String,
    pub slot: HandleSlotResponse,
    pub slot_parent_id: Bytes32,
    pub slot_parent_parent_id: Bytes32,
    pub slot_parent_inner_puzzle_hash: Bytes32,
    pub slot_confirmation_height: u32,
    pub resolved_singleton: Option<SingletonResponse>,
    pub indexed_peak_height: u32,
    pub pending_transfer: Option<PendingTransferResponse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct HandleSlotResponse {
    pub counter: u64,
    pub handle_hash: Bytes32,
    pub neighbors: SlotNeighborsResponse,
    pub expiration: u64,
    pub owner_launcher_id: Bytes32,
    pub resolved_launcher_id: Bytes32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SlotNeighborsResponse {
    pub left_value: Bytes32,
    pub right_value: Bytes32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SingletonResponse {
    pub launcher_id: Bytes32,
    pub parent_coin_id: Bytes32,
    pub amount: u64,
    pub inner_puzzle_hash: Bytes32,
    pub confirmation_height: u32,
    pub melted: bool,
    pub melt_height: Option<u32>,
    pub nft: Option<SingletonNftResponse>,
    pub indexed_peak_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SingletonNftResponse {
    pub metadata_treehash: Bytes32,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub current_owner: Option<Bytes32>,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_basis_points: u16,
    pub p2_puzzle_hash: Bytes32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PendingTransferResponse {
    pub new_owner_launcher_id: Bytes32,
    pub new_resolved_launcher_id: Bytes32,
    pub update_confirmation_height: u32,
    pub minimum_execution_height: u32,
    pub initiator_coin_id: Bytes32,
    pub executor_coin_id: Bytes32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PriceResponse {
    pub indexed_peak_height: u32,
    pub confirmed_timestamp: u64,
    pub current_base_price: u64,
    pub unrolls: Vec<PriceUnrollResponse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PriceUnrollResponse {
    pub activation_timestamp: u64,
    pub base_price: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorResponse {
    code: String,
    #[serde(default)]
    message: String,
}

/// A source of indexed handle data. Responses are untrusted and must be verified.
pub trait XchandlesIndex {
    fn handle(
        &self,
        handle: &Handle,
    ) -> impl Future<Output = Result<HandleResponse, XchandlesApiError>> + Send;
}

/// A client for the XCHandles listener API.
#[derive(Debug, Clone)]
pub struct XchandlesClient {
    client: Client,
    base_url: Url,
    registry_launcher_id: Bytes32,
}

impl XchandlesClient {
    /// Creates a client which only queries the given registry, rather than whichever
    /// registry the API treats as its default.
    pub fn new(base_url: &str, registry_launcher_id: Bytes32) -> Result<Self, XchandlesApiError> {
        let mut base_url = Url::parse(base_url)
            .map_err(|_| XchandlesApiError::InvalidUrl(base_url.to_string()))?;

        if base_url.cannot_be_a_base() {
            return Err(XchandlesApiError::InvalidUrl(base_url.to_string()));
        }

        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }

        Ok(Self {
            client: Client::builder().timeout(REQUEST_TIMEOUT).build()?,
            base_url,
            registry_launcher_id,
        })
    }

    fn url(&self, segments: &[&str], with_registry: bool) -> Url {
        let mut url = self.base_url.clone();

        url.path_segments_mut()
            .expect("base URL was checked to be a base")
            .pop_if_empty()
            .extend(segments);

        if with_registry {
            url.query_pairs_mut()
                .append_pair("launcher_id", &hex::encode(self.registry_launcher_id));
        }

        url
    }

    async fn get(&self, url: Url) -> Result<(StatusCode, Vec<u8>), XchandlesApiError> {
        let response = self.client.get(url).send().await?;
        let status = response.status();
        let body = response.bytes().await?;
        Ok((status, body.to_vec()))
    }

    pub async fn pending_transfer(
        &self,
        handle: &Handle,
    ) -> Result<Option<PendingTransferResponse>, XchandlesApiError> {
        let (status, body) = self
            .get(self.url(&["handle", handle.as_str(), "pending-transfer"], true))
            .await?;

        if status == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        decode(status, &body).map(Some)
    }

    pub async fn singleton(
        &self,
        launcher_id: Bytes32,
    ) -> Result<SingletonResponse, XchandlesApiError> {
        let (status, body) = self
            .get(self.url(&["singletons", &hex::encode(launcher_id)], false))
            .await?;
        decode(status, &body)
    }

    pub async fn price(&self) -> Result<PriceResponse, XchandlesApiError> {
        let (status, body) = self.get(self.url(&["price"], false)).await?;
        decode(status, &body)
    }
}

impl XchandlesIndex for XchandlesClient {
    async fn handle(&self, handle: &Handle) -> Result<HandleResponse, XchandlesApiError> {
        let (status, body) = self
            .get(self.url(&["handle", handle.as_str()], true))
            .await?;
        decode(status, &body)
    }
}

/// Decodes an API response body, mapping documented error codes to typed errors.
pub fn decode<T: DeserializeOwned>(
    status: StatusCode,
    body: &[u8],
) -> Result<T, XchandlesApiError> {
    if status.is_success() {
        return Ok(serde_json::from_slice(body)?);
    }

    let error = serde_json::from_slice::<ErrorResponse>(body).unwrap_or_else(|_| ErrorResponse {
        code: String::new(),
        message: String::from_utf8_lossy(body).into_owned(),
    });

    Err(match (status.as_u16(), error.code.as_str()) {
        (404, "handle_not_found") => XchandlesApiError::HandleNotFound,
        (410, "handle_expired") => XchandlesApiError::HandleExpired,
        (503, "index_stale") => XchandlesApiError::IndexStale,
        _ => XchandlesApiError::Status {
            status: status.as_u16(),
            code: error.code,
            message: error.message,
        },
    })
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    pub(crate) const HANDLE_FIXTURE: &str = include_str!("fixtures/handle_yakuhito.json");
    const TESTNET_HANDLE_FIXTURE: &str = include_str!("fixtures/testnet_handle_cloudwallet.json");
    const SINGLETON_FIXTURE: &str = include_str!("fixtures/singleton.json");
    const PRICE_FIXTURE: &str = include_str!("fixtures/price.json");
    const NOT_FOUND_FIXTURE: &str = include_str!("fixtures/error_handle_not_found.json");
    const INVALID_HANDLE_FIXTURE: &str = include_str!("fixtures/error_invalid_handle.json");

    #[test]
    fn test_decode_handle() {
        let response: HandleResponse = decode(StatusCode::OK, HANDLE_FIXTURE.as_bytes()).unwrap();

        assert_eq!(response.handle, "yakuhito");
        assert_eq!(
            response.registry_launcher_id,
            Bytes32::new(hex!(
                "6f314f69a9cf776311e1c7781a3013e4daae3696c71d8a04d049a5af1530f050"
            ))
        );
        assert_eq!(
            response.slot.handle_hash,
            Handle::parse("yakuhito").unwrap().hash()
        );
        assert_eq!(response.slot.expiration, 1_797_757_200);
        assert_eq!(response.slot_confirmation_height, 9_175_925);
        assert_eq!(response.pending_transfer, None);

        let singleton = response.resolved_singleton.unwrap();
        assert_eq!(singleton.launcher_id, response.slot.resolved_launcher_id);
        assert_eq!(singleton.amount, 1);
        assert!(!singleton.melted);
        assert!(singleton.nft.is_some());
    }

    #[test]
    fn test_decode_testnet_handle() {
        let response: HandleResponse =
            decode(StatusCode::OK, TESTNET_HANDLE_FIXTURE.as_bytes()).unwrap();
        assert_eq!(response.handle, "cloudwallet");
        assert_eq!(
            response.slot.handle_hash,
            Handle::parse("cloudwallet").unwrap().hash()
        );
    }

    #[test]
    fn test_decode_singleton_and_price() {
        let singleton: SingletonResponse =
            decode(StatusCode::OK, SINGLETON_FIXTURE.as_bytes()).unwrap();
        assert_eq!(singleton.melt_height, None);

        let price: PriceResponse = decode(StatusCode::OK, PRICE_FIXTURE.as_bytes()).unwrap();
        assert_eq!(price.current_base_price, 5000);
        assert!(price.unrolls.is_empty());
    }

    #[test]
    fn test_decode_pending_transfer() {
        let body = r#"{
            "new_owner_launcher_id": "0101010101010101010101010101010101010101010101010101010101010101",
            "new_resolved_launcher_id": "0202020202020202020202020202020202020202020202020202020202020202",
            "update_confirmation_height": 100,
            "minimum_execution_height": 132,
            "initiator_coin_id": "0303030303030303030303030303030303030303030303030303030303030303",
            "executor_coin_id": "0404040404040404040404040404040404040404040404040404040404040404"
        }"#;

        let transfer: PendingTransferResponse = decode(StatusCode::OK, body.as_bytes()).unwrap();
        assert_eq!(transfer.new_owner_launcher_id, Bytes32::new([1; 32]));
        assert_eq!(transfer.minimum_execution_height, 132);
    }

    #[test]
    fn test_decode_errors() {
        let error = decode::<HandleResponse>(StatusCode::NOT_FOUND, NOT_FOUND_FIXTURE.as_bytes());
        assert!(matches!(error, Err(XchandlesApiError::HandleNotFound)));
        assert!(!error.unwrap_err().is_unavailable());

        let body = r#"{"code":"handle_expired","message":"Handle expired","request_id":"x"}"#;
        let error = decode::<HandleResponse>(StatusCode::GONE, body.as_bytes());
        assert!(matches!(error, Err(XchandlesApiError::HandleExpired)));

        let body = r#"{"code":"index_stale","message":"Index is stale","request_id":"x"}"#;
        let error = decode::<HandleResponse>(StatusCode::SERVICE_UNAVAILABLE, body.as_bytes());
        assert!(matches!(error, Err(XchandlesApiError::IndexStale)));
        assert!(error.unwrap_err().is_unavailable());

        let error =
            decode::<HandleResponse>(StatusCode::BAD_REQUEST, INVALID_HANDLE_FIXTURE.as_bytes())
                .unwrap_err();
        assert!(
            matches!(&error, XchandlesApiError::Status { status: 400, code, .. } if code == "invalid_handle")
        );
        assert!(error.is_unavailable());

        let body = r#"{"code":"rate_limited","message":"Slow down","request_id":"x"}"#;
        let error =
            decode::<HandleResponse>(StatusCode::TOO_MANY_REQUESTS, body.as_bytes()).unwrap_err();
        assert!(matches!(
            &error,
            XchandlesApiError::Status { status: 429, .. }
        ));
        assert!(error.is_unavailable());

        let error = decode::<HandleResponse>(StatusCode::BAD_GATEWAY, b"<html>bad gateway</html>")
            .unwrap_err();
        assert!(matches!(
            &error,
            XchandlesApiError::Status { status: 502, .. }
        ));
        assert!(error.is_unavailable());

        // For example, a captive portal page.
        let error = decode::<HandleResponse>(StatusCode::OK, b"<html>Sign in</html>").unwrap_err();
        assert!(matches!(error, XchandlesApiError::InvalidResponse(..)));
        assert!(error.is_unavailable());
    }

    #[test]
    fn test_urls() {
        let launcher_id = Bytes32::new([0xab; 32]);
        let handle = Handle::parse("alice").unwrap();

        for base in ["https://api.xchandles.com", "https://api.xchandles.com/"] {
            let client = XchandlesClient::new(base, launcher_id).unwrap();
            assert_eq!(
                client.url(&["handle", handle.as_str()], true).as_str(),
                format!(
                    "https://api.xchandles.com/handle/alice?launcher_id={}",
                    hex::encode(launcher_id)
                )
            );
            assert_eq!(
                client.url(&["price"], false).as_str(),
                "https://api.xchandles.com/price"
            );
        }

        let client = XchandlesClient::new("http://localhost:8080/v1", launcher_id).unwrap();
        assert_eq!(
            client.url(&["price"], false).as_str(),
            "http://localhost:8080/v1/price"
        );

        let error = XchandlesClient::new("not a url", launcher_id).unwrap_err();
        assert!(matches!(error, XchandlesApiError::InvalidUrl(..)));
        assert!(error.is_unavailable());
    }
}
