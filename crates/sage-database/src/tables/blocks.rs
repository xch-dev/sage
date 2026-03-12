use crate::{Convert, Database, DatabaseTx, Result};
use chia_wallet_sdk::prelude::*;
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
        ON CONFLICT (height) DO UPDATE SET
            timestamp = COALESCE(excluded.timestamp, timestamp),
            header_hash = excluded.header_hash,
            is_peak = (excluded.is_peak OR is_peak)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_database;

    fn test_hash(byte: u8) -> Bytes32 {
        Bytes32::new([byte; 32])
    }

    #[tokio::test]
    async fn empty_peak_returns_none() -> anyhow::Result<()> {
        let db = test_database().await?;
        let peak = db.latest_peak().await?;
        assert!(peak.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn insert_block_and_get_peak() -> anyhow::Result<()> {
        let db = test_database().await?;
        let hash = test_hash(1);

        db.insert_block(100, hash, Some(1000), true).await?;

        let peak = db.latest_peak().await?;
        assert!(peak.is_some());
        let (height, peak_hash) = peak.unwrap();
        assert_eq!(height, 100);
        assert_eq!(peak_hash, hash);
        Ok(())
    }

    #[tokio::test]
    async fn upsert_block_updates_existing() -> anyhow::Result<()> {
        let db = test_database().await?;
        let hash1 = test_hash(1);
        let hash2 = test_hash(2);

        // Insert without timestamp
        db.insert_block(50, hash1, None, false).await?;

        // Upsert with timestamp and different hash
        db.insert_block(50, hash2, Some(500), true).await?;

        let peak = db.latest_peak().await?;
        assert!(peak.is_some());
        let (height, peak_hash) = peak.unwrap();
        assert_eq!(height, 50);
        assert_eq!(peak_hash, hash2);
        Ok(())
    }

    #[tokio::test]
    async fn latest_peak_picks_highest() -> anyhow::Result<()> {
        let db = test_database().await?;

        db.insert_block(10, test_hash(1), Some(100), true).await?;
        db.insert_block(20, test_hash(2), Some(200), true).await?;
        db.insert_block(15, test_hash(3), Some(150), true).await?;

        let (height, hash) = db.latest_peak().await?.unwrap();
        assert_eq!(height, 20);
        assert_eq!(hash, test_hash(2));
        Ok(())
    }

    #[tokio::test]
    async fn insert_height_via_tx() -> anyhow::Result<()> {
        let db = test_database().await?;
        let mut tx = db.tx().await?;
        tx.insert_height(42).await?;
        tx.commit().await?;

        // Height inserted but no peak (no header_hash)
        let peak = db.latest_peak().await?;
        assert!(peak.is_none());
        Ok(())
    }
}
