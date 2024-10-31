use chia::{
    protocol::{Bytes32, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Nft, NftInfo};
use sqlx::SqliteExecutor;

use crate::{to_bytes32, to_coin, to_lineage_proof, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct NftCollectionRow {
    pub collection_id: Bytes32,
    pub did_id: Bytes32,
    pub metadata_collection_id: String,
    pub visible: bool,
    pub name: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NftRow {
    pub launcher_id: Bytes32,
    pub collection_id: Option<Bytes32>,
    pub minter_did: Option<Bytes32>,
    pub owner_did: Option<Bytes32>,
    pub visible: bool,
    pub sensitive_content: bool,
    pub name: Option<String>,
    pub created_height: Option<u32>,
    pub metadata_hash: Option<Bytes32>,
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

    pub async fn set_nft_visible(&self, launcher_id: Bytes32, visible: bool) -> Result<()> {
        set_nft_visible(&self.pool, launcher_id, visible).await
    }

    pub async fn nft(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft(&self.pool, launcher_id).await
    }

    pub async fn fetch_nft_data(&self, hash: Bytes32) -> Result<Option<NftData>> {
        fetch_nft_data(&self.pool, hash).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn nft_collections_visible_named(
        &mut self,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<NftCollectionRow>> {
        nft_collections_visible_named(&mut *self.tx, offset, limit).await
    }

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

    pub async fn nft_collection_count(&mut self) -> Result<u32> {
        nft_collection_count(&mut *self.tx).await
    }

    pub async fn visible_nft_collection_count(&mut self) -> Result<u32> {
        visible_nft_collection_count(&mut *self.tx).await
    }

    pub async fn nfts_visible_named(&mut self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts_visible_named(&mut *self.tx, limit, offset).await
    }

    pub async fn nft_count(&mut self) -> Result<u32> {
        nft_count(&mut *self.tx).await
    }

    pub async fn visible_nft_count(&mut self) -> Result<u32> {
        visible_nft_count(&mut *self.tx).await
    }

    pub async fn insert_nft_uri(&mut self, uri: String, hash: Bytes32) -> Result<()> {
        insert_nft_uri(&mut *self.tx, uri, hash).await
    }

    pub async fn set_nft_uri_checked(&mut self, uri: String, hash: Bytes32) -> Result<()> {
        set_nft_uri_checked(&mut *self.tx, uri, hash).await
    }

    pub async fn insert_nft_data(&mut self, hash: Bytes32, nft_data: NftData) -> Result<()> {
        insert_nft_data(&mut *self.tx, hash, nft_data).await
    }

    pub async fn fetch_nft_data(&mut self, hash: Bytes32) -> Result<Option<NftData>> {
        fetch_nft_data(&mut *self.tx, hash).await
    }

    pub async fn insert_nft(&mut self, row: NftRow) -> Result<()> {
        insert_nft(&mut *self.tx, row).await
    }

    pub async fn nft_row(&mut self, launcher_id: Bytes32) -> Result<Option<NftRow>> {
        nft_row(&mut *self.tx, launcher_id).await
    }

    pub async fn delete_nft(&mut self, launcher_id: Bytes32) -> Result<()> {
        delete_nft(&mut *self.tx, launcher_id).await
    }

    pub async fn data_hash(&mut self, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
        data_hash(&mut *self.tx, launcher_id).await
    }

    pub async fn metadata_hash(&mut self, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
        metadata_hash(&mut *self.tx, launcher_id).await
    }

    pub async fn license_hash(&mut self, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
        license_hash(&mut *self.tx, launcher_id).await
    }

    pub async fn nfts_by_metadata_hash(&mut self, metadata_hash: Bytes32) -> Result<Vec<NftRow>> {
        nfts_by_metadata_hash(&mut *self.tx, metadata_hash).await
    }

    pub async fn nft(&mut self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft(&mut *self.tx, launcher_id).await
    }

    pub async fn insert_nft_collection(&mut self, row: NftCollectionRow) -> Result<()> {
        insert_nft_collection(&mut *self.tx, row).await
    }

    pub async fn nft_launcher_id(&mut self, coin_id: Bytes32) -> Result<Option<Bytes32>> {
        nft_launcher_id(&mut *self.tx, coin_id).await
    }
}

async fn insert_nft_collection(conn: impl SqliteExecutor<'_>, row: NftCollectionRow) -> Result<()> {
    let collection_id = row.collection_id.as_ref();
    let did_id = row.did_id.as_ref();
    let name = row.name.as_deref();
    let icon = row.icon.as_deref();

    sqlx::query!(
        "
        REPLACE INTO `nft_collections` (
            `collection_id`,
            `did_id`,
            `metadata_collection_id`,
            `visible`,
            `name`,
            `icon`
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        collection_id,
        did_id,
        row.metadata_collection_id,
        row.visible,
        name,
        icon
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nft_collections_visible_named(
    conn: impl SqliteExecutor<'_>,
    offset: u32,
    limit: u32,
) -> Result<Vec<NftCollectionRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `collection_id`,
            `did_id`,
            `metadata_collection_id`,
            `visible`,
            `name`,
            `icon`
        FROM `nft_collections` INDEXED BY `col_named`
        WHERE `visible` = 1
        ORDER BY `is_named` DESC, `name` ASC, `collection_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let mut collections = Vec::new();

    for row in rows {
        collections.push(NftCollectionRow {
            collection_id: to_bytes32(&row.collection_id)?,
            did_id: to_bytes32(&row.did_id)?,
            metadata_collection_id: row.metadata_collection_id,
            visible: row.visible,
            name: row.name,
            icon: row.icon,
        });
    }

    Ok(collections)
}

async fn nft_collection_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nft_collections`
        WHERE `visible` = 1
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn visible_nft_collection_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nft_collections`
        WHERE `visible` = 1
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn nft_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts`
        WHERE `visible` = 1
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn visible_nft_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts`
        WHERE `visible` = 1
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

async fn set_nft_uri_checked(
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
    nft_data: NftData,
) -> Result<()> {
    let hash = hash.as_ref();
    let data = nft_data.blob;
    let mime_type = nft_data.mime_type;

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

async fn insert_nft(conn: impl SqliteExecutor<'_>, row: NftRow) -> Result<()> {
    let launcher_id = row.launcher_id.as_ref();
    let collection_id = row.collection_id.as_deref();
    let minter_did = row.minter_did.as_deref();
    let owner_did = row.owner_did.as_deref();
    let name = row.name.as_deref();
    let metadata_hash = row.metadata_hash.as_deref();

    sqlx::query!(
        "REPLACE INTO `nfts` (
            `launcher_id`,
            `collection_id`,
            `minter_did`,
            `owner_did`,
            `visible`,
            `sensitive_content`,
            `name`,
            `created_height`,
            `metadata_hash`
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        launcher_id,
        collection_id,
        minter_did,
        owner_did,
        row.visible,
        row.sensitive_content,
        name,
        row.created_height,
        metadata_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nft_row(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<NftRow>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            `launcher_id`,
            `collection_id`,
            `minter_did`,
            `owner_did`,
            `visible`,
            `sensitive_content`,
            `name`,
            `created_height`,
            `metadata_hash`
        FROM `nfts`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(NftRow {
        launcher_id: to_bytes32(&row.launcher_id)?,
        collection_id: row.collection_id.as_deref().map(to_bytes32).transpose()?,
        minter_did: row.minter_did.as_deref().map(to_bytes32).transpose()?,
        owner_did: row.owner_did.as_deref().map(to_bytes32).transpose()?,
        visible: row.visible,
        sensitive_content: row.sensitive_content,
        name: row.name,
        created_height: row.created_height.map(TryInto::try_into).transpose()?,
        metadata_hash: row.metadata_hash.as_deref().map(to_bytes32).transpose()?,
    }))
}

async fn delete_nft(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<()> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query!("DELETE FROM `nfts` WHERE `launcher_id` = ?", launcher_id)
        .execute(conn)
        .await?;

    Ok(())
}

async fn set_nft_visible(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
    visible: bool,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query!(
        "UPDATE `nfts` SET `visible` = ? WHERE `launcher_id` = ?",
        visible,
        launcher_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nft(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            `coin_states`.`parent_coin_id`,
            `coin_states`.`puzzle_hash`,
            `coin_states`.`amount`,
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
        FROM `nft_coins`
        INNER JOIN `coin_states` INDEXED BY `coin_height` ON `nft_coins`.`coin_id` = `coin_states`.`coin_id`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `launcher_id` = ?
        AND `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        AND `coin_states`.`transaction_id` IS NULL
        AND `transaction_spends`.`transaction_id` IS NULL
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
            current_owner: row.current_owner.as_deref().map(to_bytes32).transpose()?,
            royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
            royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        },
    }))
}

async fn nfts_by_metadata_hash(
    conn: impl SqliteExecutor<'_>,
    metadata_hash: Bytes32,
) -> Result<Vec<NftRow>> {
    let metadata_hash = metadata_hash.as_ref();

    let rows = sqlx::query!(
        "
        SELECT
            `launcher_id`,
            `collection_id`,
            `minter_did`,
            `owner_did`,
            `visible`,
            `sensitive_content`,
            `name`,
            `created_height`,
            `metadata_hash`
        FROM `nfts` INDEXED BY `nft_metadata`
        WHERE `metadata_hash` = ?
        ",
        metadata_hash
    )
    .fetch_all(conn)
    .await?;

    let mut nfts = Vec::new();

    for row in rows {
        nfts.push(NftRow {
            launcher_id: to_bytes32(&row.launcher_id)?,
            collection_id: row.collection_id.as_deref().map(to_bytes32).transpose()?,
            minter_did: row.minter_did.as_deref().map(to_bytes32).transpose()?,
            owner_did: row.owner_did.as_deref().map(to_bytes32).transpose()?,
            visible: row.visible,
            sensitive_content: row.sensitive_content,
            name: row.name,
            created_height: row.created_height.map(TryInto::try_into).transpose()?,
            metadata_hash: row.metadata_hash.as_deref().map(to_bytes32).transpose()?,
        });
    }

    Ok(nfts)
}

async fn nfts_visible_named(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `launcher_id`,
            `collection_id`,
            `minter_did`,
            `owner_did`,
            `visible`,
            `sensitive_content`,
            `name`,
            `created_height`,
            `metadata_hash`
        FROM `nfts` INDEXED BY `nft_named`
        WHERE `visible` = 1
        ORDER BY `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let mut nfts = Vec::new();

    for row in rows {
        nfts.push(NftRow {
            launcher_id: to_bytes32(&row.launcher_id)?,
            collection_id: row.collection_id.as_deref().map(to_bytes32).transpose()?,
            minter_did: row.minter_did.as_deref().map(to_bytes32).transpose()?,
            owner_did: row.owner_did.as_deref().map(to_bytes32).transpose()?,
            visible: row.visible,
            sensitive_content: row.sensitive_content,
            name: row.name,
            created_height: row.created_height.map(TryInto::try_into).transpose()?,
            metadata_hash: row.metadata_hash.as_deref().map(to_bytes32).transpose()?,
        });
    }

    Ok(nfts)
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

async fn data_hash(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "SELECT `data_hash` FROM `nft_coins` WHERE `launcher_id` = ?",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    let Some(data_hash) = row.data_hash else {
        return Ok(None);
    };

    Ok(Some(to_bytes32(&data_hash)?))
}

async fn metadata_hash(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "SELECT `metadata_hash` FROM `nft_coins` WHERE `launcher_id` = ?",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    let Some(metadata_hash) = row.metadata_hash else {
        return Ok(None);
    };

    Ok(Some(to_bytes32(&metadata_hash)?))
}

async fn license_hash(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "SELECT `license_hash` FROM `nft_coins` WHERE `launcher_id` = ?",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    let Some(license_hash) = row.license_hash else {
        return Ok(None);
    };

    Ok(Some(to_bytes32(&license_hash)?))
}
async fn nft_launcher_id(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "SELECT `launcher_id` FROM `nft_coins` WHERE `coin_id` = ?",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(to_bytes32(&row.launcher_id)?))
}
