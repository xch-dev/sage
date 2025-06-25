use chia::protocol::{Bytes32, Program};
use sqlx::{query, Row, SqliteConnection, SqliteExecutor};

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
    pub created_height: Option<u32>,
}

impl Asset {
    pub fn empty(hash: Bytes32, is_visible: bool, created_height: Option<u32>) -> Self {
        Self {
            hash,
            name: None,
            icon_url: None,
            description: None,
            is_sensitive_content: false,
            is_visible,
            created_height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CatAsset {
    pub asset: Asset,
    pub ticker: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DidAsset {
    pub asset: Asset,
    pub did_info: DidCoinInfo,
}

#[derive(Debug, Clone)]
pub struct DidCoinInfo {
    pub metadata: Program,
    pub recovery_list_hash: Option<Bytes32>,
    pub num_verifications_required: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NftSortMode {
    Recent,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NftGroupSearch {
    Collection(Bytes32),
    NoCollection,
    MinterDid(Bytes32),
    NoMinterDid,
    OwnerDid(Bytes32),
    NoOwnerDid,
}

#[derive(Debug, Clone)]
pub struct NftCoinInfo {
    pub collection_id: Option<Bytes32>,
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
    pub asset: Asset,
    pub nft_info: NftCoinInfo,
}

#[derive(Debug, Clone)]
pub struct NftMetadataInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_sensitive_content: bool,
    pub collection_id: Option<Bytes32>,
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
        include_hidden: bool,
        limit: u32,
        offset: u32,
    ) -> std::result::Result<(Vec<CatAsset>, u32), DatabaseError> {
        cat_assets(&self.pool, include_hidden, limit, offset).await
    }

    pub async fn nft_asset(&self, asset_id: Bytes32) -> Result<Option<NftAsset>> {
        nft_asset(&self.pool, asset_id).await
    }

    pub async fn nft_assets(
        &self,
        name_search: Option<String>,
        group_search: Option<NftGroupSearch>,
        sort_mode: NftSortMode,
        include_hidden: bool,
        limit: u32,
        offset: u32,
    ) -> std::result::Result<(Vec<NftAsset>, u32), DatabaseError> {
        nft_assets(
            &self.pool,
            name_search,
            group_search,
            sort_mode,
            include_hidden,
            limit,
            offset,
        )
        .await
    }

    pub async fn did_asset(&self, asset_id: Bytes32) -> Result<Option<DidAsset>> {
        did_asset(&self.pool, asset_id).await
    }

    pub async fn did_assets(&self) -> Result<Vec<DidAsset>> {
        did_assets(&self.pool).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_cat(&mut self, cat: CatAsset) -> Result<()> {
        insert_cat(&mut self.tx, cat).await
    }

    pub async fn insert_did(&mut self, did: Asset, coin_info: &DidCoinInfo) -> Result<()> {
        insert_did(&mut self.tx, did, coin_info).await
    }

    pub async fn update_did_coin_info(
        &mut self,
        launcher_id: Bytes32,
        coin_info: &DidCoinInfo,
    ) -> Result<()> {
        update_did_coin_info(&mut self.tx, launcher_id, coin_info).await
    }

    pub async fn insert_nft(&mut self, nft: Asset, coin_info: &NftCoinInfo) -> Result<()> {
        insert_nft(&mut self.tx, nft, coin_info).await
    }

    pub async fn update_nft_coin_info(
        &mut self,
        launcher_id: Bytes32,
        coin_info: &NftCoinInfo,
    ) -> Result<()> {
        update_nft_coin_info(&mut self.tx, launcher_id, coin_info).await
    }

    pub async fn update_nft_metadata(
        &mut self,
        hash: Bytes32,
        metadata_info: NftMetadataInfo,
    ) -> Result<()> {
        update_nft_metadata(&mut self.tx, hash, metadata_info).await
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
    let hash = cat.asset.hash.as_ref();

    let asset_id = query!(
        "
        INSERT INTO assets (hash, kind, name, icon_url, description, is_visible, is_pending)
        VALUES (?, 0, ?, ?, ?, ?, FALSE)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description),
            is_sensitive_content = is_sensitive_content OR excluded.is_sensitive_content
        RETURNING id
        ",
        hash,
        cat.asset.name,
        cat.asset.icon_url,
        cat.asset.description,
        cat.asset.is_visible
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

async fn insert_singleton(conn: &mut SqliteConnection, kind: i64, singleton: Asset) -> Result<i64> {
    let hash = singleton.hash.as_ref();

    let asset_id = query!(
        "
        INSERT INTO assets (hash, kind, name, icon_url, description, is_sensitive_content, is_visible, is_pending, created_height)
        VALUES (?, ?, ?, ?, ?, ?, ?, FALSE, ?)
        ON CONFLICT(hash) DO UPDATE SET
            name = COALESCE(name, excluded.name),
            icon_url = COALESCE(icon_url, excluded.icon_url),
            description = COALESCE(description, excluded.description),
            is_sensitive_content = is_sensitive_content OR excluded.is_sensitive_content,
            created_height = COALESCE(MAX(created_height, excluded.created_height), created_height, excluded.created_height)
        RETURNING id
        ",
        hash,
        kind,
        singleton.name,
        singleton.icon_url,
        singleton.description,
        singleton.is_sensitive_content,
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
    did: Asset,
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
    nft: Asset,
    coin_info: &NftCoinInfo,
) -> Result<()> {
    let asset_id = insert_singleton(conn, 1, nft).await?;

    let collection_id = coin_info.collection_id.as_deref();
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
            asset_id, collection_id, minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
            royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
            edition_number, edition_total
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        collection_id,
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
    let collection_id = coin_info.collection_id.as_deref();
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
            collection_id = ?,
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
        collection_id,
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

async fn update_nft_metadata(
    conn: &mut SqliteConnection,
    hash: Bytes32,
    metadata_info: NftMetadataInfo,
) -> Result<()> {
    let hash = hash.as_ref();
    let collection_id = metadata_info.collection_id.as_deref();

    query!(
        "
        UPDATE assets SET
            name = ?,
            description = ?,
            is_sensitive_content = ?
        WHERE hash = ?
        ",
        metadata_info.name,
        metadata_info.description,
        metadata_info.is_sensitive_content,
        hash
    )
    .execute(&mut *conn)
    .await?;

    query!(
        "
        UPDATE nfts SET collection_id = ?
        WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
        ",
        collection_id,
        hash
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn cat_asset(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<CatAsset>> {
    let asset_id = asset_id.as_ref();

    query!(
        "SELECT hash, name, icon_url, description, ticker, is_visible, is_sensitive_content, created_height
        FROM assets
        INNER JOIN tokens ON tokens.asset_id = assets.id
        WHERE assets.id = 0 AND hash = ?",
        asset_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(CatAsset {
            asset: Asset {
                hash: row.hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                is_sensitive_content: row.is_sensitive_content,
                created_height: row.created_height.map(TryInto::try_into).transpose()?,
            },
            ticker: row.ticker,
        })
    })
    .transpose()
}

async fn cat_assets(
    conn: impl SqliteExecutor<'_>,
    include_hidden: bool,
    limit: u32,
    offset: u32,
) -> std::result::Result<(Vec<CatAsset>, u32), DatabaseError> {
    let rows = query!(
        "SELECT hash, name, icon_url, description, ticker, is_visible, is_sensitive_content, created_height, COUNT(*) OVER () AS total
            FROM assets
            INNER JOIN tokens ON tokens.asset_id = assets.id
            WHERE assets.id = 0 AND (? OR is_visible = 1)
            ORDER BY name DESC
            LIMIT ?
            OFFSET ?",
        include_hidden,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let total_count = rows.first().map_or(Ok(0), |row| row.total.try_into())?;

    let cats = rows
        .into_iter()
        .map(|row| {
            Ok(CatAsset {
                asset: Asset {
                    hash: row.hash.convert()?,
                    name: row.name,
                    icon_url: row.icon_url,
                    description: row.description,
                    is_visible: row.is_visible,
                    is_sensitive_content: row.is_sensitive_content,
                    created_height: row.created_height.map(TryInto::try_into).transpose()?,
                },
                ticker: row.ticker,
            })
        })
        .collect::<std::result::Result<Vec<CatAsset>, DatabaseError>>()?;

    Ok((cats, total_count))
}

async fn nft_asset(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<NftAsset>> {
    let asset_id = asset_id.as_ref();

    query!(
        "SELECT        
            assets.hash AS asset_hash, assets.name, assets.icon_url, assets.description, is_sensitive_content,
            assets.is_visible, assets.created_height, collections.hash AS 'collection_hash?', nfts.minter_hash, owner_hash,
            metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash, royalty_basis_points,
            data_hash, metadata_hash, license_hash, edition_number, edition_total
        FROM assets
        INNER JOIN nfts ON nfts.asset_id = assets.id
        LEFT JOIN collections ON collections.id = nfts.collection_id
        WHERE assets.hash = ?",
        asset_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(NftAsset {
            asset: Asset {
                hash: row.asset_hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_sensitive_content: row.is_sensitive_content,
                is_visible: row.is_visible,
                created_height: row.created_height.map(TryInto::try_into).transpose()?,
            },
            nft_info: NftCoinInfo {
                collection_id: row.collection_hash.map(Convert::convert).transpose()?,
                minter_hash: row.minter_hash.map(Convert::convert).transpose()?,
                owner_hash: row.owner_hash.map(Convert::convert).transpose()?,
                metadata: Program::from(row.metadata),
                metadata_updater_puzzle_hash: row.metadata_updater_puzzle_hash.convert()?,
                royalty_puzzle_hash: row.royalty_puzzle_hash.convert()?,
                royalty_basis_points: row.royalty_basis_points.try_into()?,
                data_hash: row.data_hash.map(Convert::convert).transpose()?,
                metadata_hash: row.metadata_hash.map(Convert::convert).transpose()?,
                license_hash: row.license_hash.map(Convert::convert).transpose()?,
                edition_number: row.edition_number.map(TryInto::try_into).transpose()?,
                edition_total: row.edition_total.map(TryInto::try_into).transpose()?,
            },
        })
    })
    .transpose()
}

async fn nft_assets(
    conn: impl SqliteExecutor<'_>,
    name_search: Option<String>,
    group_search: Option<NftGroupSearch>,
    sort_mode: NftSortMode,
    include_hidden: bool,
    limit: u32,
    offset: u32,
) -> std::result::Result<(Vec<NftAsset>, u32), DatabaseError> {
    let mut query = sqlx::QueryBuilder::new(
        "SELECT        
            assets.hash AS asset_hash, assets.name, assets.icon_url, assets.description, is_sensitive_content,
            assets.is_visible, assets.created_height, collections.hash AS 'collection_hash?', nfts.minter_hash, owner_hash,
            metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash, royalty_basis_points,
            data_hash, metadata_hash, license_hash, edition_number, edition_total,
            COUNT(*) OVER() as total_count	
        FROM assets
        INNER JOIN nfts ON nfts.asset_id = assets.id
        LEFT JOIN collections ON collections.id = nfts.collection_id
        WHERE 1=1 "
    );

    if let Some(name_search) = name_search {
        query.push("AND assets.name LIKE ?");
        query.push_bind(format!("%{}%", name_search));
    }

    if let Some(group) = group_search {
        match group {
            NftGroupSearch::Collection(id) => {
                query.push(" AND collections.hash = ");
                query.push_bind(id.as_ref().to_vec());
            }
            NftGroupSearch::NoCollection => {
                query.push(" AND collections.hash IS NULL");
            }
            NftGroupSearch::MinterDid(id) => {
                query.push(" AND nfts.minter_hash = ");
                query.push_bind(id.as_ref().to_vec());
            }
            NftGroupSearch::NoMinterDid => {
                query.push(" AND nfts.minter_hash IS NULL");
            }
            NftGroupSearch::OwnerDid(id) => {
                query.push(" AND nfts.owner_hash = ");
                query.push_bind(id.as_ref().to_vec());
            }
            NftGroupSearch::NoOwnerDid => {
                query.push(" AND nfts.owner_hash IS NULL");
            }
        }
    }
    // Add ORDER BY clause based on sort_mode
    query.push(" ORDER BY ");

    // Add visible DESC to sort order if including hidden NFTs
    if include_hidden {
        query.push("assets.is_visible DESC, ");
    }

    match sort_mode {
        NftSortMode::Recent => {
            query.push("assets.is_pending DESC, assets.created_height DESC");
        }
        NftSortMode::Name => {
            query.push("assets.is_pending DESC, assets.name ASC, nfts.edition_number ASC");
        }
    }

    query.push(" LIMIT ? OFFSET ?");
    let query = query.build().bind(limit).bind(offset);

    let rows = query.fetch_all(conn).await?;
    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;

    let nfts = rows
        .into_iter()
        .map(|row| {
            Ok(NftAsset {
                asset: Asset {
                    hash: row.get::<Vec<u8>, _>("asset_hash").convert()?,
                    name: row.get::<Option<String>, _>("name"),
                    icon_url: row.get::<Option<String>, _>("icon_url"),
                    description: row.get::<Option<String>, _>("description"),
                    is_visible: row.get::<bool, _>("is_visible"),
                    is_sensitive_content: row.get::<bool, _>("is_sensitive_content"),
                    created_height: row
                        .get::<Option<i64>, _>("created_height")
                        .map(TryInto::try_into)
                        .transpose()?,
                },
                nft_info: NftCoinInfo {
                    collection_id: row
                        .get::<Option<Vec<u8>>, _>("collection_hash")
                        .map(Convert::convert)
                        .transpose()?,
                    minter_hash: row
                        .get::<Option<Vec<u8>>, _>("minter_hash")
                        .map(Convert::convert)
                        .transpose()?,
                    owner_hash: row
                        .get::<Option<Vec<u8>>, _>("owner_hash")
                        .map(Convert::convert)
                        .transpose()?,
                    metadata: Program::from(row.get::<Vec<u8>, _>("metadata")),
                    metadata_updater_puzzle_hash: row
                        .get::<Vec<u8>, _>("metadata_updater_puzzle_hash")
                        .convert()?,
                    royalty_puzzle_hash: row.get::<Vec<u8>, _>("royalty_puzzle_hash").convert()?,
                    royalty_basis_points: row.get::<u16, _>("royalty_basis_points"),
                    data_hash: row
                        .get::<Option<Vec<u8>>, _>("data_hash")
                        .map(Convert::convert)
                        .transpose()?,
                    metadata_hash: row
                        .get::<Option<Vec<u8>>, _>("metadata_hash")
                        .map(Convert::convert)
                        .transpose()?,
                    license_hash: row
                        .get::<Option<Vec<u8>>, _>("license_hash")
                        .map(Convert::convert)
                        .transpose()?,
                    edition_number: row
                        .get::<Option<i64>, _>("edition_number")
                        .map(TryInto::try_into)
                        .transpose()?,
                    edition_total: row
                        .get::<Option<i64>, _>("edition_total")
                        .map(TryInto::try_into)
                        .transpose()?,
                },
            })
        })
        .collect::<std::result::Result<Vec<NftAsset>, DatabaseError>>()?;

    Ok((nfts, total_count))
}

async fn did_asset(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<DidAsset>> {
    let asset_id = asset_id.as_ref();

    query!(
        "SELECT assets.hash AS asset_hash, assets.name, assets.icon_url, assets.description, assets.is_visible, 
            assets.is_sensitive_content, assets.created_height, dids.metadata, dids.recovery_list_hash, 
            dids.num_verifications_required
        FROM assets
        INNER JOIN dids ON dids.asset_id = assets.id
        WHERE hash = ?",
        asset_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(DidAsset {
            asset: Asset {
                hash: row.asset_hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                is_sensitive_content: row.is_sensitive_content,
                created_height: row.created_height.map(TryInto::try_into).transpose()?,
            },
            did_info: DidCoinInfo {
                metadata: Program::from(row.metadata),
                recovery_list_hash: row.recovery_list_hash.map(Convert::convert).transpose()?,
                num_verifications_required: row.num_verifications_required.convert()?,
            },
        })
    })
    .transpose()
}

async fn did_assets(conn: impl SqliteExecutor<'_>) -> Result<Vec<DidAsset>> {
    query!(
        "SELECT assets.hash AS asset_hash, assets.name, assets.icon_url, assets.description, assets.is_visible, 
            assets.is_sensitive_content, assets.created_height, dids.metadata, dids.recovery_list_hash, dids.num_verifications_required
        FROM assets
        INNER JOIN dids ON dids.asset_id = assets.id"
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(DidAsset {
            asset: Asset {
                hash: row.asset_hash.convert()?,
                name: row.name,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                is_sensitive_content: row.is_sensitive_content,
                created_height: row.created_height.map(TryInto::try_into).transpose()?,
            },
            did_info: DidCoinInfo {
                metadata: Program::from(row.metadata),
                recovery_list_hash: row.recovery_list_hash.map(Convert::convert).transpose()?,
                num_verifications_required: row.num_verifications_required.convert()?,
            },
        })
    })
    .collect()
}
