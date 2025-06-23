use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Token,
    Nft,
    Did,
    Option,
}

impl Convert<AssetKind> for i64 {
    fn convert(self) -> Result<AssetKind> {
        Ok(match self {
            0 => AssetKind::Token,
            1 => AssetKind::Nft,
            2 => AssetKind::Did,
            3 => AssetKind::Option,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CatAsset {
    pub hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub ticker: Option<String>,
    pub is_visible: bool,
}

impl Database {
    pub async fn asset_kind(&self, asset_id: Bytes32) -> Result<Option<AssetKind>> {
        asset_kind(&self.pool, asset_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_cat(&mut self, cat: CatAsset) -> Result<()> {
        insert_cat(&mut *self.tx, cat).await
    }
}

async fn asset_kind(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<AssetKind>> {
    let asset_id = asset_id.as_ref();

    query!("SELECT kind FROM assets WHERE hash = ?", asset_id)
        .fetch_optional(conn)
        .await?
        .map(|row| row.kind.convert())
        .transpose()
}

async fn insert_cat(conn: impl SqliteExecutor<'_>, cat: CatAsset) -> Result<()> {
    let hash = cat.hash.as_ref();

    query!(
        "
        INSERT INTO assets (hash, kind, name, icon_url, description, is_visible, is_pending)
        VALUES (?, 0, ?, ?, ?, ?, FALSE)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description);

        INSERT OR IGNORE INTO tokens (asset_id, ticker)
        VALUES ((SELECT id FROM assets WHERE hash = ?), ?)
        ON CONFLICT(asset_id) DO UPDATE SET
            ticker = COALESCE(ticker, excluded.ticker);
        ",
        hash,
        cat.name,
        cat.icon_url,
        cat.description,
        cat.is_visible,
        hash,
        cat.ticker,
    )
    .execute(conn)
    .await?;

    Ok(())
}
