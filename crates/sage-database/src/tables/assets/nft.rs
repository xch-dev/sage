use chia::protocol::{Bytes32, Coin, Program};
use sqlx::{query, Row};

use crate::{Asset, AssetKind, CoinKind, CoinRow, Convert, Database, DatabaseTx, Result};

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
pub struct NftRow {
    pub asset: Asset,
    pub nft_info: NftCoinInfo,
    pub coin_row: CoinRow,
}

#[derive(Debug, Clone)]
pub struct NftMetadataInfo {
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub is_sensitive_content: bool,
    pub collection_id: Bytes32,
}

#[derive(Debug, Clone)]
pub struct NftOfferInfo {
    pub metadata: Program,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_basis_points: u16,
}

impl Database {
    pub async fn owned_nft(&self, hash: Bytes32) -> Result<Option<NftRow>> {
        let hash = hash.as_ref();

        query!(
            "
            SELECT        
                asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
                asset_description, asset_is_sensitive_content, asset_is_visible,
                collections.hash AS 'collection_hash?', collections.name AS collection_name, 
                nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
                royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
                edition_number, edition_total,
                parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash, created_height, spent_height,
                (
                    SELECT hash FROM offers
                    INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
                    WHERE offer_coins.coin_id = owned_coins.coin_id
                    AND offers.status <= 1
                    LIMIT 1
                ) AS 'offer_hash?',
                (
                    SELECT timestamp FROM blocks
                    WHERE height = owned_coins.created_height
                ) AS created_timestamp,
                (
                    SELECT timestamp FROM blocks
                    WHERE height = owned_coins.spent_height
                ) AS spent_timestamp
            FROM owned_coins
            INNER JOIN nfts ON nfts.asset_id = owned_coins.asset_id
            LEFT JOIN collections ON collections.id = nfts.collection_id
            WHERE owned_coins.asset_hash = ?
            ",
            hash
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(NftRow {
                asset: Asset {
                    hash: row.asset_hash.convert()?,
                    name: row.asset_name,
                    ticker: row.asset_ticker,
                    precision: row.asset_precision.convert()?,
                    icon_url: row.asset_icon_url,
                    description: row.asset_description,
                    is_sensitive_content: row.asset_is_sensitive_content,
                    is_visible: row.asset_is_visible,
                    kind: AssetKind::Nft,
                },
                nft_info: NftCoinInfo {
                    collection_hash: row.collection_hash.convert()?.unwrap_or_default(),
                    collection_name: row.collection_name,
                    minter_hash: row.minter_hash.convert()?,
                    owner_hash: row.owner_hash.convert()?,
                    metadata: row.metadata.into(),
                    metadata_updater_puzzle_hash: row.metadata_updater_puzzle_hash.convert()?,
                    royalty_puzzle_hash: row.royalty_puzzle_hash.convert()?,
                    royalty_basis_points: row.royalty_basis_points.convert()?,
                    data_hash: row.data_hash.convert()?,
                    metadata_hash: row.metadata_hash.convert()?,
                    license_hash: row.license_hash.convert()?,
                    edition_number: row.edition_number.convert()?,
                    edition_total: row.edition_total.convert()?,
                },
                coin_row: CoinRow {
                    coin: Coin::new(
                        row.parent_coin_hash.convert()?,
                        row.puzzle_hash.convert()?,
                        row.amount.convert()?,
                    ),
                    p2_puzzle_hash: row.p2_puzzle_hash.convert()?,
                    kind: CoinKind::Nft,
                    mempool_item_hash: None,
                    offer_hash: row.offer_hash.convert()?,
                    created_height: row.created_height.convert()?,
                    spent_height: row.spent_height.convert()?,
                    created_timestamp: row.created_timestamp.convert()?,
                    spent_timestamp: row.spent_timestamp.convert()?,
                },
            })
        })
        .transpose()
    }

    pub async fn owned_nfts(
        &self,
        name_search: Option<String>,
        group_search: Option<NftGroupSearch>,
        sort_mode: NftSortMode,
        include_hidden: bool,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<NftRow>, u32)> {
        let mut query = sqlx::QueryBuilder::new(
            "
            SELECT        
                asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
                asset_description, asset_is_sensitive_content, asset_is_visible,
                collections.hash AS collection_hash, collections.name AS collection_name, 
                owned_nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
                royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
                edition_number, edition_total,
                parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash, created_height, spent_height,
                offer_hash,created_timestamp, spent_timestamp,
                COUNT(*) OVER() as total_count	
            FROM owned_nfts
            LEFT JOIN collections ON collections.id = owned_nfts.collection_id
            WHERE 1=1
            ",
        );

        if let Some(name_search) = name_search {
            query.push("AND asset_name LIKE ");
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
                    query.push(" AND owned_nfts.minter_hash = ");
                    query.push_bind(id.as_ref().to_vec());
                }
                NftGroupSearch::NoMinterDid => {
                    query.push(" AND owned_nfts.minter_hash IS NULL");
                }
                NftGroupSearch::OwnerDid(id) => {
                    query.push(" AND owned_nfts.owner_hash = ");
                    query.push_bind(id.as_ref().to_vec());
                }
                NftGroupSearch::NoOwnerDid => {
                    query.push(" AND owner_hash IS NULL");
                }
            }
        }
        // Add ORDER BY clause based on sort_mode
        query.push(" ORDER BY ");

        // Add visible DESC to sort order if including hidden NFTs
        if include_hidden {
            query.push("asset_is_visible DESC, ");
        }

        match sort_mode {
            NftSortMode::Recent => {
                query.push("(created_height IS NULL) DESC, created_height DESC");
            }
            NftSortMode::Name => {
                query.push("asset_name ASC, edition_number ASC");
            }
        }

        query.push(" LIMIT ? OFFSET ?");
        let query = query.build().bind(limit).bind(offset);

        let rows = query.fetch_all(&self.pool).await?;
        let total_count = rows
            .first()
            .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;

        let nfts = rows
            .into_iter()
            .map(|row| {
                Ok(NftRow {
                    asset: Asset {
                        hash: row.get::<Vec<u8>, _>("asset_hash").convert()?,
                        name: row.get::<Option<String>, _>("asset_name"),
                        ticker: row.get::<Option<String>, _>("asset_ticker"),
                        precision: row.get::<i64, _>("asset_precision").convert()?,
                        icon_url: row.get::<Option<String>, _>("asset_icon_url"),
                        description: row.get::<Option<String>, _>("asset_description"),
                        is_visible: row.get::<bool, _>("asset_is_visible"),
                        is_sensitive_content: row.get::<bool, _>("asset_is_sensitive_content"),
                        kind: AssetKind::Nft,
                    },
                    nft_info: NftCoinInfo {
                        collection_hash: row
                            .get::<Option<Vec<u8>>, _>("collection_hash")
                            .convert()?
                            .unwrap_or_default(),
                        collection_name: row.get::<Option<String>, _>("collection_name"),
                        minter_hash: row.get::<Option<Vec<u8>>, _>("minter_hash").convert()?,
                        owner_hash: row.get::<Option<Vec<u8>>, _>("owner_hash").convert()?,
                        metadata: row.get::<Vec<u8>, _>("metadata").into(),
                        metadata_updater_puzzle_hash: row
                            .get::<Vec<u8>, _>("metadata_updater_puzzle_hash")
                            .convert()?,
                        royalty_puzzle_hash: row
                            .get::<Vec<u8>, _>("royalty_puzzle_hash")
                            .convert()?,
                        royalty_basis_points: row
                            .get::<i64, _>("royalty_basis_points")
                            .convert()?,
                        data_hash: row.get::<Option<Vec<u8>>, _>("data_hash").convert()?,
                        metadata_hash: row.get::<Option<Vec<u8>>, _>("metadata_hash").convert()?,
                        license_hash: row.get::<Option<Vec<u8>>, _>("license_hash").convert()?,
                        edition_number: row.get::<Option<i64>, _>("edition_number").convert()?,
                        edition_total: row.get::<Option<i64>, _>("edition_total").convert()?,
                    },
                    coin_row: CoinRow {
                        coin: Coin::new(
                            row.get::<Vec<u8>, _>("parent_coin_hash").convert()?,
                            row.get::<Vec<u8>, _>("puzzle_hash").convert()?,
                            row.get::<Vec<u8>, _>("amount").convert()?,
                        ),
                        p2_puzzle_hash: row.get::<Vec<u8>, _>("p2_puzzle_hash").convert()?,
                        kind: CoinKind::Nft,
                        mempool_item_hash: None,
                        offer_hash: row.get::<Option<Vec<u8>>, _>("offer_hash").convert()?,
                        created_height: row.get::<Option<i64>, _>("created_height").convert()?,
                        spent_height: row.get::<Option<i64>, _>("spent_height").convert()?,
                        created_timestamp: row
                            .get::<Option<i64>, _>("created_timestamp")
                            .convert()?,
                        spent_timestamp: row.get::<Option<i64>, _>("spent_timestamp").convert()?,
                    },
                })
            })
            .collect::<Result<Vec<NftRow>>>()?;

        Ok((nfts, total_count))
    }

    pub async fn distinct_minter_dids(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Bytes32>, u32)> {
        let rows = query!(
            "SELECT DISTINCT minter_hash, COUNT(*) OVER() AS total_count 
            FROM owned_nfts 
            ORDER BY minter_hash ASC
            LIMIT ? OFFSET ?",
            limit,
            offset
        )
        .fetch_all(&self.pool)
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

    pub async fn offer_nft_info(&self, hash: Bytes32) -> Result<Option<NftOfferInfo>> {
        let hash = hash.as_ref();

        query!(
            "
            SELECT
                metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash, royalty_basis_points
            FROM nfts
            INNER JOIN assets ON assets.id = nfts.asset_id
            WHERE hash = ?
            ",
            hash
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(NftOfferInfo {
                metadata: Program::from(row.metadata),
                metadata_updater_puzzle_hash: row.metadata_updater_puzzle_hash.convert()?,
                royalty_puzzle_hash: row.royalty_puzzle_hash.convert()?,
                royalty_basis_points: row.royalty_basis_points.convert()?,
            })
        })
        .transpose()
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

    pub async fn update_nft(&mut self, hash: Bytes32, coin_info: &NftCoinInfo) -> Result<()> {
        let hash = hash.as_ref();
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
        let edition_total: Option<i64> =
            coin_info.edition_total.map(TryInto::try_into).transpose()?;

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
            hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    pub async fn update_nft_metadata(
        &mut self,
        hash: Bytes32,
        metadata_info: NftMetadataInfo,
    ) -> Result<()> {
        let hash = hash.as_ref();
        let collection_id = metadata_info.collection_id.as_ref();

        query!(
            "
            UPDATE assets SET
                name = ?,
                icon_url = ?,
                description = ?,
                is_sensitive_content = ?
            WHERE hash = ?
            ",
            metadata_info.name,
            metadata_info.icon_url,
            metadata_info.description,
            metadata_info.is_sensitive_content,
            hash
        )
        .execute(&mut *self.tx)
        .await?;

        query!(
            "
            UPDATE nfts SET collection_id = (SELECT id FROM collections WHERE hash = ?)
            WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
            ",
            collection_id,
            hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
