use chia::protocol::{Bytes32, Coin};
use sqlx::SqliteExecutor;

use crate::{Asset, Convert, Database, Result};

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
    let rows = sqlx::query!(
        "SELECT 	
            height,
            timestamp,
            coin_id,
            puzzle_hash,
            parent_coin_hash,
            amount,
            is_created_in_block,
            is_spent_in_block,
            asset_hash,
            description,
            is_visible,
            is_sensitive_content,
            created_height,
            name,
            icon_url,
            kind,
            p2_puzzle_hash 
        FROM transaction_coins 
        WHERE height = ?",
        height
    )
    .fetch_all(conn)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut spent_coins = Vec::new();
    let mut created_coins = Vec::new();
    let mut timestamp = None;

    for row in rows {
        if timestamp.is_none() {
            timestamp = row.timestamp.map(|ts| ts as u32);
        }

        let coin = Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        );

        let asset = Asset {
            hash: row.asset_hash.convert()?,
            name: row.name,
            icon_url: row.icon_url,
            description: row.description,
            is_sensitive_content: row.is_sensitive_content,
            is_visible: row.is_visible,
            created_height: row.created_height.map(|h| h as u32),
        };

        let transaction_coin = TransactionCoin { coin, asset };

        // these represent whether the coins was spent and/or created in this block
        if row.is_spent_in_block == 1 {
            spent_coins.push(transaction_coin.clone());
        }

        if row.is_created_in_block == 1 {
            created_coins.push(transaction_coin);
        }
    }

    Ok(Some(Transaction {
        height,
        timestamp,
        spent: spent_coins,
        created: created_coins,
    }))
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
