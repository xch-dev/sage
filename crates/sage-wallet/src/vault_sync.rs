//! Tracking vaults while syncing.
//!
//! A vault coin isn't hinted to anything the wallet owns, so a vault is followed from its launcher
//! instead. Once the launcher coin is synced, the puzzle queue follows the vault's lineage by
//! parsing each coin's child from its spend, and each new vault coin is subscribed to by coin ID.
//! Coins sent to the vault's addresses are found by subscribing to its P2 singleton puzzle hashes.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use chia_wallet_sdk::{driver::P2Singleton, prelude::*};
use sage_database::{DatabaseTx, VaultState};

use crate::{SyncEvent, WalletError};

/// How many unused addresses each vault keeps ahead of the highest address that has received a
/// coin, so that payments to them are found.
pub const VAULT_ADDRESS_GAP: u32 = 100;

/// Starts tracking a vault, and inserts its first addresses. Returns the launcher id, which must be
/// subscribed to by coin ID from the start of the chain, and the addresses, which must be
/// subscribed to by puzzle hash.
pub async fn track_vault(
    tx: &mut DatabaseTx<'_>,
    launcher_id: Bytes32,
) -> Result<Vec<Bytes32>, WalletError> {
    tx.insert_vault(launcher_id).await?;
    insert_vault_addresses(tx).await
}

/// Inserts addresses for each tracked vault, so that it has [`VAULT_ADDRESS_GAP`] unused addresses
/// past the highest one that has received a coin. Returns the puzzle hashes of the new addresses.
pub async fn insert_vault_addresses(tx: &mut DatabaseTx<'_>) -> Result<Vec<Bytes32>, WalletError> {
    let mut puzzle_hashes = Vec::new();

    for launcher_id in tx.vault_launcher_ids().await? {
        let (max_nonce, max_used_nonce) = tx.p2_singleton_nonces(launcher_id).await?;

        let start = max_nonce.map_or(0, |nonce| nonce + 1);
        let end = max_used_nonce.map_or(0, |nonce| nonce + 1) + VAULT_ADDRESS_GAP;

        for nonce in start..end {
            let p2_singleton = P2Singleton::new(launcher_id, nonce.try_into()?);
            tx.insert_p2_singleton_p2_puzzle(p2_singleton).await?;
            puzzle_hashes.push(p2_singleton.tree_hash().into());
        }
    }

    Ok(puzzle_hashes)
}

/// Records the states of vaults before they're changed, so that events can be sent for the ones
/// whose state changed.
#[derive(Debug, Default)]
pub struct VaultStateChanges {
    before: HashMap<Bytes32, VaultState>,
}

impl VaultStateChanges {
    /// Records the state of a vault, unless it's already been recorded.
    pub async fn record(
        &mut self,
        tx: &mut DatabaseTx<'_>,
        launcher_id: Bytes32,
    ) -> Result<(), WalletError> {
        if self.before.contains_key(&launcher_id) {
            return Ok(());
        }

        if let Some(status) = tx.vault_status(launcher_id).await? {
            self.before.insert(launcher_id, status.state);
        }

        Ok(())
    }

    /// Records the state of the vault which a coin belongs to, if any.
    pub async fn record_coin(
        &mut self,
        tx: &mut DatabaseTx<'_>,
        coin_id: Bytes32,
    ) -> Result<(), WalletError> {
        if let Some(launcher_id) = tx.vault_launcher_id(coin_id).await? {
            self.record(tx, launcher_id).await?;
        }

        Ok(())
    }

    /// The events for vaults whose state has changed since it was recorded.
    pub async fn events(self, tx: &mut DatabaseTx<'_>) -> Result<Vec<SyncEvent>, WalletError> {
        let mut events = Vec::new();

        for (launcher_id, before) in self.before {
            let Some(status) = tx.vault_status(launcher_id).await? else {
                continue;
            };

            if status.state != before {
                events.push(SyncEvent::VaultStateChanged {
                    launcher_id,
                    state: status.state,
                });
            }
        }

        Ok(events)
    }
}
