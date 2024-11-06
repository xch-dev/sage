use std::ops::Range;

use chia::{
    bls::DerivableKey,
    protocol::Bytes32,
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use sage_database::DatabaseTx;

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

            tx.insert_derivation(p2_puzzle_hash, index, false, synthetic_key)
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

        let max_used = tx.max_used_derivation_index(hardened).await?;
        let next_index = tx.derivation_index(hardened).await?;

        let (mut start, mut end) = if reuse {
            let start = max_used.unwrap_or(0);
            let end = next_index.min(start + count);
            (start, end)
        } else {
            let start = max_used.map_or(0, |i| i + 1);
            let end = next_index.min(start + count);
            (start, end)
        };

        if end - start < count && reuse {
            start = start.saturating_sub(count - (end - start));
        }

        if end - start < count {
            end = next_index.min(end + count - (end - start));
        }

        if end - start < count {
            return Err(WalletError::InsufficientDerivations);
        }

        let mut p2_puzzle_hashes = Vec::new();

        for index in start..end {
            let p2_puzzle_hash = tx.p2_puzzle_hash(index, hardened).await?;
            p2_puzzle_hashes.push(p2_puzzle_hash);
        }

        tx.commit().await?;

        Ok(p2_puzzle_hashes)
    }

    pub async fn p2_puzzle_hash(
        &self,
        hardened: bool,
        reuse: bool,
    ) -> Result<Bytes32, WalletError> {
        Ok(self.p2_puzzle_hashes(1, hardened, reuse).await?[0])
    }
}
