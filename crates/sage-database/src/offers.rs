use std::time::{SystemTime, UNIX_EPOCH};

use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{
    into_row, to_bytes32, Database, DatabaseTx, OfferCatRow, OfferCatSql, OfferNftRow, OfferNftSql,
    OfferRow, OfferSql, OfferStatus, OfferXchRow, OfferXchSql, Result,
};

impl Database {
    pub async fn active_offers(&self) -> Result<Vec<OfferRow>> {
        active_offers(&self.pool).await
    }

    pub async fn get_offers(&self) -> Result<Vec<OfferRow>> {
        get_offers(&self.pool).await
    }

    pub async fn delete_offer(&self, offer_id: Bytes32) -> Result<()> {
        delete_offer(&self.pool, offer_id).await
    }

    pub async fn offer_xch(&self, offer_id: Bytes32) -> Result<Vec<OfferXchRow>> {
        offer_xch(&self.pool, offer_id).await
    }

    pub async fn offer_nfts(&self, offer_id: Bytes32) -> Result<Vec<OfferNftRow>> {
        offer_nfts(&self.pool, offer_id).await
    }

    pub async fn offer_cats(&self, offer_id: Bytes32) -> Result<Vec<OfferCatRow>> {
        offer_cats(&self.pool, offer_id).await
    }

    pub async fn get_offer(&self, offer_id: Bytes32) -> Result<Option<OfferRow>> {
        get_offer(&self.pool, offer_id).await
    }

    pub async fn update_offer_status(&self, offer_id: Bytes32, status: OfferStatus) -> Result<()> {
        update_offer_status(&self.pool, offer_id, status).await
    }

    pub async fn offer_coin_ids(&self, offer_id: Bytes32) -> Result<Vec<Bytes32>> {
        offer_coin_ids(&self.pool, offer_id).await
    }

    pub async fn coin_offer_id(&self, coin_id: Bytes32) -> Result<Option<Bytes32>> {
        coin_offer_id(&self.pool, coin_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_offer(&mut self, row: OfferRow) -> Result<()> {
        insert_offer(&mut *self.tx, row).await
    }

    pub async fn insert_offered_coin(&mut self, offer_id: Bytes32, coin_id: Bytes32) -> Result<()> {
        insert_offered_coin(&mut *self.tx, offer_id, coin_id).await
    }

    pub async fn insert_offer_xch(&mut self, row: OfferXchRow) -> Result<()> {
        insert_offer_xch(&mut *self.tx, row).await
    }

    pub async fn insert_offer_nft(&mut self, row: OfferNftRow) -> Result<()> {
        insert_offer_nft(&mut *self.tx, row).await
    }

    pub async fn insert_offer_cat(&mut self, row: OfferCatRow) -> Result<()> {
        insert_offer_cat(&mut *self.tx, row).await
    }
}

async fn insert_offer(conn: impl SqliteExecutor<'_>, row: OfferRow) -> Result<()> {
    let offer_id = row.offer_id.as_ref();
    let expiration_timestamp = row.expiration_timestamp.map(|ts| ts.to_be_bytes().to_vec());
    let status = row.status as u8;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before the UNIX epoch")
        .as_secs()
        .to_be_bytes();
    let timestamp = timestamp.as_ref();

    let fee = row.fee.to_be_bytes();
    let fee = fee.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `offers` (
            `offer_id`, `encoded_offer`, `expiration_height`,
            `expiration_timestamp`, `fee`, `status`, `inserted_timestamp`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        offer_id,
        row.encoded_offer,
        row.expiration_height,
        expiration_timestamp,
        fee,
        status,
        timestamp
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn insert_offered_coin(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
    coin_id: Bytes32,
) -> Result<()> {
    let offer_id = offer_id.as_ref();
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "INSERT OR IGNORE INTO `offered_coins` (`offer_id`, `coin_id`) VALUES (?, ?)",
        offer_id,
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_offer_xch(conn: impl SqliteExecutor<'_>, row: OfferXchRow) -> Result<()> {
    let offer_id = row.offer_id.as_ref();
    let amount = row.amount.to_be_bytes();
    let amount = amount.as_ref();
    let royalty = row.royalty.to_be_bytes();
    let royalty = royalty.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `offer_xch` (
            `offer_id`, `requested`, `amount`, `royalty`
        )
        VALUES (?, ?, ?, ?)
        ",
        offer_id,
        row.requested,
        amount,
        royalty
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_offer_nft(conn: impl SqliteExecutor<'_>, row: OfferNftRow) -> Result<()> {
    let offer_id = row.offer_id.as_ref();
    let launcher_id = row.launcher_id.as_ref();
    let royalty_puzzle_hash = row.royalty_puzzle_hash.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `offer_nfts` (
            `offer_id`, `requested`, `launcher_id`,
            `royalty_puzzle_hash`, `royalty_ten_thousandths`,
            `name`, `thumbnail`, `thumbnail_mime_type`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        offer_id,
        row.requested,
        launcher_id,
        royalty_puzzle_hash,
        row.royalty_ten_thousandths,
        row.name,
        row.thumbnail,
        row.thumbnail_mime_type
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_offer_cat(conn: impl SqliteExecutor<'_>, row: OfferCatRow) -> Result<()> {
    let offer_id = row.offer_id.as_ref();
    let asset_id = row.asset_id.as_ref();
    let amount = row.amount.to_be_bytes();
    let amount = amount.as_ref();
    let royalty = row.royalty.to_be_bytes();
    let royalty = royalty.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `offer_cats` (
            `offer_id`, `requested`, `asset_id`,
            `amount`, `royalty`, `name`, `ticker`, `icon`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        offer_id,
        row.requested,
        asset_id,
        amount,
        royalty,
        row.name,
        row.ticker,
        row.icon
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn active_offers(conn: impl SqliteExecutor<'_>) -> Result<Vec<OfferRow>> {
    sqlx::query_as!(
        OfferSql,
        "SELECT * FROM `offers` INDEXED BY `offer_status` WHERE `status` = 0"
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn get_offers(conn: impl SqliteExecutor<'_>) -> Result<Vec<OfferRow>> {
    sqlx::query_as!(
        OfferSql,
        "SELECT * FROM `offers` INDEXED BY `offer_timestamp` ORDER BY `inserted_timestamp` DESC"
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn get_offer(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<Option<OfferRow>> {
    let offer_id = offer_id.as_ref();

    sqlx::query_as!(
        OfferSql,
        "SELECT * FROM `offers` WHERE `offer_id` = ?",
        offer_id
    )
    .fetch_optional(conn)
    .await?
    .map(into_row)
    .transpose()
}

async fn delete_offer(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<()> {
    let offer_id = offer_id.as_ref();

    sqlx::query!("DELETE FROM `offers` WHERE `offer_id` = ?", offer_id)
        .execute(conn)
        .await?;

    Ok(())
}

async fn offer_xch(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<Vec<OfferXchRow>> {
    let offer_id = offer_id.as_ref();

    sqlx::query_as!(
        OfferXchSql,
        "SELECT * FROM `offer_xch` WHERE `offer_id` = ?",
        offer_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn offer_nfts(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<Vec<OfferNftRow>> {
    let offer_id = offer_id.as_ref();

    sqlx::query_as!(
        OfferNftSql,
        "SELECT * FROM `offer_nfts` WHERE `offer_id` = ?",
        offer_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn offer_cats(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<Vec<OfferCatRow>> {
    let offer_id = offer_id.as_ref();

    sqlx::query_as!(
        OfferCatSql,
        "SELECT * FROM `offer_cats` WHERE `offer_id` = ?",
        offer_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn update_offer_status(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
    status: OfferStatus,
) -> Result<()> {
    let offer_id = offer_id.as_ref();
    let status = status as u8;

    sqlx::query!(
        "UPDATE `offers` SET `status` = ? WHERE `offer_id` = ?",
        status,
        offer_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn offer_coin_ids(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<Vec<Bytes32>> {
    let offer_id = offer_id.as_ref();

    sqlx::query!(
        "SELECT `coin_id` FROM `offered_coins` WHERE `offer_id` = ?",
        offer_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| to_bytes32(&row.coin_id))
    .collect()
}

async fn coin_offer_id(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Bytes32>> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        SELECT `offer_id`
        FROM `offered_coins`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| to_bytes32(&row.offer_id))
    .transpose()
}
