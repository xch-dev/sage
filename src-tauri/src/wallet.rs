use chia::{
    bls::PublicKey,
    protocol::{Bytes32, CoinStateFilters, RejectStateReason},
};
use sage_client::Peer;
use sage_database::Database;
use tauri::{AppHandle, Emitter};
use tracing::{debug, instrument, warn};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Wallet {
    pub app_handle: AppHandle,
    pub db: Database,
    pub fingerprint: u32,
    pub _intermediate_pk: PublicKey,
    pub genesis_challenge: Bytes32,
}

impl Wallet {
    pub fn new(
        app_handle: AppHandle,
        db: Database,
        fingerprint: u32,
        intermediate_pk: PublicKey,
        genesis_challenge: Bytes32,
    ) -> Self {
        Self {
            app_handle,
            db,
            fingerprint,
            _intermediate_pk: intermediate_pk,
            genesis_challenge,
        }
    }

    pub fn fingerprint(&self) -> u32 {
        self.fingerprint
    }

    #[instrument(skip(self, peer))]
    pub async fn sync_against(&self, peer: &Peer, batch_size: u32) -> Result<()> {
        let mut tx = self.db.tx().await?;

        let (start_height, start_header_hash) = tx
            .latest_peak()
            .await?
            .map_or((None, self.genesis_challenge), |(height, header_hash)| {
                (Some(height), header_hash)
            });

        debug!(
            "Syncing from previous height {:?} with header hash {}",
            start_height, start_header_hash
        );

        let puzzle_hashes = tx.p2_puzzle_hashes().await?;

        tx.commit().await?;

        for batch in puzzle_hashes.chunks(batch_size as usize) {
            self.sync_puzzle_hashes(peer, start_height, start_header_hash, batch)
                .await?;
        }

        Ok(())
    }

    async fn sync_puzzle_hashes(
        &self,
        peer: &Peer,
        start_height: Option<u32>,
        start_header_hash: Bytes32,
        puzzle_hashes: &[Bytes32],
    ) -> Result<()> {
        let mut prev_height = start_height;
        let mut prev_header_hash = start_header_hash;

        loop {
            debug!(
                "Requesting puzzle state from previous height {:?} with header hash {}",
                prev_height, prev_header_hash
            );

            let response = peer
                .request_puzzle_state(
                    puzzle_hashes.to_vec(),
                    prev_height,
                    prev_header_hash,
                    CoinStateFilters::new(true, true, true, 0),
                    true,
                )
                .await?;

            match response {
                Ok(data) => {
                    debug!("Received {} coin states", data.coin_states.len());

                    let has_coin_states = !data.coin_states.is_empty();

                    let mut tx = self.db.tx().await?;

                    for coin_state in data.coin_states {
                        tx.try_insert_coin_state(coin_state).await?;
                    }

                    tx.commit().await?;

                    if has_coin_states {
                        if let Err(error) = self.app_handle.emit("coin-update", ()) {
                            warn!("Failed to emit coin update: {error}");
                        }
                    }

                    if data.is_finished {
                        break;
                    }

                    prev_height = Some(data.height);
                    prev_header_hash = data.header_hash;
                }
                Err(rejection) => match rejection.reason {
                    RejectStateReason::ExceededSubscriptionLimit => {
                        warn!("Subscription limit reached");
                        return Err(Error::SubscriptionLimitReached);
                    }
                    RejectStateReason::Reorg => {
                        todo!();
                    }
                },
            }
        }

        Ok(())
    }
}
