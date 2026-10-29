use chia_wallet_sdk::prelude::*;
use sqlx::{SqliteExecutor, query};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VaultState {
    Active,
    /// Recovery has been started, and the vault will move to its recovery custody once the
    /// timelock expires, unless the current custody cancels it first.
    RecoveryPending,
    Melted,
}

impl Convert<VaultState> for i64 {
    fn convert(self) -> Result<VaultState> {
        Ok(match self {
            0 => VaultState::Active,
            1 => VaultState::RecoveryPending,
            2 => VaultState::Melted,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

/// The latest vault singleton seen on chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultRow {
    pub launcher_id: Bytes32,
    /// The singleton's inner puzzle hash, which can be rebuilt by the vault config with the same
    /// custody hash, if one is known.
    pub custody_hash: Bytes32,
    pub amount: u64,
    pub state: VaultState,
}

impl Database {
    pub async fn vault(&self, launcher_id: Bytes32) -> Result<Option<VaultRow>> {
        vault(&self.pool, launcher_id).await
    }

    pub async fn vaults(&self) -> Result<Vec<VaultRow>> {
        vaults(&self.pool).await
    }
}

impl DatabaseTx<'_> {
    pub async fn vault(&mut self, launcher_id: Bytes32) -> Result<Option<VaultRow>> {
        vault(&mut *self.tx, launcher_id).await
    }

    /// Inserts or updates the vault for an existing asset with the launcher id.
    pub async fn upsert_vault(&mut self, vault: VaultRow) -> Result<()> {
        upsert_vault(&mut *self.tx, vault).await
    }
}

async fn vault(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<VaultRow>> {
    let launcher_id_ref = launcher_id.as_ref();

    query!(
        "
        SELECT custody_hash, amount, state
        FROM vaults
        INNER JOIN assets ON assets.id = vaults.asset_id
        WHERE assets.hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(VaultRow {
            launcher_id,
            custody_hash: row.custody_hash.convert()?,
            amount: row.amount.convert()?,
            state: row.state.convert()?,
        })
    })
    .transpose()
}

async fn vaults(conn: impl SqliteExecutor<'_>) -> Result<Vec<VaultRow>> {
    query!(
        "
        SELECT assets.hash AS launcher_id, custody_hash, amount, state
        FROM vaults
        INNER JOIN assets ON assets.id = vaults.asset_id
        ORDER BY vaults.id ASC
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(VaultRow {
            launcher_id: row.launcher_id.convert()?,
            custody_hash: row.custody_hash.convert()?,
            amount: row.amount.convert()?,
            state: row.state.convert()?,
        })
    })
    .collect()
}

async fn upsert_vault(conn: impl SqliteExecutor<'_>, vault: VaultRow) -> Result<()> {
    let launcher_id = vault.launcher_id.as_ref();
    let custody_hash = vault.custody_hash.as_ref();
    let amount = vault.amount.to_be_bytes().to_vec();
    let state = vault.state as i64;

    query!(
        "
        INSERT INTO vaults (asset_id, custody_hash, amount, state)
        VALUES ((SELECT id FROM assets WHERE hash = ?), ?, ?, ?)
        ON CONFLICT(asset_id) DO UPDATE SET
            custody_hash = excluded.custody_hash,
            amount = excluded.amount,
            state = excluded.state
        ",
        launcher_id,
        custody_hash,
        amount,
        state
    )
    .execute(conn)
    .await?;

    Ok(())
}
