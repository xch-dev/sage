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
    pub ticker: Option<String>,
    pub precision: u8,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub is_sensitive_content: bool,
    pub is_visible: bool,
    pub hidden_puzzle_hash: Option<Bytes32>,
    pub kind: AssetKind,
}

impl Database {
    pub async fn is_asset_owned(&self, hash: Bytes32) -> Result<bool> {
        let hash = hash.as_ref();

        let count = query!(
            "
            SELECT COUNT(*) AS count FROM owned_coins 
            INNER JOIN assets ON assets.id = owned_coins.asset_id
            WHERE assets.hash = ?
            ",
            hash
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok(count > 0)
    }

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
                ticker = ?,
                precision = ?,
                icon_url = ?,
                description = ?,
                is_sensitive_content = ?,
                is_visible = ?
            WHERE hash = ?
            ",
            kind,
            asset.name,
            asset.ticker,
            asset.precision,
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
        asset(&self.pool, hash).await
    }

    pub async fn existing_hidden_puzzle_hash(
        &self,
        asset_hash: Bytes32,
    ) -> Result<Option<Option<Bytes32>>> {
        existing_hidden_puzzle_hash(&self.pool, asset_hash).await
    }
}

impl DatabaseTx<'_> {
    pub async fn asset(&mut self, hash: Bytes32) -> Result<Option<Asset>> {
        asset(&mut *self.tx, hash).await
    }

    pub async fn insert_asset(&mut self, asset: Asset) -> Result<()> {
        insert_asset(&mut *self.tx, asset).await?;

        Ok(())
    }

    pub async fn update_hidden_puzzle_hash(
        &mut self,
        asset_hash: Bytes32,
        hidden_puzzle_hash: Option<Bytes32>,
    ) -> Result<()> {
        let asset_hash = asset_hash.as_ref();
        let hidden_puzzle_hash = hidden_puzzle_hash.as_deref();

        query!(
            "
            UPDATE assets SET hidden_puzzle_hash = ? WHERE hash = ?
            ",
            hidden_puzzle_hash,
            asset_hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    pub async fn existing_hidden_puzzle_hash(
        &mut self,
        asset_hash: Bytes32,
    ) -> Result<Option<Option<Bytes32>>> {
        existing_hidden_puzzle_hash(&mut *self.tx, asset_hash).await
    }

    pub async fn delete_asset_coins(&mut self, asset_hash: Bytes32) -> Result<()> {
        let asset_hash = asset_hash.as_ref();

        query!(
            "DELETE FROM coins WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)",
            asset_hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}

async fn insert_asset(conn: impl SqliteExecutor<'_>, asset: Asset) -> Result<()> {
    let hash = asset.hash.as_ref();
    let kind = asset.kind as i64;
    let hidden_puzzle_hash = asset.hidden_puzzle_hash.as_deref();

    query!(
        "
        INSERT INTO assets (
            hash, kind, name, ticker, precision, icon_url, description,
            is_sensitive_content, is_visible, hidden_puzzle_hash
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(excluded.name, name),
            ticker = COALESCE(excluded.ticker, ticker),
            icon_url = COALESCE(excluded.icon_url, icon_url),
            description = COALESCE(excluded.description, description),
            is_sensitive_content = is_sensitive_content OR excluded.is_sensitive_content
        ",
        hash,
        kind,
        asset.name,
        asset.ticker,
        asset.precision,
        asset.icon_url,
        asset.description,
        asset.is_sensitive_content,
        asset.is_visible,
        hidden_puzzle_hash,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn existing_hidden_puzzle_hash(
    conn: impl SqliteExecutor<'_>,
    asset_hash: Bytes32,
) -> Result<Option<Option<Bytes32>>> {
    let asset_hash = asset_hash.as_ref();

    query!(
        "
        SELECT hidden_puzzle_hash FROM assets WHERE hash = ?
        AND EXISTS (SELECT 1 FROM coins WHERE coins.asset_id = assets.id)
        ",
        asset_hash
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.hidden_puzzle_hash.convert())
    .transpose()
}

async fn asset(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<Option<Asset>> {
    let hash = hash.as_ref();

    query!(
        "
        SELECT
            hash, kind, name, ticker, precision, icon_url, description,
            is_sensitive_content, is_visible, hidden_puzzle_hash
        FROM assets
        WHERE hash = ?
        ",
        hash
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(Asset {
            hash: row.hash.convert()?,
            kind: row.kind.convert()?,
            name: row.name,
            ticker: row.ticker,
            precision: row.precision.convert()?,
            icon_url: row.icon_url,
            description: row.description,
            is_sensitive_content: row.is_sensitive_content,
            is_visible: row.is_visible,
            hidden_puzzle_hash: row.hidden_puzzle_hash.convert()?,
        })
    })
    .transpose()
}
