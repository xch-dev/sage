mod coin_states;
mod offers;
mod primitives;
mod tables;
mod utils;

pub use tables::*;

pub(crate) use utils::*;

use std::num::TryFromIntError;

use sqlx::{Sqlite, SqlitePool, Transaction};
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

    pub async fn tx(&self) -> Result<DatabaseTx<'_>> {
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
            sqlx::query!("UPDATE tokens SET ticker = ? WHERE id = 0", ticker_upper)
                .execute(&mut *tx.tx)
                .await?;
            tx.set_rust_migration_version(1).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct DatabaseTx<'a> {
    pub(crate) tx: Transaction<'a, Sqlite>,
}

impl<'a> DatabaseTx<'a> {
    pub fn new(tx: Transaction<'a, Sqlite>) -> Self {
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

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Precision lost during cast")]
    PrecisionLost(#[from] TryFromIntError),

    #[error("Invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),

    #[error("BLS error: {0}")]
    Bls(#[from] chia::bls::Error),

    #[error("Invalid enum variant")]
    InvalidEnumVariant,

    #[error("Invalid address")]
    InvalidAddress,
}

pub(crate) type Result<T> = std::result::Result<T, DatabaseError>;
