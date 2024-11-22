use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{Database, DatabaseTx, OfferRow, Result};

impl Database {}

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
    let expiration_timestamp = row.expiration_timestamp.to_be_bytes();
    let expiration_timestamp = expiration_timestamp.as_ref();
    let status = row.status as u8;

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `offers` (
            `offer_id`, `encoded_offer`,
            `expiration_height`, `expiration_timestamp`, `status`
        )
        VALUES (?, ?, ?, ?, ?)
        ",
        offer_id,
        row.encoded_offer,
        row.expiration_height,
        expiration_timestamp,
        status
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
