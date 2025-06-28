use chia::protocol::Coin;
use sqlx::SqliteExecutor;

use crate::{Asset, Database, Result};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub height: u32,
    pub timestamp: Option<u32>,
    pub spent: Vec<TransactionCoin>,
    pub created: Vec<TransactionCoin>,
}

#[derive(Debug, Clone)]
pub struct TransactionCoin {
    pub coin: Coin,
    pub asset: Asset,
}

impl Database {
    pub async fn transaction(&self, height: u32) -> Result<Option<Transaction>> {
        transaction(&self.pool, height).await
    }

    pub async fn transactions(
        &self,
        find_value: Option<String>,
        sort_ascending: bool,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Transaction>, u32)> {
        transactions(&self.pool, find_value, sort_ascending, limit, offset).await
    }
}
async fn transaction(conn: impl SqliteExecutor<'_>, height: u32) -> Result<Option<Transaction>> {
    Ok(None)
}

async fn transactions(
    conn: impl SqliteExecutor<'_>,
    find_value: Option<String>,
    sort_ascending: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Transaction>, u32)> {
    Ok((vec![], 0))
}
