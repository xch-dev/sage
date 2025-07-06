use crate::{Convert, Database, DatabaseTx, Result};
use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

impl Database {
    pub async fn unsynced_blocks(&self, limit: u32) -> Result<Vec<u32>> {
        unsynced_blocks(&self.pool, limit).await
    }

    pub async fn insert_block(
        &self,
        height: u32,
        header_hash: Bytes32,
        timestamp: Option<i64>,
        is_peak: bool,
    ) -> Result<()> {
        insert_block(&self.pool, height, header_hash, timestamp, is_peak).await
    }

    pub async fn latest_peak(&self) -> Result<Option<(u32, Bytes32)>> {
        latest_peak(&self.pool).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_height(&mut self, height: u32) -> Result<()> {
        insert_height(&mut *self.tx, height).await
    }
}

async fn insert_height(conn: impl SqliteExecutor<'_>, height: u32) -> Result<()> {
    sqlx::query!(
        "INSERT OR IGNORE INTO blocks (height, is_peak) VALUES (?, FALSE)",
        height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn unsynced_blocks(conn: impl SqliteExecutor<'_>, limit: u32) -> Result<Vec<u32>> {
    let row = sqlx::query!(
        "
        SELECT created_height AS height FROM coins
        INNER JOIN blocks ON blocks.height = coins.created_height
        WHERE blocks.timestamp IS NULL
        UNION
        SELECT spent_height AS height FROM coins
        INNER JOIN blocks ON blocks.height = coins.spent_height
        WHERE blocks.timestamp IS NULL
        ORDER BY height DESC
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?;

    row.into_iter()
        .filter_map(|r| r.height.convert().transpose())
        .collect()
}

async fn insert_block(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    header_hash: Bytes32,
    unix_timestamp: Option<i64>,
    is_peak: bool,
) -> Result<()> {
    let header_hash = header_hash.as_ref();
    sqlx::query!(
        "
        INSERT INTO blocks (height, timestamp, header_hash, is_peak) VALUES (?, ?, ?, ?)
        ON CONFLICT (height) DO UPDATE SET timestamp = COALESCE(excluded.timestamp, timestamp), header_hash = excluded.header_hash, is_peak = (excluded.is_peak OR is_peak)
        ",
        height,
        unix_timestamp,
        header_hash,
        is_peak
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
        WHERE header_hash IS NOT NULL AND is_peak = TRUE
        ORDER BY height DESC
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?
    .and_then(|row| {
        row.header_hash
            .map(|hash| Ok((row.height.convert()?, hash.convert()?)))
    })
    .transpose()
}
