use std::{
    collections::HashMap,
    net::Ipv4Addr,
    slice,
    sync::{Arc, Mutex as StdMutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{
    chia::{
        bls::{
            DerivableKey, master_to_wallet_hardened, master_to_wallet_hardened_intermediate,
            master_to_wallet_unhardened_intermediate,
        },
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
    },
    client::{Connector, PeerOptions},
    clvm_traits::clvm_quote,
    prelude::*,
    test::PeerSimulator,
};
use futures_util::future::BoxFuture;
use sage_config::TESTNET11;
use sage_database::{Database, Derivation, SharedDatabase};
use sqlx::{SqlitePool, migrate};
use tokio::{
    net::TcpListener,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    time::timeout,
};
use tokio_tungstenite::{accept_async, connect_async};
use tracing::debug;

use crate::{
    MempoolSource, PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts,
    Transaction, Wallet, WalletError, WalletKind, WalletPeer, insert_transaction,
    p2_singleton_puzzle_hash, run_conditions, sign_with_secret_keys, track_vault,
};

/// A spend of a coin whose puzzle outputs the given conditions, for transactions that a
/// dApp could put together. The coin doesn't have to exist.
pub fn quoted_spend(
    parent_coin_info: Bytes32,
    amount: u64,
    conditions: Conditions,
) -> anyhow::Result<CoinSpend> {
    let mut ctx = SpendContext::new();
    let puzzle = ctx.alloc(&clvm_quote!(conditions))?;
    let coin = Coin::new(parent_coin_info, ctx.tree_hash(puzzle).into(), amount);
    Ok(CoinSpend::new(
        coin,
        ctx.serialize(&puzzle)?,
        ctx.serialize(&NodePtr::NIL)?,
    ))
}

/// The timestamps that the coin spends must be confirmed before.
pub fn assert_before_seconds(coin_spends: &[CoinSpend]) -> anyhow::Result<Vec<u64>> {
    let mut timestamps = Vec::new();

    for coin_spend in coin_spends {
        for condition in run_conditions(&coin_spend.puzzle_reveal, &coin_spend.solution)? {
            if let Condition::AssertBeforeSecondsAbsolute(condition) = condition {
                timestamps.push(condition.seconds);
            }
        }
    }

    Ok(timestamps)
}

static INDEX: Mutex<u32> = Mutex::const_new(0);

#[derive(Debug)]
pub struct TestWallet {
    pub sim: Arc<PeerSimulator>,
    pub agg_sig: AggSigConstants,
    pub peer: Peer,
    pub pool: SqlitePool,
    pub wallet: Arc<Wallet>,
    pub master_sk: SecretKey,
    pub puzzle_hash: Bytes32,
    pub hardened_puzzle_hash: Bytes32,
    pub sender: Sender<SyncCommand>,
    pub events: Receiver<SyncEvent>,
    pub index: u32,
    pub state: Arc<Mutex<PeerState>>,
    pub options: SyncOptions,
    /// What the wallet's sync manager looks up announced mempool transactions in.
    pub mempool: Arc<FakeMempool>,
}

impl TestWallet {
    pub async fn new(balance: u64) -> anyhow::Result<Self> {
        Self::new_with_options(balance, default_test_options()).await
    }

    pub async fn next(&self, balance: u64) -> anyhow::Result<Self> {
        self.next_with_options(balance, default_test_options())
            .await
    }

    pub async fn new_with_options(balance: u64, options: SyncOptions) -> anyhow::Result<Self> {
        let sim = PeerSimulator::new().await?;
        Self::with_sim(Arc::new(sim), balance, 0, 0, options).await
    }

    pub async fn next_with_options(
        &self,
        balance: u64,
        options: SyncOptions,
    ) -> anyhow::Result<Self> {
        Self::with_sim(self.sim.clone(), balance, 0, self.index + 1, options).await
    }

    /// Creates a coin worth `balance`, and expects the wallet to sync it along with the
    /// `existing_balance` it already has on the simulator.
    async fn with_sim(
        sim: Arc<PeerSimulator>,
        balance: u64,
        existing_balance: u128,
        key_index: u32,
        options: SyncOptions,
    ) -> anyhow::Result<Self> {
        let (pool, db, shared_db) = test_databases().await?;

        let sk = BlsPair::default().sk.derive_unhardened(key_index);
        let pk = sk.public_key();
        let fingerprint = pk.get_fingerprint();
        let intermediate_pk = master_to_wallet_unhardened_intermediate(&pk);
        let genesis_challenge = TESTNET11_CONSTANTS.genesis_challenge;

        let intermediate_hardened_sk = master_to_wallet_hardened_intermediate(&sk);

        let mut tx = db.tx().await?;

        for index in 0..100 {
            let synthetic_key = intermediate_hardened_sk
                .derive_hardened(index)
                .derive_synthetic()
                .public_key();
            let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();
            tx.insert_custody_p2_puzzle(
                p2_puzzle_hash,
                synthetic_key,
                Derivation {
                    derivation_index: index,
                    is_hardened: true,
                },
            )
            .await?;
        }

        tx.commit().await?;

        let puzzle_hash =
            StandardArgs::curry_tree_hash(intermediate_pk.derive_unhardened(0).derive_synthetic());

        let hardened_puzzle_hash = StandardArgs::curry_tree_hash(
            master_to_wallet_hardened(&sk, 0)
                .derive_synthetic()
                .public_key(),
        );

        if balance > 0 {
            sim.lock().await.new_coin(puzzle_hash.into(), balance);
        }

        let state = Arc::new(Mutex::new(PeerState::default()));
        let wallet = Arc::new(Wallet::new(
            db,
            shared_db,
            fingerprint,
            WalletKind::Bls { intermediate_pk },
            genesis_challenge,
            AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            None,
        ));

        let mempool = Arc::new(FakeMempool::default());
        let (peer, sender, events) = start_sync(&sim, &wallet, &state, options, &mempool).await?;

        let mut test = TestWallet {
            sim,
            agg_sig: AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            peer,
            pool,
            wallet,
            master_sk: sk,
            puzzle_hash: puzzle_hash.into(),
            hardened_puzzle_hash: hardened_puzzle_hash.into(),
            sender,
            events,
            index: key_index,
            state,
            options,
            mempool,
        };

        test.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;
        assert_eq!(
            test.wallet.db.xch_balance().await?,
            balance as u128 + existing_balance
        );

        Ok(test)
    }

    /// Stops syncing the wallet, as if it were closed. Its database is kept.
    pub async fn stop_sync(&mut self) -> anyhow::Result<()> {
        self.sender
            .send(SyncCommand::SwitchWallet {
                wallet: None,
                delta_sync: self.options.delta_sync,
            })
            .await?;

        self.consume_until(|event| matches!(event, SyncEvent::WalletChanged { fingerprint: None }))
            .await;

        Ok(())
    }

    /// Starts syncing the wallet again with its existing database, as if it were reopened.
    pub async fn restart_sync(&mut self) -> anyhow::Result<()> {
        let state = Arc::new(Mutex::new(PeerState::default()));
        let (peer, sender, events) =
            start_sync(&self.sim, &self.wallet, &state, self.options, &self.mempool).await?;

        self.peer = peer;
        self.sender = sender;
        self.events = events;
        self.state = state;

        self.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;

        Ok(())
    }

    pub async fn resync(&mut self) -> anyhow::Result<()> {
        let balance = self.wallet.db.xch_balance().await?;
        *self = Self::with_sim(self.sim.clone(), 0, balance, self.index, self.options).await?;
        Ok(())
    }

    /// Opens a wallet for the vault on the same simulator, signed for by `signer`, as importing it
    /// and attaching the signer does. The vault's coins are synced from its launcher.
    pub async fn vault_wallet(
        &self,
        launcher_id: Bytes32,
        signer: SecretKey,
    ) -> anyhow::Result<Self> {
        let (pool, db, shared_db) = test_databases().await?;

        let mut tx = db.tx().await?;
        track_vault(&mut tx, launcher_id).await?;
        tx.commit().await?;

        let wallet = Arc::new(Wallet::new(
            db,
            shared_db,
            launcher_id.as_ref()[0].into(),
            WalletKind::Vault { launcher_id },
            TESTNET11_CONSTANTS.genesis_challenge,
            AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            None,
        ));
        wallet.set_vault_signer_keys(vec![signer.public_key()]);

        let state = Arc::new(Mutex::new(PeerState::default()));
        let mempool = Arc::new(FakeMempool::default());
        let (peer, sender, events) =
            start_sync(&self.sim, &wallet, &state, self.options, &mempool).await?;

        let puzzle_hash = p2_singleton_puzzle_hash(launcher_id, 0);

        let mut test = TestWallet {
            sim: self.sim.clone(),
            agg_sig: AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            peer,
            pool,
            wallet,
            master_sk: signer,
            puzzle_hash,
            hardened_puzzle_hash: puzzle_hash,
            sender,
            events,
            index: self.index + 1,
            state,
            options: self.options,
            mempool,
        };

        test.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;

        Ok(test)
    }

    /// Signs the coin spends with the wallet's key, or a vault wallet's signer.
    pub async fn sign(&self, coin_spends: Vec<CoinSpend>) -> anyhow::Result<SpendBundle> {
        let spend_bundle = SpendBundle::new(coin_spends, Signature::default());

        Ok(match self.wallet.kind {
            WalletKind::Vault { .. } => {
                // Like Sage, the vault's spend is checked with clear signing before it's signed,
                // so every vault transaction in the tests has to parse.
                if let Some(summary) = self
                    .wallet
                    .parse_vault_spend(&spend_bundle.coin_spends)
                    .await?
                    && summary.changes_custody()
                {
                    return Err(WalletError::VaultCustodyChange.into());
                }

                sign_with_secret_keys(
                    spend_bundle,
                    &self.agg_sig,
                    slice::from_ref(&self.master_sk),
                    false,
                )?
            }
            _ => {
                self.wallet
                    .sign_transaction(spend_bundle, &self.agg_sig, self.master_sk.clone(), false)
                    .await?
            }
        })
    }

    /// Signs and submits a vault transaction that changes the vault's custody on purpose, such as
    /// cancelling a recovery or rekeying. Like Sage's recovery endpoints, which sign these
    /// themselves, it skips the clear signing check that refuses custody changes.
    pub async fn transact_custody_change(&self, coin_spends: Vec<CoinSpend>) -> anyhow::Result<()> {
        let spend_bundle = sign_with_secret_keys(
            SpendBundle::new(coin_spends, Signature::default()),
            &self.agg_sig,
            slice::from_ref(&self.master_sk),
            false,
        )?;

        self.push_bundle(spend_bundle).await?;

        Ok(())
    }

    pub async fn transact(&self, coin_spends: Vec<CoinSpend>) -> anyhow::Result<()> {
        let spend_bundle = self.sign(coin_spends).await?;

        self.push_bundle(spend_bundle).await?;

        Ok(())
    }

    pub async fn push_bundle(&self, spend_bundle: SpendBundle) -> anyhow::Result<()> {
        let peer = self.state.lock().await.acquire_peer().expect("no peer");

        let subscriptions = insert_transaction(
            &self.wallet.db,
            &peer,
            TESTNET11_CONSTANTS.genesis_challenge,
            spend_bundle.name(),
            Transaction::from_coin_spends(spend_bundle.coin_spends)?,
            spend_bundle.aggregated_signature,
        )
        .await?;

        self.sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await?;

        Ok(())
    }

    pub async fn consume_until(&mut self, f: impl Fn(SyncEvent) -> bool) {
        loop {
            let next = timeout(Duration::from_secs(10), self.events.recv())
                .await
                .unwrap_or_else(|_| panic!("timed out listening for event"))
                .unwrap_or_else(|| panic!("missing next event"));

            debug!("Consuming event for wallet {}: {next:?}", self.index);

            if f(next) {
                return;
            }
        }
    }

    /// Waits until the check passes, consuming sync events in between.
    pub async fn wait_until<F, Fut>(&mut self, check: F) -> anyhow::Result<()>
    where
        F: Fn(Arc<Wallet>) -> Fut,
        Fut: Future<Output = anyhow::Result<bool>>,
    {
        loop {
            if check(self.wallet.clone()).await? {
                return Ok(());
            }

            timeout(Duration::from_secs(10), self.events.recv())
                .await
                .unwrap_or_else(|_| panic!("timed out waiting for the check to pass"))
                .unwrap_or_else(|| panic!("missing next event"));
        }
    }

    pub async fn wait_for_coins(&mut self) {
        self.consume_until(|event| matches!(event, SyncEvent::CoinsUpdated))
            .await;
    }

    pub async fn wait_for_puzzles(&mut self) {
        self.consume_until(|event| matches!(event, SyncEvent::PuzzleBatchSynced))
            .await;
    }

    /// The custody puzzle hash the transaction history shows as the address for a coin.
    pub async fn history_custody(&self, coin_id: Bytes32) -> anyhow::Result<Option<Bytes32>> {
        let (transactions, _) = self.wallet.db.transactions(None, false, 1000, 0).await?;

        let coin = transactions
            .iter()
            .flat_map(|transaction| transaction.created.iter().chain(&transaction.spent))
            .find(|coin| coin.coin.coin_id() == coin_id)
            .unwrap_or_else(|| panic!("coin {coin_id} is missing from the transaction history"));

        Ok(coin.custody_p2_puzzle_hash)
    }

    pub async fn new_block_with_current_time(&self) -> anyhow::Result<u64> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut sim = self.sim.lock().await;
        sim.set_next_timestamp(timestamp)?;
        sim.create_block();
        Ok(timestamp)
    }
}

/// A wallet database and shared database, in memory.
async fn test_databases() -> anyhow::Result<(SqlitePool, Database, SharedDatabase)> {
    let db_index = {
        let mut lock = INDEX.lock().await;
        let index = *lock;
        *lock += 1;
        index
    };
    let pool =
        SqlitePool::connect(&format!("file:testdb{db_index}?mode=memory&cache=shared")).await?;
    migrate!("../../migrations").run(&pool).await?;
    let db = Database::new(pool.clone());
    db.run_rust_migrations("TXCH".to_string()).await?;

    let shared_pool = SqlitePool::connect(&format!(
        "file:testshareddb{db_index}?mode=memory&cache=shared"
    ))
    .await?;
    migrate!("../../migrations").run(&shared_pool).await?;
    let shared_db = SharedDatabase::new(shared_pool);

    Ok((pool, db, shared_db))
}

/// Starts a sync manager for the wallet, connected only to the simulator.
async fn start_sync(
    sim: &PeerSimulator,
    wallet: &Arc<Wallet>,
    state: &Arc<Mutex<PeerState>>,
    options: SyncOptions,
    mempool: &Arc<FakeMempool>,
) -> anyhow::Result<(Peer, Sender<SyncCommand>, Receiver<SyncEvent>)> {
    let (mut sync_manager, sender, events) = SyncManager::new(
        options,
        state.clone(),
        Some(wallet.clone()),
        TESTNET11.clone(),
        Connector::Plain,
    );

    sync_manager.set_mempool_source(Some(mempool.clone()));

    let (peer, receiver) = sim.connect_raw().await?;

    assert!(
        sync_manager
            .try_add_peer(peer.clone(), receiver, true, false)
            .await
    );

    tokio::spawn(sync_manager.sync());

    Ok((peer, sender, events))
}

pub fn default_test_options() -> SyncOptions {
    SyncOptions {
        target_peers: 0,
        discover_peers: false,
        mempool_discovery: true,
        dns_batch_size: 0,
        connection_batch_size: 0,
        max_peer_age_seconds: 0,
        delta_sync: true,
        puzzle_batch_size_per_peer: 5,
        timeouts: Timeouts {
            sync_delay: Duration::from_millis(100),
            nft_uri_delay: Duration::from_millis(100),
            cat_delay: Duration::from_millis(100),
            puzzle_delay: Duration::from_millis(100),
            transaction_delay: Duration::from_millis(100),
            offer_delay: Duration::from_millis(100),
            ..Default::default()
        },
        testing: true,
    }
}

/// Connects to a local websocket server that accepts the connection but never answers, like a
/// peer whose connection silently died.
pub async fn unresponsive_peer() -> anyhow::Result<WalletPeer> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        let mut connections = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            if let Ok(ws) = accept_async(stream).await {
                connections.push(ws);
            }
        }
    });

    let (ws, _) = connect_async(format!("ws://{addr}")).await?;
    let (peer, _receiver) = Peer::from_websocket(ws, PeerOptions::default())?;

    Ok(WalletPeer::new(peer))
}

/// A [`MempoolSource`] that serves the spend bundles added to it, like Coinset's mempool.
#[derive(Debug, Default)]
pub struct FakeMempool {
    spend_bundles: StdMutex<HashMap<Bytes32, SpendBundle>>,
    lookups: StdMutex<Vec<Bytes32>>,
    failing: StdMutex<bool>,
}

impl FakeMempool {
    pub fn add(&self, spend_bundle: SpendBundle) {
        self.add_as(spend_bundle.name(), spend_bundle);
    }

    /// Serves the spend bundle for `transaction_id`, which doesn't have to be its id.
    pub fn add_as(&self, transaction_id: Bytes32, spend_bundle: SpendBundle) {
        self.spend_bundles
            .lock()
            .expect("fake mempool lock")
            .insert(transaction_id, spend_bundle);
    }

    /// Makes every lookup fail, like an API that's down.
    pub fn fail(&self) {
        *self.failing.lock().expect("fake mempool lock") = true;
    }

    /// The transaction ids that have been looked up, in order.
    pub fn lookups(&self) -> Vec<Bytes32> {
        self.lookups.lock().expect("fake mempool lock").clone()
    }
}

impl MempoolSource for FakeMempool {
    fn spend_bundle(
        &self,
        transaction_id: Bytes32,
    ) -> BoxFuture<'_, Result<Option<SpendBundle>, WalletError>> {
        self.lookups
            .lock()
            .expect("fake mempool lock")
            .push(transaction_id);

        if *self.failing.lock().expect("fake mempool lock") {
            return Box::pin(async { Err(WalletError::Coinset("unavailable".to_string())) });
        }

        let spend_bundle = self
            .spend_bundles
            .lock()
            .expect("fake mempool lock")
            .get(&transaction_id)
            .cloned();
        Box::pin(async move { Ok(spend_bundle) })
    }
}
