use crate::{PeerState, SyncEvent, WalletError, WalletPeer};

use futures_util::{stream::FuturesUnordered, StreamExt};
use sage_database::Database;
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::{error, info};

#[derive(Debug)]
#[allow(dead_code)]
pub struct BlockTimeQueue {
    db: Database,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl BlockTimeQueue {
    pub fn new(
        db: Database,
        state: Arc<Mutex<PeerState>>,
        sync_sender: mpsc::Sender<SyncEvent>,
    ) -> Self {
        Self {
            db,
            state,
            sync_sender,
        }
    }

    pub async fn start(mut self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&mut self) -> Result<(), WalletError> {
        // Look for missing created timestamps, ordered by height highest to lowest.
        // Followed by spent timestamps, ordered by height highest to lowest.
        let peers = self.state.lock().await.peers();
        let limit = 5 * u32::try_from(peers.len())?;

        let heights = self.db.unsynced_blocks(limit).await?;

        if heights.is_empty() {
            return Ok(());
        }

        info!("Looking up timestamps with heights: {heights:?}");

        let mut tasks = FuturesUnordered::new();
        let mut heights = heights.into_iter();

        for peer in peers {
            for _ in 0..5 {
                if let Some(height) = heights.next() {
                    tasks.push(self.fetch_block(peer.clone(), height));
                }
            }
        }

        while let Some(result) = tasks.next().await {
            if let Err(error) = result {
                error!("Error fetching and processing blockinfo: {error}");
            }
        }

        self.sync_sender.send(SyncEvent::CoinsUpdated).await.ok();

        Ok(())
    }

    async fn fetch_block(&self, peer: WalletPeer, height: u32) -> Result<(), WalletError> {
        match peer.block_timestamp(height).await {
            Ok((header_hash, timestamp)) => {
                self.db
                    .insert_block(height, header_hash, Some(timestamp.try_into()?), false)
                    .await?;
            }
            Err(error) => {
                error!("Failed to fetch block {height} timestamp: {error}");
                return Err(error);
            }
        }

        Ok(())
    }
}
