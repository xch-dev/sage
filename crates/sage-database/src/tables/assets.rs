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
pub struct SingletonAsset {
    pub hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub is_visible: bool,
    pub created_height: Option<u32>,
}

impl SingletonAsset {
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

#[derive(Debug, Clone)]
pub struct NftCoinInfo {
    pub minter_hash: Option<Bytes32>,
    pub owner_hash: Option<Bytes32>,
    pub metadata: Program,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_basis_points: u16,
    pub data_hash: Option<Bytes32>,
    pub metadata_hash: Option<Bytes32>,
    pub license_hash: Option<Bytes32>,
    pub edition_number: Option<u64>,
    pub edition_total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct NftAsset {
    pub singleton: SingletonAsset,
    pub nft_info: NftCoinInfo,
}

impl Database {
    pub async fn asset_kind(&self, asset_id: Bytes32) -> Result<Option<AssetKind>> {
        asset_kind(&self.pool, asset_id).await
    }

    pub async fn cat_asset(&self, asset_id: Bytes32) -> Result<Option<CatAsset>> {
        cat_asset(&self.pool, asset_id).await
    }

    pub async fn cat_assets(
        &self,
        limit: u32,
        offset: u32,
        include_hidden: bool,
    ) -> Result<Vec<CatAsset>> {
        cat_assets(&self.pool, limit, offset, include_hidden).await
    }

    pub async fn nft_asset(&self, asset_id: Bytes32) -> Result<Option<NftAsset>> {
        nft_asset(&self.pool, asset_id).await
    }

    pub async fn nft_assets(
        &self,
        limit: u32,
        offset: u32,
        include_hidden: bool,
    ) -> Result<Vec<NftAsset>> {
        nft_assets(&self.pool, limit, offset, include_hidden).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_cat(&mut self, cat: CatAsset) -> Result<()> {
        insert_cat(&mut self.tx, cat).await
    }

    pub async fn insert_did(&mut self, did: SingletonAsset, coin_info: &DidCoinInfo) -> Result<()> {
        insert_did(&mut self.tx, did, coin_info).await
    }

    pub async fn update_did_coin_info(
        &mut self,
        launcher_id: Bytes32,
        coin_info: &DidCoinInfo,
    ) -> Result<()> {
        update_did_coin_info(&mut self.tx, launcher_id, coin_info).await
    }

    pub async fn insert_nft(&mut self, nft: SingletonAsset, coin_info: &NftCoinInfo) -> Result<()> {
        insert_nft(&mut self.tx, nft, coin_info).await
    }

    pub async fn update_nft_coin_info(
        &mut self,
        launcher_id: Bytes32,
        coin_info: &NftCoinInfo,
    ) -> Result<()> {
        update_nft_coin_info(&mut self.tx, launcher_id, coin_info).await
    }
}

async fn asset_kind(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<Option<AssetKind>> {
    let hash = hash.as_ref();

    query!("SELECT kind FROM assets WHERE hash = ?", hash)
        .fetch_optional(conn)
        .await?
        .map(|row| row.kind.convert())
        .transpose()
}

async fn insert_cat(conn: &mut SqliteConnection, cat: CatAsset) -> Result<()> {
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

    Ok(())
}

async fn insert_singleton(
    conn: &mut SqliteConnection,
    kind: i64,
    singleton: SingletonAsset,
) -> Result<i64> {
    let hash = singleton.hash.as_ref();

    let asset_id = query!(
        "
        INSERT INTO assets (hash, kind, name, icon_url, description, is_visible, is_pending, created_height)
        VALUES (?, ?, ?, ?, ?, ?, FALSE, ?)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description),
            created_height = COALESCE(MAX(created_height, excluded.created_height), created_height, excluded.created_height)
        RETURNING id
        ",
        hash,
        kind,
        singleton.name,
        singleton.icon_url,
        singleton.description,
        singleton.is_visible,
        singleton.created_height
    )
    .fetch_one(&mut *conn)
    .await?
    .id;

    Ok(asset_id)
}

async fn insert_did(
    conn: &mut SqliteConnection,
    did: SingletonAsset,
    coin_info: &DidCoinInfo,
) -> Result<()> {
    let asset_id = insert_singleton(conn, 2, did).await?;

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

    Ok(())
}

async fn update_did_coin_info(
    conn: &mut SqliteConnection,
    launcher_id: Bytes32,
    coin_info: &DidCoinInfo,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();
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
        WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
        ",
        metadata,
        recovery_list_hash,
        num_verifications_required,
        launcher_id
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn insert_nft(
    conn: &mut SqliteConnection,
    nft: SingletonAsset,
    coin_info: &NftCoinInfo,
) -> Result<()> {
    let asset_id = insert_singleton(conn, 1, nft).await?;

    let minter_hash = coin_info.minter_hash.as_deref();
    let owner_hash = coin_info.owner_hash.as_deref();
    let metadata = coin_info.metadata.as_slice();
    let metadata_updater_puzzle_hash = coin_info.metadata_updater_puzzle_hash.as_ref();
    let royalty_puzzle_hash = coin_info.royalty_puzzle_hash.as_ref();
    let data_hash = coin_info.data_hash.as_deref();
    let metadata_hash = coin_info.metadata_hash.as_deref();
    let license_hash = coin_info.license_hash.as_deref();
    let edition_number: Option<i64> = coin_info
        .edition_number
        .map(TryInto::try_into)
        .transpose()?;
    let edition_total: Option<i64> = coin_info.edition_total.map(TryInto::try_into).transpose()?;

    query!(
        "
        INSERT OR IGNORE INTO nfts (
            asset_id, minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
            royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
            edition_number, edition_total
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        minter_hash,
        owner_hash,
        metadata,
        metadata_updater_puzzle_hash,
        royalty_puzzle_hash,
        coin_info.royalty_basis_points,
        data_hash,
        metadata_hash,
        license_hash,
        edition_number,
        edition_total
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn update_nft_coin_info(
    conn: &mut SqliteConnection,
    launcher_id: Bytes32,
    coin_info: &NftCoinInfo,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();
    let minter_hash = coin_info.minter_hash.as_deref();
    let owner_hash = coin_info.owner_hash.as_deref();
    let metadata = coin_info.metadata.as_slice();
    let metadata_updater_puzzle_hash = coin_info.metadata_updater_puzzle_hash.as_ref();
    let royalty_puzzle_hash = coin_info.royalty_puzzle_hash.as_ref();
    let data_hash = coin_info.data_hash.as_deref();
    let metadata_hash = coin_info.metadata_hash.as_deref();
    let license_hash = coin_info.license_hash.as_deref();
    let edition_number: Option<i64> = coin_info
        .edition_number
        .map(TryInto::try_into)
        .transpose()?;
    let edition_total: Option<i64> = coin_info.edition_total.map(TryInto::try_into).transpose()?;

    query!(
        "
        UPDATE nfts
        SET
            minter_hash = ?,
            owner_hash = ?,
            metadata = ?,
            metadata_updater_puzzle_hash = ?,
            royalty_puzzle_hash = ?,
            royalty_basis_points = ?,
            data_hash = ?,
            metadata_hash = ?,
            license_hash = ?,
            edition_number = ?,
            edition_total = ?
        WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
        ",
        minter_hash,
        owner_hash,
        metadata,
        metadata_updater_puzzle_hash,
        royalty_puzzle_hash,
        coin_info.royalty_basis_points,
        data_hash,
        metadata_hash,
        license_hash,
        edition_number,
        edition_total,
        launcher_id
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn cat_asset(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<CatAsset>> {
    let asset_id = asset_id.as_ref();

    query!(
        "SELECT hash, name, icon_url, description, ticker, is_visible
        FROM assets
        INNER JOIN tokens ON tokens.asset_id = assets.id
        WHERE hash = ?",
        asset_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(CatAsset {
            hash: row.hash.convert()?,
            name: row.name,
            icon_url: row.icon_url,
            description: row.description,
            ticker: row.ticker,
            is_visible: row.is_visible,
        })
    })
    .transpose()
}

async fn cat_assets(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
    include_hidden: bool,
) -> Result<Vec<CatAsset>> {
    query!(
        "SELECT hash, name, icon_url, description, ticker, is_visible
            FROM assets
            INNER JOIN tokens ON tokens.asset_id = assets.id
            WHERE ? OR is_visible = 1
            ORDER BY name DESC
            LIMIT ?
            OFFSET ?",
        include_hidden,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(CatAsset {
            hash: row.hash.convert()?,
            name: row.name,
            icon_url: row.icon_url,
            description: row.description,
            ticker: row.ticker,
            is_visible: row.is_visible,
        })
    })
    .collect()
}

async fn nft_asset(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<NftAsset>> {
    let asset_id = asset_id.as_ref();

    query!(
        "SELECT        
            hash, name, icon_url, description, is_visible, created_height,
            minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash,
            royalty_basis_points, data_hash, metadata_hash, license_hash, edition_number,edition_total
            edition_total
        FROM assets
        INNER JOIN nfts ON nfts.asset_id = assets.id
        WHERE hash = ?",
        asset_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(NftAsset {
            singleton: SingletonAsset {
                hash: row.hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                created_height: row.created_height.map(|h| h.try_into()).transpose()?,
            },
            nft_info: NftCoinInfo {
                minter_hash: row.minter_hash.map(|h| h.convert()).transpose()?,
                owner_hash: row.owner_hash.map(|h| h.convert()).transpose()?,
                metadata: Program::from(row.metadata),
                metadata_updater_puzzle_hash: row.metadata_updater_puzzle_hash.convert()?,
                royalty_puzzle_hash: row.royalty_puzzle_hash.convert()?,
                royalty_basis_points: row.royalty_basis_points.try_into()?,
                data_hash: row.data_hash.map(|h| h.convert()).transpose()?,
                metadata_hash: row.metadata_hash.map(|h| h.convert()).transpose()?,
                license_hash: row.license_hash.map(|h| h.convert()).transpose()?,
                edition_number: row.edition_number.map(|n| n.try_into()).transpose()?,
                edition_total: row.edition_total.map(|n| n.try_into()).transpose()?,
            },
        })
    })
    .transpose()
}

async fn nft_assets(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
    include_hidden: bool,
) -> Result<Vec<NftAsset>> {
    query!(
        "SELECT        
            hash, name, icon_url, description, is_visible, created_height,
            minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash,
            royalty_basis_points, data_hash, metadata_hash, license_hash, edition_number,edition_total
        FROM assets
        INNER JOIN nfts ON nfts.asset_id = assets.id
        WHERE ? OR is_visible = 1
        ORDER BY name DESC
        LIMIT ?
        OFFSET ?",
        include_hidden,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NftAsset {
            singleton: SingletonAsset {
                hash: row.hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                created_height: row.created_height.map(|h| h.try_into()).transpose()?,
            },
            nft_info: NftCoinInfo {
                minter_hash: row.minter_hash.map(|h| h.convert()).transpose()?,
                owner_hash: row.owner_hash.map(|h| h.convert()).transpose()?,
                metadata: Program::from(row.metadata),
                metadata_updater_puzzle_hash: row.metadata_updater_puzzle_hash.convert()?,
                royalty_puzzle_hash: row.royalty_puzzle_hash.convert()?,
                royalty_basis_points: row.royalty_basis_points.try_into()?,
                data_hash: row.data_hash.map(|h| h.convert()).transpose()?,
                metadata_hash: row.metadata_hash.map(|h| h.convert()).transpose()?,
                license_hash: row.license_hash.map(|h| h.convert()).transpose()?,
                edition_number: row.edition_number.map(|n| n.try_into()).transpose()?,
                edition_total: row.edition_total.map(|n| n.try_into()).transpose()?,
            },
        })
    })
    .collect()
}
