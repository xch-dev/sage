use chia_wallet_sdk::prelude::*;
use sqlx::{SqliteExecutor, query};

use crate::{Convert, Database, DatabaseTx, Result};

/// A vault recovery that was started on chain, recorded from the spend that started it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultRecoveryRow {
    /// The custody hash of the vault in the recovery state.
    pub custody_hash: Bytes32,
    /// The left side of the recovery state's 1 of 2, which is the custody that can cancel it.
    pub custody_member_hash: Bytes32,
    /// The right side of the recovery state's 1 of 2, which finishes recovery after the timelock.
    pub recovery_member_hash: Bytes32,
    /// The puzzle hash of the timelocked member that finishes recovery.
    pub finish_puzzle_hash: Bytes32,
    pub recovery_timelock: u64,
    /// Whether this wallet started the recovery.
    pub is_started_locally: bool,
}

impl Database {
    pub async fn vault_recovery(&self, custody_hash: Bytes32) -> Result<Option<VaultRecoveryRow>> {
        vault_recovery(&self.pool, custody_hash).await
    }
}

impl DatabaseTx<'_> {
    pub async fn vault_recovery(
        &mut self,
        custody_hash: Bytes32,
    ) -> Result<Option<VaultRecoveryRow>> {
        vault_recovery(&mut *self.tx, custody_hash).await
    }

    /// Records a recovery. A recovery that's already recorded is kept as started locally if it
    /// was, since sync records the recoveries that this wallet started once they're confirmed.
    pub async fn insert_vault_recovery(&mut self, row: VaultRecoveryRow) -> Result<()> {
        let custody_hash = row.custody_hash.as_ref();
        let custody_member_hash = row.custody_member_hash.as_ref();
        let recovery_member_hash = row.recovery_member_hash.as_ref();
        let finish_puzzle_hash = row.finish_puzzle_hash.as_ref();
        let recovery_timelock: i64 = row.recovery_timelock.try_into()?;

        query!(
            "
            INSERT INTO vault_recoveries (
                custody_hash, custody_member_hash, recovery_member_hash, finish_puzzle_hash,
                recovery_timelock, is_started_locally
            )
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(custody_hash) DO UPDATE SET
                is_started_locally = is_started_locally OR excluded.is_started_locally
            ",
            custody_hash,
            custody_member_hash,
            recovery_member_hash,
            finish_puzzle_hash,
            recovery_timelock,
            row.is_started_locally,
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}

pub(crate) async fn vault_recovery(
    conn: impl SqliteExecutor<'_>,
    custody_hash: Bytes32,
) -> Result<Option<VaultRecoveryRow>> {
    let custody_hash_ref = custody_hash.as_ref();

    query!(
        "
        SELECT
            custody_member_hash, recovery_member_hash, finish_puzzle_hash, recovery_timelock,
            is_started_locally
        FROM vault_recoveries
        WHERE custody_hash = ?
        ",
        custody_hash_ref
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(VaultRecoveryRow {
            custody_hash,
            custody_member_hash: row.custody_member_hash.convert()?,
            recovery_member_hash: row.recovery_member_hash.convert()?,
            finish_puzzle_hash: row.finish_puzzle_hash.convert()?,
            recovery_timelock: row.recovery_timelock.convert()?,
            is_started_locally: row.is_started_locally,
        })
    })
    .transpose()
}
