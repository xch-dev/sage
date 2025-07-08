use chia::protocol::{Bytes32, Program};
use sqlx::{query, Row, SqliteConnection, SqliteExecutor};

use crate::{Asset, AssetKind, Convert, Database, DatabaseError, DatabaseTx, Result};

pub static XCH_ASSET_HASH: Bytes32 = Bytes32::new([0; 32]);

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
    pub collection_hash: Bytes32,
    pub collection_name: Option<String>,
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

#[derive(Debug, Clone)]
pub struct RequestedNft {
    pub metadata: Program,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_basis_points: u16,
}

impl Database {
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

    pub async fn distinct_minter_dids(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Bytes32>, u32)> {
        distinct_minter_dids(&self.pool, limit, offset).await
    }

    pub async fn requested_nft(&self, launcher_id: Bytes32) -> Result<Option<RequestedNft>> {
        requested_nft(&self.pool, launcher_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_nft(&mut self, hash: Bytes32, coin_info: &NftCoinInfo) -> Result<()> {
        let hash = hash.as_ref();
        let collection_id = coin_info.collection_hash.as_ref();
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
        let edition_total: Option<i64> =
            coin_info.edition_total.map(TryInto::try_into).transpose()?;

        query!(
            "
            INSERT OR IGNORE INTO nfts (
                asset_id, collection_id, minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
                royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
                edition_number, edition_total
            )
            VALUES ((SELECT id FROM assets WHERE hash = ?), (SELECT id FROM collections WHERE hash = ?), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            hash,
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
        .execute(&mut *self.tx)
        .await?;

        Ok(())
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

async fn update_nft_coin_info(
    conn: &mut SqliteConnection,
    launcher_id: Bytes32,
    coin_info: &NftCoinInfo,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();
    let collection_hash = coin_info.collection_hash.as_ref();
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
            collection_id = (SELECT id FROM collections WHERE hash = ?),
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
        collection_hash,
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
        UPDATE nfts SET collection_id = (SELECT id FROM collections WHERE hash = ?)
        WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
        ",
        collection_id,
        hash
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn nft_asset(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<NftAsset>> {
    let asset_id = asset_id.as_ref();

    query!(
        "SELECT        
            assets.hash AS asset_hash, assets.name, assets.icon_url,
            assets.description, is_sensitive_content, assets.is_visible,
            collections.hash AS 'collection_hash?', collections.name AS 'collection_name?', 
            nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
            royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
            edition_number, edition_total
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
                kind: AssetKind::Nft,
            },
            nft_info: NftCoinInfo {
                collection_hash: row.collection_hash.convert()?.unwrap_or_default(),
                collection_name: row.collection_name,
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
            assets.hash AS asset_hash, assets.name, assets.icon_url,
            assets.description, is_sensitive_content, assets.is_visible,
            collections.hash AS collection_hash, collections.name AS collection_name, 
            nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
            royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
            edition_number, edition_total,
            COUNT(*) OVER() as total_count	
        FROM assets
        INNER JOIN nfts ON nfts.asset_id = assets.id
        LEFT JOIN collections ON collections.id = nfts.collection_id
        WHERE 1=1 ",
    );

    if let Some(name_search) = name_search {
        query.push("AND assets.name LIKE ");
        query.push_bind(format!("%{name_search}%"));
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
            query.push("assets.created_height DESC");
        }
        NftSortMode::Name => {
            query.push("assets.name ASC, nfts.edition_number ASC");
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
                    kind: AssetKind::Nft,
                },
                nft_info: NftCoinInfo {
                    collection_hash: row.get::<Vec<u8>, _>("collection_hash").convert()?,
                    collection_name: row.get::<Option<String>, _>("collection_name"),
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

async fn distinct_minter_dids(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Bytes32>, u32)> {
    let rows = query!(
        "SELECT DISTINCT minter_hash, COUNT(*) OVER() AS total_count FROM nfts LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.total_count.try_into())?;

    let dids = rows
        .into_iter()
        .filter_map(|row| row.minter_hash.convert().transpose())
        .collect::<Result<Vec<_>>>()?;

    Ok((dids, total_count))
}

async fn requested_nft(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<RequestedNft>> {
    let launcher_id = launcher_id.as_ref();

    query!(
        "
        SELECT
            metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash, royalty_basis_points
        FROM nfts
        INNER JOIN assets ON assets.id = nfts.asset_id
        WHERE hash = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(RequestedNft {
            metadata: Program::from(row.metadata),
            metadata_updater_puzzle_hash: row.metadata_updater_puzzle_hash.convert()?,
            royalty_puzzle_hash: row.royalty_puzzle_hash.convert()?,
            royalty_basis_points: row.royalty_basis_points.convert()?,
        })
    })
    .transpose()
}
