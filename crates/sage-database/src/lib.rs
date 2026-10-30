mod maintenance;
mod nft_metadata;
mod serialized_primitives;
mod tables;
mod utils;

#[cfg(test)]
mod tests;

pub use maintenance::*;
pub use nft_metadata::*;
pub use serialized_primitives::*;
pub use tables::*;

pub(crate) use utils::*;

use std::num::TryFromIntError;

use sqlx::{Sqlite, SqlitePool, Transaction as SqliteTransaction};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Database {
    pub(crate) pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Begins a transaction that may write.
    pub async fn tx(&self) -> Result<DatabaseTx<'_>> {
        let tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        Ok(DatabaseTx::new(tx))
    }

    /// Begins a read-only transaction, for callers that need several reads to see
    /// a consistent snapshot but never write.
    pub async fn read_tx(&self) -> Result<DatabaseTx<'_>> {
        let tx = self.pool.begin().await?;
        Ok(DatabaseTx::new(tx))
    }

    pub async fn run_rust_migrations(&self, ticker: String) -> Result<()> {
        let mut tx = self.tx().await?;

        let version = tx.rust_migration_version().await?;

        info!("The current Sage migration version is {version}");

        if version < 1 {
            let ticker_upper = ticker.to_uppercase();
            info!("Migrating to version 1 - setting chia token ticker to {ticker_upper}");
            sqlx::query!("UPDATE assets SET ticker = ? WHERE id = 0", ticker_upper)
                .execute(&mut *tx.tx)
                .await?;
            tx.set_rust_migration_version(1).await?;
        }

        if version < 2 {
            info!("Migrating to version 2 - parsing display names of mutable NFTs");
            tx.backfill_nft_display_names().await?;
            tx.set_rust_migration_version(2).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct DatabaseTx<'a> {
    pub(crate) tx: SqliteTransaction<'a, Sqlite>,
}

impl<'a> DatabaseTx<'a> {
    pub fn new(tx: SqliteTransaction<'a, Sqlite>) -> Self {
        Self { tx }
    }

    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }

    pub async fn rollback(self) -> Result<()> {
        Ok(self.tx.rollback().await?)
    }

    pub async fn rust_migration_version(&mut self) -> Result<i64> {
        let row = sqlx::query_scalar!("SELECT version FROM rust_migrations LIMIT 1")
            .fetch_one(&mut *self.tx)
            .await?;

        Ok(row)
    }

    pub async fn set_rust_migration_version(&mut self, version: i64) -> Result<()> {
        sqlx::query!("UPDATE rust_migrations SET version = ?", version)
            .execute(&mut *self.tx)
            .await?;

        Ok(())
    }
}

/// The database shared by every wallet on a network, for what isn't specific to one wallet:
/// the address book, and the handle mappings Sage has verified.
///
/// It's created with the same migrations as wallet databases, so that its queries are
/// checked against the same schema at compile time, but only the `xchandles` and `contacts`
/// tables are used in it. Queries for those tables are only available here, so they can't
/// be run against a wallet's database by mistake, and none of them read the wallet tables,
/// which are always empty here.
///
/// The `xchandles` table was first created for wallet databases, which may still have rows
/// in it from before, but nothing reads them anymore.
#[derive(Debug, Clone)]
pub struct SharedDatabase {
    pub(crate) pool: SqlitePool,
}

impl SharedDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Begins a transaction that may write.
    pub async fn tx(&self) -> Result<SharedDatabaseTx<'_>> {
        let tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        Ok(SharedDatabaseTx { tx })
    }
}

#[derive(Debug)]
pub struct SharedDatabaseTx<'a> {
    pub(crate) tx: SqliteTransaction<'a, Sqlite>,
}

impl SharedDatabaseTx<'_> {
    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }

    pub async fn rollback(self) -> Result<()> {
        Ok(self.tx.rollback().await?)
    }
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Precision lost during cast")]
    PrecisionLost(#[from] TryFromIntError),

    #[error("Invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),

    #[error("BLS error: {0}")]
    Bls(#[from] chia_wallet_sdk::chia::bls::Error),

    #[error("Invalid enum variant")]
    InvalidEnumVariant,

    #[error("Invalid address")]
    InvalidAddress,

    #[error("Option underlying not found")]
    OptionUnderlyingNotFound,

    #[error("Public key not found for puzzle hash")]
    PublicKeyNotFound,

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid vault config")]
    InvalidVaultConfig,
}

pub(crate) type Result<T> = std::result::Result<T, DatabaseError>;
