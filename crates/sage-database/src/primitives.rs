use chia::{
    protocol::{Bytes32, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Cat, Did, DidInfo, Nft, NftInfo};
use sqlx::SqliteExecutor;

use crate::{
    error::Result, to_bytes, to_bytes32, to_coin, to_lineage_proof, Database, DatabaseError,
    DatabaseTx,
};

#[derive(Debug, Clone)]
pub struct CatRow {
    pub asset_id: Bytes32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub ticker: Option<String>,
    pub precision: u8,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NftRow {
    pub launcher_id: Bytes32,
    pub coin_id: Bytes32,
    pub p2_puzzle_hash: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
    pub current_owner: Option<Bytes32>,
    pub data_hash: Option<Bytes32>,
    pub metadata_json: Option<String>,
    pub metadata_hash: Option<Bytes32>,
    pub license_hash: Option<Bytes32>,
    pub edition_number: Option<u32>,
    pub edition_total: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct NftUri {
    pub uri: String,
    pub kind: NftUriKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NftUriKind {
    Data,
    Metadata,
    License,
}

#[derive(Debug, Clone)]
pub struct UncachedNftInfo {
    pub info: NftInfo<Program>,
    pub coin_id: Bytes32,
}

impl Database {
    pub async fn maybe_insert_cat(&self, row: CatRow) -> Result<()> {
        maybe_insert_cat(&self.pool, row).await
    }

    pub async fn update_cat(&self, row: CatRow) -> Result<()> {
        update_cat(&self.pool, row).await
    }

    pub async fn cats(&self) -> Result<Vec<CatRow>> {
        cats(&self.pool).await
    }

    pub async fn unidentified_cat(&self) -> Result<Option<Bytes32>> {
        unidentified_cat(&self.pool).await
    }

    pub async fn insert_cat_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(&self.pool, coin_id, lineage_proof, p2_puzzle_hash, asset_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn update_nft(&self, row: NftRow) -> Result<()> {
        update_nft(&self.pool, row).await
    }

    pub async fn delete_old_nfts(&self) -> Result<()> {
        delete_old_nfts(&self.pool).await
    }

    pub async fn nfts(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts(&self.pool, limit, offset).await
    }

    pub async fn nft_count(&self) -> Result<u32> {
        nft_count(&self.pool).await
    }

    pub async fn insert_nft_uri(
        &self,
        nft_id: Bytes32,
        uri: String,
        kind: NftUriKind,
    ) -> Result<()> {
        insert_nft_uri(&self.pool, nft_id, uri, kind).await
    }

    pub async fn clear_nft_uris(&self, nft_id: Bytes32) -> Result<()> {
        clear_nft_uris(&self.pool, nft_id).await
    }

    pub async fn nft_uris(&self, nft_id: Bytes32) -> Result<Vec<NftUri>> {
        nft_uris(&self.pool, nft_id).await
    }

    pub async fn nft(&self, launcher_id: Bytes32) -> Result<Option<NftRow>> {
        nft(&self.pool, launcher_id).await
    }

    pub async fn insert_nft_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
    ) -> Result<()> {
        insert_nft_coin(&self.pool, coin_id, lineage_proof, nft_info).await
    }

    pub async fn nft_coin(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&self.pool, launcher_id).await
    }

    pub async fn updated_nft_coins(&self, limit: u32) -> Result<Vec<UncachedNftInfo>> {
        updated_nft_coins(&self.pool, limit).await
    }

    pub async fn insert_did_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        did_info: DidInfo<Program>,
    ) -> Result<()> {
        insert_did_coin(&self.pool, coin_id, lineage_proof, did_info).await
    }

    pub async fn did_coin(&self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_coin(&self.pool, launcher_id).await
    }

    pub async fn did_coins(&self) -> Result<Vec<Did<Program>>> {
        did_coins(&self.pool).await
    }

    pub async fn insert_unknown_coin(&self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&self.pool, coin_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn maybe_insert_cat(&mut self, row: CatRow) -> Result<()> {
        maybe_insert_cat(&mut *self.tx, row).await
    }

    pub async fn update_cat(&mut self, row: CatRow) -> Result<()> {
        update_cat(&mut *self.tx, row).await
    }

    pub async fn cats(&mut self) -> Result<Vec<CatRow>> {
        cats(&mut *self.tx).await
    }

    pub async fn unidentified_cat(&mut self) -> Result<Option<Bytes32>> {
        unidentified_cat(&mut *self.tx).await
    }

    pub async fn insert_cat_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(
            &mut *self.tx,
            coin_id,
            lineage_proof,
            p2_puzzle_hash,
            asset_id,
        )
        .await
    }

    pub async fn cat_coin(&mut self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&mut *self.tx, coin_id).await
    }

    pub async fn update_nft(&mut self, row: NftRow) -> Result<()> {
        update_nft(&mut *self.tx, row).await
    }

    pub async fn delete_old_nfts(&mut self) -> Result<()> {
        delete_old_nfts(&mut *self.tx).await
    }

    pub async fn nfts(&mut self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts(&mut *self.tx, limit, offset).await
    }

    pub async fn nft_count(&mut self) -> Result<u32> {
        nft_count(&mut *self.tx).await
    }

    pub async fn insert_nft_uri(
        &mut self,
        nft_id: Bytes32,
        uri: String,
        kind: NftUriKind,
    ) -> Result<()> {
        insert_nft_uri(&mut *self.tx, nft_id, uri, kind).await
    }

    pub async fn clear_nft_uris(&mut self, nft_id: Bytes32) -> Result<()> {
        clear_nft_uris(&mut *self.tx, nft_id).await
    }

    pub async fn nft_uris(&mut self, nft_id: Bytes32) -> Result<Vec<NftUri>> {
        nft_uris(&mut *self.tx, nft_id).await
    }

    pub async fn nft(&mut self, launcher_id: Bytes32) -> Result<Option<NftRow>> {
        nft(&mut *self.tx, launcher_id).await
    }

    pub async fn insert_nft_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
    ) -> Result<()> {
        insert_nft_coin(&mut *self.tx, coin_id, lineage_proof, nft_info).await
    }

    pub async fn nft_coin(&mut self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&mut *self.tx, launcher_id).await
    }

    pub async fn updated_nft_coins(&mut self, limit: u32) -> Result<Vec<UncachedNftInfo>> {
        updated_nft_coins(&mut *self.tx, limit).await
    }

    pub async fn insert_did_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        did_info: DidInfo<Program>,
    ) -> Result<()> {
        insert_did_coin(&mut *self.tx, coin_id, lineage_proof, did_info).await
    }

    pub async fn did_coin(&mut self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_coin(&mut *self.tx, launcher_id).await
    }

    pub async fn did_coins(&mut self) -> Result<Vec<Did<Program>>> {
        did_coins(&mut *self.tx).await
    }

    pub async fn insert_unknown_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&mut *self.tx, coin_id).await
    }
}

async fn maybe_insert_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `cats` (
            `asset_id`,
            `name`,
            `description`,
            `ticker`,
            `precision`,
            `icon_url`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.description,
        row.ticker,
        row.precision,
        row.icon_url
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn update_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `cats` (
            `asset_id`,
            `name`,
            `description`,
            `ticker`,
            `precision`,
            `icon_url`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.description,
        row.ticker,
        row.precision,
        row.icon_url
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn cats(conn: impl SqliteExecutor<'_>) -> Result<Vec<CatRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `asset_id`,
            `name`,
            `description`,
            `ticker`,
            `precision`,
            `icon_url`
        FROM `cats`
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(CatRow {
                asset_id: to_bytes32(&row.asset_id)?,
                name: row.name,
                description: row.description,
                ticker: row.ticker,
                precision: row.precision.try_into()?,
                icon_url: row.icon_url,
            })
        })
        .collect()
}

async fn unidentified_cat(conn: impl SqliteExecutor<'_>) -> Result<Option<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `asset_id` FROM `cat_coins`
        WHERE `asset_id` NOT IN (SELECT `asset_id` FROM `cats`)
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?;
    rows.map(|row| to_bytes32(&row.asset_id)).transpose()
}

async fn insert_cat_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    p2_puzzle_hash: Bytes32,
    asset_id: Bytes32,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let asset_id = asset_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `cat_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `p2_puzzle_hash`,
            `asset_id`
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        p2_puzzle_hash,
        asset_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cat.parent_parent_coin_id, cat.parent_inner_puzzle_hash, cat.parent_amount,
            cat.p2_puzzle_hash, cat.asset_id
        FROM `coin_states` AS cs
        INNER JOIN `cat_coins` AS cat
        ON cs.coin_id = cat.coin_id
        WHERE cs.coin_id = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Cat {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        lineage_proof: Some(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        asset_id: to_bytes32(&row.asset_id)?,
    }))
}

async fn update_nft(conn: impl SqliteExecutor<'_>, row: NftRow) -> Result<()> {
    let launcher_id = row.launcher_id.as_ref();
    let coin_id = row.coin_id.as_ref();
    let p2_puzzle_hash = row.p2_puzzle_hash.as_ref();
    let royalty_puzzle_hash = row.royalty_puzzle_hash.as_ref();
    let royalty_ten_thousandths = row.royalty_ten_thousandths;
    let current_owner = row.current_owner.as_deref();
    let data_hash = row.data_hash.as_deref();
    let metadata_json = row.metadata_json.as_ref();
    let metadata_hash = row.metadata_hash.as_deref();
    let license_hash = row.license_hash.as_deref();
    let edition_number = row.edition_number;
    let edition_total = row.edition_total;

    sqlx::query!(
        "
        REPLACE INTO `nfts` (
            `launcher_id`,
            `coin_id`,
            `p2_puzzle_hash`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `current_owner`,
            `data_hash`,
            `metadata_json`,
            `metadata_hash`,
            `license_hash`,
            `edition_number`,
            `edition_total`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        launcher_id,
        coin_id,
        p2_puzzle_hash,
        royalty_puzzle_hash,
        royalty_ten_thousandths,
        current_owner,
        data_hash,
        metadata_json,
        metadata_hash,
        license_hash,
        edition_number,
        edition_total
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn clear_nft_uris(conn: impl SqliteExecutor<'_>, nft_id: Bytes32) -> Result<()> {
    let nft_id = nft_id.as_ref();

    sqlx::query!(
        "
        DELETE FROM `nft_uris` WHERE `nft_id` = ?
        ",
        nft_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_nft_uri(
    conn: impl SqliteExecutor<'_>,
    nft_id: Bytes32,
    uri: String,
    kind: NftUriKind,
) -> Result<()> {
    let nft_id = nft_id.as_ref();
    let kind = match kind {
        NftUriKind::Data => 0,
        NftUriKind::Metadata => 1,
        NftUriKind::License => 2,
    };

    sqlx::query!(
        "
        INSERT INTO `nft_uris` (`nft_id`, `uri`, `kind`) VALUES (?, ?, ?)
        ",
        nft_id,
        uri,
        kind
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nfts(conn: impl SqliteExecutor<'_>, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `launcher_id`,
            `coin_id`,
            `p2_puzzle_hash`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `current_owner`,
            `data_hash`,
            `metadata_json`,
            `metadata_hash`,
            `license_hash`,
            `edition_number`,
            `edition_total`
        FROM `nfts`
        ORDER BY `launcher_id`
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(NftRow {
                launcher_id: to_bytes32(&row.launcher_id)?,
                coin_id: to_bytes32(&row.coin_id)?,
                p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
                royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
                royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
                current_owner: row
                    .current_owner
                    .map(|owner| to_bytes32(&owner))
                    .transpose()?,
                data_hash: row.data_hash.as_deref().map(to_bytes32).transpose()?,
                metadata_json: row.metadata_json,
                metadata_hash: row.metadata_hash.as_deref().map(to_bytes32).transpose()?,
                license_hash: row.license_hash.as_deref().map(to_bytes32).transpose()?,
                edition_number: row.edition_number.map(u32::try_from).transpose()?,
                edition_total: row.edition_total.map(u32::try_from).transpose()?,
            })
        })
        .collect()
}

async fn nft(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<NftRow>> {
    let launcher_id = launcher_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT
            `launcher_id`,
            `coin_id`,
            `p2_puzzle_hash`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `current_owner`,
            `data_hash`,
            `metadata_json`,
            `metadata_hash`,
            `license_hash`,
            `edition_number`,
            `edition_total`
        FROM `nfts`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        Ok(NftRow {
            launcher_id: to_bytes32(&row.launcher_id)?,
            coin_id: to_bytes32(&row.coin_id)?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
            royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
            royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
            current_owner: row
                .current_owner
                .map(|owner| to_bytes32(&owner))
                .transpose()?,
            data_hash: row.data_hash.as_deref().map(to_bytes32).transpose()?,
            metadata_json: row.metadata_json,
            metadata_hash: row.metadata_hash.as_deref().map(to_bytes32).transpose()?,
            license_hash: row.license_hash.as_deref().map(to_bytes32).transpose()?,
            edition_number: row.edition_number.map(u32::try_from).transpose()?,
            edition_total: row.edition_total.map(u32::try_from).transpose()?,
        })
    })
    .transpose()
}

async fn nft_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts`
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn nft_uris(conn: impl SqliteExecutor<'_>, nft_id: Bytes32) -> Result<Vec<NftUri>> {
    let nft_id = nft_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT `uri`, `kind` FROM `nft_uris` WHERE `nft_id` = ?
        ",
        nft_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(NftUri {
                uri: row.uri,
                kind: match row.kind {
                    0 => NftUriKind::Data,
                    1 => NftUriKind::Metadata,
                    2 => NftUriKind::License,
                    _ => return Err(DatabaseError::InvalidEnumVariant),
                },
            })
        })
        .collect()
}

async fn insert_nft_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    nft_info: NftInfo<Program>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = nft_info.launcher_id.as_ref();
    let metadata = nft_info.metadata.as_ref();
    let metadata_updater_puzzle_hash = nft_info.metadata_updater_puzzle_hash.as_ref();
    let current_owner = nft_info.current_owner.map(|owner| owner.to_vec());
    let royalty_puzzle_hash = nft_info.royalty_puzzle_hash.as_ref();
    let p2_puzzle_hash = nft_info.p2_puzzle_hash.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `nft_coins` (
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
            `p2_puzzle_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        nft_info.royalty_ten_thousandths,
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nft_coin(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Nft<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            nft.parent_parent_coin_id, nft.parent_inner_puzzle_hash, nft.parent_amount,
            nft.launcher_id, nft.metadata, nft.metadata_updater_puzzle_hash,
            nft.current_owner, nft.royalty_puzzle_hash, nft.royalty_ten_thousandths,
            nft.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `nft_coins` AS nft
        ON cs.coin_id = nft.coin_id
        WHERE nft.launcher_id = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        proof: Proof::Lineage(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        info: NftInfo {
            launcher_id: to_bytes32(&row.launcher_id)?,
            metadata: row.metadata.into(),
            metadata_updater_puzzle_hash: to_bytes32(&row.metadata_updater_puzzle_hash)?,
            current_owner: row
                .current_owner
                .map(|owner| to_bytes32(&owner))
                .transpose()?,
            royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
            royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        },
    }))
}

async fn updated_nft_coins(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
) -> Result<Vec<UncachedNftInfo>> {
    let rows = sqlx::query!(
        "
        SELECT
            nft.parent_parent_coin_id, nft.parent_inner_puzzle_hash, nft.parent_amount,
            nft.launcher_id, nft.metadata, nft.metadata_updater_puzzle_hash,
            nft.current_owner, nft.royalty_puzzle_hash, nft.royalty_ten_thousandths,
            nft.p2_puzzle_hash, nft.coin_id
        FROM `nft_coins` AS nft
        INNER JOIN `coin_states` AS cs
        ON cs.coin_id = nft.coin_id
        LEFT JOIN `nfts` AS n
        ON nft.launcher_id = n.launcher_id
        WHERE cs.spent_height IS NULL
        AND (n.launcher_id IS NULL OR n.coin_id != cs.coin_id)
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(UncachedNftInfo {
                coin_id: to_bytes32(&row.coin_id)?,
                info: NftInfo {
                    launcher_id: to_bytes32(&row.launcher_id)?,
                    metadata: row.metadata.into(),
                    metadata_updater_puzzle_hash: to_bytes32(&row.metadata_updater_puzzle_hash)?,
                    current_owner: row
                        .current_owner
                        .map(|owner| to_bytes32(&owner))
                        .transpose()?,
                    royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
                    royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
                    p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
                },
            })
        })
        .collect()
}

async fn delete_old_nfts(conn: impl SqliteExecutor<'_>) -> Result<()> {
    sqlx::query!(
        "
        DELETE FROM `nfts`
        WHERE NOT EXISTS (
            SELECT 1
            FROM `nft_coins` nc
            JOIN `coin_states` cs ON nc.`coin_id` = cs.`coin_id`
            WHERE nc.`launcher_id` = nfts.`launcher_id`
            AND cs.`spent_height` IS NULL
        );
        "
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_did_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    did_info: DidInfo<Program>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = did_info.launcher_id.as_ref();
    let recovery_list_hash = did_info.recovery_list_hash.as_deref();
    let num_verifications_required = did_info.num_verifications_required.to_be_bytes();
    let num_verifications_required = num_verifications_required.as_ref();
    let metadata = did_info.metadata.as_ref();
    let p2_puzzle_hash = did_info.p2_puzzle_hash.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `did_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `launcher_id`,
            `recovery_list_hash`,
            `num_verifications_required`,
            `metadata`,
            `p2_puzzle_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        launcher_id,
        recovery_list_hash,
        num_verifications_required,
        metadata,
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn did_coin(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Did<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did
        ON cs.coin_id = did.coin_id
        WHERE did.launcher_id = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        proof: Proof::Lineage(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        info: DidInfo {
            launcher_id: to_bytes32(&row.launcher_id)?,
            recovery_list_hash: row
                .recovery_list_hash
                .map(|hash| to_bytes32(&hash))
                .transpose()?,
            num_verifications_required: u64::from_be_bytes(to_bytes(
                &row.num_verifications_required,
            )?),
            metadata: row.metadata.into(),
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        },
    }))
}

async fn did_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Did<Program>>> {
    let rows = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did
        ON cs.coin_id = did.coin_id
        WHERE cs.spent_height IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(Did {
                coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                proof: Proof::Lineage(to_lineage_proof(
                    &row.parent_parent_coin_id,
                    &row.parent_inner_puzzle_hash,
                    &row.parent_amount,
                )?),
                info: DidInfo::<Program> {
                    launcher_id: to_bytes32(&row.launcher_id)?,
                    recovery_list_hash: row
                        .recovery_list_hash
                        .map(|hash| to_bytes32(&hash))
                        .transpose()?,
                    num_verifications_required: u64::from_be_bytes(to_bytes(
                        &row.num_verifications_required,
                    )?),
                    metadata: row.metadata.into(),
                    p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
                },
            })
        })
        .collect()
}

async fn insert_unknown_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `unknown_coins` (`coin_id`) VALUES (?)
        ",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
