use crate::{Asset, AssetKind, Convert, Database, DatabaseTx, Result};
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

#[derive(Debug, Clone)]
pub struct OfferedAsset {
    pub offer_id: Bytes32,
    pub asset: Asset,
    pub is_requested: bool,
    pub amount: u64,
    pub royalty: u64,
}

impl Database {
    pub async fn offer(&self, offer_id: Bytes32) -> Result<Option<OfferRow>> {
        offer(&self.pool, offer_id).await
    }

    pub async fn offer_xch_assets(&self, offer_id: Bytes32) -> Result<Vec<OfferedAsset>> {
        offer_xch_assets(&self.pool, offer_id).await
    }

    pub async fn offer_cat_assets(&self, offer_id: Bytes32) -> Result<Vec<OfferedAsset>> {
        offer_assets(&self.pool, offer_id, AssetKind::Token).await
    }

    pub async fn offer_nft_assets(&self, offer_id: Bytes32) -> Result<Vec<OfferedAsset>> {
        offer_assets(&self.pool, offer_id, AssetKind::Nft).await
    }

    pub async fn delete_offer(&self, offer_id: Bytes32) -> Result<()> {
        delete_offer(&self.pool, offer_id).await
    }

    pub async fn offers(&self, status: Option<OfferStatus>) -> Result<Vec<OfferRow>> {
        offers(&self.pool, status).await
    }

    pub async fn update_offer_status(&self, offer_id: Bytes32, status: OfferStatus) -> Result<()> {
        update_offer_status(&self.pool, offer_id, status).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_offer(&mut self, offer: OfferRow) -> Result<()> {
        insert_offer(&mut *self.tx, offer).await
    }

    pub async fn insert_offered_coin(&mut self, offer_id: Bytes32, coin_id: Bytes32) -> Result<()> {
        insert_offered_coin(&mut *self.tx, offer_id, coin_id).await
    }

    pub async fn insert_offer_xch(
        &mut self,
        offer_id: Bytes32,
        xch: u64,
        royalty: u64,
        is_requested: bool,
    ) -> Result<()> {
        insert_offer_xch(&mut *self.tx, offer_id, xch, royalty, is_requested).await
    }

    pub async fn insert_offer_asset(
        &mut self,
        offer_id: Bytes32,
        asset_id: Bytes32,
        amount: u64,
        royalty: u64,
        is_requested: bool,
    ) -> Result<()> {
        insert_offer_asset(
            &mut *self.tx,
            offer_id,
            asset_id,
            amount,
            royalty,
            is_requested,
        )
        .await
    }

    pub async fn update_offer_status(
        &mut self,
        offer_id: Bytes32,
        status: OfferStatus,
    ) -> Result<()> {
        update_offer_status(&mut *self.tx, offer_id, status).await
    }
}

async fn offer_assets(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
    kind: AssetKind,
) -> Result<Vec<OfferedAsset>> {
    let offer_id_ref = offer_id.as_ref();
    let kind_u8 = kind as u8;
    let rows = sqlx::query!(
        "
        SELECT
            offers.hash as offer_id, assets.hash as asset_id,
            amount, royalty, is_requested, 
            assets.description, assets.is_sensitive_content,
            assets.is_visible, assets.icon_url, assets.name
        FROM offer_assets 
        INNER JOIN assets ON offer_assets.asset_id = assets.id
        INNER JOIN offers ON offer_assets.offer_id = offers.id
        WHERE offers.hash = ? AND kind = ?
        ",
        offer_id_ref,
        kind_u8
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(OfferedAsset {
                offer_id: row.offer_id.convert()?,
                asset: Asset {
                    hash: row.asset_id.convert()?,
                    description: row.description,
                    is_sensitive_content: row.is_sensitive_content,
                    is_visible: row.is_visible,
                    icon_url: row.icon_url,
                    kind,
                    name: row.name,
                },
                amount: row.amount.convert()?,
                royalty: row.royalty.convert()?,
                is_requested: row.is_requested,
            })
        })
        .collect()
}

async fn offer_xch_assets(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
) -> Result<Vec<OfferedAsset>> {
    let offer_id_ref = offer_id.as_ref();
    let rows = sqlx::query!(
        "
        SELECT
            offers.hash as offer_id, assets.hash as asset_id,
            amount, royalty, is_requested, 
            assets.description, assets.is_sensitive_content,
            assets.is_visible, assets.icon_url, assets.name
        FROM offer_assets 
        INNER JOIN assets ON offer_assets.asset_id = assets.id
        INNER JOIN offers ON offer_assets.offer_id = offers.id
        WHERE asset_id = 0 AND offer_id = ?
        ",
        offer_id_ref,
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(OfferedAsset {
                offer_id: row.offer_id.convert()?,
                asset: Asset {
                    hash: row.asset_id.convert()?,
                    description: row.description,
                    is_sensitive_content: row.is_sensitive_content,
                    is_visible: row.is_visible,
                    icon_url: row.icon_url,
                    kind: AssetKind::Token,
                    name: row.name,
                },
                amount: row.amount.convert()?,
                royalty: row.royalty.convert()?,
                is_requested: row.is_requested,
            })
        })
        .collect()
}

async fn insert_offer(conn: impl SqliteExecutor<'_>, offer: OfferRow) -> Result<()> {
    let offer_id_ref = offer.offer_id.as_ref();

    let expiration_height: Option<i64> = offer.expiration_height.map(Into::into);
    let expiration_timestamp: Option<i64> = offer
        .expiration_timestamp
        .map(TryInto::try_into)
        .transpose()?;
    let inserted_timestamp: i64 = offer.inserted_timestamp.try_into()?;
    let fee = offer.fee.to_be_bytes().to_vec();

    sqlx::query(
        "
        INSERT OR IGNORE INTO offers (
            hash, encoded_offer, fee, status,
            expiration_height, expiration_timestamp, inserted_timestamp
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
    )
    .bind(offer_id_ref)
    .bind(offer.encoded_offer)
    .bind(fee)
    .bind(offer.status as u8)
    .bind(expiration_height)
    .bind(expiration_timestamp)
    .bind(inserted_timestamp)
    .execute(conn)
    .await?;
    Ok(())
}

async fn insert_offer_asset(
    conn: impl SqliteExecutor<'_>,
    offer_id: Bytes32,
    asset_id: Bytes32,
    amount: u64,
    royalty: u64,
    is_requested: bool,
) -> Result<()> {
    let offer_id_ref = offer_id.as_ref();
    let asset_id_ref = asset_id.as_ref();

    let amount = amount.to_be_bytes().to_vec();
    let royalty = royalty.to_be_bytes().to_vec();

    sqlx::query(
        "
        INSERT OR IGNORE INTO offer_assets (offer_id, asset_id, amount, royalty, is_requested) 
        VALUES (
            (SELECT id FROM offers WHERE hash = ?), 
            (SELECT id FROM assets WHERE hash = ?), 
            ?, ?, ?
        )
        ",
    )
    .bind(offer_id_ref)
    .bind(asset_id_ref)
    .bind(amount)
    .bind(royalty)
    .bind(is_requested)
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_offer_xch(
    conn: impl SqliteExecutor<'_>,
    offer_hash: Bytes32,
    xch: u64,
    royalty: u64,
    is_requested: bool,
) -> Result<()> {
    let offer_id_ref = offer_hash.as_ref();

    let xch = xch.to_be_bytes().to_vec();
    let royalty = royalty.to_be_bytes().to_vec();

    sqlx::query(
        "
        INSERT OR IGNORE INTO offer_assets (offer_id, asset_id, amount, royalty, is_requested) 
        VALUES (
            (SELECT id FROM offers WHERE hash = ?),
            0, ?, ?, ?
        )
        ",
    )
    .bind(offer_id_ref)
    .bind(xch)
    .bind(royalty)
    .bind(is_requested)
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_offered_coin(
    conn: impl SqliteExecutor<'_>,
    offer_hash: Bytes32,
    coin_hash: Bytes32,
) -> Result<()> {
    let offer_id_ref = offer_hash.as_ref();
    let coin_hash_ref = coin_hash.as_ref();
    sqlx::query(
        "INSERT OR IGNORE INTO offer_coins (offer_id, coin_id) 
        VALUES ((SELECT id FROM offers WHERE hash = ?), (SELECT id FROM coins WHERE hash = ?))",
    )
    .bind(offer_id_ref)
    .bind(coin_hash_ref)
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

async fn offers(
    conn: impl SqliteExecutor<'_>,
    status: Option<OfferStatus>,
) -> Result<Vec<OfferRow>> {
    let status_value = status.map(|s| s as u8);
    let rows = sqlx::query!(
        "SELECT
            hash as offer_id,
            encoded_offer,
            fee,
            status,
            expiration_height,
            expiration_timestamp,
            inserted_timestamp
        FROM offers 
        WHERE status = ? OR ? IS NULL",
        status_value,
        status_value
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
