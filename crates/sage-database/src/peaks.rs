use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{to_bytes32, Database, DatabaseTx, Result};

impl Database {
    pub async fn insert_peak(&self, height: u32, header_hash: Bytes32) -> Result<()> {
        insert_peak(&self.pool, height, header_hash).await
    }

    pub async fn latest_peak(&self) -> Result<Option<(u32, Bytes32)>> {
        latest_peak(&self.pool).await
    }
}

impl DatabaseTx<'_> {
    pub async fn latest_peak(&mut self) -> Result<Option<(u32, Bytes32)>> {
        latest_peak(&mut *self.tx).await
    }
}

async fn insert_peak(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    header_hash: Bytes32,
) -> Result<()> {
    let header_hash = header_hash.as_ref();
    sqlx::query!(
        "
        REPLACE INTO `peaks` (`height`, `header_hash`)
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
        SELECT `height`, `header_hash`
        FROM `peaks`
        ORDER BY `height` DESC
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?
    .map(|row| Ok((row.height.try_into()?, to_bytes32(&row.header_hash)?)))
    .transpose()
}
