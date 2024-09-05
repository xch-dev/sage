use std::{collections::HashMap, time::Duration};

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    puzzles::nft::NftMetadata,
};
use clvmr::Allocator;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_database::{Database, NftRow, NftUriKind};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::info;

use crate::{ParseError, WalletError};

use super::SyncEvent;

#[derive(Debug)]
pub struct NftQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

struct Uris {
    data: Vec<String>,
    metadata: Vec<String>,
    license: Vec<String>,
}

impl NftQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self { db, sync_sender }
    }

    pub async fn start(self) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let nfts = self.db.updated_nft_coins(100).await?;

        if nfts.is_empty() {
            return Ok(());
        }

        info!("Caching data for {} NFTs", nfts.len());

        let mut partial_rows = HashMap::new();
        let mut futures = FuturesUnordered::new();

        for nft in &nfts {
            let mut allocator = Allocator::new();

            let metadata_ptr = nft
                .info
                .metadata
                .to_clvm(&mut allocator)
                .map_err(|_| ParseError::AllocateMetadata)?;

            let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

            for uri in metadata
                .as_ref()
                .map(|m| m.metadata_uris.clone())
                .into_iter()
                .flatten()
                .take(5)
            {
                let launcher_id = nft.info.launcher_id;

                futures.push(async move {
                    let result = timeout(Duration::from_secs(5), reqwest::get(uri.clone())).await;
                    (launcher_id, uri, result)
                });
            }

            let partial_row = NftRow {
                launcher_id: nft.info.launcher_id,
                coin_id: nft.coin_id,
                p2_puzzle_hash: nft.info.p2_puzzle_hash,
                royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                current_owner: nft.info.current_owner,
                data_hash: metadata.as_ref().and_then(|meta| meta.data_hash),
                metadata_hash: metadata.as_ref().and_then(|meta| meta.metadata_hash),
                license_hash: metadata.as_ref().and_then(|meta| meta.license_hash),
                edition_number: metadata
                    .as_ref()
                    .and_then(|meta| meta.edition_number.try_into().ok()),
                edition_total: metadata
                    .as_ref()
                    .and_then(|meta| meta.edition_total.try_into().ok()),
                metadata_json: None,
            };

            let uris = Uris {
                data: metadata
                    .as_ref()
                    .map(|meta| meta.data_uris.clone())
                    .into_iter()
                    .flatten()
                    .collect(),
                metadata: metadata
                    .as_ref()
                    .map(|meta| meta.metadata_uris.clone())
                    .into_iter()
                    .flatten()
                    .collect(),
                license: metadata
                    .as_ref()
                    .map(|meta| meta.license_uris.clone())
                    .into_iter()
                    .flatten()
                    .collect(),
            };

            partial_rows.insert(nft.info.launcher_id, (partial_row, uris));
        }

        while let Some((launcher_id, uri, result)) = futures.next().await {
            let text = match result {
                Ok(Ok(response)) => timeout(Duration::from_secs(3), response.text()).await,
                Err(_timeout) => {
                    info!("Timeout fetching {} for {}", uri, launcher_id);
                    continue;
                }
                Ok(Err(error)) => {
                    info!("Error fetching {} for {}: {}", uri, launcher_id, error);
                    continue;
                }
            };

            match text {
                Ok(Ok(text)) => {
                    partial_rows
                        .get_mut(&launcher_id)
                        .expect("missing partial row")
                        .0
                        .metadata_json = Some(text);
                }
                Ok(Err(error)) => {
                    info!("Error reading {} for {}: {}", uri, launcher_id, error);
                }
                Err(_timeout) => {
                    info!("Timeout reading {} for {}", uri, launcher_id);
                }
            }
        }

        for (row, uris) in partial_rows.into_values() {
            let launcher_id = row.launcher_id;

            let mut tx = self.db.tx().await?;
            tx.update_nft(row).await?;
            tx.clear_nft_uris(launcher_id).await?;

            for uri in uris.data {
                tx.insert_nft_uri(launcher_id, uri, NftUriKind::Data)
                    .await?;
            }

            for uri in uris.metadata {
                tx.insert_nft_uri(launcher_id, uri, NftUriKind::Metadata)
                    .await?;
            }

            for uri in uris.license {
                tx.insert_nft_uri(launcher_id, uri, NftUriKind::License)
                    .await?;
            }

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftUpdate).await.ok();

        Ok(())
    }
}
