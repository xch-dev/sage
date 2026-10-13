use std::{sync::Arc, time::Duration};

use anyhow::{Result, bail};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use bip39::Mnemonic;
use chia_wallet_sdk::{
    chia::{
        bls::master_to_wallet_unhardened,
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
    },
    prelude::*,
    test::PeerSimulator,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rustls::crypto::aws_lc_rs::default_provider;
use sage::Sage;
use sage_api::{
    Amount, GetKey, GetPeers, GetSyncStatus, GetVersion, ImportKey, Login, MakeOffer, OfferAmount,
    SendXch,
};
use sage_api_macro::impl_endpoints;
use sage_wallet::{SyncCommand, SyncEvent};
use serde::{Serialize, de::DeserializeOwned};
use tempfile::TempDir;
use tokio::{
    sync::{Mutex, mpsc},
    time::timeout,
};
use tower::ServiceExt;
use tracing::debug;

use crate::make_router;

struct TestApp {
    sage: Arc<Mutex<Sage>>,
    router: Router<()>,
    rng: ChaCha8Rng,
    sim: PeerSimulator,
    events: mpsc::Receiver<SyncEvent>,
    _dir: TempDir,
}

impl TestApp {
    pub async fn new() -> Result<Self> {
        let _ = default_provider().install_default();

        let dir = TempDir::new()?;
        let rng = ChaCha8Rng::seed_from_u64(1337);
        let sim = PeerSimulator::new().await?;

        let mut sage = Sage::new(dir.path(), true);

        // Make sure we don't attempt to connect to actual nodes
        sage.config.network.target_peers = 1;
        sage.config.network.discover_peers = false;
        sage.config.network.default_network = "testnet11".to_string();

        let events = sage.initialize().await?;

        let sage = Arc::new(Mutex::new(sage));
        let router = make_router(sage.clone());

        let app = Self {
            sage,
            router,
            rng,
            sim,
            events,
            _dir: dir,
        };

        let (peer, receiver) = app.sim.connect_raw().await?;

        app.sage
            .lock()
            .await
            .command_sender
            .send(SyncCommand::AddPeer { peer, receiver })
            .await?;

        Ok(app)
    }

    async fn call_rpc<T: Serialize, R: DeserializeOwned>(&self, path: &str, body: T) -> Result<R> {
        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body)?))?;

        let response = self.router.clone().oneshot(req).await?;
        let status = response.status();

        if status != StatusCode::OK {
            let body = response.into_body();
            let body = axum::body::to_bytes(body, usize::MAX).await?;
            bail!(
                "RPC request failed with status {status}: {}",
                String::from_utf8(body.to_vec())?
            );
        }

        let body = response.into_body();
        let body = axum::body::to_bytes(body, usize::MAX).await?;

        Ok(serde_json::from_slice(&body)?)
    }

    async fn setup_bls(&mut self, balance: u64) -> Result<u32> {
        let mnemonic = Mnemonic::from_entropy(&self.rng.r#gen::<[u8; 16]>())?;

        if balance > 0 {
            let master_sk = SecretKey::from_seed(&mnemonic.to_seed(""));
            let p2_puzzle_hash = StandardArgs::curry_tree_hash(
                master_to_wallet_unhardened(&master_sk, 0)
                    .public_key()
                    .derive_synthetic(),
            );

            self.sim.lock().await.create_block();

            self.sim
                .lock()
                .await
                .new_coin(p2_puzzle_hash.into(), balance);
        }

        let fingerprint = self
            .import_key(ImportKey {
                name: "Alice".to_string(),
                key: mnemonic.to_string(),
                derivation_index: 0,
                hardened: None,
                unhardened: None,
                save_secrets: true,
                login: true,
                emoji: None,
            })
            .await?
            .fingerprint;

        self.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;

        Ok(fingerprint)
    }

    async fn consume_until(&mut self, f: impl Fn(SyncEvent) -> bool) {
        loop {
            let next = timeout(Duration::from_secs(10), self.events.recv())
                .await
                .unwrap_or_else(|_| panic!("timed out listening for event"))
                .unwrap_or_else(|| panic!("missing next event"));

            debug!("Consuming event: {next:?}");

            if f(next) {
                return;
            }
        }
    }

    async fn wait_for_coins(&mut self) {
        self.consume_until(|event| matches!(event, SyncEvent::CoinsUpdated))
            .await;
    }

    #[allow(unused)]
    async fn wait_for_puzzles(&mut self) {
        self.consume_until(|event| matches!(event, SyncEvent::PuzzleBatchSynced))
            .await;
    }
}

impl_endpoints! {
    impl TestApp {
        (repeat pub async fn endpoint(&self, body: sage_api::Endpoint) -> Result<sage_api::EndpointResponse> {
            self.call_rpc(&format!("/{}", endpoint_string), body).await
        })
    }
}

#[tokio::test]
async fn test_rpc_version() -> Result<()> {
    let app = TestApp::new().await?;

    let response = app.get_version(GetVersion {}).await?;

    assert_eq!(response.version, env!("CARGO_PKG_VERSION"));

    Ok(())
}

#[tokio::test]
async fn test_initial_state() -> Result<()> {
    let mut app = TestApp::new().await?;

    let fingerprint = app.setup_bls(0).await?;

    let key = app
        .get_key(GetKey { fingerprint: None })
        .await?
        .key
        .expect("should be logged in");

    assert_eq!(key.fingerprint, fingerprint);

    let peers = app.get_peers(GetPeers {}).await?.peers;

    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].peak_height, 0);
    assert!(!peers[0].user_managed);

    let status = app.get_sync_status(GetSyncStatus {}).await?;

    assert_eq!(status.synced_coins, 0);
    assert_eq!(status.total_coins, 0);
    assert_eq!(status.selectable_balance.to_u64(), Some(0));
    assert_eq!(status.unhardened_derivation_index, 1000);
    assert_eq!(status.hardened_derivation_index, 0);
    assert_eq!(
        status.receive_address,
        "txch19hutewzq3z4l6y3fsw5laatre79tuz5p43jlvag0yz466xx9l7vs4vnpem"
    );

    Ok(())
}

#[tokio::test]
async fn test_send_xch() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(1000).await?;

    let bob = app.setup_bls(1000).await?;
    let bob_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    app.login(Login { fingerprint: alice }).await?;

    let balance = app
        .get_sync_status(GetSyncStatus {})
        .await?
        .selectable_balance
        .to_u64();
    assert_eq!(balance, Some(1000));

    app.wait_for_coins().await;

    app.send_xch(SendXch {
        address: bob_address,
        amount: Amount::u64(1000),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit: true,
        coin_ids: None,
    })
    .await?;

    app.wait_for_coins().await;

    let balance = app
        .get_sync_status(GetSyncStatus {})
        .await?
        .selectable_balance
        .to_u64();
    assert_eq!(balance, Some(0));

    app.login(Login { fingerprint: bob }).await?;

    app.wait_for_coins().await;

    let balance = app
        .get_sync_status(GetSyncStatus {})
        .await?
        .selectable_balance
        .to_u64();
    assert_eq!(balance, Some(2000));

    Ok(())
}

#[tokio::test]
async fn test_send_xch_with_coin_ids() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(1000).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let selectable = async |app: &TestApp| -> Result<Vec<Coin>> {
        Ok(app
            .sage
            .lock()
            .await
            .wallet()?
            .db
            .selectable_xch_coins()
            .await?)
    };
    let send = |coin_ids: Vec<String>, address: String, amount: u64| SendXch {
        address,
        amount: Amount::u64(amount),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit: true,
        coin_ids: Some(coin_ids),
    };

    let own = app.get_sync_status(GetSyncStatus {}).await?.receive_address;
    let start = selectable(&app).await?;
    assert_eq!(start.len(), 1);
    let first = start[0].coin_id();

    // A chosen coin is spent: 400 to ourselves, 600 change.
    app.send_xch(send(vec![first.to_string()], own.clone(), 400))
        .await?;
    app.wait_for_coins().await;

    let coins = selectable(&app).await?;
    let mut amounts: Vec<u64> = coins.iter().map(|coin| coin.amount).collect();
    amounts.sort_unstable();
    assert_eq!(amounts, vec![400, 600]);

    // A coin this wallet does not have, and one it has already spent, are refused.
    for coin_id in [Bytes32::new([7; 32]), first] {
        let error = app
            .send_xch(send(vec![coin_id.to_string()], own.clone(), 1))
            .await
            .expect_err("a coin the wallet cannot select must be refused");
        assert!(error.to_string().contains("cannot be selected"), "{error}");
    }

    // A coin backing an open offer is refused too: choosing it must not cancel the offer.
    let locked = coins
        .iter()
        .find(|coin| coin.amount == 400)
        .expect("400 coin");
    let free = coins
        .iter()
        .find(|coin| coin.amount == 600)
        .expect("600 coin");
    app.make_offer(MakeOffer {
        requested_assets: vec![OfferAmount {
            asset_id: None,
            hidden_puzzle_hash: None,
            amount: Amount::u64(1),
        }],
        offered_assets: vec![OfferAmount {
            asset_id: None,
            hidden_puzzle_hash: None,
            amount: Amount::u64(400),
        }],
        fee: Amount::u64(0),
        receive_address: None,
        expires_at_second: None,
        auto_import: true,
        coin_ids: Some(vec![locked.coin_id().to_string()]),
    })
    .await?;

    let error = app
        .send_xch(send(vec![locked.coin_id().to_string()], own.clone(), 1))
        .await
        .expect_err("a coin locked in an offer must be refused");
    assert!(error.to_string().contains("cannot be selected"), "{error}");

    app.send_xch(send(vec![free.coin_id().to_string()], own, 100))
        .await?;

    Ok(())
}
