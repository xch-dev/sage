use crate::wallet_peer::WalletPeer; // Import WalletPeer from wallet_peer module
use crate::{PeerState, WalletError};

use sage_database::Database; // Import the Database type from sage_database crate
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
            info!("*******Loop initiated*********");
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&mut self) -> Result<(), WalletError> {
        let cts_null_ht = self.db.find_created_timestamp_null().await?;
        if let Some(cts_null_ht) = cts_null_ht {
            // If Some(i64) is returned, log or print the value //look in blockinfo and then chia blokchain
            println!(
                "Found created timestamp (null) with height: {}",
                cts_null_ht
            );
            //do something
            let cts_null_ht_u32: u32 = cts_null_ht as u32;
            let check_blockinfo = self.db.get_timestamp_blockinfo(cts_null_ht_u32).await;
            //this is to handle an error in the case of zero rows. not ideal imo gdn 20250225
            match check_blockinfo {
                Ok(unix_time) => {
                    // If the value is found, use the unix_time
                    println!(
                        "Found unix_time for height {}: {}",
                        cts_null_ht_u32, unix_time
                    );
                    // Do something with unix_time (maybe update coin_states)
                }
                Err(e) => {
                    // Handle the error case where no row in block_info found
                    println!(
                        "Error fetching unix_time for height {}: {}",
                        cts_null_ht_u32, e
                    );
                    //look on Chia blockchain if possible through an error
                    let Some(peer) = self.state.lock().await.acquire_peer() else {
                        return Ok(());
                    };

                    let height = cts_null_ht_u32;
                    match fetch_block_timestamp(&peer, height).await {
                        Ok(Some(timestamp)) => {
                            info!("Timestamp from blockchain source: {}", timestamp);
                            let timestamp_u32: u32 = timestamp as u32;
                            //update coininfo and coinstates in two places gdn 20250225
                            // Call update_coinstates method here
                            // self.update_coinstates(height, timestamp_u32).await?;
                            // Ok(())
                            if let Err(e) = self.update_coinstates(height, timestamp_u32).await {
                                error!("Failed to update coinstates: {:?}", e);
                                return Err(e); // Return error if update fails
                            }
                        }
                        Ok(None) => {
                            error!("No timestamp found for block {}", height);
                            return Err(WalletError::PeerMisbehaved); // Return an error directly
                        }
                        Err(e) => {
                            error!("Failed to fetch block {} timestamp: {:?}", height, e);
                            return Err(e); // Propagate the error directly
                        }
                    }
                }
            }
            // Match on the Result from get_timestamp_blockinfo
        } else {
            // If None is returned, log or print that no result was found //move on to looking for spent timestamp nulls
            println!("No created timestamp (null) found. Looking for spent timestamp(null).");
            // Check for spent timestamp if no created timestamp is found
            let sts_null = self.db.find_spent_timestamp_null().await?;
            if let Some(sts_null) = sts_null {
                println!("Found spent timestamp (null) with height: {}", sts_null);
                //do something else
            } else {
                println!("No spent timestamp (null) found. ***End Batch***");
                //exit gracefully here
            }
        }

        Ok(())
    }
    // Define update_coinstates as a separate method of BlockTimeQueue
    async fn update_coinstates(&self, height: u32, timestamp_u32: u32) -> Result<(), WalletError> {
        // Call the insert_created_timestamp function
        let insert_created = self
            .db
            .insert_created_timestamp(height, timestamp_u32)
            .await?;
        info!(
            "Created timestamp update query attempted... {}",
            insert_created
        );

        // Call the insert_spent_timestamp function
        let insert_spent = self
            .db
            .insert_spent_timestamp(height, timestamp_u32)
            .await?;
        info!("Spent timestamp insert query attempted... {}", insert_spent);

        // Log success for both operations
        info!(
            "Coinstates created_unixtime and spent_unixtimestamp for height {} with timestamp {}",
            height, timestamp_u32
        );

        //call insert timestamp and height into blockinfo //build me gdn 20250225******************
        self.db
            .insert_timestamp_height(height, timestamp_u32)
            .await?;
        info!("Blockinfo insert height and timestamp query attempted...",);

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
