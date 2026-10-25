use std::{collections::HashMap, time::Duration};

use chia_wallet_sdk::prelude::*;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_assets::{Data, Download, base64_data_uri, download_mintgarden_archive, download_uri};
use sage_database::{Database, DatabaseTx, FileUri, NftMetadataInfo, ResizedImageKind};
use tokio::{sync::mpsc, time::sleep};
use tracing::{debug, info, warn};

use crate::{SyncEvent, WalletError, compute_nft_info};
use chia_wallet_sdk::types::TESTNET11_CONSTANTS;
use sage_config::Network;

#[derive(Debug)]
pub struct NftUriQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
    network: Network,
}

impl NftUriQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>, network: Network) -> Self {
        Self {
            db,
            sync_sender,
            network,
        }
    }

    pub async fn start(self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let batch = self.db.candidates_for_download(60 * 60 * 24, 3, 25).await?;

        if batch.is_empty() {
            return Ok(());
        }

        info!("Processing batch of {} NFT URIs", batch.len());

        let mut groups: HashMap<Bytes32, Vec<FileUri>> = HashMap::new();

        for item in batch {
            groups.entry(item.hash).or_default().push(item);
        }

        let mut futures = FuturesUnordered::new();
        let testnet = self.network.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

        for (hash, items) in groups {
            futures.push(fetch_file(hash, items, testnet));
        }

        while let Some(fetched) = futures.next().await {
            let mut tx = self.db.tx().await?;

            if let Some((data, is_hash_match)) = fetched.data {
                save_data(&mut tx, fetched.hash, data, is_hash_match).await?;
            }

            for uri in fetched.checked_uris {
                tx.update_checked_uri(fetched.hash, uri).await?;
            }

            for uri in fetched.failed_uris {
                tx.update_failed_uri(fetched.hash, uri).await?;
            }

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftData).await.ok();

        Ok(())
    }
}

#[derive(Debug)]
struct Fetched {
    hash: Bytes32,
    data: Option<(Data, bool)>,
    checked_uris: Vec<String>,
    failed_uris: Vec<String>,
}

/// Tries MintGarden's archive first, since it's content addressed and will
/// work even if the NFT's own URIs have gone offline. If it doesn't have the
/// file, falls back to fetching each of the NFT's URIs.
async fn fetch_file(hash: Bytes32, items: Vec<FileUri>, testnet: bool) -> Fetched {
    let uris = items.into_iter().map(|item| item.uri);

    let archive_unavailable = match download_mintgarden_archive(hash).await {
        Ok(Some(download)) => {
            return Fetched {
                hash,
                data: Some((download.into_data(testnet).await, true)),
                checked_uris: uris.collect(),
                failed_uris: Vec::new(),
            };
        }
        Ok(None) => false,
        Err(error) => {
            debug!("Error fetching {hash} from MintGarden archive: {error}");
            error.is_transient()
        }
    };

    let mut futures = FuturesUnordered::new();

    for uri in uris {
        futures.push(async move {
            let result = download_uri(&uri, Some(hash), true).await;
            (uri, result)
        });
    }

    let mut best: Option<Download> = None;
    let mut checked_uris = Vec::new();
    let mut failed_uris = Vec::new();

    while let Some((uri, result)) = futures.next().await {
        match result {
            Ok(download) => {
                if download.hash != hash {
                    warn!(
                        "Hash mismatch for URI {uri} (expected {hash} but found {})",
                        download.hash
                    );
                }

                // Prefer a file that matches the hash, but fall back to one
                // that doesn't so that there's at least something to show.
                if best
                    .as_ref()
                    .is_none_or(|best| best.hash != hash && download.hash == hash)
                {
                    best = Some(download);
                }

                checked_uris.push(uri);
            }
            Err(error) => {
                debug!("Error fetching URI {uri}: {error}");
                failed_uris.push(uri);
            }
        }
    }

    let best = best.filter(|download| {
        // A mismatching file is saved permanently, so don't settle for one if
        // the archive might have had the right file but was temporarily
        // unavailable (such as being rate limited).
        let keep = download.hash == hash || !archive_unavailable;

        if !keep {
            debug!("Not saving mismatched file for {hash} since the archive was unavailable");
        }

        keep
    });

    let data = match best {
        Some(download) => {
            let is_hash_match = download.hash == hash;
            Some((download.into_data(testnet).await, is_hash_match))
        }
        None => None,
    };

    Fetched {
        hash,
        data,
        checked_uris,
        failed_uris,
    }
}

async fn save_data(
    tx: &mut DatabaseTx<'_>,
    hash: Bytes32,
    data: Data,
    is_hash_match: bool,
) -> Result<(), WalletError> {
    let icon_url = data
        .thumbnail
        .as_ref()
        .map(|thumbnail| base64_data_uri(&thumbnail.icon, "image/png"));

    if let Some(icon_url) = icon_url {
        tx.update_nft_data_hash_urls(hash, icon_url).await?;
    }

    for nft in tx.nfts_with_metadata_hash(hash).await? {
        let info = compute_nft_info(nft.minter_hash, &data.blob);

        let collection_id = info.collection.as_ref().map(|col| col.hash);

        if let Some(collection) = info.collection {
            tx.insert_collection(collection).await?;
        }

        tx.update_nft_metadata(
            nft.hash,
            NftMetadataInfo {
                name: info.name,
                description: info.description,
                is_sensitive_content: info.sensitive_content,
                collection_id: collection_id.unwrap_or_default(),
            },
        )
        .await?;
    }

    tx.update_file(hash, data.blob, data.mime_type, is_hash_match)
        .await?;

    if let Some(thumbnail) = data.thumbnail {
        tx.insert_resized_image(hash, ResizedImageKind::Icon, thumbnail.icon)
            .await?;

        tx.insert_resized_image(hash, ResizedImageKind::Thumbnail, thumbnail.thumbnail)
            .await?;
    }

    Ok(())
}
