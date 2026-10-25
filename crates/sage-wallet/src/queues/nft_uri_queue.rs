use std::{collections::HashMap, time::Duration};

use chia_wallet_sdk::prelude::*;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_assets::{Data, HEDGE_DELAY, RaceOptions, base64_data_uri, nft_sources, race_sources};
use sage_database::{Database, DatabaseTx, FileUri, NftMetadataInfo, ResizedImageKind};
use tokio::{sync::mpsc, time::sleep};
use tracing::{debug, info, warn};

use crate::{SyncEvent, WalletError, compute_nft_info};
use chia_wallet_sdk::types::TESTNET11_CONSTANTS;
use sage_config::Network;

/// The time limit for downloading a file, across all of its sources.
const FETCH_DEADLINE: Duration = Duration::from_mins(1);

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
            let has_data = self.db.file_has_data(hash).await?;
            futures.push(fetch_file(hash, items, has_data, testnet));
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

/// Fetches a file from MintGarden's archive, which is tried first since it's
/// content addressed and will work even if the NFT's own URIs have gone
/// offline, and then from the NFT's URIs (see [`nft_sources`]).
///
/// If `has_data` is set, a file that doesn't match its hash has already been
/// saved, and this is a periodic retry to see if the right file has become
/// available (for example, after MintGarden archives it).
async fn fetch_file(hash: Bytes32, items: Vec<FileUri>, has_data: bool, testnet: bool) -> Fetched {
    let uris: Vec<String> = items.into_iter().map(|item| item.uri).collect();

    let outcome = race_sources(
        nft_sources(&uris, hash),
        RaceOptions {
            expected_hash: Some(hash),
            hedge_delay: HEDGE_DELAY,
            deadline: FETCH_DEADLINE,
        },
    )
    .await;

    if let Some(download) = outcome.matched {
        return Fetched {
            hash,
            data: Some((download.into_data(testnet).await, true)),
            checked_uris: uris,
            failed_uris: Vec::new(),
        };
    }

    // Fall back to a file that doesn't match the hash, so that there's at least
    // something to show. It will be retried later, and replaced if the right
    // file turns up. If one has already been saved, keep it rather than doing
    // the work of saving it again.
    if let Some(download) = outcome.mismatch {
        warn!(
            "Hash mismatch for {hash} (found {}), from URIs {uris:?}",
            download.hash
        );

        let data = if has_data {
            None
        } else {
            Some((download.into_data(testnet).await, false))
        };

        return Fetched {
            hash,
            data,
            checked_uris: uris,
            failed_uris: Vec::new(),
        };
    }

    if let Some(error) = &outcome.last_error {
        debug!("Error fetching {hash} from URIs {uris:?}: {error}");
    }

    // Only count this against the URIs if it wasn't down to a temporary
    // problem, so that a file isn't given up on while the archive or a
    // gateway is rate limiting.
    if outcome.transient_failure {
        Fetched {
            hash,
            data: None,
            checked_uris: uris,
            failed_uris: Vec::new(),
        }
    } else {
        Fetched {
            hash,
            data: None,
            checked_uris: Vec::new(),
            failed_uris: uris,
        }
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

    // The file may be replacing one that didn't match, so clear out the old
    // resized images rather than adding to them.
    tx.delete_resized_images(hash).await?;

    if let Some(thumbnail) = data.thumbnail {
        tx.insert_resized_image(hash, ResizedImageKind::Icon, thumbnail.icon)
            .await?;

        tx.insert_resized_image(hash, ResizedImageKind::Thumbnail, thumbnail.thumbnail)
            .await?;
    }

    Ok(())
}
