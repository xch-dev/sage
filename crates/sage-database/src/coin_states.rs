use chia::protocol::{Bytes32, CoinState};
use sqlx::SqliteExecutor;

use crate::{error::Result, to_coin, to_coin_state, Database, DatabaseTx};

impl Database {
    pub async fn try_insert_coin_state(&self, coin_state: CoinState) -> Result<()> {
        try_insert_coin_state(&self.pool, coin_state).await
    }

    pub async fn remove_coin_state(&self, coin_id: Bytes32) -> Result<()> {
        remove_coin_state(&self.pool, coin_id).await
    }

    pub async fn unsynced_coin_states(&self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coin_states(&self.pool, limit).await
    }

    pub async fn mark_coin_synced(&self, coin_id: Bytes32) -> Result<()> {
        mark_coin_synced(&self.pool, coin_id).await
    }

    pub async fn total_coin_count(&self) -> Result<u32> {
        total_coin_count(&self.pool).await
    }

    pub async fn synced_coin_count(&self) -> Result<u32> {
        synced_coin_count(&self.pool).await
    }

    pub async fn coin_state(&self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&self.pool, coin_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn try_insert_coin_state(&mut self, coin_state: CoinState) -> Result<()> {
        try_insert_coin_state(&mut *self.tx, coin_state).await
    }

    pub async fn remove_coin_state(&mut self, coin_id: Bytes32) -> Result<()> {
        remove_coin_state(&mut *self.tx, coin_id).await
    }

    pub async fn unsynced_coin_states(&mut self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coin_states(&mut *self.tx, limit).await
    }

    pub async fn mark_coin_synced(&mut self, coin_id: Bytes32) -> Result<()> {
        mark_coin_synced(&mut *self.tx, coin_id).await
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
}

async fn try_insert_coin_state(conn: impl SqliteExecutor<'_>, coin_state: CoinState) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();
    let coin_id_ref = coin_id.as_ref();
    let parent_coin_id = coin_state.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_state.coin.puzzle_hash.as_ref();
    let amount = coin_state.coin.amount.to_be_bytes();
    let amount_ref = amount.as_ref();
    sqlx::query!(
        "
        INSERT OR IGNORE INTO `coin_states` (
            `coin_id`,
            `parent_coin_id`,
            `puzzle_hash`,
            `amount`,
            `created_height`,
            `spent_height`,
            `synced`,
            `hint`
        )
        VALUES (
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            EXISTS (SELECT * FROM `derivations` WHERE `p2_puzzle_hash` = ?),
            NULL
        )
        ",
        coin_id_ref,
        parent_coin_id,
        puzzle_hash,
        amount_ref,
        coin_state.created_height,
        coin_state.spent_height,
        puzzle_hash
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn remove_coin_state(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    sqlx::query!(
        "
        DELETE FROM `coin_states`
        WHERE `coin_id` = ?
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

async fn mark_coin_synced(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `synced` = 1
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
