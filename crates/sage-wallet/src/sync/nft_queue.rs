use std::time::Duration;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    puzzles::nft::NftMetadata,
};
use clvmr::Allocator;
use sage_database::{Database, NftRow, NftUriKind};
use tokio::{sync::mpsc, time::sleep};
use tracing::info;

use crate::{ParseError, WalletError};

use super::SyncEvent;

#[derive(Debug)]
pub struct NftQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl NftQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self { db, sync_sender }
    }

    pub async fn start(self) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        self.db.delete_nfts().await?;

        let nfts = self.db.updated_nft_coins().await?;

        if nfts.is_empty() {
            return Ok(());
        }

        info!("Caching data for {} NFTs", nfts.len());

        for nft in nfts {
            let mut allocator = Allocator::new();

            let metadata_ptr = nft
                .info
                .metadata
                .to_clvm(&mut allocator)
                .map_err(|_| ParseError::AllocateMetadata)?;

            let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

            let mut tx = self.db.tx().await?;

            tx.update_nft(NftRow {
                launcher_id: nft.info.launcher_id,
                coin_id: nft.coin.coin_id(),
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
                metadata_json: metadata.as_ref().map(|_| "{}".to_string()),
            })
            .await?;

            if let Some(metadata) = metadata {
                tx.clear_nft_uris(nft.info.launcher_id).await?;

                for uri in metadata.data_uris {
                    tx.insert_nft_uri(nft.info.launcher_id, uri, NftUriKind::Data)
                        .await?;
                }

                for uri in metadata.metadata_uris {
                    tx.insert_nft_uri(nft.info.launcher_id, uri, NftUriKind::Metadata)
                        .await?;
                }

                for uri in metadata.license_uris {
                    tx.insert_nft_uri(nft.info.launcher_id, uri, NftUriKind::License)
                        .await?;
                }
            }

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftUpdate).await.ok();

        Ok(())
    }
}
