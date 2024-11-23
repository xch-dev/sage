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
