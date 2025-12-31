use chia::protocol::Bytes32;
use sqlx::{SqliteExecutor, SqlitePool};

use crate::{Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone, Copy)]
pub struct TransactionHistory {
    pub hash: Bytes32,
    pub height: u32,
    pub fee: u64,
    pub confirmed_timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TransactionHistoryCoin {
    pub coin_id: Bytes32,
    pub is_input: bool,
}

#[derive(Debug, Clone)]
pub struct TransactionHistoryWithCoins {
    pub hash: Bytes32,
    pub height: u32,
    pub fee: u64,
    pub confirmed_timestamp: u64,
    pub coins: Vec<TransactionHistoryCoin>,
}

impl Database {
    pub async fn transaction_history_by_id(
        &self,
        hash: Bytes32,
    ) -> Result<Option<TransactionHistoryWithCoins>> {
        transaction_history_by_id(&self.pool, hash).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_transaction_history(
        &mut self,
        hash: Bytes32,
        height: u32,
        fee: u64,
        confirmed_timestamp: u64,
    ) -> Result<()> {
        insert_transaction_history(&mut *self.tx, hash, height, fee, confirmed_timestamp).await
    }

    pub async fn insert_transaction_history_coin(
        &mut self,
        transaction_hash: Bytes32,
        coin_id: Bytes32,
        is_input: bool,
    ) -> Result<()> {
        insert_transaction_history_coin(&mut *self.tx, transaction_hash, coin_id, is_input).await
    }

    pub async fn mempool_coin_ids(
        &mut self,
        mempool_item_id: Bytes32,
    ) -> Result<Vec<(Bytes32, bool)>> {
        mempool_coin_ids(&mut *self.tx, mempool_item_id).await
    }

    pub async fn mempool_item_fee(&mut self, mempool_item_id: Bytes32) -> Result<Option<u64>> {
        mempool_item_fee(&mut *self.tx, mempool_item_id).await
    }
}

async fn insert_transaction_history(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    height: u32,
    fee: u64,
    confirmed_timestamp: u64,
) -> Result<()> {
    let hash = hash.as_ref();
    let fee = fee.to_be_bytes().to_vec();
    let confirmed_timestamp: i64 = confirmed_timestamp.try_into()?;

    sqlx::query!(
        "
        INSERT OR IGNORE INTO transaction_history (hash, height, fee, confirmed_timestamp)
        VALUES (?, ?, ?, ?)
        ",
        hash,
        height,
        fee,
        confirmed_timestamp
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_transaction_history_coin(
    conn: impl SqliteExecutor<'_>,
    transaction_hash: Bytes32,
    coin_id: Bytes32,
    is_input: bool,
) -> Result<()> {
    let transaction_hash = transaction_hash.as_ref();
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO transaction_history_coins (transaction_history_id, coin_id, is_input)
        VALUES ((SELECT id FROM transaction_history WHERE hash = ?), ?, ?)
        ",
        transaction_hash,
        coin_id,
        is_input
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn transaction_history_by_id(
    pool: &SqlitePool,
    hash: Bytes32,
) -> Result<Option<TransactionHistoryWithCoins>> {
    let hash_bytes = hash.as_ref();

    let row = sqlx::query!(
        "
        SELECT hash, height, fee, confirmed_timestamp
        FROM transaction_history
        WHERE hash = ?
        ",
        hash_bytes
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let coins = sqlx::query!(
        "
        SELECT coin_id, is_input
        FROM transaction_history_coins
        WHERE transaction_history_id = (SELECT id FROM transaction_history WHERE hash = ?)
        ",
        hash_bytes
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| {
        Ok(TransactionHistoryCoin {
            coin_id: r.coin_id.convert()?,
            is_input: r.is_input,
        })
    })
    .collect::<Result<Vec<_>>>()?;

    Ok(Some(TransactionHistoryWithCoins {
        hash: row.hash.convert()?,
        height: row.height.try_into()?,
        fee: row.fee.convert()?,
        confirmed_timestamp: (row.confirmed_timestamp as u64),
        coins,
    }))
}

async fn mempool_coin_ids(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
) -> Result<Vec<(Bytes32, bool)>> {
    let mempool_item_id = mempool_item_id.as_ref();

    sqlx::query!(
        "
        SELECT coins.hash AS coin_id, mempool_coins.is_input
        FROM mempool_coins
        INNER JOIN mempool_items ON mempool_items.id = mempool_coins.mempool_item_id
        INNER JOIN coins ON coins.id = mempool_coins.coin_id
        WHERE mempool_items.hash = ?
        ",
        mempool_item_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| Ok((row.coin_id.convert()?, row.is_input)))
    .collect()
}

async fn mempool_item_fee(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
) -> Result<Option<u64>> {
    let mempool_item_id = mempool_item_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT fee FROM mempool_items WHERE hash = ?
        ",
        mempool_item_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(|r| r.fee.convert()).transpose()
}
