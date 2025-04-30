use sqlx::SqliteExecutor;

use crate::{Database, DatabaseTx, Result};
//use std::error::Error;

impl Database {
    pub async fn find_created_timestamp_null(&self, limit: u32) -> Result<Vec<u32>> {
        find_created_timestamp_null(&self.pool, limit).await
    }

    pub async fn find_spent_timestamp_null(&self, limit: u32) -> Result<Vec<u32>> {
        find_spent_timestamp_null(&self.pool, limit).await
    }

    pub async fn check_blockinfo(&self, height: u32) -> Result<Option<i64>> {
        check_blockinfo(&self.pool, height).await
    }

    pub async fn update_created_timestamp(&self, height: u32, timestamp: i64) -> Result<()> {
        update_created_timestamp(&self.pool, height, timestamp).await
    }

    pub async fn update_spent_timestamp(&self, height: u32, timestamp: i64) -> Result<()> {
        update_spent_timestamp(&self.pool, height, timestamp).await
    }

    pub async fn insert_timestamp_height(&self, height: u32, timestamp: i64) -> Result<()> {
        insert_timestamp_height(&self.pool, height, timestamp).await
    }
}

impl DatabaseTx<'_> {
    pub async fn find_created_timestamp_null(&mut self, limit: u32) -> Result<Vec<u32>> {
        find_created_timestamp_null(&mut *self.tx, limit).await
    }

    pub async fn find_spent_timestamp_null(&mut self, limit: u32) -> Result<Vec<u32>> {
        find_spent_timestamp_null(&mut *self.tx, limit).await
    }
}

async fn find_created_timestamp_null(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
) -> Result<Vec<u32>> {
    let row = sqlx::query!(
        "
            SELECT DISTINCT `created_height`
            FROM `coin_states`
            WHERE `created_unixtime` IS NULL
            AND `created_height` IS NOT NULL
            ORDER BY `created_height` DESC 
            LIMIT ?;
        ",
        limit
    )
    .fetch_all(conn)
    .await?;

    row.into_iter()
        .filter_map(|r| r.created_height)
        .map(|height| Ok(height.try_into()?))
        .collect::<Result<Vec<_>>>()
}

async fn find_spent_timestamp_null(conn: impl SqliteExecutor<'_>, limit: u32) -> Result<Vec<u32>> {
    let row = sqlx::query!(
        "
            SELECT DISTINCT `spent_height`
            FROM `coin_states`
            WHERE `spent_unixtime` IS NULL
            AND `spent_height` IS NOT NULL
            ORDER BY `spent_height` DESC 
            LIMIT ?;
        ",
        limit
    )
    .fetch_all(conn)
    .await?;

    row.into_iter()
        .filter_map(|r| r.spent_height)
        .map(|height| Ok(height.try_into()?))
        .collect::<Result<Vec<_>>>()
}

async fn check_blockinfo(conn: impl SqliteExecutor<'_>, height: u32) -> Result<Option<i64>> {
    let row = sqlx::query!(
        "
            SELECT `unix_time`
            FROM `blockinfo`
            WHERE `height` = ?
        ",
        height
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| r.unix_time))
}

async fn update_created_timestamp(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    timestamp: i64,
) -> Result<()> {
    sqlx::query!(
        "
        UPDATE OR IGNORE `coin_states`
        SET `created_unixtime` = ?
        WHERE `created_height` = ?
        ",
        timestamp,
        height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn update_spent_timestamp(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    timestamp: i64,
) -> Result<()> {
    sqlx::query!(
        "
        UPDATE OR IGNORE `coin_states`
        SET `spent_unixtime` = ?
        WHERE `spent_height` = ?
        ",
        timestamp,
        height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_timestamp_height(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    unix_timestamp: i64,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT OR IGNORE INTO `blockinfo` (
            `height`,
            `unix_time`
        )
        VALUES (?, ?)
        ",
        height,
        unix_timestamp
    )
    .execute(conn)
    .await?;

    Ok(())
}
