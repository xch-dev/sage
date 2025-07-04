use chia::{
    bls::Signature,
    protocol::{Bytes32, Coin, CoinSpend},
};
use sqlx::{query, SqliteConnection, SqliteExecutor};

use crate::{Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct MempoolItem {
    pub hash: Bytes32,
    pub aggregated_signature: Signature,
    pub fee: u64,
    pub submitted_timestamp: i64,
}

impl Database {
    pub async fn mempool_items_to_submit(
        &self,
        check_every_seconds: i64,
        limit: i64,
    ) -> Result<Vec<MempoolItem>> {
        mempool_items_to_submit(&self.pool, check_every_seconds, limit).await
    }

    pub async fn mempool_coin_spends(&self, mempool_item_id: Bytes32) -> Result<Vec<CoinSpend>> {
        mempool_coin_spends(&self.pool, mempool_item_id).await
    }

    pub async fn update_mempool_item_time(&self, mempool_item_id: Bytes32) -> Result<()> {
        update_mempool_item_time(&self.pool, mempool_item_id).await
    }

    pub async fn mempool_items(&self) -> Result<Vec<MempoolItem>> {
        mempool_items(&self.pool).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_mempool_item(
        &mut self,
        hash: Bytes32,
        aggregated_signature: Signature,
        fee: u64,
    ) -> Result<()> {
        insert_mempool_item(&mut *self.tx, hash, aggregated_signature, fee).await
    }

    pub async fn insert_mempool_coin(
        &mut self,
        mempool_item_id: Bytes32,
        coin_id: Bytes32,
        is_input: bool,
        is_output: bool,
    ) -> Result<()> {
        insert_mempool_coin(&mut *self.tx, mempool_item_id, coin_id, is_input, is_output).await
    }

    pub async fn insert_mempool_spend(
        &mut self,
        mempool_item_id: Bytes32,
        coin_spend: CoinSpend,
        seq: usize,
    ) -> Result<()> {
        insert_mempool_spend(&mut *self.tx, mempool_item_id, coin_spend, seq).await
    }

    pub async fn mempool_items_for_input(&mut self, coin_id: Bytes32) -> Result<Vec<Bytes32>> {
        mempool_items_for_input(&mut *self.tx, coin_id).await
    }

    pub async fn mempool_items_for_output(&mut self, coin_id: Bytes32) -> Result<Vec<Bytes32>> {
        mempool_items_for_output(&mut *self.tx, coin_id).await
    }

    pub async fn remove_mempool_item(&mut self, mempool_item_id: Bytes32) -> Result<()> {
        remove_mempool_item(&mut self.tx, mempool_item_id).await
    }
}

async fn insert_mempool_item(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    aggregated_signature: Signature,
    fee: u64,
) -> Result<()> {
    let hash = hash.as_ref();
    let aggregated_signature = aggregated_signature.to_bytes();
    let aggregated_signature = aggregated_signature.as_ref();
    let fee = fee.to_be_bytes().to_vec();

    query!(
        "
        INSERT INTO mempool_items (hash, aggregated_signature, fee) VALUES (?, ?, ?)
        ",
        hash,
        aggregated_signature,
        fee
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_mempool_coin(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
    coin_id: Bytes32,
    is_input: bool,
    is_output: bool,
) -> Result<()> {
    let mempool_item_id = mempool_item_id.as_ref();
    let coin_id = coin_id.as_ref();

    query!(
        "
        INSERT INTO mempool_coins (mempool_item_id, coin_id, is_input, is_output)
        VALUES ((SELECT id FROM mempool_items WHERE hash = ?), (SELECT id FROM coins WHERE hash = ?), ?, ?)
        ",
        mempool_item_id,
        coin_id,
        is_input,
        is_output
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_mempool_spend(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
    coin_spend: CoinSpend,
    seq: usize,
) -> Result<()> {
    let mempool_item_id = mempool_item_id.as_ref();
    let coin_id = coin_spend.coin.coin_id();
    let coin_id = coin_id.as_ref();
    let parent_coin_hash = coin_spend.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_spend.coin.puzzle_hash.as_ref();
    let amount = coin_spend.coin.amount.to_be_bytes().to_vec();
    let puzzle_reveal = coin_spend.puzzle_reveal.into_bytes();
    let solution = coin_spend.solution.into_bytes();
    let seq: i64 = seq.try_into()?;

    query!(
        "
        INSERT INTO mempool_spends (mempool_item_id, coin_hash, parent_coin_hash, puzzle_hash, amount, puzzle_reveal, solution, seq)
        VALUES ((SELECT id FROM mempool_items WHERE hash = ?), ?, ?, ?, ?, ?, ?, ?)
        ",
        mempool_item_id,
        coin_id,
        parent_coin_hash,
        puzzle_hash,
        amount,
        puzzle_reveal,
        solution,
        seq
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn mempool_items_to_submit(
    conn: impl SqliteExecutor<'_>,
    check_every_seconds: i64,
    limit: i64,
) -> Result<Vec<MempoolItem>> {
    query!(
        "
        SELECT hash, aggregated_signature, fee, submitted_timestamp
        FROM mempool_items
        WHERE submitted_timestamp IS NULL OR unixepoch() - submitted_timestamp >= ?
        LIMIT ?
        ",
        check_every_seconds,
        limit
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(MempoolItem {
            hash: row.hash.convert()?,
            aggregated_signature: row.aggregated_signature.convert()?,
            fee: row.fee.convert()?,
            submitted_timestamp: row.submitted_timestamp.unwrap_or(0),
        })
    })
    .collect()
}

async fn mempool_coin_spends(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
) -> Result<Vec<CoinSpend>> {
    let mempool_item_id = mempool_item_id.as_ref();

    query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount, puzzle_reveal, solution
        FROM mempool_spends
        INNER JOIN mempool_items ON mempool_items.id = mempool_spends.mempool_item_id
        WHERE mempool_items.hash = ?
        ORDER BY seq ASC
        ",
        mempool_item_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(CoinSpend::new(
            Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            row.puzzle_reveal.into(),
            row.solution.into(),
        ))
    })
    .collect()
}

async fn mempool_items_for_input(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Vec<Bytes32>> {
    let coin_id = coin_id.as_ref();

    query!(
        "
        SELECT mempool_items.hash AS mempool_item_hash 
        FROM mempool_items
        INNER JOIN mempool_coins ON mempool_coins.mempool_item_id = mempool_items.id
        INNER JOIN coins ON coins.hash = ?
        WHERE mempool_coins.is_input = TRUE
        ",
        coin_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.mempool_item_hash.convert())
    .collect()
}

async fn mempool_items_for_output(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Vec<Bytes32>> {
    let coin_id = coin_id.as_ref();

    query!(
        "
        SELECT mempool_items.hash AS mempool_item_hash 
        FROM mempool_items
        INNER JOIN mempool_coins ON mempool_coins.mempool_item_id = mempool_items.id
        INNER JOIN coins ON coins.hash = ?
        WHERE mempool_coins.is_output = TRUE
        ",
        coin_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.mempool_item_hash.convert())
    .collect()
}

async fn remove_mempool_item(conn: &mut SqliteConnection, mempool_item_id: Bytes32) -> Result<()> {
    let mempool_item_id = mempool_item_id.as_ref();

    query!(
        "
        DELETE FROM coins WHERE created_height IS NULL AND id IN (
            SELECT coin_id FROM mempool_coins
            INNER JOIN mempool_items ON mempool_items.id = mempool_coins.mempool_item_id
            WHERE hash = ? AND is_output = TRUE
        )
        ",
        mempool_item_id
    )
    .execute(&mut *conn)
    .await?;

    query!("DELETE FROM mempool_items WHERE hash = ?", mempool_item_id)
        .execute(conn)
        .await?;

    Ok(())
}

async fn update_mempool_item_time(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
) -> Result<()> {
    let mempool_item_id = mempool_item_id.as_ref();

    query!(
        "UPDATE mempool_items SET submitted_timestamp = unixepoch() WHERE hash = ?",
        mempool_item_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn mempool_items(conn: impl SqliteExecutor<'_>) -> Result<Vec<MempoolItem>> {
    query!(
        "
        SELECT hash, aggregated_signature, fee, submitted_timestamp
        FROM mempool_items
        ORDER BY submitted_timestamp DESC, hash ASC
        ",
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(MempoolItem {
            hash: row.hash.convert()?,
            aggregated_signature: row.aggregated_signature.convert()?,
            fee: row.fee.convert()?,
            submitted_timestamp: row.submitted_timestamp.unwrap_or(0),
        })
    })
    .collect()
}
