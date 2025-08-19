use std::time::Duration;

use sage_assets::DexieCat;
use sage_database::{Asset, AssetKind, Database};
//use serde::Deserialize;
use tokio::time::{sleep, timeout};

use crate::{SyncEvent, SyncState, WalletError};

#[derive(Debug)]
pub struct CatQueue {
    db: Database,
    state: SyncState,
    testnet: bool,
}

impl CatQueue {
    pub fn new(db: Database, state: SyncState, testnet: bool) -> Self {
        Self { db, state, testnet }
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
            tx.insert_asset(Asset {
                hash: cat.hash,
                name: cat.name,
                ticker: cat.ticker,
                precision: 3,
                icon_url: cat.icon_url,
                description: cat.description,
                is_sensitive_content: false,
                is_visible: true,
                hidden_puzzle_hash: None,
                kind: AssetKind::Token,
            })
            .await?;
        }

        tx.commit().await?;

        self.state.events.send(SyncEvent::CatInfo).await.ok();

        Ok(())
    }
}
