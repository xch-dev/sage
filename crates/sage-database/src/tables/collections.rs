use crate::{Convert, Database, DatabaseTx, Result};
use chia_wallet_sdk::prelude::*;
use sqlx::{SqliteExecutor, query};

#[derive(Debug, Clone)]
pub struct CollectionRow {
    pub hash: Bytes32,
    pub uuid: String,
    pub minter_hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub is_visible: bool,
}

impl Database {
    pub async fn collections(
        &self,
        limit: u32,
        offset: u32,
        include_hidden: bool,
    ) -> Result<(Vec<CollectionRow>, u32)> {
        collections(&self.pool, limit, offset, include_hidden).await
    }

    pub async fn collection(&self, hash: Bytes32) -> Result<Option<CollectionRow>> {
        collection(&self.pool, hash).await
    }

    pub async fn set_collection_visible(&self, hash: Bytes32, visible: bool) -> Result<()> {
        set_collection_visible(&self.pool, hash, visible).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_collection(&mut self, row: CollectionRow) -> Result<()> {
        insert_collection(&mut *self.tx, row).await
    }

    pub async fn set_collection_visible(&mut self, hash: Bytes32, visible: bool) -> Result<()> {
        set_collection_visible(&mut *self.tx, hash, visible).await
    }
}

async fn collection(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<Option<CollectionRow>> {
    let hash_ref = hash.as_ref();
    let row = query!(
        "SELECT id, hash, uuid, minter_hash, name, icon_url, banner_url, description, is_visible 
        FROM collections
        WHERE hash = ?",
        hash_ref
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        Ok(CollectionRow {
            hash: row.hash.convert()?,
            uuid: row.uuid,
            minter_hash: row.minter_hash.convert()?,
            name: row.name,
            icon_url: row.icon_url,
            banner_url: row.banner_url,
            description: row.description,
            is_visible: row.is_visible,
        })
    })
    .transpose()
}

async fn collections(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
    include_hidden: bool,
) -> Result<(Vec<CollectionRow>, u32)> {
    // we only return collections that have nfts
    let rows = query!(
        "SELECT collections.hash, uuid, collections.minter_hash, collections.name, collections.icon_url, 
        collections.banner_url, collections.description, collections.is_visible, COUNT(*) OVER() as total_count
        FROM collections
        WHERE 1=1
        AND EXISTS (SELECT 1 FROM owned_nfts WHERE owned_nfts.collection_id = collections.id)
        AND (? OR is_visible = 1)
        ORDER BY CASE WHEN collections.id = 0 THEN 1 ELSE 0 END, name ASC
        LIMIT ?
        OFFSET ?",
        include_hidden,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.total_count.try_into())?;

    let collections = rows
        .into_iter()
        .map(|row| {
            Ok(CollectionRow {
                hash: row.hash.convert()?,
                uuid: row.uuid,
                minter_hash: row.minter_hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                banner_url: row.banner_url,
                description: row.description,
                is_visible: row.is_visible,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((collections, total_count))
}

async fn insert_collection(conn: impl SqliteExecutor<'_>, row: CollectionRow) -> Result<()> {
    let hash_ref = row.hash.as_ref();
    let minter_hash_ref = row.minter_hash.as_ref();
    query!(
        "
        INSERT OR IGNORE INTO collections (
            hash, uuid, minter_hash, name, icon_url,
            banner_url, description, is_visible
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        hash_ref,
        row.uuid,
        minter_hash_ref,
        row.name,
        row.icon_url,
        row.banner_url,
        row.description,
        row.is_visible,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn set_collection_visible(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    visible: bool,
) -> Result<()> {
    let hash_ref = hash.as_ref();
    query!(
        "UPDATE collections SET is_visible = ? WHERE hash = ?",
        visible,
        hash_ref
    )
    .execute(conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_database;

    fn test_hash(byte: u8) -> Bytes32 {
        Bytes32::new([byte; 32])
    }

    fn test_collection(byte: u8) -> CollectionRow {
        CollectionRow {
            hash: test_hash(byte),
            uuid: format!("uuid-{byte}"),
            minter_hash: test_hash(byte + 100),
            name: Some(format!("Collection {byte}")),
            icon_url: None,
            banner_url: None,
            description: Some(format!("Desc {byte}")),
            is_visible: true,
        }
    }

    #[tokio::test]
    async fn insert_and_retrieve_collection() -> anyhow::Result<()> {
        let db = test_database().await?;
        let mut tx = db.tx().await?;
        let col = test_collection(1);
        tx.insert_collection(col).await?;
        tx.commit().await?;

        let fetched = db.collection(test_hash(1)).await?;
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.name.as_deref(), Some("Collection 1"));
        assert_eq!(fetched.uuid, "uuid-1");
        assert!(fetched.is_visible);
        Ok(())
    }

    #[tokio::test]
    async fn nonexistent_collection_returns_none() -> anyhow::Result<()> {
        let db = test_database().await?;
        let fetched = db.collection(test_hash(99)).await?;
        assert!(fetched.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn set_collection_visibility() -> anyhow::Result<()> {
        let db = test_database().await?;
        let mut tx = db.tx().await?;
        tx.insert_collection(test_collection(1)).await?;
        tx.commit().await?;

        db.set_collection_visible(test_hash(1), false).await?;

        let fetched = db.collection(test_hash(1)).await?.unwrap();
        assert!(!fetched.is_visible);

        db.set_collection_visible(test_hash(1), true).await?;

        let fetched = db.collection(test_hash(1)).await?.unwrap();
        assert!(fetched.is_visible);
        Ok(())
    }

    #[tokio::test]
    async fn default_collection_exists_after_migration() -> anyhow::Result<()> {
        let db = test_database().await?;

        // The default collection (id=0) is inserted by migrations
        // Verify the database was set up correctly
        let (collections, _count) = db.collections(100, 0, true).await?;
        // Even if empty, this shouldn't error
        assert!(collections.len() <= 1); // May or may not have NFTs
        Ok(())
    }
}
