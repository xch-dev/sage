use std::{sync::Arc, time::Duration};

use chia::{
    bls::{
        master_to_wallet_hardened, master_to_wallet_hardened_intermediate,
        master_to_wallet_unhardened_intermediate, DerivableKey, SecretKey,
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
use sage_config::TESTNET11;
use sage_database::Database;
use sqlx::{migrate, SqlitePool};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::timeout,
};
use tracing::debug;

use crate::{
    insert_transaction, PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts,
    Transaction, Wallet,
};

static INDEX: Mutex<u32> = Mutex::const_new(0);

#[derive(Debug)]
pub struct TestWallet {
    pub sim: Arc<PeerSimulator>,
    pub agg_sig: AggSigConstants,
    pub peer: Peer,
    pub wallet: Arc<Wallet>,
    pub master_sk: SecretKey,
    pub puzzle_hash: Bytes32,
    pub hardened_puzzle_hash: Bytes32,
    pub sender: Sender<SyncCommand>,
    pub events: Receiver<SyncEvent>,
    pub index: u32,
    pub state: Arc<Mutex<PeerState>>,
}

impl TestWallet {
    pub async fn new(balance: u64) -> anyhow::Result<Self> {
        let sim = PeerSimulator::new().await?;
        Self::with_sim(Arc::new(sim), balance, 0).await
    }

    pub async fn next(&self, balance: u64) -> anyhow::Result<Self> {
        Self::with_sim(self.sim.clone(), balance, self.index + 1).await
    }

    async fn with_sim(
        sim: Arc<PeerSimulator>,
        balance: u64,
        key_index: u32,
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
        db.run_rust_migrations().await?;

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
            tx.insert_derivation(p2_puzzle_hash, index, true, synthetic_key)
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
            sim.mint_coin(puzzle_hash.into(), balance).await;
        }

        let state = Arc::new(Mutex::new(PeerState::default()));
        let wallet = Arc::new(Wallet::new(
            db,
            fingerprint,
            intermediate_pk,
            genesis_challenge,
        ));

        let (mut sync_manager, sender, events) = SyncManager::new(
            SyncOptions {
                target_peers: 0,
                discover_peers: false,
                dns_batch_size: 0,
                connection_batch_size: 0,
                max_peer_age_seconds: 0,
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
            },
            state.clone(),
            Some(wallet.clone()),
            TESTNET11.clone(),
            Connector::Plain,
        );

        let (peer, receiver) = sim.connect_raw().await?;

        assert!(
            sync_manager
                .try_add_peer(peer.clone(), receiver, true, false)
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
            sender,
            events,
            index: key_index,
            state,
        };

        test.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;
        assert_eq!(test.wallet.db.balance().await?, balance as u128);

        Ok(test)
    }

    pub async fn transact(&self, coin_spends: Vec<CoinSpend>) -> anyhow::Result<()> {
        let spend_bundle = self
            .wallet
            .sign_transaction(coin_spends, &self.agg_sig, self.master_sk.clone(), false)
            .await?;

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

            debug!("Consuming event: {next:?}");

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
}
