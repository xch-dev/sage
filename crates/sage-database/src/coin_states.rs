use chia::protocol::{Bytes32, CoinState};
use sqlx::SqliteExecutor;

use crate::{to_bytes32, to_coin, to_coin_state, Database, DatabaseTx, Result};

impl Database {
    pub async fn unsynced_coin_states(&self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coin_states(&self.pool, limit).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_coin_state(
        &mut self,
        coin_state: CoinState,
        synced: bool,
        transaction_id: Option<Bytes32>,
    ) -> Result<()> {
        insert_coin_state(&mut *self.tx, coin_state, synced, transaction_id).await
    }

    pub async fn sync_coin(&mut self, coin_id: Bytes32, hint: Bytes32) -> Result<()> {
        sync_coin(&mut *self.tx, coin_id, hint).await
    }

    pub async fn remove_coin_transaction_id(&mut self, coin_id: Bytes32) -> Result<()> {
        remove_coin_transaction_id(&mut *self.tx, coin_id).await
    }

    pub async fn total_coin_count(&mut self) -> Result<u32> {
        total_coin_count(&mut *self.tx).await
    }

    pub async fn synced_coin_count(&mut self) -> Result<u32> {
        synced_coin_count(&mut *self.tx).await
    }

    pub async fn coin_state(&mut self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&mut *self.tx, coin_id).await
    }

    pub async fn insert_unknown_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&mut *self.tx, coin_id).await
    }

    pub async fn unspent_nft_coin_ids(&mut self) -> Result<Vec<Bytes32>> {
        unspent_nft_coin_ids(&mut *self.tx).await
    }

    pub async fn unspent_did_coin_ids(&mut self) -> Result<Vec<Bytes32>> {
        unspent_did_coin_ids(&mut *self.tx).await
    }

    pub async fn unspent_cat_coin_ids(&mut self) -> Result<Vec<Bytes32>> {
        unspent_cat_coin_ids(&mut *self.tx).await
    }
}

async fn insert_coin_state(
    conn: impl SqliteExecutor<'_>,
    coin_state: CoinState,
    synced: bool,
    transaction_id: Option<Bytes32>,
) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();
    let coin_id_ref = coin_id.as_ref();
    let parent_coin_id = coin_state.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_state.coin.puzzle_hash.as_ref();
    let amount = coin_state.coin.amount.to_be_bytes();
    let amount_ref = amount.as_ref();
    let transaction_id = transaction_id.as_deref();

    sqlx::query!(
        "
        REPLACE INTO `coin_states` (
            `coin_id`,
            `parent_coin_id`,
            `puzzle_hash`,
            `amount`,
            `created_height`,
            `spent_height`,
            `synced`,
            `transaction_id`
        )
        VALUES (
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?
        )
        ",
        coin_id_ref,
        parent_coin_id,
        puzzle_hash,
        amount_ref,
        coin_state.created_height,
        coin_state.spent_height,
        synced,
        transaction_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn insert_unknown_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `unknown_coins` (`coin_id`) VALUES (?)
        ",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn unsynced_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: usize,
) -> Result<Vec<CoinState>> {
    let limit: i64 = limit.try_into()?;
    let rows = sqlx::query!(
        "
        SELECT *
        FROM `coin_states`
        WHERE `synced` = 0 AND `created_height` IS NOT NULL
        ORDER BY `spent_height` ASC
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|row| {
            to_coin_state(
                to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                row.created_height,
                row.spent_height,
            )
        })
        .collect()
}

async fn sync_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32, hint: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let hint = hint.as_ref();
    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `synced` = 1, `hint` = ?
        WHERE `coin_id` = ?
        ",
        hint,
        coin_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn remove_coin_transaction_id(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `transaction_id` = NULL
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn total_coin_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `total`
        FROM `coin_states`
        "
    )
    .fetch_one(conn)
    .await?;
    Ok(row.total.try_into()?)
}

async fn synced_coin_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `synced`
        FROM `coin_states`
        WHERE `synced` = 1
        "
    )
    .fetch_one(conn)
    .await?;
    Ok(row.synced.try_into()?)
}

async fn coin_state(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<CoinState>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT *
        FROM `coin_states`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(to_coin_state(
        to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        row.created_height,
        row.spent_height,
    )?))
}

async fn unspent_nft_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `coin_states`.`coin_id`
        FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `nft_coins` ON `coin_states`.`coin_id` = `nft_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}

async fn unspent_did_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `coin_states`.`coin_id`
        FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `did_coins` ON `coin_states`.`coin_id` = `did_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}

async fn unspent_cat_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `coin_states`.`coin_id`
        FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `cat_coins` ON `coin_states`.`coin_id` = `cat_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}
