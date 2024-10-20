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
    pub expiration_height: Option<u32>,
}

impl Database {
    pub async fn update_transaction_mempool_time(
        &self,
        transaction_id: Bytes32,
        timestamp: i64,
    ) -> Result<()> {
        update_transaction_mempool_time(&self.pool, transaction_id, timestamp).await
    }

    pub async fn delete_confirmed_transactions(&self) -> Result<()> {
        delete_confirmed_transactions(&self.pool).await
    }

    pub async fn transactions(&self) -> Result<Vec<TransactionRow>> {
        transactions(&self.pool).await
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

    pub async fn transactions_for_coin(&mut self, coin_id: Bytes32) -> Result<Vec<Bytes32>> {
        transactions_for_coin(&mut *self.tx, coin_id).await
    }

    pub async fn resubmittable_transactions(
        &mut self,
        threshold: i64,
    ) -> Result<Vec<(Bytes32, Signature)>> {
        resubmittable_transactions(&mut *self.tx, threshold).await
    }

    pub async fn coin_spends(&mut self, transaction_id: Bytes32) -> Result<Vec<CoinSpend>> {
        coin_spends(&mut *self.tx, transaction_id).await
    }

    pub async fn confirm_coins(&mut self, transaction_id: Bytes32) -> Result<()> {
        confirm_coins(&mut *self.tx, transaction_id).await
    }

    pub async fn remove_transaction(&mut self, transaction_id: Bytes32) -> Result<()> {
        remove_transaction(&mut *self.tx, transaction_id).await
    }

    pub async fn delete_coins(&mut self, transaction_id: Bytes32) -> Result<()> {
        delete_coins(&mut *self.tx, transaction_id).await
    }

    pub async fn transaction_for_spent_coin(&mut self, coin_id: Bytes32) -> Result<Vec<Bytes32>> {
        transaction_for_spent_coin(&mut *self.tx, coin_id).await
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

async fn transactions(conn: impl SqliteExecutor<'_>) -> Result<Vec<TransactionRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `transaction_id`,
            `fee`,
            `submitted_at`,
            `expiration_height`
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
                expiration_height: row.expiration_height.map(TryInto::try_into).transpose()?,
            })
        })
        .collect()
}

async fn transactions_for_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Vec<Bytes32>> {
    let coin_id = coin_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT `transaction_id`
        FROM `transaction_spends`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.transaction_id))
        .collect()
}

async fn delete_confirmed_transactions(conn: impl SqliteExecutor<'_>) -> Result<()> {
    sqlx::query!(
        "
        DELETE FROM `transactions` WHERE `transaction_id` IN (
            SELECT `transaction_spends`.`transaction_id`
            FROM `transaction_spends`
            INNER JOIN `coin_states` ON `transaction_spends`.`coin_id` = `coin_states`.`coin_id`
            WHERE `coin_states`.`spent_height` IS NOT NULL
        )
        "
    )
    .execute(conn)
    .await?;

    Ok(())
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
        FROM `transaction_spends`
        WHERE `transaction_id` = ?
        ORDER BY `coin_id` ASC
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

async fn delete_coins(conn: impl SqliteExecutor<'_>, transaction_id: Bytes32) -> Result<()> {
    let transaction_id = transaction_id.as_ref();

    sqlx::query!(
        "
        DELETE FROM `coin_states`
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
) -> Result<Vec<Bytes32>> {
    let coin_id = coin_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT `transaction_id`
        FROM `transaction_spends`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.transaction_id))
        .collect()
}
