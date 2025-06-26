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
    pub async fn transaction_block(&self, height: u32) -> Result<Option<Transaction>> {
        transaction_block(&self.pool, height).await
    }

    pub async fn transaction_blocks(
        &self,
        find_value: Option<String>,
        sort_ascending: bool,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Transaction>, u32)> {
        transaction_blocks(&self.pool, find_value, sort_ascending, limit, offset).await
    }
}
async fn transaction_block(
    conn: impl SqliteExecutor<'_>,
    height: u32,
) -> Result<Option<Transaction>> {
    Ok(None)
}

async fn transaction_blocks(
    conn: impl SqliteExecutor<'_>,
    find_value: Option<String>,
    sort_ascending: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Transaction>, u32)> {
    Ok((vec![], 0))
}
