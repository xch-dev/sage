use std::ops::Range;

use chia_wallet_sdk::{
    chia::{
        bls::DerivableKey,
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
    },
    prelude::*,
};
use sage_database::{DatabaseTx, Derivation};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Inserts a range of unhardened derivations to the database.
    pub async fn insert_unhardened_derivations(
        &self,
        tx: &mut DatabaseTx<'_>,
        range: Range<u32>,
    ) -> Result<Vec<Bytes32>, WalletError> {
        let mut puzzle_hashes = Vec::new();

        for index in range {
            let synthetic_key = self
                .intermediate_pk
                .derive_unhardened(index)
                .derive_synthetic();

            let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();

            tx.insert_custody_p2_puzzle(
                p2_puzzle_hash,
                synthetic_key,
                Derivation {
                    derivation_index: index,
                    is_hardened: false,
                },
            )
            .await?;

            puzzle_hashes.push(p2_puzzle_hash);
        }

        Ok(puzzle_hashes)
    }

    pub async fn p2_puzzle_hashes(
        &self,
        count: u32,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<Bytes32>, WalletError> {
        let mut tx = self.db.tx().await?;

        let unused_index = tx.unused_derivation_index(hardened).await?;
        let next_index = tx.derivation_index(hardened).await?;

        let range = if reuse {
            let start = unused_index.saturating_sub(count);
            let end = next_index.min(start + count);
            start..end
        } else {
            unused_index..(unused_index + count)
        };

        if range.len() < count as usize {
            return Err(WalletError::InsufficientDerivations);
        }

        let mut p2_puzzle_hashes = Vec::new();

        for index in range {
            let p2_puzzle_hash = tx.custody_p2_puzzle_hash(index, hardened).await?;
            p2_puzzle_hashes.push(p2_puzzle_hash);
        }

        tx.commit().await?;

        Ok(p2_puzzle_hashes)
    }

    pub async fn change_p2_puzzle_hash(&self) -> Result<Bytes32, WalletError> {
        if let Some(change_p2_puzzle_hash) = self.change_p2_puzzle_hash {
            return Ok(change_p2_puzzle_hash);
        }
        Ok(self.p2_puzzle_hashes(1, false, true).await?[0])
    }
}
