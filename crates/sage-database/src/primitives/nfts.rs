use chia::{
    protocol::{Bytes32, Program},
    puzzles::LineageProof,
};
use chia_wallet_sdk::{Nft, NftInfo};
use sqlx::SqliteExecutor;

use crate::{
    into_row, to_bytes32, CoinStateRow, CoinStateSql, CollectionRow, CollectionSql, Database,
    DatabaseTx, FullNftCoinSql, IntoRow, NftRow, NftSql, Result,
};

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

    pub async fn spendable_nft(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        spendable_nft(&self.pool, launcher_id).await
    }

    pub async fn fetch_nft_data(&self, hash: Bytes32) -> Result<Option<NftData>> {
        fetch_nft_data(&self.pool, hash).await
    }

    pub async fn collection(&self, collection_id: Bytes32) -> Result<CollectionRow> {
        collection(&self.pool, collection_id).await
    }

    pub async fn collection_count(&self) -> Result<u32> {
        collection_count(&self.pool).await
    }

    pub async fn visible_collection_count(&self) -> Result<u32> {
        visible_collection_count(&self.pool).await
    }

    pub async fn nfts_visible_named(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts_visible_named(&self.pool, limit, offset).await
    }

    pub async fn nfts_visible_recent(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts_visible_recent(&self.pool, limit, offset).await
    }

    pub async fn nfts_named(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts_named(&self.pool, limit, offset).await
    }

    pub async fn nfts_recent(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        nfts_recent(&self.pool, limit, offset).await
    }

    pub async fn nft_count(&self) -> Result<u32> {
        nft_count(&self.pool).await
    }

    pub async fn visible_nft_count(&self) -> Result<u32> {
        visible_nft_count(&self.pool).await
    }

    pub async fn collection_nfts_visible_named(
        &self,
        collection_id: Bytes32,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<NftRow>> {
        collection_nfts_visible_named(&self.pool, collection_id, limit, offset).await
    }

    pub async fn collection_nfts_visible_recent(
        &self,
        collection_id: Bytes32,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<NftRow>> {
        collection_nfts_visible_recent(&self.pool, collection_id, limit, offset).await
    }

    pub async fn collection_nfts_named(
        &self,
        collection_id: Bytes32,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<NftRow>> {
        collection_nfts_named(&self.pool, collection_id, limit, offset).await
    }

    pub async fn collection_nfts_recent(
        &self,
        collection_id: Bytes32,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<NftRow>> {
        collection_nfts_recent(&self.pool, collection_id, limit, offset).await
    }

    pub async fn no_collection_nfts_visible_named(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<NftRow>> {
        no_collection_nfts_visible_named(&self.pool, limit, offset).await
    }

    pub async fn no_collection_nfts_visible_recent(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<NftRow>> {
        no_collection_nfts_visible_recent(&self.pool, limit, offset).await
    }

    pub async fn no_collection_nfts_named(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        no_collection_nfts_named(&self.pool, limit, offset).await
    }

    pub async fn no_collection_nfts_recent(&self, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
        no_collection_nfts_recent(&self.pool, limit, offset).await
    }

    pub async fn collection_nft_count(&self, collection_id: Bytes32) -> Result<u32> {
        collection_nft_count(&self.pool, collection_id).await
    }

    pub async fn collection_visible_nft_count(&self, collection_id: Bytes32) -> Result<u32> {
        collection_visible_nft_count(&self.pool, collection_id).await
    }

    pub async fn no_collection_nft_count(&self) -> Result<u32> {
        no_collection_nft_count(&self.pool).await
    }

    pub async fn no_collection_visible_nft_count(&self) -> Result<u32> {
        no_collection_visible_nft_count(&self.pool).await
    }

    pub async fn collections_visible_named(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CollectionRow>> {
        collections_visible_named(&self.pool, offset, limit).await
    }

    pub async fn collections_named(&self, limit: u32, offset: u32) -> Result<Vec<CollectionRow>> {
        collections_named(&self.pool, offset, limit).await
    }

    pub async fn nft_row(&self, launcher_id: Bytes32) -> Result<Option<NftRow>> {
        nft_row(&self.pool, launcher_id).await
    }

    pub async fn nft(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft(&self.pool, launcher_id).await
    }

    pub async fn collection_name(&self, collection_id: Bytes32) -> Result<Option<String>> {
        collection_name(&self.pool, collection_id).await
    }

    pub async fn data_hash(&self, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
        data_hash(&self.pool, launcher_id).await
    }

    pub async fn metadata_hash(&self, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
        metadata_hash(&self.pool, launcher_id).await
    }

    pub async fn license_hash(&self, launcher_id: Bytes32) -> Result<Option<Bytes32>> {
        license_hash(&self.pool, launcher_id).await
    }

    pub async fn created_unspent_nft_coin_states(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_nft_coin_states(&self.pool, limit, offset).await
    }

    pub async fn created_unspent_nft_coin_state(
        &self,
        launcher_id: Bytes32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_nft_coin_state(&self.pool, launcher_id).await
    }

    pub async fn nft_by_coin_id(&self, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_by_coin_id(&self.pool, coin_id).await
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

    pub async fn nft_row_by_coin(&mut self, coin_id: Bytes32) -> Result<Option<NftRow>> {
        nft_row_by_coin(&mut *self.tx, coin_id).await
    }

    pub async fn nfts_by_metadata_hash(&mut self, metadata_hash: Bytes32) -> Result<Vec<NftRow>> {
        nfts_by_metadata_hash(&mut *self.tx, metadata_hash).await
    }

    pub async fn nft(&mut self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft(&mut *self.tx, launcher_id).await
    }

    pub async fn insert_collection(&mut self, row: CollectionRow) -> Result<()> {
        insert_collection(&mut *self.tx, row).await
    }
}

async fn insert_collection(conn: impl SqliteExecutor<'_>, row: CollectionRow) -> Result<()> {
    let collection_id = row.collection_id.as_ref();
    let did_id = row.did_id.as_ref();
    let name = row.name.as_deref();
    let icon = row.icon.as_deref();

    sqlx::query!(
        "
        REPLACE INTO `collections` (
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

async fn collection_name(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
) -> Result<Option<String>> {
    let collection_id = collection_id.as_ref();

    let row = sqlx::query!(
        "SELECT `name` FROM `collections` WHERE `collection_id` = ?",
        collection_id
    )
    .fetch_one(conn)
    .await?;

    Ok(row.name)
}

async fn collection(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
) -> Result<CollectionRow> {
    let collection_id = collection_id.as_ref();

    sqlx::query_as!(
        CollectionSql,
        "
        SELECT `collection_id`, `did_id`, `metadata_collection_id`, `visible`, `name`, `icon`
        FROM `collections` WHERE `collection_id` = ?
        ",
        collection_id
    )
    .fetch_one(conn)
    .await?
    .into_row()
}

async fn collections_visible_named(
    conn: impl SqliteExecutor<'_>,
    offset: u32,
    limit: u32,
) -> Result<Vec<CollectionRow>> {
    sqlx::query_as!(
        CollectionSql,
        "
        SELECT `collection_id`, `did_id`, `metadata_collection_id`, `visible`, `name`, `icon`
        FROM `collections` INDEXED BY `col_name`
        WHERE `visible` = 1
        ORDER BY `is_named` DESC, `name` ASC, `collection_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn collections_named(
    conn: impl SqliteExecutor<'_>,
    offset: u32,
    limit: u32,
) -> Result<Vec<CollectionRow>> {
    sqlx::query_as!(
        CollectionSql,
        "
        SELECT `collection_id`, `did_id`, `metadata_collection_id`, `visible`, `name`, `icon`
        FROM `collections` INDEXED BY `col_name`
        ORDER BY `visible` DESC, `is_named` DESC, `name` ASC, `collection_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn collection_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `collections`
        WHERE `visible` = 1
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn visible_collection_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `collections`
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
        WHERE `is_owned` = 1 AND `visible` = 1
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
        WHERE `is_owned` = 1 AND `visible` = 1
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn collection_nft_count(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
) -> Result<u32> {
    let collection_id = collection_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` = ? AND `visible` = 1
        ",
        collection_id
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn collection_visible_nft_count(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
) -> Result<u32> {
    let collection_id = collection_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` = ? AND `visible` = 1
        ",
        collection_id
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn no_collection_nft_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` IS NULL AND `visible` = 1
        "
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn no_collection_visible_nft_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` IS NULL AND `visible` = 1
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
    let coin_id = row.coin_id.as_ref();
    let collection_id = row.collection_id.as_deref();
    let minter_did = row.minter_did.as_deref();
    let owner_did = row.owner_did.as_deref();
    let name = row.name.as_deref();
    let metadata_hash = row.metadata_hash.as_deref();

    sqlx::query!(
        "REPLACE INTO `nfts` (
            `launcher_id`,
            `coin_id`,
            `collection_id`,
            `minter_did`,
            `owner_did`,
            `visible`,
            `sensitive_content`,
            `name`,
            `is_owned`,
            `created_height`,
            `metadata_hash`
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        launcher_id,
        coin_id,
        collection_id,
        minter_did,
        owner_did,
        row.visible,
        row.sensitive_content,
        name,
        row.is_owned,
        row.created_height,
        metadata_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nft_row(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<NftRow>> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    .map(into_row)
    .transpose()
}

async fn nft_row_by_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<NftRow>> {
    let coin_id = coin_id.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    .map(into_row)
    .transpose()
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

async fn spendable_nft(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Nft<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        FullNftCoinSql,
        "
        SELECT
            `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`,
            `launcher_id`, `metadata`, `metadata_updater_puzzle_hash`, `current_owner`,
            `royalty_puzzle_hash`, `royalty_ten_thousandths`, `p2_puzzle_hash`
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

    Ok(Some(sql.into_row()?))
}

async fn nft(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        FullNftCoinSql,
        "
        SELECT
            `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`,
            `launcher_id`, `metadata`, `metadata_updater_puzzle_hash`, `current_owner`,
            `royalty_puzzle_hash`, `royalty_ten_thousandths`, `p2_puzzle_hash`
        FROM `nft_coins`
        INNER JOIN `coin_states` INDEXED BY `coin_height` ON `nft_coins`.`coin_id` = `coin_states`.`coin_id`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `launcher_id` = ?
        AND `spent_height` IS NULL
        AND `transaction_spends`.`transaction_id` IS NULL
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(sql.into_row()?))
}

async fn nfts_by_metadata_hash(
    conn: impl SqliteExecutor<'_>,
    metadata_hash: Bytes32,
) -> Result<Vec<NftRow>> {
    let metadata_hash = metadata_hash.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_metadata`
        WHERE `metadata_hash` = ?
        ",
        metadata_hash
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn nfts_visible_named(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_name`
        WHERE `is_owned` = 1 AND `visible` = 1
        ORDER BY `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn nfts_visible_recent(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_recent`
        WHERE `is_owned` = 1 AND `visible` = 1
        ORDER BY `is_pending` DESC, `created_height` DESC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn nfts_named(conn: impl SqliteExecutor<'_>, limit: u32, offset: u32) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_name`
        WHERE `is_owned` = 1
        ORDER BY `visible` DESC, `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn nfts_recent(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_recent`
        WHERE `is_owned` = 1
        ORDER BY `visible` DESC, `is_pending` DESC, `created_height` DESC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn collection_nfts_visible_named(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    let collection_id = collection_id.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_name`
        WHERE `is_owned` = 1 AND `collection_id` = ? AND `visible` = 1
        ORDER BY `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        collection_id,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn collection_nfts_visible_recent(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    let collection_id = collection_id.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` = ? AND `visible` = 1
        ORDER BY `is_pending` DESC, `created_height` DESC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        collection_id,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn collection_nfts_named(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    let collection_id = collection_id.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_name`
        WHERE `is_owned` = 1 AND `collection_id` = ?
        ORDER BY `visible` DESC, `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        collection_id,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn collection_nfts_recent(
    conn: impl SqliteExecutor<'_>,
    collection_id: Bytes32,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    let collection_id = collection_id.as_ref();

    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` = ?
        ORDER BY `visible` DESC, `is_pending` DESC, `created_height` DESC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        collection_id,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn no_collection_nfts_visible_named(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_name`
        WHERE `is_owned` = 1 AND `collection_id` IS NULL AND `visible` = 1
        ORDER BY `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn no_collection_nfts_visible_recent(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` IS NULL AND `visible` = 1
        ORDER BY `is_pending` DESC, `created_height` DESC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn no_collection_nfts_named(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_name`
        WHERE `is_owned` = 1 AND `collection_id` IS NULL
        ORDER BY `visible` DESC, `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn no_collection_nfts_recent(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<NftRow>> {
    sqlx::query_as!(
        NftSql,
        "
        SELECT * FROM `nfts` INDEXED BY `nft_col_recent`
        WHERE `is_owned` = 1 AND `collection_id` IS NULL
        ORDER BY `visible` DESC, `is_pending` DESC, `created_height` DESC, `launcher_id` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
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
        INSERT OR IGNORE INTO `nft_coins` (
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

async fn created_unspent_nft_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`
        FROM `coin_states`
        INNER JOIN `nft_coins` ON `coin_states`.coin_id = `nft_coins`.coin_id
        WHERE `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        ORDER BY `created_height`, `coin_states`.`coin_id` LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn created_unspent_nft_coin_state(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Vec<CoinStateRow>> {
    let launcher_id = launcher_id.as_ref();

    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`
        FROM `coin_states`
        INNER JOIN `nft_coins` ON `coin_states`.coin_id = `nft_coins`.coin_id
        WHERE `launcher_id` = ?
        AND `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        ",
        launcher_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn nft_by_coin_id(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<Nft<Program>>> {
    let coin_id = coin_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        FullNftCoinSql,
        "
        SELECT
            `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`,
            `launcher_id`, `metadata`, `metadata_updater_puzzle_hash`, `current_owner`,
            `royalty_puzzle_hash`, `royalty_ten_thousandths`, `p2_puzzle_hash`
        FROM `nft_coins`
        INNER JOIN `coin_states` INDEXED BY `coin_height` ON `nft_coins`.`coin_id` = `coin_states`.`coin_id`
        WHERE `coin_states`.`coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(sql.into_row()?))
}
