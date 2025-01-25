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
pub use transactions::*;

pub(crate) use utils::*;

use std::num::TryFromIntError;

use sqlx::{Sqlite, SqlitePool, Transaction};
use thiserror::Error;
use tracing::{debug, info};

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

        if version < 1 {
            info!("Migrating to version 1 (fixed collection id calculation)");

            for collection_id in tx.collection_ids().await? {
                let collection = tx
                    .collection(collection_id)
                    .await?
                    .expect("collection not found");

                let new_collection_id =
                    calculate_collection_id(collection.did_id, &collection.metadata_collection_id);

                if collection_id == new_collection_id {
                    continue;
                }

                debug!("Migrating collection {collection_id} to {new_collection_id}");

                tx.update_collection_id(collection_id, new_collection_id)
                    .await?;

                tx.update_nft_collection_ids(collection_id, new_collection_id)
                    .await?;
            }

            tx.set_rust_migration_version(1).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Count NFTs matching the given search parameters
    pub async fn count_nfts(&self, params: NftSearchParams) -> Result<u32> {
        let mut conditions = Vec::new();
        let mut args = Vec::new();

        if !params.include_hidden {
            conditions.push("visible = 1");
        }

        if let Some(group) = params.group {
            match group {
                NftGroup::Collection(id) => {
                    conditions.push("collection_id = ?");
                    args.push(id.to_string());
                }
                NftGroup::NoCollection => conditions.push("collection_id IS NULL"),
                NftGroup::MinterDid(did) => {
                    conditions.push("minter_did = ?");
                    args.push(did.to_string());
                }
                NftGroup::NoMinterDid => conditions.push("minter_did IS NULL"),
                NftGroup::OwnerDid(did) => {
                    conditions.push("owner_did = ?");
                    args.push(did.to_string());
                }
                NftGroup::NoOwnerDid => conditions.push("owner_did IS NULL"),
            }
        }

        if let Some(name) = params.name {
            conditions.push("name LIKE ?");
            args.push(format!("%{}%", name));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query = format!("SELECT COUNT(*) FROM nfts {}", where_clause);

        let mut query = sqlx::query_scalar(&query);
        for arg in args {
            query = query.bind(arg);
        }

        let count: i64 = query.fetch_one(&self.pool).await?;

        Ok(count.try_into()?)
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
