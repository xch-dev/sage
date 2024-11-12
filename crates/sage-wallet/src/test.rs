use std::{sync::Arc, time::Duration};

use chia::{
    bls::{master_to_wallet_unhardened_intermediate, DerivableKey, SecretKey},
    protocol::{Bytes32, CoinSpend},
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::{
    test_secret_key, AggSigConstants, Connector, Network, Peer, PeerSimulator, TESTNET11_CONSTANTS,
};
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

#[derive(Debug)]
pub struct TestWallet {
    pub sim: PeerSimulator,
    pub peer: Peer,
    pub wallet: Arc<Wallet>,
    pub master_sk: SecretKey,
    pub puzzle_hash: Bytes32,
    pub sender: Sender<SyncCommand>,
    pub events: Receiver<SyncEvent>,
}

impl TestWallet {
    pub async fn new(pool: SqlitePool, balance: u64) -> anyhow::Result<Self> {
        migrate!("../../migrations").run(&pool).await?;
        let db = Database::new(pool);

        let sk = test_secret_key()?;
        let pk = sk.public_key();
        let fingerprint = pk.get_fingerprint();
        let intermediate_pk = master_to_wallet_unhardened_intermediate(&pk);
        let genesis_challenge = TESTNET11_CONSTANTS.genesis_challenge;

        let sim = PeerSimulator::new().await?;
        let puzzle_hash =
            StandardArgs::curry_tree_hash(intermediate_pk.derive_unhardened(0).derive_synthetic());

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
                dns_batch_size: 0,
                connection_batch_size: 0,
                max_peer_age_seconds: 0,
                timeouts: Timeouts {
                    sync_delay: Duration::from_millis(100),
                    nft_uri_delay: Duration::from_millis(100),
                    cat_delay: Duration::from_millis(100),
                    puzzle_delay: Duration::from_millis(100),
                    transaction_delay: Duration::from_millis(100),
                    ..Default::default()
                },
            },
            state,
            Some(wallet.clone()),
            "testnet11".to_string(),
            Network::default_testnet11(),
            Connector::Plain,
        );

        let (peer, receiver) = sim.connect_raw().await?;

        assert!(sync_manager.try_add_peer(peer.clone(), receiver).await);

        tokio::spawn(sync_manager.sync());

        let mut test = TestWallet {
            sim,
            peer,
            wallet,
            master_sk: sk,
            puzzle_hash: puzzle_hash.into(),
            sender,
            events,
        };

        test.consume_until(SyncEvent::Subscribed).await;
        assert_eq!(test.wallet.db.balance().await?, balance as u128);

        Ok(test)
    }

    pub async fn transact(&self, coin_spends: Vec<CoinSpend>) -> anyhow::Result<()> {
        let spend_bundle = self
            .wallet
            .sign_transaction(
                coin_spends,
                &AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
                self.master_sk.clone(),
            )
            .await?;

        let mut tx = self.wallet.db.tx().await?;

        let subscriptions = insert_transaction(
            &mut tx,
            spend_bundle.name(),
            Transaction::from_coin_spends(spend_bundle.coin_spends)?,
            spend_bundle.aggregated_signature,
        )
        .await?;

        tx.commit().await?;

        self.sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await?;

        Ok(())
    }

    pub async fn consume_until(&mut self, event: SyncEvent) {
        loop {
            let next = timeout(Duration::from_secs(10), self.events.recv())
                .await
                .expect("timed out listening for events")
                .expect("missing event");

            debug!("Consuming event: {next:?}");

            if event == next {
                return;
            }
        }
    }
}
