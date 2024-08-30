use std::sync::Arc;

use chia_wallet_sdk::Peer;
use tokio::sync::Mutex;

use crate::wallet::Wallet;

use super::sync_state::SyncState;

pub async fn sync_wallet(wallet: Arc<Wallet>, peer: Peer) {}

pub async fn lookup_puzzles(wallet: Arc<Wallet>, state: Arc<Mutex<SyncState>>) {}

/*
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

                    let mut tx = self.db.tx().await?;

                    for coin_state in data.coin_states {
                        tx.try_insert_coin_state(coin_state).await?;
                    }

                    tx.commit().await?;

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
    } */
