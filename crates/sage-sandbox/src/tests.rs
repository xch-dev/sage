use std::time::Duration;

use anyhow::Result;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chia_wallet_sdk::{prelude::*, test::PeerSimulator};
use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
    sync::{broadcast, mpsc},
    time::{sleep, timeout},
};
use tower::ServiceExt;

use crate::{SandboxWallet, WalletInfo, bridge_router, forward_events};

const MOJOS_PER_XCH: u64 = 1_000_000_000_000;

struct TestBridge {
    router: Router,
    wallets: Vec<WalletInfo>,
    _sim: PeerSimulator,
    _dir: TempDir,
}

impl TestBridge {
    async fn start() -> Result<Self> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let dir = TempDir::new()?;
        let sim = PeerSimulator::new().await?;
        // Nothing in these tests looks up a handle, so the URL is never reached.
        let xchandles_url = "http://127.0.0.1:9/xchandles";

        let mut wallets = Vec::new();

        for (name, seed) in [("alice", 1), ("bob", 2)] {
            wallets.push(
                SandboxWallet::start(name, seed, &dir.path().join(name), &sim, xchandles_url)
                    .await?,
            );
        }

        let infos = wallets.iter().map(|wallet| wallet.info.clone()).collect();

        Ok(Self {
            router: bridge_router(wallets),
            wallets: infos,
            _sim: sim,
            _dir: dir,
        })
    }

    fn wallet(&self, name: &str) -> &WalletInfo {
        self.wallets
            .iter()
            .find(|wallet| wallet.name == name)
            .expect("unknown test wallet")
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .body(body.map_or_else(Body::empty, |body| Body::from(body.to_string())))
            .expect("invalid request");

        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router failed");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("unreadable body");
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).expect("response isn't JSON")
        };

        (status, body)
    }

    async fn invoke(&self, wallet: &str, command: &str, args: Value) -> (StatusCode, Value) {
        self.request("POST", &format!("/{wallet}/invoke/{command}"), Some(args))
            .await
    }

    async fn balance(&self, wallet: &str) -> u64 {
        let (status, body) = self
            .invoke(wallet, "get_sync_status", json!({ "req": {} }))
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["selectable_balance"]
            .as_u64()
            .expect("balance isn't a number")
    }
}

#[tokio::test]
async fn test_wallets_start_funded_with_fixed_keys() -> Result<()> {
    let bridge = TestBridge::start().await?;

    let (status, body) = bridge.request("GET", "/", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["name"], "alice");
    assert_eq!(body[1]["name"], "bob");

    // The mnemonics are fixed so that fingerprints and addresses are the same on every run.
    assert_eq!(bridge.wallet("alice").fingerprint, 2_635_807_873);
    assert_eq!(bridge.wallet("bob").fingerprint, 1_364_469_178);

    assert_eq!(bridge.balance("alice").await, 100 * MOJOS_PER_XCH);
    assert_eq!(bridge.balance("bob").await, 100 * MOJOS_PER_XCH);

    Ok(())
}

#[tokio::test]
async fn test_send_between_wallets() -> Result<()> {
    let bridge = TestBridge::start().await?;
    let alice = bridge.wallet("alice").address.clone();

    let (status, body) = bridge
        .invoke(
            "bob",
            "send_xch",
            json!({
                "req": {
                    "address": alice,
                    "amount": (5 * MOJOS_PER_XCH).to_string(),
                    "fee": "0",
                    "auto_submit": true,
                }
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // The simulator confirms the spend as soon as it's submitted, and alice syncs it shortly after.
    timeout(Duration::from_secs(10), async {
        while bridge.balance("alice").await != 105 * MOJOS_PER_XCH {
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await?;

    assert_eq!(bridge.balance("bob").await, 95 * MOJOS_PER_XCH);

    Ok(())
}

#[tokio::test]
async fn test_hand_written_commands() -> Result<()> {
    let bridge = TestBridge::start().await?;
    let alice = bridge.wallet("alice");

    let (status, body) = bridge.invoke("alice", "initialize", json!({})).await;
    assert_eq!((status, body), (StatusCode::OK, Value::Null));

    // A testnet address is valid on the sandbox's network, and a mainnet one isn't.
    let (_, body) = bridge
        .invoke(
            "alice",
            "validate_address",
            json!({ "address": alice.address }),
        )
        .await;
    assert_eq!(body, Value::Bool(true));

    let mainnet = Address::new(Bytes32::default(), "xch".to_string()).encode()?;
    let (_, body) = bridge
        .invoke("alice", "validate_address", json!({ "address": mainnet }))
        .await;
    assert_eq!(body, Value::Bool(false));

    let (status, body) = bridge
        .invoke(
            "alice",
            "wallet_config",
            json!({ "fingerprint": alice.fingerprint }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "Alice");

    let (_, body) = bridge.invoke("alice", "network_config", json!({})).await;
    assert_eq!(body["default_network"], "testnet11");

    let (_, body) = bridge.invoke("alice", "get_logs", json!({})).await;
    assert_eq!(body, json!([]));

    Ok(())
}

#[tokio::test]
async fn test_errors_match_the_desktop_app() -> Result<()> {
    let bridge = TestBridge::start().await?;

    let (status, body) = bridge.invoke("alice", "no_such_command", json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["kind"], "api");
    assert_eq!(
        body["reason"],
        "`no_such_command` isn't available in the sandbox"
    );

    // An error from Sage itself keeps its kind and reason.
    let (status, body) = bridge
        .invoke(
            "alice",
            "send_xch",
            json!({ "req": { "address": "txch1bad", "amount": "1", "fee": "0" } }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["kind"], "api");
    assert!(
        body["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("Address")),
        "{body}"
    );

    let (status, body) = bridge
        .invoke("alice", "send_xch", json!({ "req": { "address": 1 } }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["reason"]
            .as_str()
            .is_some_and(|reason| reason.starts_with("Invalid request")),
        "{body}"
    );

    let (status, body) = bridge.invoke("carol", "get_sync_status", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["reason"], "There is no `carol` wallet");

    Ok(())
}

#[tokio::test]
async fn test_xchandles_stand_in_reports_a_stale_index() -> Result<()> {
    let bridge = TestBridge::start().await?;

    let (status, body) = bridge
        .request("GET", "/xchandles/v1/handles/alice", None)
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "index_stale");

    Ok(())
}

fn event_json(event: &ApiEvent) -> Value {
    serde_json::to_value(event).expect("event isn't serializable")
}

#[tokio::test(start_paused = true)]
async fn test_refresh_events_are_debounced() -> Result<()> {
    let (sender, receiver) = mpsc::channel(16);
    let (events, mut listener) = broadcast::channel(16);
    forward_events(receiver, events);

    // A burst of refreshes of the same kind collapses into one, sent after the debounce.
    sender.send(SyncEvent::CoinsUpdated).await?;
    sender.send(SyncEvent::CoinsUpdated).await?;
    sender
        .send(SyncEvent::TransactionUpdated {
            transaction_id: Bytes32::default(),
        })
        .await?;
    sender.send(SyncEvent::CatInfo).await?;

    // Events that carry a payload are sent at once, ahead of the pending refreshes.
    sender
        .send(SyncEvent::TransactionFailed {
            transaction_id: Bytes32::default(),
            error: Some("rejected".to_string()),
        })
        .await?;

    // Events the frontend doesn't listen for are dropped.
    sender
        .send(SyncEvent::WalletChanged { fingerprint: None })
        .await?;

    let first = listener.recv().await?;
    assert_eq!(
        event_json(&first),
        json!({
            "type": "transaction_failed",
            "transaction_id": Bytes32::default().to_string(),
            "error": "rejected",
        })
    );

    let second = listener.recv().await?;
    let third = listener.recv().await?;
    assert_eq!(event_json(&second), json!({ "type": "coin_state" }));
    assert_eq!(event_json(&third), json!({ "type": "cat_info" }));

    // Nothing else was sent.
    sleep(Duration::from_secs(1)).await;
    assert!(listener.try_recv().is_err());

    // Once flushed, the same kind of refresh is sent again.
    sender.send(SyncEvent::CoinsUpdated).await?;
    let fourth = listener.recv().await?;
    assert_eq!(event_json(&fourth), json!({ "type": "coin_state" }));

    Ok(())
}
