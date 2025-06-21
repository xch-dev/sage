use crate::{to_bytes32, Database, DatabaseTx, Result};
use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

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

    pub async fn insert_peak(&self, height: u32, header_hash: Bytes32) -> Result<()> {
        insert_peak(&self.pool, height, header_hash).await
    }

    pub async fn latest_peak(&self) -> Result<Option<(u32, Bytes32)>> {
        latest_peak(&self.pool).await
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

async fn insert_peak(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    header_hash: Bytes32,
) -> Result<()> {
    let header_hash = header_hash.as_ref();
    sqlx::query!(
        "
        REPLACE INTO blocks (height, header_hash)
        VALUES (?, ?)
        ",
        height,
        header_hash
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn latest_peak(conn: impl SqliteExecutor<'_>) -> Result<Option<(u32, Bytes32)>> {
    sqlx::query!(
        "
        SELECT height, header_hash
        FROM blocks
        WHERE header_hash IS NOT NULL
        ORDER BY height DESC
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?
    .and_then(|row| {
        row.header_hash
            .map(|hash| Ok((row.height.try_into()?, to_bytes32(&hash)?)))
    })
    .transpose()
}
