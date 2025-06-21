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

    pub async fn check_block(&self, height: u32) -> Result<Option<i64>> {
        check_block(&self.pool, height).await
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
            SELECT DISTINCT created_height
            FROM coins
            LEFT JOIN blocks ON coins.created_height = blocks.height
            WHERE blocks.timestamp IS NULL
                AND coins.created_height IS NOT NULL
            ORDER BY coins.created_height DESC 
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
            SELECT DISTINCT spent_height
            FROM coins
            LEFT JOIN blocks ON coins.spent_height = blocks.height
            WHERE blocks.timestamp IS NULL
                AND coins.spent_height IS NOT NULL
            ORDER BY coins.spent_height DESC 
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

async fn check_block(conn: impl SqliteExecutor<'_>, height: u32) -> Result<Option<i64>> {
    let row = sqlx::query!(
        "
            SELECT timestamp
            FROM blocks
            WHERE height = ?
        ",
        height
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| r.timestamp).flatten())
}

async fn insert_timestamp_height(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    unix_timestamp: i64,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT OR IGNORE INTO blocks (
            height,
            timestamp
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
