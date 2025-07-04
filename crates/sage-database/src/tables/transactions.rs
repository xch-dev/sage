use chia::protocol::{Bytes32, Coin};
use sqlx::{Row, SqliteExecutor};

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
    pub p2_puzzle_hash: Option<Bytes32>,
    pub ticker: Option<String>,
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

// Helper function to create a TransactionCoin from a database row
fn create_transaction_coin(row: &sqlx::sqlite::SqliteRow) -> Result<TransactionCoin> {
    let coin = Coin::new(
        row.get::<Vec<u8>, _>("parent_coin_hash").convert()?,
        row.get::<Vec<u8>, _>("puzzle_hash").convert()?,
        row.get::<Vec<u8>, _>("amount").convert()?,
    );

    let asset = Asset {
        hash: row.get::<Vec<u8>, _>("asset_hash").convert()?,
        name: row.get::<Option<String>, _>("name"),
        icon_url: row.get::<Option<String>, _>("icon_url"),
        description: row.get::<Option<String>, _>("description"),
        is_sensitive_content: row.get::<bool, _>("is_sensitive_content"),
        is_visible: row.get::<bool, _>("is_visible"),
        created_height: row
            .get::<Option<i64>, _>("created_height")
            .map(|h| h as u32),
        kind: row
            .get::<Option<i64>, _>("asset_kind")
            .map(Convert::convert)
            .transpose()?
            .unwrap_or(crate::AssetKind::Token),
    };

    let p2_puzzle_hash = row.get::<Option<Vec<u8>>, _>("p2_puzzle_hash").convert()?;

    Ok(TransactionCoin {
        coin,
        asset,
        p2_puzzle_hash,
        ticker: row.get::<Option<String>, _>("ticker"),
    })
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
            asset_description,
            asset_is_visible,
            asset_is_sensitive_content,
            asset_created_height,
            asset_name,
            asset_icon_url,
            asset_kind,
            p2_puzzle_hash,
            ticker
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
            name: row.asset_name,
            icon_url: row.asset_icon_url,
            description: row.asset_description,
            is_sensitive_content: row.asset_is_sensitive_content,
            is_visible: row.asset_is_visible,
            created_height: row.asset_created_height.map(|h| h as u32),
            kind: row.asset_kind.convert()?,
        };

        let transaction_coin = TransactionCoin {
            coin,
            asset,
            p2_puzzle_hash: row.p2_puzzle_hash.convert()?,
            ticker: row.ticker,
        };

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
    let mut query = sqlx::QueryBuilder::new(
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
            asset_description,
            asset_is_visible,
            asset_is_sensitive_content,
            asset_created_height,
            asset_name,
            asset_icon_url,
            asset_kind,
            p2_puzzle_hash,
            ticker,
            COUNT(*) OVER() as total_count
        FROM transaction_coins
        WHERE 1=1",
    );

    if let Some(find_value) = find_value {
        query.push(" AND (name LIKE %?% OR ticker LIKE %?%)");
        query.push_bind(find_value.clone());
        query.push_bind(find_value);
    }

    if !sort_ascending {
        query.push(" ORDER BY height DESC");
    }

    query.push(" LIMIT ? OFFSET ?");
    let query = query.build().bind(limit).bind(offset);

    let rows = query.fetch_all(conn).await?;
    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;

    let transactions = group_rows_into_transactions(rows)?;

    Ok((transactions, total_count as u32))
}

// Helper function to group rows by height and create Transaction structs
fn group_rows_into_transactions(rows: Vec<sqlx::sqlite::SqliteRow>) -> Result<Vec<Transaction>> {
    use std::collections::HashMap;

    let mut transactions_by_height: HashMap<
        u32,
        (Option<u32>, Vec<TransactionCoin>, Vec<TransactionCoin>),
    > = HashMap::new();

    for row in rows {
        let height: u32 = row.get("height");
        let timestamp: Option<i64> = row.get("timestamp");
        let is_spent_in_block: i64 = row.get("is_spent_in_block");
        let is_created_in_block: i64 = row.get("is_created_in_block");

        let transaction_coin = create_transaction_coin(&row)?;

        let entry = transactions_by_height
            .entry(height)
            .or_insert_with(|| (timestamp.map(|ts| ts as u32), Vec::new(), Vec::new()));

        // these represent whether the coin was spent and/or created in this block
        if is_spent_in_block == 1 {
            entry.1.push(transaction_coin.clone());
        }

        if is_created_in_block == 1 {
            entry.2.push(transaction_coin);
        }
    }

    let mut transactions = Vec::new();
    for (height, (timestamp, spent_coins, created_coins)) in transactions_by_height {
        transactions.push(Transaction {
            height,
            timestamp,
            spent: spent_coins,
            created: created_coins,
        });
    }

    Ok(transactions)
}
