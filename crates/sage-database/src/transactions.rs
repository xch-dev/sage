use chia::{
    bls::Signature,
    protocol::{Bytes32, Coin, Program},
};
use sqlx::SqliteExecutor;

use crate::{Database, DatabaseTx, Result};

impl Database {
    pub async fn update_transaction_mempool_time(
        &self,
        transaction_id: Bytes32,
        timestamp: i64,
    ) -> Result<()> {
        update_transaction_mempool_time(&self.pool, transaction_id, timestamp).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_transaction(
        &mut self,
        transaction_id: Bytes32,
        aggregated_signature: Signature,
        fee: u64,
        expiration_height: Option<u32>,
    ) -> Result<()> {
        insert_transaction(
            &mut *self.tx,
            transaction_id,
            aggregated_signature,
            fee,
            expiration_height,
        )
        .await
    }

    pub async fn insert_transaction_spend(
        &mut self,
        coin: Coin,
        transaction_id: Bytes32,
        puzzle_reveal: Program,
        solution: Program,
    ) -> Result<()> {
        insert_transaction_spend(&mut *self.tx, coin, transaction_id, puzzle_reveal, solution).await
    }
}

async fn insert_transaction(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
    aggregated_signature: Signature,
    fee: u64,
    expiration_height: Option<u32>,
) -> Result<()> {
    let transaction_id = transaction_id.as_ref();
    let aggregated_signature = aggregated_signature.to_bytes();
    let aggregated_signature = aggregated_signature.as_ref();
    let fee = fee.to_be_bytes();
    let fee = fee.as_ref();

    sqlx::query!(
        "
        INSERT INTO `transactions` (
            `transaction_id`,
            `aggregated_signature`,
            `fee`,
            `expiration_height`
        )
        VALUES (?, ?, ?, ?)
        ",
        transaction_id,
        aggregated_signature,
        fee,
        expiration_height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_transaction_spend(
    conn: impl SqliteExecutor<'_>,
    coin: Coin,
    transaction_id: Bytes32,
    puzzle_reveal: Program,
    solution: Program,
) -> Result<()> {
    let coin_id = coin.coin_id();
    let coin_id = coin_id.as_ref();
    let transaction_id = transaction_id.as_ref();
    let parent_coin_id = coin.parent_coin_info.as_ref();
    let puzzle_hash = coin.puzzle_hash.as_ref();
    let amount = coin.amount.to_be_bytes();
    let amount = amount.as_ref();
    let puzzle_reveal = puzzle_reveal.as_ref();
    let solution = solution.as_ref();

    sqlx::query!(
        "
        INSERT INTO `transaction_spends` (
            `coin_id`,
            `transaction_id`,
            `parent_coin_id`,
            `puzzle_hash`,
            `amount`,
            `puzzle_reveal`,
            `solution`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        transaction_id,
        parent_coin_id,
        puzzle_hash,
        amount,
        puzzle_reveal,
        solution
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn update_transaction_mempool_time(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
    timestamp: i64,
) -> Result<()> {
    let transaction_id = transaction_id.as_ref();

    sqlx::query!(
        "
        UPDATE `transactions`
        SET `submitted_at` = ?
        WHERE `transaction_id` = ?
        ",
        timestamp,
        transaction_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
