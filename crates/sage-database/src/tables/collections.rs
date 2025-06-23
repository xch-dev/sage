use crate::{Convert, Database, Result};
use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

#[derive(Debug, Clone)]
pub struct CollectionRow {
    pub id: i64,
    pub hash: Bytes32,
    pub uuid: String,
    pub minter_hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub is_visible: bool,
    pub created_height: Option<i64>,
}

impl Database {
    pub async fn get_collections(
        &self,
        limit: u32,
        offset: u32,
        include_hidden: bool,
    ) -> Result<Vec<CollectionRow>> {
        get_collections(&self.pool, limit, offset, include_hidden).await
    }
}

async fn get_collections(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
    include_hidden: bool,
) -> Result<Vec<CollectionRow>> {
    let rows = query!(
        "SELECT id, hash, uuid, minter_hash, name, icon_url, banner_url, description, is_visible, created_height 
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

    let collections = rows
        .into_iter()
        .map(|row| {
            Ok(CollectionRow {
                id: row.id,
                hash: row.hash.convert()?,
                uuid: row.uuid,
                minter_hash: row.minter_hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                banner_url: row.banner_url,
                description: row.description,
                is_visible: row.is_visible,
                created_height: row.created_height,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(collections)
}
