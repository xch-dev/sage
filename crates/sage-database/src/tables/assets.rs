use chia::protocol::{Bytes32, Program};
use sqlx::{query, SqliteConnection, SqliteExecutor};

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

impl CatAsset {
    pub fn empty(hash: Bytes32, is_visible: bool) -> Self {
        Self {
            hash,
            name: None,
            icon_url: None,
            description: None,
            ticker: None,
            is_visible,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DidAsset {
    pub hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub is_visible: bool,
    pub created_height: Option<u32>,
}

impl DidAsset {
    pub fn empty(hash: Bytes32, is_visible: bool, created_height: Option<u32>) -> Self {
        Self {
            hash,
            name: None,
            icon_url: None,
            description: None,
            is_visible,
            created_height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DidCoinInfo {
    pub metadata: Program,
    pub recovery_list_hash: Option<Bytes32>,
    pub num_verifications_required: u64,
}

impl Database {
    pub async fn asset_kind(&self, asset_id: Bytes32) -> Result<Option<AssetKind>> {
        asset_kind(&self.pool, asset_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_cat(&mut self, cat: CatAsset) -> Result<i64> {
        insert_cat(&mut self.tx, cat).await
    }

    pub async fn insert_did(&mut self, did: DidAsset, coin_info: &DidCoinInfo) -> Result<i64> {
        insert_did(&mut self.tx, did, coin_info).await
    }

    pub async fn update_did_coin_info(
        &mut self,
        asset_id: i64,
        coin_info: &DidCoinInfo,
    ) -> Result<i64> {
        update_did_coin_info(&mut self.tx, asset_id, coin_info).await
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

async fn insert_cat(conn: &mut SqliteConnection, cat: CatAsset) -> Result<i64> {
    let hash = cat.hash.as_ref();

    let asset_id = query!(
        "
        INSERT INTO assets (hash, kind, name, icon_url, description, is_visible, is_pending)
        VALUES (?, 0, ?, ?, ?, ?, FALSE)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description)
        RETURNING id
        ",
        hash,
        cat.name,
        cat.icon_url,
        cat.description,
        cat.is_visible
    )
    .fetch_one(&mut *conn)
    .await?
    .id;

    query!(
        "
        INSERT INTO tokens (asset_id, ticker)
        VALUES (?, ?)
        ON CONFLICT(asset_id) DO UPDATE SET
            ticker = COALESCE(ticker, excluded.ticker)
        ",
        asset_id,
        cat.ticker,
    )
    .execute(&mut *conn)
    .await?;

    Ok(asset_id)
}

async fn insert_did(
    conn: &mut SqliteConnection,
    did: DidAsset,
    coin_info: &DidCoinInfo,
) -> Result<i64> {
    let hash = did.hash.as_ref();

    let asset_id = query!(
        "
        INSERT INTO assets (hash, kind, name, icon_url, description, is_visible, is_pending, created_height)
        VALUES (?, 2, ?, ?, ?, ?, FALSE, ?)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description),
            created_height = COALESCE(MAX(created_height, excluded.created_height), created_height, excluded.created_height)
        RETURNING id
        ",
        hash,
        did.name,
        did.icon_url,
        did.description,
        did.is_visible,
        did.created_height
    )
    .fetch_one(&mut *conn)
    .await?
    .id;

    let metadata = coin_info.metadata.as_slice();
    let recovery_list_hash = coin_info.recovery_list_hash.as_deref();
    let num_verifications_required: i64 = coin_info.num_verifications_required.try_into()?;

    query!(
        "
        INSERT OR IGNORE INTO dids (asset_id, metadata, recovery_list_hash, num_verifications_required)
        VALUES (?, ?, ?, ?)
        ",
        asset_id,
        metadata,
        recovery_list_hash,
        num_verifications_required
    )
    .execute(&mut *conn)
    .await?;

    Ok(asset_id)
}

async fn update_did_coin_info(
    conn: &mut SqliteConnection,
    asset_id: i64,
    coin_info: &DidCoinInfo,
) -> Result<i64> {
    let metadata = coin_info.metadata.as_slice();
    let recovery_list_hash = coin_info.recovery_list_hash.as_deref();
    let num_verifications_required: i64 = coin_info.num_verifications_required.try_into()?;

    query!(
        "
        UPDATE dids
        SET
            metadata = ?,
            recovery_list_hash = ?,
            num_verifications_required = ?
        WHERE asset_id = ?
        ",
        metadata,
        recovery_list_hash,
        num_verifications_required,
        asset_id
    )
    .execute(&mut *conn)
    .await?;

    Ok(asset_id)
}
