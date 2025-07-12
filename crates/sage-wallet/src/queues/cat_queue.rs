use std::time::Duration;

use sage_assets::DexieCat;
use sage_database::{Asset, AssetKind, Database, TokenAsset};
//use serde::Deserialize;
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};

use crate::{SyncEvent, WalletError};

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
        let cats = timeout(Duration::from_secs(120), DexieCat::fetch_all(self.testnet)).await??;

        if cats.is_empty() {
            return Ok(());
        }

        let mut tx = self.db.tx().await?;

        for cat in cats {
            tx.insert_cat(TokenAsset {
                asset: Asset {
                    hash: cat.hash,
                    name: cat.name,
                    icon_url: cat.icon_url,
                    description: cat.description,
                    is_sensitive_content: false,
                    is_visible: true,
                    kind: AssetKind::Token,
                },
                ticker: cat.ticker,
                precision: 3,
            })
            .await?;
        }

        tx.commit().await?;

        self.sync_sender.send(SyncEvent::CatInfo).await.ok();

        Ok(())
    }
}
