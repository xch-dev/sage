use std::time::Duration;

use sage_database::{CatRow, Database};
use tokio::{sync::mpsc, time::sleep};
use tracing::info;

use crate::{try_lookup_cat, SyncEvent, WalletError};

#[derive(Debug)]
pub struct CatQueue {
    db: Database,
    testnet: bool,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl CatQueue {
    pub fn new(db: Database, testnet: bool, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self {
            db,
            testnet,
            sync_sender,
        }
    }

    pub async fn start(self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let Some(asset_id) = self.db.unfetched_cat().await? else {
            return Ok(());
        };

        info!(
            "Looking up CAT with asset id {} from spacescan.io",
            asset_id
        );

        let asset = try_lookup_cat(asset_id, self.testnet).await;

        self.db
            .update_cat(CatRow {
                asset_id,
                name: asset.name,
                ticker: asset.ticker,
                description: asset.description,
                icon: asset.icon_url,
                visible: true,
                fetched: true,
            })
            .await?;

        self.sync_sender.send(SyncEvent::CatInfo).await.ok();

        Ok(())
    }
}
