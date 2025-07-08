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
pub struct Asset {
    pub hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub is_sensitive_content: bool,
    pub is_visible: bool,
    pub kind: AssetKind,
}

impl Database {
    pub async fn insert_asset(&self, asset: Asset) -> Result<()> {
        insert_asset(&self.pool, asset).await?;

        Ok(())
    }

    pub async fn update_asset(&self, asset: Asset) -> Result<()> {
        let hash = asset.hash.as_ref();
        let kind = asset.kind as i64;

        query!(
            "
            UPDATE assets SET
                kind = ?,
                name = ?,
                icon_url = ?,
                description = ?,
                is_sensitive_content = ?,
                is_visible = ?
            WHERE hash = ?
            ",
            kind,
            asset.name,
            asset.icon_url,
            asset.description,
            asset.is_sensitive_content,
            asset.is_visible,
            hash,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn asset_kind(&self, hash: Bytes32) -> Result<Option<AssetKind>> {
        let hash = hash.as_ref();

        query!("SELECT kind FROM assets WHERE hash = ?", hash)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| row.kind.convert())
            .transpose()
    }

    pub async fn asset(&self, hash: Bytes32) -> Result<Option<Asset>> {
        let hash = hash.as_ref();

        query!(
            "
            SELECT
                hash, kind, name, icon_url, description,
                is_sensitive_content, is_visible
            FROM assets
            WHERE hash = ?
            ",
            hash
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(Asset {
                hash: row.hash.convert()?,
                kind: row.kind.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_sensitive_content: row.is_sensitive_content,
                is_visible: row.is_visible,
            })
        })
        .transpose()
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_asset(&mut self, asset: Asset) -> Result<()> {
        insert_asset(&mut *self.tx, asset).await?;

        Ok(())
    }
}

async fn insert_asset(conn: impl SqliteExecutor<'_>, asset: Asset) -> Result<()> {
    let hash = asset.hash.as_ref();
    let kind = asset.kind as i64;

    query!(
        "
        INSERT INTO assets (
            hash, kind, name, icon_url, description,
            is_sensitive_content, is_visible
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description),
            is_sensitive_content = is_sensitive_content OR excluded.is_sensitive_content
        ",
        hash,
        kind,
        asset.name,
        asset.icon_url,
        asset.description,
        asset.is_sensitive_content,
        asset.is_visible,
    )
    .execute(conn)
    .await?;

    Ok(())
}
