mod coin_states;
mod derivations;
mod offers;
mod peaks;
mod primitives;
mod rows;
mod transactions;
mod utils;

pub use primitives::*;
pub use rows::*;
use tracing::info;
pub use transactions::*;

pub(crate) use utils::*;

use std::num::TryFromIntError;

use sqlx::{Sqlite, SqlitePool, Transaction};
use thiserror::Error;

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

    pub async fn run_rust_migrations(&self) -> Result<()> {
        let mut tx = self.tx().await?;

        let version = tx.rust_migration_version().await?;

        info!("The current Sage migration version is {version}");

        if version == 0 {
            info!("Migrating to version 1 (fixed collection id calculation)");

            for collection_id in tx.collection_ids().await? {
                let collection = tx
                    .collection(collection_id)
                    .await?
                    .expect("collection not found");

                tx.update_collection(
                    collection_id,
                    CollectionRow {
                        collection_id: calculate_collection_id(
                            collection.did_id,
                            &collection.metadata_collection_id,
                        ),
                        did_id: collection.did_id,
                        metadata_collection_id: collection.metadata_collection_id,
                        visible: collection.visible,
                        name: collection.name,
                        icon: collection.icon,
                    },
                )
                .await?;
            }

            tx.set_rust_migration_version(1).await?;
        }

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
        let row = sqlx::query_scalar!("SELECT `version` FROM `rust_migrations` LIMIT 1")
            .fetch_one(&mut *self.tx)
            .await?;

        Ok(row)
    }

    pub async fn set_rust_migration_version(&mut self, version: i64) -> Result<()> {
        sqlx::query!("UPDATE `rust_migrations` SET `version` = ?", version)
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

    #[error("Invalid offer status {0}")]
    InvalidOfferStatus(i64),
}

pub(crate) type Result<T> = std::result::Result<T, DatabaseError>;
