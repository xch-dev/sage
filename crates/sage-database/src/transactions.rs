use chia::{
    bls::Signature,
    protocol::{Bytes32, Coin, CoinSpend, Program},
};
use sqlx::SqliteExecutor;

use crate::{to_bytes, to_bytes32, Database, DatabaseTx, Result};

#[derive(Debug, Clone, Copy)]
pub struct TransactionRow {
    pub transaction_id: Bytes32,
    pub fee: u64,
    pub submitted_at: Option<i64>,
}

impl Database {
    pub async fn update_transaction_mempool_time(
        &self,
        transaction_id: Bytes32,
        timestamp: i64,
    ) -> Result<()> {
        update_transaction_mempool_time(&self.pool, transaction_id, timestamp).await
    }

    pub async fn transactions(&self) -> Result<Vec<TransactionRow>> {
        transactions(&self.pool).await
    }

    pub async fn resubmittable_transactions(
        &self,
        threshold: i64,
    ) -> Result<Vec<(Bytes32, Signature)>> {
        resubmittable_transactions(&self.pool, threshold).await
    }

    pub async fn coin_spends(&self, transaction_id: Bytes32) -> Result<Vec<CoinSpend>> {
        coin_spends(&self.pool, transaction_id).await
    }

    pub async fn coin_transaction_id(&self, coin_id: Bytes32) -> Result<Option<Bytes32>> {
        coin_transaction_id(&self.pool, coin_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_pending_transaction(
        &mut self,
        transaction_id: Bytes32,
        aggregated_signature: Signature,
        fee: u64,
    ) -> Result<()> {
        insert_pending_transaction(&mut *self.tx, transaction_id, aggregated_signature, fee).await
    }

    pub async fn insert_transaction_spend(
        &mut self,
        transaction_id: Bytes32,
        coin_spend: CoinSpend,
        index: usize,
    ) -> Result<()> {
        insert_transaction_spend(&mut *self.tx, transaction_id, coin_spend, index).await
    }

    pub async fn confirm_coins(&mut self, transaction_id: Bytes32) -> Result<()> {
        confirm_coins(&mut *self.tx, transaction_id).await
    }

    pub async fn remove_transaction(&mut self, transaction_id: Bytes32) -> Result<()> {
        remove_transaction(&mut *self.tx, transaction_id).await
    }

    pub async fn transaction_for_spent_coin(
        &mut self,
        coin_id: Bytes32,
    ) -> Result<Option<Bytes32>> {
        transaction_for_spent_coin(&mut *self.tx, coin_id).await
    }

    pub async fn transaction_coin_ids(&mut self, transaction_id: Bytes32) -> Result<Vec<Bytes32>> {
        transaction_coin_ids(&mut *self.tx, transaction_id).await
    }
}

async fn insert_pending_transaction(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
    aggregated_signature: Signature,
    fee: u64,
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
            `fee`
        )
        VALUES (?, ?, ?)
        ",
        transaction_id,
        aggregated_signature,
        fee
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_transaction_spend(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
    coin_spend: CoinSpend,
    index: usize,
) -> Result<()> {
    let coin_id = coin_spend.coin.coin_id();
    let coin_id = coin_id.as_ref();
    let index: i64 = index.try_into()?;
    let transaction_id = transaction_id.as_ref();
    let parent_coin_id = coin_spend.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_spend.coin.puzzle_hash.as_ref();
    let amount = coin_spend.coin.amount.to_be_bytes();
    let amount = amount.as_ref();
    let puzzle_reveal = coin_spend.puzzle_reveal.as_ref();
    let solution = coin_spend.solution.as_ref();

    sqlx::query!(
        "
        INSERT INTO `transaction_spends` (
            `coin_id`,
            `index`,
            `transaction_id`,
            `parent_coin_id`,
            `puzzle_hash`,
            `amount`,
            `puzzle_reveal`,
            `solution`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        index,
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

async fn transactions(conn: impl SqliteExecutor<'_>) -> Result<Vec<TransactionRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `transaction_id`,
            `fee`,
            `submitted_at`
        FROM `transactions`
        ORDER BY `submitted_at` DESC, `transaction_id` ASC
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TransactionRow {
                transaction_id: to_bytes32(&row.transaction_id)?,
                fee: u64::from_be_bytes(to_bytes(&row.fee)?),
                submitted_at: row.submitted_at,
            })
        })
        .collect()
}

async fn coin_transaction_id(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        SELECT `transaction_id`
        FROM `transaction_spends`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| to_bytes32(&row.transaction_id))
    .transpose()
}

async fn resubmittable_transactions(
    conn: impl SqliteExecutor<'_>,
    threshold: i64,
) -> Result<Vec<(Bytes32, Signature)>> {
    let rows = sqlx::query!(
        "
        SELECT
            `transaction_id`,
            `aggregated_signature`
        FROM `transactions`
        WHERE `submitted_at` IS NULL OR `submitted_at` <= ?
        ",
        threshold
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok((
                to_bytes32(&row.transaction_id)?,
                Signature::from_bytes(&to_bytes(&row.aggregated_signature)?)?,
            ))
        })
        .collect()
}

async fn coin_spends(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
) -> Result<Vec<CoinSpend>> {
    let transaction_id = transaction_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT
            `parent_coin_id`,
            `puzzle_hash`,
            `amount`,
            `puzzle_reveal`,
            `solution`
        FROM `transaction_spends` INDEXED BY `indexed_spend`
        WHERE `transaction_id` = ?
        ORDER BY `index` ASC
        ",
        transaction_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(CoinSpend::new(
                Coin {
                    parent_coin_info: to_bytes32(&row.parent_coin_id)?,
                    puzzle_hash: to_bytes32(&row.puzzle_hash)?,
                    amount: u64::from_be_bytes(to_bytes(&row.amount)?),
                },
                Program::from(row.puzzle_reveal),
                Program::from(row.solution),
            ))
        })
        .collect()
}

async fn remove_transaction(conn: impl SqliteExecutor<'_>, transaction_id: Bytes32) -> Result<()> {
    let transaction_id = transaction_id.as_ref();

    sqlx::query!(
        "
        DELETE FROM `transactions`
        WHERE `transaction_id` = ?
        ",
        transaction_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn confirm_coins(conn: impl SqliteExecutor<'_>, transaction_id: Bytes32) -> Result<()> {
    let transaction_id = transaction_id.as_ref();

    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `transaction_id` = NULL
        WHERE `transaction_id` = ?
        ",
        transaction_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn transaction_for_spent_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT `transaction_id`
        FROM `transaction_spends`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(to_bytes32(&row.transaction_id)?))
}

async fn transaction_coin_ids(
    conn: impl SqliteExecutor<'_>,
    transaction_id: Bytes32,
) -> Result<Vec<Bytes32>> {
    let transaction_id = transaction_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT `coin_id` FROM `transaction_spends` WHERE `transaction_id` = ?
        ",
        transaction_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}
