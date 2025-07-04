use crate::{Convert, Database, DatabaseTx, Result};
use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum OfferStatus {
    Pending = 0,
    Active = 1,
    Completed = 2,
    Cancelled = 3,
    Expired = 4,
}

#[derive(Debug, Clone)]
pub struct OfferRow {
    pub offer_id: Bytes32,
    pub encoded_offer: String,
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
    pub fee: u64,
    pub status: OfferStatus,
    pub inserted_timestamp: u64,
}

impl Database {
    pub async fn offer(&self, offer_id: Bytes32) -> Result<Option<OfferRow>> {
        offer(&self.pool, offer_id).await
    }

    pub async fn delete_offer(&self, offer_id: Bytes32) -> Result<()> {
        delete_offer(&self.pool, offer_id).await
    }

    pub async fn active_offers(&self) -> Result<Vec<OfferRow>> {
        active_offers(&self.pool).await
    }

    pub async fn update_offer_status(&self, offer_id: Bytes32, status: OfferStatus) -> Result<()> {
        update_offer_status(&self.pool, offer_id, status).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_offer(&mut self, offer: OfferRow) -> Result<()> {
        insert_offer(&mut *self.tx, offer).await
    }

    pub async fn update_offer_status(
        &mut self,
        offer_id: Bytes32,
        status: OfferStatus,
    ) -> Result<()> {
        update_offer_status(&mut *self.tx, offer_id, status).await
    }
}

async fn insert_offer(conn: impl SqliteExecutor<'_>, offer: OfferRow) -> Result<()> {
    let offer_id_ref = offer.offer_id.as_ref();
    sqlx::query(
        "INSERT INTO offers (hash, encoded_offer, fee, status, expiration_height, expiration_timestamp, inserted_timestamp) 
        VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(offer_id_ref)
        .bind(offer.encoded_offer)
        .bind(offer.fee as i64)
        .bind(offer.status as u8)
        .bind(offer.expiration_height.map(|h| h as i64))
        .bind(offer.expiration_timestamp.map(|t| t as i64))
        .bind(offer.inserted_timestamp as i64)
        .execute(conn)
        .await?;
    Ok(())
}

async fn offer(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<Option<OfferRow>> {
    let offer_id_ref = offer_id.as_ref();
    let row = sqlx::query!(
        "SELECT
            hash as offer_id,
            encoded_offer,
            fee,
            status,
            expiration_height,
            expiration_timestamp,
            inserted_timestamp
        FROM offers WHERE hash = ?",
        offer_id_ref
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        Ok(OfferRow {
            offer_id: row.offer_id.convert()?,
            encoded_offer: row.encoded_offer,
            expiration_height: row.expiration_height.map(|h| h as u32),
            expiration_timestamp: row.expiration_timestamp.map(|t| t as u64),
            fee: row.fee.convert()?,
            status: match row.status {
                0 => OfferStatus::Pending,
                1 => OfferStatus::Active,
                2 => OfferStatus::Completed,
                3 => OfferStatus::Cancelled,
                4 => OfferStatus::Expired,
                _ => return Err(crate::DatabaseError::InvalidEnumVariant),
            },
            inserted_timestamp: row.inserted_timestamp as u64,
        })
    })
    .transpose()
}

async fn active_offers(conn: impl SqliteExecutor<'_>) -> Result<Vec<OfferRow>> {
    let rows = sqlx::query!(
        "SELECT
            hash as offer_id,
            encoded_offer,
            fee,
            status,
            expiration_height,
            expiration_timestamp,
            inserted_timestamp
        FROM offers WHERE status = ?",
        OfferStatus::Active as u8
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(OfferRow {
                offer_id: row.offer_id.convert()?,
                encoded_offer: row.encoded_offer,
                expiration_height: row.expiration_height.map(|h| h as u32),
                expiration_timestamp: row.expiration_timestamp.map(|t| t as u64),
                fee: row.fee.convert()?,
                status: match row.status {
                    0 => OfferStatus::Pending,
                    1 => OfferStatus::Active,
                    2 => OfferStatus::Completed,
                    3 => OfferStatus::Cancelled,
                    4 => OfferStatus::Expired,
                    _ => return Err(crate::DatabaseError::InvalidEnumVariant),
                },
                inserted_timestamp: row.inserted_timestamp as u64,
            })
        })
        .collect()
}

async fn delete_offer(conn: impl SqliteExecutor<'_>, offer_id: Bytes32) -> Result<()> {
    let offer_id_ref = offer_id.as_ref();
    sqlx::query("DELETE FROM offers WHERE hash = ?")
        .bind(offer_id_ref)
        .execute(conn)
        .await?;
    Ok(())
}

async fn update_offer_status(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
    status: OfferStatus,
) -> Result<()> {
    let offer_id_bytes = offer_id.to_vec();
    sqlx::query("UPDATE offers SET status = ? WHERE hash = ?")
        .bind(status as u8)
        .bind(&offer_id_bytes)
        .execute(conn)
        .await?;
    Ok(())
}
