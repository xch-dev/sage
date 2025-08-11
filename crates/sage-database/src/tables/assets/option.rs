use chia::protocol::{Bytes32, Coin};
use chia_wallet_sdk::driver::{OptionType, OptionUnderlying};
use sqlx::{query, Row, SqliteExecutor};

use crate::{Asset, AssetKind, CoinKind, CoinRow, Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionSortMode {
    Name,
    CreatedHeight,
    ExpirationSeconds,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionCoinInfo {
    pub underlying_coin_hash: Bytes32,
    pub underlying_delegated_puzzle_hash: Bytes32,
    pub strike_asset_hash: Bytes32,
    pub strike_amount: u64,
}

#[derive(Debug, Clone)]
pub struct OptionRow {
    pub asset: Asset,
    pub underlying_asset: Asset,
    pub underlying_amount: u64,
    pub strike_asset: Asset,
    pub strike_amount: u64,
    pub expiration_seconds: u64,
    pub coin_row: CoinRow,
    pub underlying_coin_id: Bytes32,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionOfferInfo {
    pub underlying_coin_hash: Bytes32,
    pub underlying_delegated_puzzle_hash: Bytes32,
}

impl Database {
    pub async fn owned_options(
        &self,
        limit: u32,
        offset: u32,
        sort_mode: OptionSortMode,
        ascending: bool,
        find_value: Option<String>,
        include_hidden: bool,
    ) -> Result<(Vec<OptionRow>, u32)> {
        owned_options(
            &self.pool,
            limit,
            offset,
            sort_mode,
            ascending,
            find_value,
            include_hidden,
        )
        .await
    }

    pub async fn owned_option(&self, launcher_id: Bytes32) -> Result<Option<OptionRow>> {
        let launcher_id_ref = launcher_id.as_ref();

        query!(
            "
            SELECT
                asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
                asset_description, asset_is_visible, asset_is_sensitive_content,
                asset_hidden_puzzle_hash, owned_coins.created_height, owned_coins.spent_height,
                owned_coins.parent_coin_hash, owned_coins.puzzle_hash, owned_coins.amount, owned_coins.p2_puzzle_hash,
                offer_hash AS 'offer_hash?', created_timestamp, spent_timestamp,
                clawback_expiration_seconds AS 'clawback_timestamp?',
                p2_options.expiration_seconds AS option_expiration_seconds,

                strike_asset.hash AS strike_asset_hash, strike_asset.name AS strike_asset_name,
                strike_asset.ticker AS strike_asset_ticker, strike_asset.precision AS strike_asset_precision,
                strike_asset.icon_url AS strike_asset_icon_url, strike_asset.description AS strike_asset_description,
                strike_asset.is_visible AS strike_asset_is_visible, strike_asset.is_sensitive_content AS strike_asset_is_sensitive_content,
                strike_asset.hidden_puzzle_hash AS strike_asset_hidden_puzzle_hash, strike_asset.kind AS strike_asset_kind,

                underlying_asset.hash AS underlying_asset_hash, underlying_asset.name AS underlying_asset_name,
                underlying_asset.ticker AS underlying_asset_ticker, underlying_asset.precision AS underlying_asset_precision,
                underlying_asset.icon_url AS underlying_asset_icon_url, underlying_asset.description AS underlying_asset_description,
                underlying_asset.is_visible AS underlying_asset_is_visible, underlying_asset.is_sensitive_content AS underlying_asset_is_sensitive_content,
                underlying_asset.hidden_puzzle_hash AS underlying_asset_hidden_puzzle_hash, underlying_asset.kind AS underlying_asset_kind,
                
                strike_amount, 
                underlying_coin.amount AS underlying_amount,
                underlying_coin.hash AS underlying_coin_id
            FROM owned_coins
            INNER JOIN options ON options.asset_id = owned_coins.asset_id
            INNER JOIN p2_options ON p2_options.option_asset_id = options.asset_id
            INNER JOIN coins AS underlying_coin ON underlying_coin.id = options.underlying_coin_id
            INNER JOIN assets AS strike_asset ON strike_asset.id = options.strike_asset_id
            INNER JOIN assets AS underlying_asset ON underlying_asset.id = underlying_coin.asset_id
            WHERE asset_hash = ?
            ",
            launcher_id_ref
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(OptionRow {
                asset: Asset {
                    hash: row.asset_hash.convert()?,
                    name: row.asset_name,
                    ticker: row.asset_ticker,
                    precision: row.asset_precision.convert()?,
                    icon_url: row.asset_icon_url,
                    description: row.asset_description,
                    is_visible: row.asset_is_visible,
                    is_sensitive_content: row.asset_is_sensitive_content,
                    hidden_puzzle_hash: row.asset_hidden_puzzle_hash.convert()?,
                    kind: AssetKind::Option,
                },
                underlying_asset: Asset {
                    hash: row.underlying_asset_hash.convert()?,
                    name: row.underlying_asset_name,
                    ticker: row.underlying_asset_ticker,
                    precision: row.underlying_asset_precision.convert()?,
                    icon_url: row.underlying_asset_icon_url,
                    description: row.underlying_asset_description,
                    is_visible: row.underlying_asset_is_visible,
                    is_sensitive_content: row.underlying_asset_is_sensitive_content,
                    hidden_puzzle_hash: row.underlying_asset_hidden_puzzle_hash.convert()?,
                    kind: row.underlying_asset_kind.convert()?,
                },
                underlying_amount: row.underlying_amount.convert()?,
                strike_asset: Asset {
                    hash: row.strike_asset_hash.convert()?,
                    name: row.strike_asset_name,
                    ticker: row.strike_asset_ticker,
                    precision: row.strike_asset_precision.convert()?,
                    icon_url: row.strike_asset_icon_url,
                    description: row.strike_asset_description,
                    is_visible: row.strike_asset_is_visible,
                    is_sensitive_content: row.strike_asset_is_sensitive_content,
                    hidden_puzzle_hash: row.strike_asset_hidden_puzzle_hash.convert()?,
                    kind: row.strike_asset_kind.convert()?,
                },
                strike_amount: row.strike_amount.convert()?,
                underlying_coin_id: row.underlying_coin_id.convert()?,
                expiration_seconds: row.option_expiration_seconds.convert()?,
                coin_row: CoinRow {
                    coin: Coin::new(
                        row.parent_coin_hash.convert()?,
                        row.puzzle_hash.convert()?,
                        row.amount.convert()?,
                    ),
                    p2_puzzle_hash: row.p2_puzzle_hash.convert()?,
                    kind: CoinKind::Option,
                    mempool_item_hash: None,
                    offer_hash: row.offer_hash.convert()?,
                    clawback_timestamp: row.clawback_timestamp.convert()?,
                    created_height: row.created_height.convert()?,
                    spent_height: row.spent_height.convert()?,
                    created_timestamp: row.created_timestamp.convert()?,
                    spent_timestamp: row.spent_timestamp.convert()?,
                },
            })
        })
        .transpose()
    }

    pub async fn option_underlying(
        &self,
        launcher_id: Bytes32,
    ) -> Result<Option<OptionUnderlying>> {
        let launcher_id_ref = launcher_id.as_ref();

        let Some(row) = query!(
            "
            SELECT
                creator_puzzle_hash, expiration_seconds,
                (
                    SELECT amount FROM coins
                    WHERE coins.p2_puzzle_id = p2_options.p2_puzzle_id LIMIT 1
                ) AS underlying_amount,
                (SELECT hash FROM assets WHERE id = strike_asset_id) AS strike_asset_hash,
                strike_amount, strike_assets.hidden_puzzle_hash AS strike_hidden_puzzle_hash
            FROM p2_options
            INNER JOIN options ON options.asset_id = p2_options.option_asset_id
            INNER JOIN assets AS strike_assets ON strike_assets.id = options.strike_asset_id
            WHERE option_asset_id = (SELECT id FROM assets WHERE hash = ?)
            ",
            launcher_id_ref
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let asset_hash: Bytes32 = row.strike_asset_hash.convert()?;
        let amount: u64 = row.strike_amount.convert()?;
        let hidden_puzzle_hash: Option<Bytes32> = row.strike_hidden_puzzle_hash.convert()?;

        Ok(Some(OptionUnderlying::new(
            launcher_id,
            row.creator_puzzle_hash.convert()?,
            row.expiration_seconds.convert()?,
            row.underlying_amount.convert()?,
            if asset_hash == Bytes32::default() {
                OptionType::Xch { amount }
            } else if let Some(hidden_puzzle_hash) = hidden_puzzle_hash {
                OptionType::RevocableCat {
                    asset_id: asset_hash,
                    hidden_puzzle_hash,
                    amount,
                }
            } else {
                OptionType::Cat {
                    asset_id: asset_hash,
                    amount,
                }
            },
        )))
    }

    pub async fn offer_option_info(&self, hash: Bytes32) -> Result<Option<OptionOfferInfo>> {
        let hash = hash.as_ref();

        query!(
            "
            SELECT
                (SELECT hash FROM coins WHERE coins.id = underlying_coin_id) AS underlying_coin_hash,
                underlying_delegated_puzzle_hash
            FROM options
            INNER JOIN assets ON assets.id = options.asset_id
            WHERE hash = ?
            ",
            hash
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(OptionOfferInfo {
                underlying_coin_hash: row.underlying_coin_hash.convert()?,
                underlying_delegated_puzzle_hash: row.underlying_delegated_puzzle_hash.convert()?,
            })
        })
        .transpose()
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_option(&mut self, hash: Bytes32, coin_info: &OptionCoinInfo) -> Result<()> {
        let hash = hash.as_ref();
        let underlying_coin_hash = coin_info.underlying_coin_hash.as_ref();
        let underlying_delegated_puzzle_hash = coin_info.underlying_delegated_puzzle_hash.as_ref();
        let strike_asset_hash = coin_info.strike_asset_hash.as_ref();
        let strike_amount = coin_info.strike_amount.to_be_bytes().to_vec();

        query!(
            "
            INSERT OR IGNORE INTO options (
                asset_id, underlying_coin_id, underlying_delegated_puzzle_hash, strike_asset_id, strike_amount
            )
            VALUES (
                (SELECT id FROM assets WHERE hash = ?),
                (SELECT id FROM coins WHERE hash = ?),
                ?,
                (SELECT id FROM assets WHERE hash = ?),
                ?
            )
            ",
            hash,
            underlying_coin_hash,
            underlying_delegated_puzzle_hash,
            strike_asset_hash,
            strike_amount
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}

async fn owned_options(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
    sort_mode: OptionSortMode,
    ascending: bool,
    find_value: Option<String>,
    include_hidden: bool,
) -> Result<(Vec<OptionRow>, u32)> {
    let mut query = sqlx::QueryBuilder::new(
        "SELECT
            asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
            asset_description, asset_is_visible, asset_is_sensitive_content,
            asset_hidden_puzzle_hash, owned_coins.created_height, owned_coins.spent_height,
            owned_coins.parent_coin_hash, owned_coins.puzzle_hash, owned_coins.amount, owned_coins.p2_puzzle_hash,
            offer_hash, created_timestamp, spent_timestamp,
            clawback_expiration_seconds AS clawback_timestamp,
            p2_options.expiration_seconds AS option_expiration_seconds,

            strike_asset.hash AS strike_asset_hash, strike_asset.name AS strike_asset_name,
            strike_asset.ticker AS strike_asset_ticker, strike_asset.precision AS strike_asset_precision,
            strike_asset.icon_url AS strike_asset_icon_url, strike_asset.description AS strike_asset_description,
            strike_asset.is_visible AS strike_asset_is_visible, strike_asset.is_sensitive_content AS strike_asset_is_sensitive_content,
            strike_asset.hidden_puzzle_hash AS strike_asset_hidden_puzzle_hash, strike_asset.kind AS strike_asset_kind,

            underlying_asset.hash AS underlying_asset_hash, underlying_asset.name AS underlying_asset_name,
            underlying_asset.ticker AS underlying_asset_ticker, underlying_asset.precision AS underlying_asset_precision,
            underlying_asset.icon_url AS underlying_asset_icon_url, underlying_asset.description AS underlying_asset_description,
            underlying_asset.is_visible AS underlying_asset_is_visible, underlying_asset.is_sensitive_content AS underlying_asset_is_sensitive_content,
            underlying_asset.hidden_puzzle_hash AS underlying_asset_hidden_puzzle_hash, underlying_asset.kind AS underlying_asset_kind,

            strike_amount, underlying_coin.amount AS underlying_amount, underlying_coin.hash AS underlying_coin_id,

            COUNT(*) OVER() as total_count
        FROM owned_coins
        INNER JOIN options ON options.asset_id = owned_coins.asset_id
        INNER JOIN p2_options ON p2_options.option_asset_id = options.asset_id
        INNER JOIN coins AS underlying_coin ON underlying_coin.id = options.underlying_coin_id
        INNER JOIN assets AS strike_asset ON strike_asset.id = options.strike_asset_id
        INNER JOIN assets AS underlying_asset ON underlying_asset.id = underlying_coin.asset_id
        WHERE 1=1",
    );

    if !include_hidden {
        query.push(" AND asset_is_visible = 1");
    }

    if let Some(find_value) = find_value {
        query.push(" AND (asset_name LIKE ");
        query.push_bind(format!("%{find_value}%"));
        query.push(" OR asset_ticker LIKE ");
        query.push_bind(format!("%{find_value}%"));
        query.push(" OR underlying_asset.name LIKE ");
        query.push_bind(format!("%{find_value}%"));
        query.push(" OR underlying_asset.ticker LIKE ");
        query.push_bind(format!("%{find_value}%"));
        query.push(" OR strike_asset.name LIKE ");
        query.push_bind(format!("%{find_value}%"));
        query.push(" OR strike_asset.ticker LIKE ");
        query.push_bind(format!("%{find_value}%"));

        // If find_value looks like a valid asset ID (64 hex chars), search by asset hash
        if find_value.len() == 64 && find_value.chars().all(|c| c.is_ascii_hexdigit()) {
            query.push(" OR asset_hash = X'");
            query.push(find_value.clone());
            query.push("' OR underlying_asset.hash = X'");
            query.push(find_value.clone());
            query.push("' OR strike_asset.hash = X'");
            query.push(find_value);
            query.push("'");
        }

        query.push(")");
    }

    // Add ORDER BY clause based on sort_mode and ascending
    query.push(" ORDER BY ");
    let order_column = match sort_mode {
        OptionSortMode::Name => "asset_name",
        OptionSortMode::CreatedHeight => "owned_coins.created_height",
        OptionSortMode::ExpirationSeconds => "p2_options.expiration_seconds",
    };
    query.push(order_column);

    if ascending {
        query.push(" ASC");
    } else {
        query.push(" DESC");
    }

    query.push(" LIMIT ");
    query.push_bind(limit);
    query.push(" OFFSET ");
    query.push_bind(offset);

    let built_query = query.build();
    let rows = built_query.fetch_all(conn).await?;

    let total_count: u32 = rows
        .first()
        .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;

    let options = rows
        .into_iter()
        .map(|row| {
            Ok(OptionRow {
                asset: Asset {
                    hash: row.get::<Vec<u8>, _>("asset_hash").convert()?,
                    name: row.get::<Option<String>, _>("asset_name"),
                    ticker: row.get::<Option<String>, _>("asset_ticker"),
                    precision: row.get::<i64, _>("asset_precision").convert()?,
                    icon_url: row.get::<Option<String>, _>("asset_icon_url"),
                    description: row.get::<Option<String>, _>("asset_description"),
                    is_visible: row.get::<bool, _>("asset_is_visible"),
                    is_sensitive_content: row.get::<bool, _>("asset_is_sensitive_content"),
                    hidden_puzzle_hash: row
                        .get::<Option<Vec<u8>>, _>("asset_hidden_puzzle_hash")
                        .convert()?,
                    kind: AssetKind::Option,
                },
                underlying_amount: row.get::<Vec<u8>, _>("underlying_amount").convert()?,
                underlying_coin_id: row.get::<Vec<u8>, _>("underlying_coin_id").convert()?,
                underlying_asset: Asset {
                    hash: row.get::<Vec<u8>, _>("underlying_asset_hash").convert()?,
                    name: row.get::<Option<String>, _>("underlying_asset_name"),
                    ticker: row.get::<Option<String>, _>("underlying_asset_ticker"),
                    precision: row.get::<i64, _>("underlying_asset_precision").convert()?,
                    icon_url: row.get::<Option<String>, _>("underlying_asset_icon_url"),
                    description: row.get::<Option<String>, _>("underlying_asset_description"),
                    is_visible: row.get::<bool, _>("underlying_asset_is_visible"),
                    is_sensitive_content: row
                        .get::<bool, _>("underlying_asset_is_sensitive_content"),
                    hidden_puzzle_hash: row
                        .get::<Option<Vec<u8>>, _>("underlying_asset_hidden_puzzle_hash")
                        .convert()?,
                    kind: row
                        .get::<Option<i64>, _>("underlying_asset_kind")
                        .map(Convert::convert)
                        .transpose()?
                        .unwrap_or(AssetKind::Token),
                },
                strike_amount: row.get::<Vec<u8>, _>("strike_amount").convert()?,
                strike_asset: Asset {
                    hash: row.get::<Vec<u8>, _>("strike_asset_hash").convert()?,
                    name: row.get::<Option<String>, _>("strike_asset_name"),
                    ticker: row.get::<Option<String>, _>("strike_asset_ticker"),
                    precision: row.get::<i64, _>("strike_asset_precision").convert()?,
                    icon_url: row.get::<Option<String>, _>("strike_asset_icon_url"),
                    description: row.get::<Option<String>, _>("strike_asset_description"),
                    is_visible: row.get::<bool, _>("strike_asset_is_visible"),
                    is_sensitive_content: row.get::<bool, _>("strike_asset_is_sensitive_content"),
                    hidden_puzzle_hash: row
                        .get::<Option<Vec<u8>>, _>("strike_asset_hidden_puzzle_hash")
                        .convert()?,
                    kind: row
                        .get::<Option<i64>, _>("strike_asset_kind")
                        .map(Convert::convert)
                        .transpose()?
                        .unwrap_or(AssetKind::Token),
                },
                expiration_seconds: row.get::<i64, _>("option_expiration_seconds").convert()?,
                coin_row: CoinRow {
                    coin: Coin::new(
                        row.get::<Vec<u8>, _>("parent_coin_hash").convert()?,
                        row.get::<Vec<u8>, _>("puzzle_hash").convert()?,
                        row.get::<Vec<u8>, _>("amount").convert()?,
                    ),
                    p2_puzzle_hash: row.get::<Vec<u8>, _>("p2_puzzle_hash").convert()?,
                    kind: CoinKind::Option,
                    mempool_item_hash: None,
                    offer_hash: row.get::<Option<Vec<u8>>, _>("offer_hash").convert()?,
                    clawback_timestamp: row
                        .get::<Option<i64>, _>("clawback_timestamp")
                        .convert()?,
                    created_height: row.get::<Option<i64>, _>("created_height").convert()?,
                    spent_height: row.get::<Option<i64>, _>("spent_height").convert()?,
                    created_timestamp: row.get::<Option<i64>, _>("created_timestamp").convert()?,
                    spent_timestamp: row.get::<Option<i64>, _>("spent_timestamp").convert()?,
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((options, total_count))
}
