mod recipients;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use anyhow::{Result, bail};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use bip39::Mnemonic;
use chia_wallet_sdk::{
    chia::{
        bls::master_to_wallet_unhardened,
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
    },
    driver::TransferNftById,
    prelude::*,
    test::PeerSimulator,
    types::puzzles::{ANY_METADATA_UPDATER_HASH, HandleNftMetadata},
    utils::Address,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rustls::crypto::aws_lc_rs::default_provider;
use sage::Sage;
use sage_api::{
    Action as ApiAction, AddNftUri, Amount, BulkMintNfts, CreateDid, CreateTransaction, GetDids,
    GetKey, GetNft, GetNftMetadataUpdater, GetOffers, GetPeers, GetSyncStatus, GetVersion,
    Id as ApiId, ImportKey, ImportOffer, Login, MakeOffer, NewNftUri, NftUpdaterKind, NftUriKind,
    OfferAmount, OfferAsset, ResolveHandle, SendXch, SignCoinSpends, UpdateNftAction, ViewOffer,
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
    xchandles: MockXchandles,
    _dir: TempDir,
}

/// A stand-in for the XCHandles API, so that tests never reach the real one. Paths without
/// a configured response answer that the index is stale, so lookups are unavailable and
/// fall back to the mappings stored in the wallet.
#[derive(Debug, Clone, Default)]
struct MockXchandles {
    responses: Arc<StdMutex<HashMap<String, (StatusCode, String)>>>,
}

impl MockXchandles {
    async fn start(&self) -> Result<String> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}", listener.local_addr()?);
        let router = Router::new()
            .fallback(mock_xchandles_response)
            .with_state(self.clone());

        tokio::spawn(async move { axum::serve(listener, router).await });

        Ok(url)
    }

    fn respond(&self, path: &str, status: StatusCode, body: &str) {
        self.responses
            .lock()
            .expect("mock lock poisoned")
            .insert(path.to_string(), (status, body.to_string()));
    }
}

async fn mock_xchandles_response(State(mock): State<MockXchandles>, uri: Uri) -> Response {
    let response = mock
        .responses
        .lock()
        .expect("mock lock poisoned")
        .get(uri.path())
        .cloned();

    match response {
        Some((status, body)) => (status, body).into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            r#"{"code":"index_stale","message":"The index is behind the chain"}"#,
        )
            .into_response(),
    }
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

        let xchandles = MockXchandles::default();
        let xchandles_url = xchandles.start().await?;

        for network in &mut sage.network_list.networks {
            if let Some(mut config) = network.xchandles() {
                config.api_url.clone_from(&xchandles_url);
                network.xchandles = Some(config);
            }
        }

        let sage = Arc::new(Mutex::new(sage));
        let router = make_router(sage.clone());

        let app = Self {
            sage,
            router,
            rng,
            sim,
            events,
            xchandles,
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
        confirmed_handles: Vec::new(),
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
        confirmed_handles: Vec::new(),
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
        confirmed_handles: Vec::new(),
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

#[tokio::test]
async fn test_resolve_handle_rejects_invalid_handles() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    for handle in ["@Alice", "al", "alice.xch", "@@alice"] {
        let error = app
            .resolve_handle(ResolveHandle {
                handle: handle.to_string(),
            })
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("status 400"), "{handle}: {error}");
        assert!(error.contains("Invalid handle"), "{handle}: {error}");
    }

    Ok(())
}

#[tokio::test]
async fn test_resolve_handle_rejects_unsupported_networks() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    // A network which doesn't inherit from a built-in network has no XCHandles deployment,
    // and must not fall back to another network's. The tests configure their own API, so
    // that is removed too.
    for network in &mut app.sage.lock().await.network_list.networks {
        network.inherit = None;
        network.xchandles = None;
    }

    let error = app
        .resolve_handle(ResolveHandle {
            handle: "alice".to_string(),
        })
        .await
        .unwrap_err()
        .to_string();

    assert!(error.contains("status 400"), "{error}");
    assert!(error.contains("not supported on this network"), "{error}");

    Ok(())
}

fn find_nft<'a>(assets: &'a [OfferAsset], nft_id: &str) -> &'a OfferAsset {
    assets
        .iter()
        .find(|asset| asset.asset.asset_id.as_deref() == Some(nft_id))
        .expect("missing nft in offer summary")
}

/// Mints an NFT with the any metadata updater and `XCHandles` metadata, as a handle NFT would be.
async fn mint_mutable_nft(app: &mut TestApp) -> Result<String> {
    app.create_did(CreateDid {
        name: "Alice".to_string(),
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;
    app.wait_for_coins().await;

    let did_id = app.get_dids(GetDids {}).await?.dids[0].launcher_id.clone();
    let did_id = Address::decode(&did_id)?.puzzle_hash;

    let (coin_spends, launcher_id) = {
        let wallet = app.sage.lock().await.wallet()?;
        let mut ctx = SpendContext::new();

        let metadata = ctx.alloc_hashed(&HandleNftMetadata {
            display_name: Some("@alice".to_string()),
            image_uris: vec!["https://example.com/alice.png".to_string()],
            image_hash: Some(Bytes32::new([1; 32])),
            ..Default::default()
        })?;

        let actions = [
            Action::fee(0),
            Action::mint_nft_from_did(
                Id::Existing(did_id),
                metadata,
                ANY_METADATA_UPDATER_HASH.into(),
                Bytes32::default(),
                0,
                1,
            ),
            Action::update_nft(
                Id::New(1),
                vec![],
                Some(TransferNftById::new(Some(Id::Existing(did_id)), vec![])),
            ),
        ];

        let outputs = wallet.spend(&mut ctx, vec![], &actions).await?;
        let launcher_id = outputs
            .nfts
            .values()
            .next()
            .expect("missing nft")
            .info
            .launcher_id;

        (ctx.take(), launcher_id)
    };

    app.sign_coin_spends(SignCoinSpends {
        coin_spends: serde_json::from_value(serde_json::to_value(&coin_spends)?)?,
        auto_submit: true,
        partial: false,
        aggregated_signature: None,
    })
    .await?;
    app.wait_for_coins().await;

    Ok(Address::new(launcher_id, "nft".to_string()).encode()?)
}

#[tokio::test]
async fn test_mutable_nft() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(1000).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let nft_id = mint_mutable_nft(&mut app).await?;

    let nft = app
        .get_nft(GetNft {
            nft_id: nft_id.clone(),
        })
        .await?
        .nft
        .expect("missing nft");
    assert_eq!(nft.metadata_updater_kind, NftUpdaterKind::Any);
    assert_eq!(nft.name.as_deref(), Some("@alice"));
    assert_eq!(nft.data_uris, ["https://example.com/alice.png"]);
    assert_eq!(nft.edition_number, None);

    // Appending a URI is rejected before anything is signed, by both endpoints that can do it.
    let error = app
        .add_nft_uri(AddNftUri {
            nft_id: nft_id.clone(),
            uri: "https://example.com/other.png".to_string(),
            fee: Amount::u64(0),
            kind: NftUriKind::Data,
            auto_submit: false,
        })
        .await
        .expect_err("adding a URI to a mutable NFT must fail");
    assert!(
        error.to_string().contains("default metadata updater"),
        "{error}"
    );

    let error = app
        .create_transaction(CreateTransaction {
            confirmed_handles: Vec::new(),
            actions: vec![ApiAction::UpdateNft(UpdateNftAction {
                id: ApiId::Existing {
                    asset_id: nft_id.clone(),
                },
                new_uris: vec![NewNftUri {
                    kind: NftUriKind::Data,
                    uri: "https://example.com/other.png".to_string(),
                }],
                transfer: None,
            })],
            selected_coin_ids: vec![],
            auto_submit: false,
        })
        .await
        .expect_err("adding a URI to a mutable NFT must fail");
    assert!(
        error.to_string().contains("default metadata updater"),
        "{error}"
    );

    // The NFT is flagged as mutable on the maker side of an offer.
    let xch = OfferAmount {
        asset_id: None,
        hidden_puzzle_hash: None,
        amount: Amount::u64(1),
    };
    let nft_amount = OfferAmount {
        asset_id: Some(nft_id.clone()),
        hidden_puzzle_hash: None,
        amount: Amount::u64(1),
    };

    let offer = app
        .make_offer(MakeOffer {
            confirmed_handles: Vec::new(),
            requested_assets: vec![xch.clone()],
            offered_assets: vec![nft_amount.clone()],
            fee: Amount::u64(0),
            receive_address: None,
            expires_at_second: None,
            auto_import: true,
            coin_ids: None,
        })
        .await?
        .offer;

    let summary = app.view_offer(ViewOffer { offer }).await?.offer;
    let asset = find_nft(&summary.maker, &nft_id);
    assert_eq!(asset.nft_metadata_updater_kind, Some(NftUpdaterKind::Any));
    assert_eq!(asset.asset.name.as_deref(), Some("@alice"));
    assert!(
        summary
            .taker
            .iter()
            .all(|asset| asset.nft_metadata_updater_kind.is_none())
    );

    // The same is true for offers imported into the wallet.
    let offers = app.get_offers(GetOffers {}).await?.offers;
    let asset = find_nft(&offers[0].summary.maker, &nft_id);
    assert_eq!(asset.nft_metadata_updater_kind, Some(NftUpdaterKind::Any));
    assert_eq!(asset.asset.name.as_deref(), Some("@alice"));

    // And on the taker side, when the NFT is requested.
    let offer = app
        .make_offer(MakeOffer {
            confirmed_handles: Vec::new(),
            requested_assets: vec![nft_amount],
            offered_assets: vec![xch],
            fee: Amount::u64(0),
            receive_address: None,
            expires_at_second: None,
            auto_import: false,
            coin_ids: None,
        })
        .await?
        .offer;

    let summary = app.view_offer(ViewOffer { offer }).await?.offer;
    let asset = find_nft(&summary.taker, &nft_id);
    assert_eq!(asset.nft_metadata_updater_kind, Some(NftUpdaterKind::Any));
    assert_eq!(asset.asset.name.as_deref(), Some("@alice"));

    // NFTs minted by Sage use the default, append-only updater and aren't flagged.
    let did_id = app.get_dids(GetDids {}).await?.dids[0].launcher_id.clone();
    let default_nft_id = app
        .bulk_mint_nfts(serde_json::from_value::<BulkMintNfts>(serde_json::json!({
            "mints": [{}],
            "did_id": did_id,
            "fee": 0,
            "auto_submit": true,
        }))?)
        .await?
        .nft_ids
        .remove(0);
    app.wait_for_coins().await;

    let nft = app
        .get_nft(GetNft {
            nft_id: default_nft_id,
        })
        .await?
        .nft
        .expect("missing nft");
    assert_eq!(nft.metadata_updater_kind, NftUpdaterKind::Default);
    assert_eq!(nft.edition_number, Some(1));

    Ok(())
}

/// Removes an NFT's saved on-chain info from a wallet's database, as for offers imported
/// before that info was saved.
async fn forget_nft_info(app: &TestApp, fingerprint: u32, nft_id: &str) -> Result<()> {
    let path = app.sage.lock().await.wallet_db_path(fingerprint)?;
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
    let launcher_id = Address::decode(nft_id)?.puzzle_hash;

    sqlx::query("DELETE FROM nfts WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)")
        .bind(launcher_id.to_vec())
        .execute(&pool)
        .await?;

    pool.close().await;

    Ok(())
}

#[tokio::test]
async fn test_requested_mutable_nft() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(1000).await?;
    let bob = app.setup_bls(1000).await?;

    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let nft_id = mint_mutable_nft(&mut app).await?;

    app.login(Login { fingerprint: bob }).await?;
    app.wait_for_coins().await;

    // Bob doesn't own the NFT, so it's looked up from the network.
    let updater = app
        .get_nft_metadata_updater(GetNftMetadataUpdater {
            nft_id: nft_id.clone(),
        })
        .await?;
    assert_eq!(updater.metadata_updater_kind, Some(NftUpdaterKind::Any));
    assert_eq!(updater.name.as_deref(), Some("@alice"));

    let offer = app
        .make_offer(MakeOffer {
            confirmed_handles: Vec::new(),
            requested_assets: vec![OfferAmount {
                asset_id: Some(nft_id.clone()),
                hidden_puzzle_hash: None,
                amount: Amount::u64(1),
            }],
            offered_assets: vec![OfferAmount {
                asset_id: None,
                hidden_puzzle_hash: None,
                amount: Amount::u64(1),
            }],
            fee: Amount::u64(0),
            receive_address: None,
            expires_at_second: None,
            auto_import: false,
            coin_ids: None,
        })
        .await?
        .offer;

    // Importing saves the requested NFT's info, even if it wasn't known beforehand.
    forget_nft_info(&app, bob, &nft_id).await?;
    app.import_offer(ImportOffer { offer }).await?;

    let offers = app.get_offers(GetOffers {}).await?.offers;
    let asset = find_nft(&offers[0].summary.taker, &nft_id);
    assert_eq!(asset.nft_metadata_updater_kind, Some(NftUpdaterKind::Any));
    assert_eq!(asset.asset.name.as_deref(), Some("@alice"));

    // Without saved info, the NFT is reported as unknown rather than append-only.
    forget_nft_info(&app, bob, &nft_id).await?;

    let offers = app.get_offers(GetOffers {}).await?.offers;
    let asset = find_nft(&offers[0].summary.taker, &nft_id);
    assert_eq!(
        asset.nft_metadata_updater_kind,
        Some(NftUpdaterKind::Unknown)
    );

    Ok(())
}

#[tokio::test]
async fn test_nft_metadata_updater_of_unspent_singleton() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(1000).await?;
    let bob = app.setup_bls(0).await?;
    let bob_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let coin_id = app
        .sage
        .lock()
        .await
        .wallet()?
        .db
        .selectable_xch_coins()
        .await?[0]
        .coin_id();

    // The coin's only odd child is unspent, so it looks like a singleton that has never
    // been spent since it was launched.
    app.send_xch(SendXch {
        confirmed_handles: Vec::new(),
        address: bob_address,
        amount: Amount::u64(1),
        fee: Amount::u64(1),
        memos: vec![],
        clawback: None,
        auto_submit: true,
        coin_ids: None,
    })
    .await?;
    app.wait_for_coins().await;

    app.login(Login { fingerprint: bob }).await?;
    app.wait_for_coins().await;

    let updater = app
        .get_nft_metadata_updater(GetNftMetadataUpdater {
            nft_id: Address::new(coin_id, "nft".to_string()).encode()?,
        })
        .await?;
    assert_eq!(updater.metadata_updater_kind, None);
    assert_eq!(updater.name, None);

    Ok(())
}
