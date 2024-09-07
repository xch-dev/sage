use chia::{
    protocol::{Bytes32, Program},
    puzzles::LineageProof,
};
use chia_wallet_sdk::NftInfo;
use sqlx::SqliteExecutor;

use crate::{to_bytes32, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct NftDisplayInfo {
    pub coin_id: Bytes32,
    pub info: NftInfo<Program>,
    pub data_hash: Option<Bytes32>,
    pub metadata_hash: Option<Bytes32>,
    pub license_hash: Option<Bytes32>,
}

#[derive(Debug, Clone)]
pub struct NftData {
    pub blob: Vec<u8>,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct NftUri {
    pub hash: Bytes32,
    pub uri: String,
}

impl Database {
    pub async fn unchecked_nft_uris(&self, limit: u32) -> Result<Vec<NftUri>> {
        unchecked_nft_uris(&self.pool, limit).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_nft_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
        data_hash: Option<Bytes32>,
        metadata_hash: Option<Bytes32>,
        license_hash: Option<Bytes32>,
    ) -> Result<()> {
        insert_nft_coin(
            &mut *self.tx,
            coin_id,
            lineage_proof,
            nft_info,
            data_hash,
            metadata_hash,
            license_hash,
        )
        .await
    }

    pub async fn fetch_nfts(&mut self, limit: u32, offset: u32) -> Result<Vec<NftDisplayInfo>> {
        fetch_nfts(&mut *self.tx, limit, offset).await
    }

    pub async fn fetch_nft(&mut self, launcher_id: Bytes32) -> Result<Option<NftDisplayInfo>> {
        fetch_nft(&mut *self.tx, launcher_id).await
    }

    pub async fn nft_count(&mut self) -> Result<u32> {
        nft_count(&mut *self.tx).await
    }

    pub async fn insert_nft_uri(&mut self, uri: String, hash: Bytes32) -> Result<()> {
        insert_nft_uri(&mut *self.tx, uri, hash).await
    }

    pub async fn mark_nft_uri_checked(&mut self, uri: String, hash: Bytes32) -> Result<()> {
        mark_nft_uri_checked(&mut *self.tx, uri, hash).await
    }

    pub async fn insert_nft_data(
        &mut self,
        hash: Bytes32,
        data: Vec<u8>,
        mime_type: String,
    ) -> Result<()> {
        insert_nft_data(&mut *self.tx, hash, data, mime_type).await
    }

    pub async fn fetch_nft_data(&mut self, hash: Bytes32) -> Result<Option<NftData>> {
        fetch_nft_data(&mut *self.tx, hash).await
    }
}

async fn insert_nft_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    nft_info: NftInfo<Program>,
    data_hash: Option<Bytes32>,
    metadata_hash: Option<Bytes32>,
    license_hash: Option<Bytes32>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = nft_info.launcher_id.as_ref();
    let metadata = nft_info.metadata.as_ref();
    let metadata_updater_puzzle_hash = nft_info.metadata_updater_puzzle_hash.as_ref();
    let current_owner = nft_info.current_owner.as_deref();
    let royalty_puzzle_hash = nft_info.royalty_puzzle_hash.as_ref();
    let royalty_ten_thousandths = nft_info.royalty_ten_thousandths;
    let p2_puzzle_hash = nft_info.p2_puzzle_hash.as_ref();
    let data_hash = data_hash.as_deref();
    let metadata_hash = metadata_hash.as_deref();
    let license_hash = license_hash.as_deref();

    sqlx::query!(
        "
        INSERT INTO `nft_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `launcher_id`,
            `metadata`,
            `metadata_updater_puzzle_hash`,
            `current_owner`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `p2_puzzle_hash`,
            `data_hash`,
            `metadata_hash`,
            `license_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        launcher_id,
        metadata,
        metadata_updater_puzzle_hash,
        current_owner,
        royalty_puzzle_hash,
        royalty_ten_thousandths,
        p2_puzzle_hash,
        data_hash,
        metadata_hash,
        license_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn fetch_nfts(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftDisplayInfo>> {
    let rows = sqlx::query!(
        "
        SELECT
            `nft_coins`.`coin_id`,
            `launcher_id`,
            `metadata`,
            `metadata_updater_puzzle_hash`,
            `current_owner`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `p2_puzzle_hash`,
            `data_hash`,
            `metadata_hash`,
            `license_hash`
        FROM `nft_coins`
        INNER JOIN `coin_states` AS cs ON `nft_coins`.`coin_id` = `cs`.`coin_id`
        WHERE `cs`.`spent_height` IS NULL
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let mut nfts = Vec::new();

    for row in rows {
        let coin_id = to_bytes32(&row.coin_id)?;

        let info = NftInfo {
            launcher_id: to_bytes32(&row.launcher_id)?,
            metadata: row.metadata.into(),
            metadata_updater_puzzle_hash: to_bytes32(&row.metadata_updater_puzzle_hash)?,
            current_owner: row.current_owner.as_deref().map(to_bytes32).transpose()?,
            royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
            royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        };

        let data_hash = row.data_hash.as_deref().map(to_bytes32).transpose()?;
        let metadata_hash = row.metadata_hash.as_deref().map(to_bytes32).transpose()?;
        let license_hash = row.license_hash.as_deref().map(to_bytes32).transpose()?;

        nfts.push(NftDisplayInfo {
            coin_id,
            info,
            data_hash,
            metadata_hash,
            license_hash,
        });
    }

    Ok(nfts)
}

async fn fetch_nft(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<NftDisplayInfo>> {
    let launcher_id = launcher_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT
            `coin_id`,
            `launcher_id`,
            `metadata`,
            `metadata_updater_puzzle_hash`,
            `current_owner`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `p2_puzzle_hash`,
            `data_hash`,
            `metadata_hash`,
            `license_hash`
        FROM `nft_coins`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        let coin_id = to_bytes32(&row.coin_id)?;

        let info = NftInfo {
            launcher_id: to_bytes32(&row.launcher_id)?,
            metadata: row.metadata.into(),
            metadata_updater_puzzle_hash: to_bytes32(&row.metadata_updater_puzzle_hash)?,
            current_owner: row.current_owner.as_deref().map(to_bytes32).transpose()?,
            royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
            royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        };

        let data_hash = row.data_hash.as_deref().map(to_bytes32).transpose()?;
        let metadata_hash = row.metadata_hash.as_deref().map(to_bytes32).transpose()?;
        let license_hash = row.license_hash.as_deref().map(to_bytes32).transpose()?;

        Ok(NftDisplayInfo {
            coin_id,
            info,
            data_hash,
            metadata_hash,
            license_hash,
        })
    })
    .transpose()
}

async fn nft_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nft_coins`
        INNER JOIN `coin_states` AS cs ON `nft_coins`.`coin_id` = `cs`.`coin_id`
        WHERE `cs`.`spent_height` IS NULL
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn insert_nft_uri(conn: impl SqliteExecutor<'_>, uri: String, hash: Bytes32) -> Result<()> {
    let hash = hash.as_ref();

    sqlx::query!(
        "INSERT OR IGNORE INTO `nft_uris` (`hash`, `uri`, `checked`) VALUES (?, ?, ?)",
        hash,
        uri,
        false
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn unchecked_nft_uris(conn: impl SqliteExecutor<'_>, limit: u32) -> Result<Vec<NftUri>> {
    let rows = sqlx::query!(
        "SELECT `hash`, `uri` FROM `nft_uris` WHERE `checked` = 0 LIMIT ?",
        limit
    )
    .fetch_all(conn)
    .await?;

    let mut uris = Vec::new();

    for row in rows {
        let hash = to_bytes32(&row.hash)?;
        let uri = row.uri;

        uris.push(NftUri { hash, uri });
    }

    Ok(uris)
}

async fn mark_nft_uri_checked(
    conn: impl SqliteExecutor<'_>,
    uri: String,
    hash: Bytes32,
) -> Result<()> {
    let hash = hash.as_ref();

    sqlx::query!(
        "UPDATE `nft_uris` SET `checked` = 1 WHERE `hash` = ? AND `uri` = ?",
        hash,
        uri
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_nft_data(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    data: Vec<u8>,
    mime_type: String,
) -> Result<()> {
    let hash = hash.as_ref();

    sqlx::query!(
        "INSERT OR IGNORE INTO `nft_data` (`hash`, `data`, `mime_type`) VALUES (?, ?, ?)",
        hash,
        data,
        mime_type
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn fetch_nft_data(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<Option<NftData>> {
    let hash = hash.as_ref();

    let row = sqlx::query!(
        "SELECT `data`, `mime_type` FROM `nft_data` WHERE `hash` = ?",
        hash
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|row| NftData {
        blob: row.data,
        mime_type: row.mime_type,
    }))
}
