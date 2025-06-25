use chia::{
    bls::Signature,
    protocol::{Bytes32, Coin, CoinSpend},
};
use sqlx::{query, SqliteConnection, SqliteExecutor};

use crate::{Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct TransactionRow {
    pub hash: Bytes32,
    pub aggregated_signature: Signature,
    pub fee: u64,
}

impl Database {
    pub async fn transactions_to_submit(
        &self,
        check_every_seconds: i64,
        limit: i64,
    ) -> Result<Vec<TransactionRow>> {
        transactions_to_submit(&self.pool, check_every_seconds, limit).await
    }

    pub async fn transaction_coin_spends(&self, transaction_id: Bytes32) -> Result<Vec<CoinSpend>> {
        transaction_coin_spends(&self.pool, transaction_id).await
    }

    pub async fn update_transaction_time(&self, transaction_id: Bytes32) -> Result<()> {
        update_transaction_time(&self.pool, transaction_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_transaction(
        &mut self,
        hash: Bytes32,
        aggregated_signature: Signature,
        fee: u64,
    ) -> Result<()> {
        insert_transaction(&mut *self.tx, hash, aggregated_signature, fee).await
    }

    pub async fn insert_transaction_coin(
        &mut self,
        transaction_id: Bytes32,
        coin_id: Bytes32,
        is_input: bool,
        is_output: bool,
    ) -> Result<()> {
        insert_transaction_coin(&mut *self.tx, transaction_id, coin_id, is_input, is_output).await
    }

    pub async fn insert_transaction_spend(
        &mut self,
        transaction_id: Bytes32,
        coin_spend: CoinSpend,
        seq: usize,
    ) -> Result<()> {
        insert_transaction_spend(&mut *self.tx, transaction_id, coin_spend, seq).await
    }

    pub async fn transactions_for_output(&mut self, coin_id: Bytes32) -> Result<Vec<Bytes32>> {
        transactions_for_output(&mut *self.tx, coin_id).await
    }

    pub async fn remove_transaction(&mut self, transaction_id: Bytes32) -> Result<()> {
        remove_transaction(&mut self.tx, transaction_id).await
    }
}

async fn insert_transaction(
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
        INSERT INTO transactions (hash, aggregated_signature, fee) VALUES (?, ?, ?)
        ",
        hash,
        aggregated_signature,
        fee
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_transaction_coin(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
    coin_id: Bytes32,
    is_input: bool,
    is_output: bool,
) -> Result<()> {
    let transaction_id = transaction_id.as_ref();
    let coin_id = coin_id.as_ref();

    query!(
        "
        INSERT INTO transaction_coins (transaction_id, coin_id, is_input, is_output)
        VALUES ((SELECT id FROM transactions WHERE hash = ?), (SELECT id FROM coins WHERE hash = ?), ?, ?)
        ",
        transaction_id,
        coin_id,
        is_input,
        is_output
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_transaction_spend(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
    coin_spend: CoinSpend,
    seq: usize,
) -> Result<()> {
    let transaction_id = transaction_id.as_ref();
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
        INSERT INTO transaction_spends (transaction_id, coin_hash, parent_coin_hash, puzzle_hash, amount, puzzle_reveal, solution, seq)
        VALUES ((SELECT id FROM transactions WHERE hash = ?), ?, ?, ?, ?, ?, ?, ?)
        ",
        transaction_id,
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

async fn transactions_to_submit(
    conn: impl SqliteExecutor<'_>,
    check_every_seconds: i64,
    limit: i64,
) -> Result<Vec<TransactionRow>> {
    query!(
        "
        SELECT hash, aggregated_signature, fee
        FROM transactions
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
        Ok(TransactionRow {
            hash: row.hash.convert()?,
            aggregated_signature: row.aggregated_signature.convert()?,
            fee: row.fee.convert()?,
        })
    })
    .collect()
}

async fn transaction_coin_spends(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
) -> Result<Vec<CoinSpend>> {
    let transaction_id = transaction_id.as_ref();

    query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount, puzzle_reveal, solution
        FROM transaction_spends
        INNER JOIN transactions ON transactions.id = transaction_spends.transaction_id
        WHERE transactions.hash = ?
        ORDER BY seq ASC
        ",
        transaction_id
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

async fn transactions_for_output(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Vec<Bytes32>> {
    let coin_id = coin_id.as_ref();

    query!(
        "
        SELECT transactions.hash AS transaction_hash FROM transactions
        INNER JOIN transaction_coins ON transaction_coins.transaction_id = transactions.id
        INNER JOIN coins ON coins.hash = ?
        WHERE transaction_coins.is_output = TRUE
        ",
        coin_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.transaction_hash.convert())
    .collect()
}

async fn remove_transaction(conn: &mut SqliteConnection, transaction_id: Bytes32) -> Result<()> {
    let transaction_id = transaction_id.as_ref();

    query!(
        "
        DELETE FROM coins WHERE created_height IS NULL AND id IN (
            SELECT coin_id FROM transaction_coins
            INNER JOIN transactions ON transactions.id = transaction_coins.transaction_id
            WHERE hash = ? AND is_output = TRUE
        )
        ",
        transaction_id
    )
    .execute(&mut *conn)
    .await?;

    query!("DELETE FROM transactions WHERE hash = ?", transaction_id)
        .execute(conn)
        .await?;

    Ok(())
}

async fn update_transaction_time(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
) -> Result<()> {
    let transaction_id = transaction_id.as_ref();

    query!(
        "UPDATE transactions SET submitted_timestamp = unixepoch() WHERE hash = ?",
        transaction_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
