use crate::{Convert, Database, DatabaseTx, Result};
use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

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
    let rows = query!(
        "SELECT hash, uuid, minter_hash, name, icon_url, banner_url, description, is_visible,
        COUNT(*) OVER() as total_count
        FROM collections
        WHERE ? OR is_visible = 1
        ORDER BY name DESC
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
