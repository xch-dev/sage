use std::time::{SystemTime, UNIX_EPOCH};

use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{into_row, Database, DatabaseTx, OfferRow, OfferSql, Result};

impl Database {
    pub async fn active_offers(&self) -> Result<Vec<OfferRow>> {
        active_offers(&self.pool).await
    }

    pub async fn get_offers(&self) -> Result<Vec<OfferRow>> {
        get_offers(&self.pool).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_offer(&mut self, row: OfferRow) -> Result<()> {
        insert_offer(&mut *self.tx, row).await
    }

    pub async fn insert_offer_coin(&mut self, offer_id: Bytes32, coin_id: Bytes32) -> Result<()> {
        insert_offer_coin(&mut *self.tx, offer_id, coin_id).await
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

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `offers` (
            `offer_id`, `encoded_offer`, `expiration_height`,
            `expiration_timestamp`, `status`, `inserted_timestamp`
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        offer_id,
        row.encoded_offer,
        row.expiration_height,
        expiration_timestamp,
        status,
        timestamp
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn insert_offer_coin(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
    coin_id: Bytes32,
) -> Result<()> {
    let offer_id = offer_id.as_ref();
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "INSERT OR IGNORE INTO `offer_coins` (`offer_id`, `coin_id`) VALUES (?, ?)",
        offer_id,
        coin_id
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
