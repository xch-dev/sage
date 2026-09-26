use crate::{Asset, Convert, Database, DatabaseTx, Result};
use chia_wallet_sdk::prelude::*;
use sqlx::{QueryBuilder, Row, Sqlite, SqliteExecutor};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OfferSearchSide {
    #[default]
    Any,
    Offered,
    Requested,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OfferSortColumn {
    #[default]
    Created,
    Expiration,
}

#[derive(Debug, Clone, Default)]
pub struct OffersPageParams {
    pub status: Option<OfferStatus>,
    pub find_text: Option<String>,
    pub find_id: Option<Bytes32>,
    pub side: OfferSearchSide,
    pub sort: OfferSortColumn,
    pub ascending: bool,
    pub limit: Option<u32>,
    pub offset: u32,
}

impl Database {
    pub async fn offer(&self, offer_id: Bytes32) -> Result<Option<OfferRow>> {
        offer(&self.pool, offer_id).await
    }

    pub async fn offer_assets(&self, offer_id: Bytes32) -> Result<Vec<OfferedAsset>> {
        offer_assets(&self.pool, offer_id).await
    }

    pub async fn delete_offer(&self, offer_id: Bytes32) -> Result<()> {
        delete_offer(&self.pool, offer_id).await
    }

    pub async fn offers(&self, status: Option<OfferStatus>) -> Result<Vec<OfferRow>> {
        offers(&self.pool, status).await
    }

    pub async fn offers_page(&self, params: OffersPageParams) -> Result<(Vec<OfferRow>, u32)> {
        offers_page(&self.pool, params).await
    }

    pub async fn update_offer_status(&self, offer_id: Bytes32, status: OfferStatus) -> Result<()> {
        update_offer_status(&self.pool, offer_id, status).await
    }

    pub async fn offers_for_asset(
        &self,
        asset_id: Bytes32,
        status: Option<OfferStatus>,
    ) -> Result<Vec<OfferRow>> {
        offers_for_asset(&self.pool, asset_id, status).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_offer(&mut self, offer: OfferRow) -> Result<()> {
        insert_offer(&mut *self.tx, offer).await
    }

    pub async fn insert_offered_coin(&mut self, offer_id: Bytes32, coin_id: Bytes32) -> Result<()> {
        insert_offered_coin(&mut *self.tx, offer_id, coin_id).await
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

    pub async fn offers_for_asset(
        &mut self,
        asset_id: Bytes32,
        status: Option<OfferStatus>,
    ) -> Result<Vec<OfferRow>> {
        offers_for_asset(&mut *self.tx, asset_id, status).await
    }
}

async fn offers_for_asset(
    conn: impl SqliteExecutor<'_>,
    asset_id: Bytes32,
    status: Option<OfferStatus>,
) -> Result<Vec<OfferRow>> {
    let status_value = status.map(|s| s as u8);
    let asset_id_ref = asset_id.as_ref();

    let rows = sqlx::query!(
        "SELECT
            offers.hash as offer_id,
            encoded_offer,
            fee,
            status,
            expiration_height,
            expiration_timestamp,
            inserted_timestamp
        FROM offers
        INNER JOIN offer_assets ON offers.id = offer_assets.offer_id
        INNER JOIN assets ON offer_assets.asset_id = assets.id
        WHERE assets.hash = ? AND offers.status = ? OR ? IS NULL
        ORDER BY inserted_timestamp DESC",
        asset_id_ref,
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

async fn offer_assets(
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
            assets.is_visible, assets.icon_url, assets.name,
            assets.ticker, assets.precision, assets.kind,
            assets.hidden_puzzle_hash
        FROM offer_assets 
        INNER JOIN assets ON offer_assets.asset_id = assets.id
        INNER JOIN offers ON offer_assets.offer_id = offers.id
        WHERE offers.hash = ?
        ",
        offer_id_ref
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
                    kind: row.kind.convert()?,
                    name: row.name,
                    ticker: row.ticker,
                    precision: row.precision.convert()?,
                    hidden_puzzle_hash: row.hidden_puzzle_hash.convert()?,
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
        WHERE status = ? OR ? IS NULL
        ORDER BY inserted_timestamp DESC",
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

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn offer_status(value: i64) -> Result<OfferStatus> {
    Ok(match value {
        0 => OfferStatus::Pending,
        1 => OfferStatus::Active,
        2 => OfferStatus::Completed,
        3 => OfferStatus::Cancelled,
        4 => OfferStatus::Expired,
        _ => return Err(crate::DatabaseError::InvalidEnumVariant),
    })
}

async fn offers_page(
    conn: impl SqliteExecutor<'_>,
    params: OffersPageParams,
) -> Result<(Vec<OfferRow>, u32)> {
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT
            hash AS offer_id, encoded_offer, fee, status, expiration_height,
            expiration_timestamp, inserted_timestamp, COUNT(*) OVER() AS total_count
        FROM offers
        WHERE 1=1",
    );

    if let Some(status) = params.status {
        query.push(" AND status = ");
        query.push_bind(status as u8);
    }

    let find_text = params
        .find_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if find_text.is_some() || params.find_id.is_some() {
        query.push(" AND (");

        if let Some(find_id) = params.find_id {
            query.push("offers.hash = ");
            query.push_bind(find_id.to_vec());
            query.push(" OR ");
        }

        // Driving from offer_assets lets exact matches use the unique hash indexes.
        query.push(
            "offers.id IN (
                SELECT oa.offer_id FROM offer_assets oa
                INNER JOIN assets a ON a.id = oa.asset_id
                WHERE ",
        );

        match params.side {
            OfferSearchSide::Any => {}
            OfferSearchSide::Offered => {
                query.push("oa.is_requested = 0 AND ");
            }
            OfferSearchSide::Requested => {
                query.push("oa.is_requested = 1 AND ");
            }
        }

        query.push("(0");

        if let Some(find_text) = find_text {
            let pattern = format!("%{}%", escape_like(find_text));
            query.push(" OR a.name LIKE ");
            query.push_bind(pattern.clone());
            query.push(" ESCAPE '\\' OR a.ticker LIKE ");
            query.push_bind(pattern);
            query.push(" ESCAPE '\\'");
        }

        if let Some(find_id) = params.find_id {
            query.push(" OR a.hash = ");
            query.push_bind(find_id.to_vec());
        }

        query.push(")))");
    }

    let direction = if params.ascending { "ASC" } else { "DESC" };

    match params.sort {
        OfferSortColumn::Created => {
            query.push(format!(
                " ORDER BY inserted_timestamp {direction}, offers.id {direction}"
            ));
        }
        OfferSortColumn::Expiration => {
            query.push(format!(
                " ORDER BY expiration_timestamp IS NULL, expiration_timestamp {direction},
                  inserted_timestamp DESC, offers.id DESC"
            ));
        }
    }

    if params.limit.is_some() || params.offset > 0 {
        // SQLite requires LIMIT before OFFSET; -1 means unlimited.
        query.push(" LIMIT ");
        query.push_bind(params.limit.map_or(-1, i64::from));
        query.push(" OFFSET ");
        query.push_bind(params.offset);
    }

    let rows = query.build().fetch_all(conn).await?;

    let total = rows
        .first()
        .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;

    let offers = rows
        .into_iter()
        .map(|row| {
            Ok(OfferRow {
                offer_id: row.get::<Vec<u8>, _>("offer_id").convert()?,
                encoded_offer: row.get("encoded_offer"),
                expiration_height: row
                    .get::<Option<i64>, _>("expiration_height")
                    .map(|h| h as u32),
                expiration_timestamp: row
                    .get::<Option<i64>, _>("expiration_timestamp")
                    .map(|t| t as u64),
                fee: row.get::<Vec<u8>, _>("fee").convert()?,
                status: offer_status(row.get("status"))?,
                inserted_timestamp: row.get::<i64, _>("inserted_timestamp") as u64,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((offers, total))
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    const SPACE: Bytes32 = Bytes32::new([0xa1; 32]);
    const MARMOT: Bytes32 = Bytes32::new([0xb2; 32]);
    const PURE: Bytes32 = Bytes32::new([0xc3; 32]);

    async fn setup() -> Database {
        // One connection so every query sees the same in-memory database.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
        let db = Database::new(pool);

        for (hash, name, ticker) in [
            (SPACE, "Spacebucks", "SBX"),
            (MARMOT, "Marmot Coin", "MRMT"),
            (PURE, "100%_Pure", "PURE"),
        ] {
            sqlx::query(
                "INSERT INTO assets (hash, kind, name, ticker, precision, is_visible)
                 VALUES (?, 0, ?, ?, 3, 1)",
            )
            .bind(hash.to_vec())
            .bind(name)
            .bind(ticker)
            .execute(&db.pool)
            .await
            .unwrap();
        }

        // id, status, inserted, expiration, offered, requested
        for (id, status, inserted, expiration, offered, requested) in [
            (1, OfferStatus::Active, 100, Some(5000), SPACE, MARMOT),
            (2, OfferStatus::Completed, 200, None, MARMOT, PURE),
            (3, OfferStatus::Active, 300, Some(4000), PURE, SPACE),
            (4, OfferStatus::Cancelled, 300, Some(6000), MARMOT, SPACE),
        ] {
            insert_test_offer(&db, id, status, inserted, expiration, offered, requested).await;
        }

        db
    }

    async fn insert_test_offer(
        db: &Database,
        id: u8,
        status: OfferStatus,
        inserted: u64,
        expiration: Option<u64>,
        offered: Bytes32,
        requested: Bytes32,
    ) {
        let offer_id = Bytes32::new([id; 32]);
        let mut tx = db.tx().await.unwrap();
        tx.insert_offer(OfferRow {
            offer_id,
            encoded_offer: format!("offer{id}"),
            expiration_height: None,
            expiration_timestamp: expiration,
            fee: 0,
            status,
            inserted_timestamp: inserted,
        })
        .await
        .unwrap();
        tx.insert_offer_asset(offer_id, offered, 1, 0, false).await.unwrap();
        tx.insert_offer_asset(offer_id, requested, 1, 0, true).await.unwrap();
        tx.commit().await.unwrap();
    }

    async fn ids(db: &Database, params: OffersPageParams) -> (Vec<u8>, u32) {
        let (rows, total) = db.offers_page(params).await.unwrap();
        (rows.iter().map(|row| row.offer_id[0]).collect(), total)
    }

    fn text(value: &str) -> OffersPageParams {
        OffersPageParams {
            find_text: Some(value.to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn defaults_return_all_newest_first() {
        let db = setup().await;
        // 4 and 3 share inserted_timestamp 300; row id breaks the tie.
        assert_eq!(ids(&db, OffersPageParams::default()).await, (vec![4, 3, 2, 1], 4));
    }

    #[tokio::test]
    async fn filters_by_status() {
        let db = setup().await;
        let params = OffersPageParams {
            status: Some(OfferStatus::Active),
            ..Default::default()
        };
        assert_eq!(ids(&db, params).await, (vec![3, 1], 2));
    }

    #[tokio::test]
    async fn searches_name_and_ticker_case_insensitively() {
        let db = setup().await;
        // SPACE is offered in 1 and requested in 3 and 4.
        assert_eq!(ids(&db, text("space")).await, (vec![4, 3, 1], 3));
        assert_eq!(ids(&db, text("mrmt")).await, (vec![4, 2, 1], 3));
        assert_eq!(ids(&db, text("  space  ")).await, (vec![4, 3, 1], 3));
        assert_eq!(ids(&db, text("   ")).await, (vec![4, 3, 2, 1], 4));
    }

    #[tokio::test]
    async fn restricts_search_to_side() {
        let db = setup().await;
        let offered = OffersPageParams {
            side: OfferSearchSide::Offered,
            ..text("space")
        };
        let requested = OffersPageParams {
            side: OfferSearchSide::Requested,
            ..text("space")
        };
        assert_eq!(ids(&db, offered).await, (vec![1], 1));
        assert_eq!(ids(&db, requested).await, (vec![4, 3], 2));
    }

    #[tokio::test]
    async fn matches_exact_asset_and_offer_ids() {
        let db = setup().await;
        let asset = OffersPageParams {
            find_text: Some(hex::encode(SPACE)),
            find_id: Some(SPACE),
            ..Default::default()
        };
        let offer = OffersPageParams {
            find_text: Some(hex::encode([2u8; 32])),
            find_id: Some(Bytes32::new([2; 32])),
            ..Default::default()
        };
        assert_eq!(ids(&db, asset).await, (vec![4, 3, 1], 3));
        assert_eq!(ids(&db, offer).await, (vec![2], 1));
    }

    #[tokio::test]
    async fn escapes_like_wildcards() {
        let db = setup().await;
        // Only "100%_Pure" contains a literal % or _.
        assert_eq!(ids(&db, text("%")).await, (vec![3, 2], 2));
        assert_eq!(ids(&db, text("_")).await, (vec![3, 2], 2));
    }

    #[tokio::test]
    async fn sorts_by_expiration_with_non_expiring_last() {
        let db = setup().await;
        let asc = OffersPageParams {
            sort: OfferSortColumn::Expiration,
            ascending: true,
            ..Default::default()
        };
        let desc = OffersPageParams {
            sort: OfferSortColumn::Expiration,
            ascending: false,
            ..Default::default()
        };
        assert_eq!(ids(&db, asc).await, (vec![3, 1, 4, 2], 4));
        assert_eq!(ids(&db, desc).await, (vec![4, 1, 3, 2], 4));
    }

    #[tokio::test]
    async fn pages_with_total() {
        let db = setup().await;
        let first = OffersPageParams {
            limit: Some(2),
            ..Default::default()
        };
        let second = OffersPageParams {
            limit: Some(2),
            offset: 2,
            ..Default::default()
        };
        let offset_only = OffersPageParams {
            offset: 3,
            ..Default::default()
        };
        assert_eq!(ids(&db, first).await, (vec![4, 3], 4));
        assert_eq!(ids(&db, second).await, (vec![2, 1], 4));
        assert_eq!(ids(&db, offset_only).await, (vec![1], 4));
    }

    #[tokio::test]
    async fn paging_is_stable_with_equal_timestamps() {
        let db = setup().await;
        for id in 10..20 {
            insert_test_offer(&db, id, OfferStatus::Active, 999, None, SPACE, MARMOT).await;
        }
        let mut seen = Vec::new();
        for page in 0..5 {
            let params = OffersPageParams {
                limit: Some(3),
                offset: page * 3,
                ..Default::default()
            };
            seen.extend(ids(&db, params).await.0);
        }
        let mut unique = seen.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(seen.len(), 14);
        assert_eq!(unique.len(), 14);
    }
}
