#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use axum::{Router, body::Body};
use http_body_util::BodyExt;
use hyper::{Request, StatusCode};
use sage::Sage;
use sage_rpc::make_router;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tower::ServiceExt;

const TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Create a Sage instance in a temp directory with no wallet logged in.
fn test_app() -> (Router, TempDir) {
    let dir = TempDir::new().unwrap();
    let sage = Sage::new(dir.path());
    let router = make_router(Arc::new(Mutex::new(sage)));
    (router, dir)
}

/// Create a Sage instance with a logged-in test wallet.
///
/// Replaces the dummy command channel with a draining receiver so that
/// `switch_wallet` (called by `import_key` with `login: true`) succeeds.
async fn test_app_with_wallet() -> (Router, TempDir) {
    let dir = TempDir::new().unwrap();
    let mut sage = Sage::new(dir.path());

    // Replace the dead command channel with one that has a draining receiver.
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    sage.command_sender = tx;
    tokio::spawn(async move {
        while rx.recv().await.is_some() {}
    });

    let sage = Arc::new(Mutex::new(sage));
    let router = make_router(sage);

    let body = serde_json::json!({
        "name": "Test Wallet",
        "key": TEST_MNEMONIC,
        "derivation_index": 100,
        "save_secrets": false,
        "login": true
    });
    let (status, resp) = post_json(&router, "/import_key", &body.to_string()).await;
    assert_eq!(status, StatusCode::OK, "import_key setup failed: {resp}");

    (router, dir)
}

/// POST JSON to an endpoint and return `(status, body_string)`.
async fn post_json(router: &Router, path: &str, body: &str) -> (StatusCode, String) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

// ---------------------------------------------------------------------------
// Tier 1: Pure endpoints (no wallet/DB needed)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_version_returns_200() {
    let (router, _dir) = test_app();
    let (status, body) = post_json(&router, "/get_version", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["version"].is_string());
    assert!(!json["version"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn generate_mnemonic_12_words() {
    let (router, _dir) = test_app();
    let (status, body) =
        post_json(&router, "/generate_mnemonic", r#"{"use_24_words": false}"#).await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mnemonic = json["mnemonic"].as_str().unwrap();
    assert_eq!(mnemonic.split_whitespace().count(), 12);
}

#[tokio::test]
async fn generate_mnemonic_24_words() {
    let (router, _dir) = test_app();
    let (status, body) =
        post_json(&router, "/generate_mnemonic", r#"{"use_24_words": true}"#).await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mnemonic = json["mnemonic"].as_str().unwrap();
    assert_eq!(mnemonic.split_whitespace().count(), 24);
}

#[tokio::test]
async fn get_keys_empty() {
    let (router, _dir) = test_app();
    let (status, body) = post_json(&router, "/get_keys", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["keys"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_networks_returns_defaults() {
    let (router, _dir) = test_app();
    let (status, body) = post_json(&router, "/get_networks", "{}").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("mainnet"));
}

#[tokio::test]
async fn get_network_returns_mainnet() {
    let (router, _dir) = test_app();
    let (status, body) = post_json(&router, "/get_network", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["network"].is_object());
}

// ---------------------------------------------------------------------------
// Tier 2: Error mapping (wallet-required endpoints return 401)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_sync_status_unauthorized() {
    let (router, _dir) = test_app();
    let (status, _body) = post_json(&router, "/get_sync_status", "{}").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nfts_unauthorized() {
    let (router, _dir) = test_app();
    let (status, _body) = post_json(
        &router,
        "/get_nfts",
        r#"{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": false}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn send_xch_unauthorized() {
    let (router, _dir) = test_app();
    let (status, _body) = post_json(
        &router,
        "/send_xch",
        r#"{"address": "xch1test", "amount": "0", "fee": "0", "memos": [], "auto_submit": false}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Tier 3: HTTP/JSON error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn malformed_json_returns_400() {
    let (router, _dir) = test_app();
    let (status, _body) = post_json(&router, "/get_version", "NOT JSON").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_content_type_returns_415() {
    let (router, _dir) = test_app();
    let req = Request::builder()
        .method("POST")
        .uri("/get_version")
        .body(Body::from("{}"))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let (router, _dir) = test_app();
    let (status, _body) = post_json(&router, "/nonexistent", "{}").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn wrong_method_returns_405() {
    let (router, _dir) = test_app();
    let req = Request::builder()
        .method("GET")
        .uri("/get_version")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// ---------------------------------------------------------------------------
// Tier 4: Disk-writing endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn set_delta_sync_persists() {
    let (router, dir) = test_app();
    let (status, _body) =
        post_json(&router, "/set_delta_sync", r#"{"delta_sync": true}"#).await;
    assert_eq!(status, StatusCode::OK);

    let config_path = dir.path().join("wallets.toml");
    let content = std::fs::read_to_string(config_path).unwrap();
    assert!(content.contains("delta_sync = true"));
}

#[tokio::test]
async fn rename_key_unknown_fingerprint() {
    let (router, _dir) = test_app();
    let (status, _body) = post_json(
        &router,
        "/rename_key",
        r#"{"fingerprint": 999999, "name": "test"}"#,
    )
    .await;
    // UnknownFingerprint → ErrorKind::NotFound → 404
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Tier 5: Key import round-trips
// ---------------------------------------------------------------------------

#[tokio::test]
async fn import_key_then_get_keys() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(&router, "/get_keys", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["keys"].as_array().unwrap().len(), 1);
    assert_eq!(json["keys"][0]["name"].as_str().unwrap(), "Test Wallet");
}

#[tokio::test]
async fn import_key_then_delete_key() {
    let (router, _dir) = test_app_with_wallet().await;

    // Read the fingerprint from get_keys
    let (_, body) = post_json(&router, "/get_keys", "{}").await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let fingerprint = json["keys"][0]["fingerprint"].as_u64().unwrap();

    let (status, _body) = post_json(
        &router,
        "/delete_key",
        &format!(r#"{{"fingerprint": {fingerprint}}}"#),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = post_json(&router, "/get_keys", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["keys"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn import_key_with_login_fails_without_sync_manager() {
    // Uses test_app() (dead command channel) to verify the error path.
    let (router, _dir) = test_app();

    let body = serde_json::json!({
        "name": "Test",
        "key": TEST_MNEMONIC,
        "derivation_index": 1,
        "save_secrets": false,
        "login": true
    });
    let (status, _body) = post_json(&router, "/import_key", &body.to_string()).await;

    // switch_wallet sends on the dead channel → Send error → Internal → 500
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

// ---------------------------------------------------------------------------
// Tier 6: Wallet-authenticated endpoints (logged-in test wallet)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_sync_status_with_wallet() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(&router, "/get_sync_status", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["balance"], 0);
    assert_eq!(json["total_coins"], 0);
    assert_eq!(json["synced_coins"], 0);
    assert!(json["receive_address"].as_str().unwrap().starts_with("xch"));
}

#[tokio::test]
async fn get_coins_empty_wallet() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(
        &router,
        "/get_coins",
        r#"{"offset": 0, "limit": 50}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["coins"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], 0);
}

#[tokio::test]
async fn get_cats_empty_wallet() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(&router, "/get_cats", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["cats"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_dids_empty_wallet() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(&router, "/get_dids", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["dids"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_nfts_empty_wallet() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(
        &router,
        "/get_nfts",
        r#"{"offset": 0, "limit": 10, "sort_mode": "name", "include_hidden": false}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["nfts"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], 0);
}

#[tokio::test]
async fn get_transactions_empty_wallet() {
    let (router, _dir) = test_app_with_wallet().await;

    let (status, body) = post_json(
        &router,
        "/get_transactions",
        r#"{"offset": 0, "limit": 50, "ascending": false}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["transactions"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], 0);
}

#[tokio::test]
async fn get_derivations_has_imported_keys() {
    let (router, _dir) = test_app_with_wallet().await;

    // We imported with derivation_index=100, so there should be derivations.
    let (status, body) = post_json(
        &router,
        "/get_derivations",
        r#"{"hardened": false, "offset": 0, "limit": 10}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let derivations = json["derivations"].as_array().unwrap();
    assert!(!derivations.is_empty());
    assert_eq!(json["total"], 100);
}
