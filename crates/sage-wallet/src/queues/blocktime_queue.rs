use crate::wallet_peer::WalletPeer;
use crate::{PeerState, WalletError};

use sage_database::Database;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex, time::sleep};
use tracing::{error, info};

#[derive(Debug)]
#[allow(dead_code)]
pub struct BlockTimeQueue {
    db: Database,
    state: Arc<Mutex<PeerState>>,
}

impl BlockTimeQueue {
    pub fn new(db: Database, state: Arc<Mutex<PeerState>>) -> Self {
        info!("BlockTimeQueue initialized with an SQLite connection pool...");
        Self { db, state }
    }

    pub async fn start(mut self, delay: Duration) -> Result<(), WalletError> {
        loop {
            info!("\nLoop initiated");
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&mut self) -> Result<(), WalletError> {
        let cts_null_ht = self.db.find_created_timestamp_null().await?;
        //look for created height timestamp nulls first. highest to lowest.
        if let Some(cts_null_ht) = cts_null_ht {
            info!(
                "Found created timestamp (null) with height: {}",
                cts_null_ht
            );
            let cts_null_ht_u32: u32 = cts_null_ht as u32;
            //block_info to blockchain function with created as input
            self.fetch_and_process_blockinfo(cts_null_ht_u32).await?;
        //move on to looking for spent timestamp nulls
        } else {
            info!("Looking for spent timestamp(null).");
            //block_info to blockchain function with spent as input
            let sts_null_ht = self.db.find_spent_timestamp_null().await?;
            //first look for created height timestamp nulls
            if let Some(sts_null_ht) = sts_null_ht {
                info!("Found spent timestamp (null) with height: {}", sts_null_ht);
                let sts_null_ht_u32: u32 = sts_null_ht as u32;
                self.fetch_and_process_blockinfo(sts_null_ht_u32).await?;
            } else {
                info!("No spent timestamp (null) found. ***End Batch***");
                //exit gracefully here
            }
        }

        Ok(())
    }
    //
    async fn fetch_and_process_blockinfo(&self, height: u32) -> Result<(), WalletError> {
        // Attempt to get the blockinfo timestamp for the given height
        let check_blockinfo = self.db.check_blockinfo(height).await;

        // Handle the case when blockinfo is found or not
        match check_blockinfo {
            Ok(unix_time) => {
                // If the value is found, use the unix_time
                info!(
                    "******Found blockinfo for height {}: {}*******",
                    height, unix_time
                );
                // Do something with unix_time (maybe update coin_states)
                // Update the coininfo and coinstates in two places
                let timestamp_u32: u32 = unix_time as u32;
                if let Err(e) = self.update_coinstates(height, timestamp_u32).await {
                    error!("Failed to update coinstates: {:?}", e);
                    return Err(e); // Return error if update fails
                }
            }
            Err(e) => {
                // Handle the error case where no row in block_info found
                info!("Error must go to blockchain for height {}: {}", height, e);
                // Try to fetch the timestamp from the blockchain
                let Some(peer) = self.state.lock().await.acquire_peer() else {
                    return Ok(()); // If no peer is available, return early
                };

                match fetch_block_timestamp(&peer, height).await {
                    Ok(Some(timestamp)) => {
                        info!("Timestamp from blockchain source: {}", timestamp);
                        let timestamp_u32: u32 = timestamp as u32;

                        // Update the coininfo and coinstates in two places
                        if let Err(e) = self.update_coinstates(height, timestamp_u32).await {
                            error!("Failed to update coinstates: {:?}", e);
                            return Err(e); // Return error if update fails
                        }
                    }
                    Ok(None) => {
                        error!("No timestamp found for block {}", height);
                        return Err(WalletError::PeerMisbehaved); // Return an error if no timestamp found
                    }
                    Err(e) => {
                        error!("Failed to fetch block {} timestamp: {:?}", height, e);
                        return Err(e); // Propagate the error directly if fetch fails
                    }
                }
            }
        }

        Ok(())
    }
    //
    async fn insert_blockinfo(&self, height: u32, timestamp_u32: u32) -> Result<(), WalletError> {
        // Insert the timestamp and height into blockinfo
        self.db
            .insert_timestamp_height(height, timestamp_u32)
            .await?;
        info!("Blockinfo insert height and timestamp query attempted...");
        Ok(())
    }
    async fn update_coinstates(&self, height: u32, timestamp_u32: u32) -> Result<(), WalletError> {
        // Call the update_created_timestamp function
        let update_created = self
            .db
            .update_created_timestamp(height, timestamp_u32)
            .await?;
        info!(
            "Created timestamp update query attempted... {}",
            update_created
        );

        // Call the update_spent_timestamp function
        let update_spent = self
            .db
            .update_spent_timestamp(height, timestamp_u32)
            .await?;
        info!("Spent timestamp update query attempted... {}", update_spent);

        // Log success for both operations
        info!(
            "Coinstates created_unixtime and spent_unixtimestamp for height {} with timestamp {}",
            height, timestamp_u32
        );

        // Call insert_blockinfo to handle blockinfo insert
        self.insert_blockinfo(height, timestamp_u32).await?;

        Ok(())
    }
}

async fn fetch_block_timestamp(peer: &WalletPeer, height: u32) -> Result<Option<u64>, WalletError> {
    match peer.block_timestamp(height).await {
        Ok(Some(timestamp)) => Ok(Some(timestamp)), // Successfully fetched timestamp
        Ok(None) => Ok(None),                       // No timestamp found, return None
        Err(e) => Err(e),                           // Propagate the WalletError
    }
}
