use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::{
    bls::{
        master_to_wallet_hardened, master_to_wallet_hardened_intermediate,
        master_to_wallet_unhardened_intermediate, DerivableKey, SecretKey, Signature,
    },
    protocol::{Bytes32, CoinSpend, SpendBundle},
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::{
    client::{Connector, Peer},
    signer::AggSigConstants,
    test::{BlsPair, PeerSimulator},
    types::TESTNET11_CONSTANTS,
};
use sage_config::{NetworkConfig, TESTNET11};
use sage_database::{Database, Derivation};
use sqlx::{migrate, SqlitePool};
use tokio::{
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
    time::timeout,
};
use tracing::debug;

use crate::{
    insert_transaction, SyncCommand, SyncEvent, SyncManager, SyncOptions, SyncState, Timeouts,
    Transaction, Wallet,
};

static INDEX: Mutex<u32> = Mutex::const_new(0);

#[derive(Debug)]
pub struct TestWallet {
    pub sim: Arc<PeerSimulator>,
    pub agg_sig: AggSigConstants,
    pub peer: Peer,
    pub wallet: Wallet,
    pub master_sk: SecretKey,
    pub puzzle_hash: Bytes32,
    pub hardened_puzzle_hash: Bytes32,
    pub events: Receiver<SyncEvent>,
    pub index: u32,
    pub state: SyncState,
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
        Self::with_sim(Arc::new(sim), balance, 0, options).await
    }

    pub async fn next_with_options(
        &self,
        balance: u64,
        options: SyncOptions,
    ) -> anyhow::Result<Self> {
        Self::with_sim(self.sim.clone(), balance, self.index + 1, options).await
    }

    async fn with_sim(
        sim: Arc<PeerSimulator>,
        balance: u64,
        key_index: u32,
        options: SyncOptions,
    ) -> anyhow::Result<Self> {
        let db_index = {
            let mut lock = INDEX.lock().await;
            let index = *lock;
            *lock += 1;
            index
        };
        let pool =
            SqlitePool::connect(&format!("file:testdb{db_index}?mode=memory&cache=shared")).await?;
        migrate!("../../migrations").run(&pool).await?;
        let db = Database::new(pool);
        db.run_rust_migrations("TXCH".to_string()).await?;

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

        let wallet = Wallet::new(
            db,
            fingerprint,
            intermediate_pk,
            genesis_challenge,
            AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
        );

        let (command_sender, command_receiver) = mpsc::channel(100);
        let (event_sender, event_receiver) = mpsc::channel(100);

        let state = SyncState::new(options, command_sender, event_sender);
        state.update_network(TESTNET11.clone()).await;
        state
            .update_network_config(NetworkConfig {
                default_network: "testnet11".to_string(),
                target_peers: 1,
                discover_peers: false,
            })
            .await;
        state
            .login_wallet(wallet.clone(), sage_config::Wallet::default())
            .await;

        let mut sync_manager = SyncManager::new(state.clone(), command_receiver, Connector::Plain);

        let (peer, receiver) = sim.connect_raw().await?;

        let network_config = state.network_config.lock().await.clone();

        assert!(
            sync_manager
                .try_add_peer(peer.clone(), receiver, true, false, &network_config)
                .await
        );

        tokio::spawn(sync_manager.sync());

        let mut test = TestWallet {
            sim,
            agg_sig: AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            peer,
            wallet,
            master_sk: sk,
            puzzle_hash: puzzle_hash.into(),
            hardened_puzzle_hash: hardened_puzzle_hash.into(),
            events: event_receiver,
            index: key_index,
            state,
        };

        test.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;
        assert_eq!(test.wallet.db.xch_balance().await?, balance as u128);

        Ok(test)
    }

    pub async fn resync(&mut self) -> anyhow::Result<()> {
        let options = self.state.options;
        *self = Self::with_sim(self.sim.clone(), 0, self.index, options).await?;
        Ok(())
    }

    pub async fn transact(&self, coin_spends: Vec<CoinSpend>) -> anyhow::Result<()> {
        let spend_bundle = self
            .wallet
            .sign_transaction(
                SpendBundle::new(coin_spends, Signature::default()),
                &self.agg_sig,
                self.master_sk.clone(),
                false,
            )
            .await?;

        self.push_bundle(spend_bundle).await?;

        Ok(())
    }

    pub async fn push_bundle(&self, spend_bundle: SpendBundle) -> anyhow::Result<()> {
        let peer = self
            .state
            .peers
            .lock()
            .await
            .acquire_peer()
            .expect("no peer");

        let subscriptions = insert_transaction(
            &self.wallet.db,
            &peer,
            TESTNET11_CONSTANTS.genesis_challenge,
            spend_bundle.name(),
            Transaction::from_coin_spends(spend_bundle.coin_spends)?,
            spend_bundle.aggregated_signature,
        )
        .await?;

        self.state
            .commands
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

    pub async fn wait_for_coins(&mut self) {
        self.consume_until(|event| matches!(event, SyncEvent::CoinsUpdated { .. }))
            .await;
    }

    pub async fn wait_for_puzzles(&mut self) {
        self.consume_until(|event| matches!(event, SyncEvent::PuzzleBatchSynced))
            .await;
    }

    pub async fn new_block_with_current_time(&self) -> anyhow::Result<u64> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut sim = self.sim.lock().await;
        sim.set_next_timestamp(timestamp)?;
        sim.create_block();
        Ok(timestamp)
    }
}

pub fn default_test_options() -> SyncOptions {
    SyncOptions {
        dns_batch_size: 0,
        connection_batch_size: 0,
        max_peer_age_seconds: 0,
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
