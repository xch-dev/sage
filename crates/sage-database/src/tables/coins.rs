use chia::{
    protocol::{Bytes32, Coin, CoinState, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::driver::{
    Cat, CatInfo, Did, DidInfo, Nft, NftInfo, OptionContract, OptionInfo,
};
use sqlx::{query, Row, SqliteExecutor};

use crate::{AssetKind, Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoinKind {
    Xch,
    Cat,
    Did,
    Option,
    Nft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoinSortMode {
    CoinId,
    Amount,
    CreatedHeight,
    SpentHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoinFilterMode {
    All,
    Owned,
    Spent,
    Spendable,
}

#[derive(Debug, Clone, Copy)]
pub enum AssetFilter {
    Id(Bytes32),
    Nfts,
    Dids,
}

#[derive(Debug, Clone, Copy)]
pub struct CoinRow {
    pub coin: Coin,
    pub p2_puzzle_hash: Bytes32,
    pub kind: CoinKind,
    pub mempool_item_hash: Option<Bytes32>,
    pub offer_hash: Option<Bytes32>,
    pub created_height: Option<u32>,
    pub spent_height: Option<u32>,
    pub created_timestamp: Option<u32>,
    pub spent_timestamp: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct UnsyncedCoin {
    pub coin_state: CoinState,
    pub is_asset_unsynced: bool,
    pub is_children_unsynced: bool,
}

impl Database {
    pub async fn coins_by_ids(&self, coin_ids: &[String]) -> Result<Vec<CoinRow>> {
        coins_by_ids(&self.pool, coin_ids).await
    }

    pub async fn coin_records(
        &self,
        asset_filter: AssetFilter,
        limit: u32,
        offset: u32,
        sort_mode: CoinSortMode,
        ascending: bool,
        filter_mode: CoinFilterMode,
    ) -> Result<(Vec<CoinRow>, u32)> {
        coin_records(
            &self.pool,
            asset_filter,
            limit,
            offset,
            sort_mode,
            ascending,
            filter_mode,
        )
        .await
    }

    pub async fn are_coins_spendable(&self, coin_ids: &[String]) -> Result<bool> {
        are_coins_spendable(&self.pool, coin_ids).await
    }

    pub async fn total_coin_count(&self) -> Result<u32> {
        total_coin_count(&self.pool).await
    }

    pub async fn spendable_xch_coin_count(&self) -> Result<u32> {
        spendable_coin_count(&self.pool, Bytes32::default()).await
    }

    pub async fn spendable_cat_coin_count(&self, asset_id: Bytes32) -> Result<u32> {
        spendable_coin_count(&self.pool, asset_id).await
    }

    pub async fn synced_coin_count(&self) -> Result<u32> {
        synced_coin_count(&self.pool).await
    }

    pub async fn unsynced_coins(&self, limit: usize) -> Result<Vec<UnsyncedCoin>> {
        unsynced_coins(&self.pool, limit).await
    }

    pub async fn update_coin(
        &self,
        coin_id: Bytes32,
        asset_hash: Bytes32,
        p2_puzzle_hash: Bytes32,
        hidden_puzzle_hash: Option<Bytes32>,
    ) -> Result<()> {
        update_coin(
            &self.pool,
            coin_id,
            asset_hash,
            p2_puzzle_hash,
            hidden_puzzle_hash,
        )
        .await
    }

    pub async fn subscription_coin_ids(&self) -> Result<Vec<Bytes32>> {
        subscription_coin_ids(&self.pool).await
    }

    pub async fn xch_balance(&self) -> Result<u128> {
        token_balance(&self.pool, Bytes32::default()).await
    }

    pub async fn cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        token_balance(&self.pool, asset_id).await
    }

    pub async fn token_balance(&self, asset_id: Bytes32) -> Result<u128> {
        token_balance(&self.pool, asset_id).await
    }

    pub async fn spendable_xch_balance(&self) -> Result<u128> {
        spendable_token_balance(&self.pool, Bytes32::default()).await
    }

    pub async fn spendable_cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        spendable_token_balance(&self.pool, asset_id).await
    }

    pub async fn spendable_xch_coins(&self) -> Result<Vec<Coin>> {
        spendable_xch_coins(&self.pool).await
    }

    pub async fn spendable_cat_coins(&self, asset_id: Bytes32) -> Result<Vec<Cat>> {
        spendable_cat_coins(&self.pool, asset_id).await
    }

    pub async fn coin_kind(&self, coin_id: Bytes32) -> Result<Option<CoinKind>> {
        coin_kind(&self.pool, coin_id).await
    }

    pub async fn xch_coin(&self, coin_id: Bytes32) -> Result<Option<Coin>> {
        xch_coin(&self.pool, coin_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn did_coin(&self, coin_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_coin(&self.pool, coin_id).await
    }

    pub async fn nft_coin(&self, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&self.pool, coin_id).await
    }

    pub async fn option_coin(&self, coin_id: Bytes32) -> Result<Option<OptionContract>> {
        option_coin(&self.pool, coin_id).await
    }

    pub async fn did(&self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        did(&self.pool, launcher_id).await
    }

    pub async fn spendable_did(&self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        spendable_did(&self.pool, launcher_id).await
    }

    pub async fn nft(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft(&self.pool, launcher_id).await
    }

    pub async fn spendable_nft(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        spendable_nft(&self.pool, launcher_id).await
    }

    pub async fn option(&self, launcher_id: Bytes32) -> Result<Option<OptionContract>> {
        option(&self.pool, launcher_id).await
    }

    pub async fn spendable_option(&self, launcher_id: Bytes32) -> Result<Option<OptionContract>> {
        spendable_option(&self.pool, launcher_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_coin(&mut self, coin_state: CoinState) -> Result<()> {
        insert_coin(&mut *self.tx, coin_state).await
    }

    pub async fn is_known_coin(&mut self, coin_id: Bytes32) -> Result<bool> {
        is_known_coin(&mut *self.tx, coin_id).await
    }

    pub async fn update_coin(
        &mut self,
        coin_id: Bytes32,
        asset_hash: Bytes32,
        p2_puzzle_hash: Bytes32,
        hidden_puzzle_hash: Option<Bytes32>,
    ) -> Result<()> {
        update_coin(
            &mut *self.tx,
            coin_id,
            asset_hash,
            p2_puzzle_hash,
            hidden_puzzle_hash,
        )
        .await
    }

    pub async fn set_children_synced(&mut self, coin_id: Bytes32) -> Result<()> {
        set_children_synced(&mut *self.tx, coin_id).await
    }

    pub async fn set_transaction_children_unsynced(
        &mut self,
        mempool_item_id: Bytes32,
    ) -> Result<()> {
        set_transaction_children_unsynced(&mut *self.tx, mempool_item_id).await
    }

    pub async fn delete_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        delete_coin(&mut *self.tx, coin_id).await
    }

    pub async fn insert_lineage_proof(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
    ) -> Result<()> {
        insert_lineage_proof(&mut *self.tx, coin_id, lineage_proof).await
    }
}

async fn are_coins_spendable(conn: impl SqliteExecutor<'_>, coin_ids: &[String]) -> Result<bool> {
    if coin_ids.is_empty() {
        return Ok(false);
    }

    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT COUNT(*) AS count
        FROM spendable_coins
        WHERE 1=1
        AND coin_hash IN (",
    );

    let mut separated = query.separated(", ");
    for coin_id in coin_ids {
        separated.push(format!("X'{coin_id}'"));
    }
    separated.push_unseparated(")");

    let count: i64 = query.build().fetch_one(conn).await?.get("count");

    #[allow(clippy::cast_possible_wrap)]
    Ok(count == coin_ids.len() as i64)
}

async fn insert_coin(conn: impl SqliteExecutor<'_>, coin_state: CoinState) -> Result<()> {
    let hash = coin_state.coin.coin_id();
    let hash = hash.as_ref();
    let parent_coin_hash = coin_state.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_state.coin.puzzle_hash.as_ref();
    let amount = coin_state.coin.amount.to_be_bytes().to_vec();

    query!(
        "
        INSERT INTO coins
            (hash, parent_coin_hash, puzzle_hash, amount, created_height, spent_height)
        VALUES
            (?, ?, ?, ?, ?, ?)
        ON CONFLICT(hash) DO UPDATE SET
            created_height = excluded.created_height,
            spent_height = excluded.spent_height
        ",
        hash,
        parent_coin_hash,
        puzzle_hash,
        amount,
        coin_state.created_height,
        coin_state.spent_height,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn is_known_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<bool> {
    let coin_id_ref = coin_id.as_ref();

    let row = query!(
        "SELECT COUNT(*) AS count FROM coins WHERE hash = ?",
        coin_id_ref
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count > 0)
}

async fn unsynced_coins(conn: impl SqliteExecutor<'_>, limit: usize) -> Result<Vec<UnsyncedCoin>> {
    let limit = i64::try_from(limit)?;

    query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, created_height, spent_height,
            (asset_id IS NULL) AS is_asset_unsynced,
            (spent_height IS NOT NULL AND is_children_synced = FALSE) AS is_children_unsynced
        FROM coins
        WHERE asset_id IS NULL OR (spent_height IS NOT NULL AND is_children_synced = FALSE)
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(UnsyncedCoin {
            coin_state: CoinState::new(
                Coin::new(
                    row.parent_coin_hash.convert()?,
                    row.puzzle_hash.convert()?,
                    row.amount.convert()?,
                ),
                row.spent_height.convert()?,
                row.created_height.convert()?,
            ),
            is_asset_unsynced: row.is_asset_unsynced != 0,
            is_children_unsynced: row.is_children_unsynced != 0,
        })
    })
    .collect()
}

async fn delete_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id_ref = coin_id.as_ref();

    query!("DELETE FROM coins WHERE hash = ?", coin_id_ref)
        .execute(conn)
        .await?;

    Ok(())
}

async fn update_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    asset_hash: Bytes32,
    p2_puzzle_hash: Bytes32,
    hidden_puzzle_hash: Option<Bytes32>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let asset_hash = asset_hash.as_ref();
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let hidden_puzzle_hash = hidden_puzzle_hash.as_deref();

    query!(
        "
        UPDATE coins SET
            asset_id = (SELECT id FROM assets WHERE hash = ?),
            p2_puzzle_id = (SELECT id FROM p2_puzzles WHERE hash = ?),
            hidden_puzzle_hash = ?
        WHERE hash = ?
        ",
        asset_hash,
        p2_puzzle_hash,
        hidden_puzzle_hash,
        coin_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn set_children_synced(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    query!(
        "UPDATE coins SET is_children_synced = TRUE WHERE hash = ?",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn set_transaction_children_unsynced(
    conn: impl SqliteExecutor<'_>,
    mempool_item_id: Bytes32,
) -> Result<()> {
    let mempool_item_id = mempool_item_id.as_ref();

    query!(
        "
        UPDATE coins SET is_children_synced = FALSE WHERE id IN (
            SELECT coin_id FROM mempool_coins
            INNER JOIN mempool_items ON mempool_items.id = mempool_coins.mempool_item_id
            WHERE mempool_items.hash = ? AND is_input = TRUE
        )
        ",
        mempool_item_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_lineage_proof(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_hash = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes().to_vec();

    query!(
        "INSERT OR IGNORE INTO lineage_proofs
            (coin_id, parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount)
        VALUES
            ((SELECT id FROM coins WHERE hash = ?), ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_hash,
        parent_inner_puzzle_hash,
        parent_amount,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn subscription_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM coins WHERE asset_id != 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
}

async fn spendable_coin_count(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u32> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "SELECT COUNT(*) AS count FROM spendable_coins WHERE asset_hash = ?",
        asset_id_ref
    )
    .fetch_one(conn)
    .await?
    .count
    .convert()
}

async fn total_coin_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    query!("SELECT COUNT(*) AS count FROM coins")
        .fetch_one(conn)
        .await?
        .count
        .convert()
}

async fn synced_coin_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    query!(
        "
        SELECT COUNT(*) AS count FROM coins
        WHERE asset_id IS NOT NULL
        AND (spent_height IS NULL OR is_children_synced = TRUE)
        "
    )
    .fetch_one(conn)
    .await?
    .count
    .convert()
}

async fn token_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "SELECT amount FROM owned_coins WHERE asset_hash = ?",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        let amount: u64 = row.amount.convert()?;
        Ok(amount as u128)
    })
    .sum()
}

async fn spendable_token_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "SELECT amount FROM spendable_coins WHERE asset_hash = ?",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        let amount: u64 = row.amount.convert()?;
        Ok(amount as u128)
    })
    .sum()
}

async fn coins_by_ids(conn: impl SqliteExecutor<'_>, coin_ids: &[String]) -> Result<Vec<CoinRow>> {
    let mut query = sqlx::QueryBuilder::new(
        "
       SELECT
            parent_coin_hash, puzzle_hash, amount, spent_height, created_height, p2_puzzles.hash AS p2_puzzle_hash,
            (
                SELECT hash FROM mempool_items
                INNER JOIN mempool_coins ON mempool_coins.mempool_item_id = mempool_items.id
                WHERE mempool_coins.coin_id = coins.id
                AND mempool_coins.is_input = TRUE
                LIMIT 1
            ) AS mempool_item_hash,
            (
                SELECT hash FROM offers
                INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
                WHERE offer_coins.coin_id = coins.id
                AND offers.status <= 1
                LIMIT 1
            ) AS offer_hash,
            created_blocks.timestamp AS created_timestamp,
            spent_blocks.timestamp AS spent_timestamp
        FROM coins
            INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
            LEFT JOIN blocks AS created_blocks ON created_blocks.height = coins.created_height
            LEFT JOIN blocks AS spent_blocks ON spent_blocks.height = coins.spent_height            
        WHERE coins.hash IN (",
    );
    let mut separated = query.separated(", ");

    for coin_id in coin_ids {
        separated.push(format!("X'{coin_id}'"));
    }
    separated.push_unseparated(")");

    let rows = query.build().fetch_all(conn).await?;

    let coins = rows
        .into_iter()
        .map(|row| {
            Ok(CoinRow {
                coin: Coin::new(
                    row.get::<Vec<u8>, _>("parent_coin_hash").convert()?,
                    row.get::<Vec<u8>, _>("puzzle_hash").convert()?,
                    row.get::<Vec<u8>, _>("amount").convert()?,
                ),
                p2_puzzle_hash: row.get::<Vec<u8>, _>("p2_puzzle_hash").convert()?,
                mempool_item_hash: row
                    .get::<Option<Vec<u8>>, _>("mempool_item_hash")
                    .map(Convert::convert)
                    .transpose()?,
                offer_hash: row
                    .get::<Option<Vec<u8>>, _>("offer_hash")
                    .map(Convert::convert)
                    .transpose()?,
                kind: CoinKind::Xch,
                created_height: row.get::<Option<u32>, _>("created_height"),
                spent_height: row.get::<Option<u32>, _>("spent_height"),
                created_timestamp: row
                    .get::<Option<i64>, _>("created_timestamp")
                    .map(TryInto::try_into)
                    .transpose()?,
                spent_timestamp: row
                    .get::<Option<i64>, _>("spent_timestamp")
                    .map(TryInto::try_into)
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(coins)
}

async fn coin_records(
    conn: impl SqliteExecutor<'_>,
    asset_filter: AssetFilter,
    limit: u32,
    offset: u32,
    sort_mode: CoinSortMode,
    ascending: bool,
    filter_mode: CoinFilterMode,
) -> Result<(Vec<CoinRow>, u32)> {
    let table = match filter_mode {
        CoinFilterMode::Owned => "owned_coins",
        CoinFilterMode::Spent => "spent_coins",
        CoinFilterMode::All => "wallet_coins",
        CoinFilterMode::Spendable => "spendable_coins",
    };

    let mut query = sqlx::QueryBuilder::new(format!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount,
            spent_height, created_height, p2_puzzle_hash,
            (
                SELECT hash FROM mempool_items
                INNER JOIN mempool_coins ON mempool_coins.mempool_item_id = mempool_items.id
                WHERE mempool_coins.coin_id = {table}.coin_id
                AND mempool_coins.is_input = TRUE
                LIMIT 1
            ) AS mempool_item_hash,
            (
                SELECT hash FROM offers
                INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
                WHERE offer_coins.coin_id = {table}.coin_id
                AND offers.status <= 1
                LIMIT 1
            ) AS offer_hash,
            created_blocks.timestamp AS created_timestamp,
            spent_blocks.timestamp AS spent_timestamp,
            COUNT(*) OVER () AS total_count
        FROM {table}
            LEFT JOIN blocks AS created_blocks ON created_blocks.height = {table}.created_height
            LEFT JOIN blocks AS spent_blocks ON spent_blocks.height = {table}.spent_height  
        ",
    ));

    match asset_filter {
        AssetFilter::Id(asset_id) => {
            query.push(" WHERE asset_hash = ");
            query.push_bind(asset_id.to_vec());
        }
        AssetFilter::Nfts => {
            query.push(" WHERE asset_kind = 1");
        }
        AssetFilter::Dids => {
            query.push(" WHERE asset_kind = 2");
        }
    }

    query.push(" ORDER BY ");
    match sort_mode {
        CoinSortMode::CoinId => query.push("coin_hash"),
        CoinSortMode::Amount => query.push("amount"),
        CoinSortMode::CreatedHeight => query.push("created_height"),
        CoinSortMode::SpentHeight => query.push("spent_height"),
    };
    if ascending {
        query.push(" ASC");
    } else {
        query.push(" DESC");
    }

    query.push(" LIMIT ");
    query.push_bind(limit as i64);
    query.push(" OFFSET ");
    query.push_bind(offset as i64);

    let rows = query.build().fetch_all(conn).await?;
    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;
    let coins = rows
        .into_iter()
        .map(|row| {
            Ok(CoinRow {
                coin: Coin::new(
                    row.get::<Vec<u8>, _>("parent_coin_hash").convert()?,
                    row.get::<Vec<u8>, _>("puzzle_hash").convert()?,
                    row.get::<Vec<u8>, _>("amount").convert()?,
                ),
                p2_puzzle_hash: row.get::<Vec<u8>, _>("p2_puzzle_hash").convert()?,
                mempool_item_hash: row
                    .get::<Option<Vec<u8>>, _>("mempool_item_hash")
                    .map(Convert::convert)
                    .transpose()?,
                offer_hash: row
                    .get::<Option<Vec<u8>>, _>("offer_hash")
                    .map(Convert::convert)
                    .transpose()?,
                kind: CoinKind::Xch,
                created_height: row.get::<Option<u32>, _>("created_height"),
                spent_height: row.get::<Option<u32>, _>("spent_height"),
                created_timestamp: row
                    .get::<Option<i64>, _>("created_timestamp")
                    .map(TryInto::try_into)
                    .transpose()?,
                spent_timestamp: row
                    .get::<Option<i64>, _>("spent_timestamp")
                    .map(TryInto::try_into)
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((coins, total_count))
}

async fn spendable_xch_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Coin>> {
    query!("SELECT parent_coin_hash, puzzle_hash, amount FROM spendable_coins WHERE asset_id = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ))
        })
        .collect()
}

async fn spendable_cat_coins(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Vec<Cat>> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, hidden_puzzle_hash, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount
        FROM spendable_coins
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = spendable_coins.coin_id
        WHERE asset_hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Cat::new(
            Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            Some(LineageProof {
                parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
                parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
                parent_amount: row.parent_amount.convert()?,
            }),
            CatInfo::new(
                asset_id,
                row.hidden_puzzle_hash.convert()?,
                row.p2_puzzle_hash.convert()?,
            ),
        ))
    })
    .collect()
}

async fn coin_kind(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<CoinKind>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "SELECT asset_id, kind FROM coins INNER JOIN assets ON assets.id = asset_id WHERE coins.hash = ?",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    let Some(asset_id) = row.asset_id else {
        return Err(DatabaseError::InvalidEnumVariant);
    };

    let kind: AssetKind = row.kind.convert()?;

    Ok(Some(match kind {
        AssetKind::Token => {
            if asset_id == 0 {
                CoinKind::Xch
            } else {
                CoinKind::Cat
            }
        }
        AssetKind::Nft => CoinKind::Nft,
        AssetKind::Did => CoinKind::Did,
        AssetKind::Option => CoinKind::Option,
    }))
}

async fn xch_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Coin>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount
        FROM owned_coins
        WHERE coin_hash = ? AND asset_id = 0
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Coin::new(
        row.parent_coin_hash.convert()?,
        row.puzzle_hash.convert()?,
        row.amount.convert()?,
    )))
}

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, hidden_puzzle_hash, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount, asset_hash AS asset_id
        FROM owned_coins
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE coin_hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Cat::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Some(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        CatInfo::new(
            row.asset_id.convert()?,
            row.hidden_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn did_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Did<Program>>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, recovery_list_hash, num_verifications_required, metadata
        FROM owned_coins
        INNER JOIN dids ON dids.asset_id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE coin_hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        DidInfo::new(
            row.launcher_id.convert()?,
            row.recovery_list_hash.convert()?,
            row.num_verifications_required.convert()?,
            row.metadata.into(),
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn nft_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, metadata, metadata_updater_puzzle_hash,
            owner_hash, royalty_puzzle_hash, royalty_basis_points
        FROM owned_coins
        INNER JOIN nfts ON nfts.asset_id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE coin_hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        NftInfo::new(
            row.launcher_id.convert()?,
            row.metadata.into(),
            row.metadata_updater_puzzle_hash.convert()?,
            row.owner_hash.convert()?,
            row.royalty_puzzle_hash.convert()?,
            row.royalty_basis_points.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn option_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, underlying_coin_hash, underlying_delegated_puzzle_hash
        FROM owned_coins
        INNER JOIN options ON options.asset_id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE coin_hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(OptionContract::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        OptionInfo::new(
            row.launcher_id.convert()?,
            row.underlying_coin_hash.convert()?,
            row.underlying_delegated_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn did(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, recovery_list_hash, num_verifications_required, metadata
        FROM owned_coins
        INNER JOIN dids ON dids.asset_id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE asset_hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        DidInfo::new(
            row.launcher_id.convert()?,
            row.recovery_list_hash.convert()?,
            row.num_verifications_required.convert()?,
            row.metadata.into(),
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn spendable_did(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Did<Program>>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, recovery_list_hash, num_verifications_required, metadata
        FROM spendable_coins
        INNER JOIN dids ON dids.asset_id = spendable_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = spendable_coins.coin_id
        WHERE asset_hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        DidInfo::new(
            row.launcher_id.convert()?,
            row.recovery_list_hash.convert()?,
            row.num_verifications_required.convert()?,
            row.metadata.into(),
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn nft(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, metadata, metadata_updater_puzzle_hash,
            owner_hash, royalty_puzzle_hash, royalty_basis_points
        FROM owned_coins
        INNER JOIN nfts ON nfts.asset_id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE asset_hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        NftInfo::new(
            row.launcher_id.convert()?,
            row.metadata.into(),
            row.metadata_updater_puzzle_hash.convert()?,
            row.owner_hash.convert()?,
            row.royalty_puzzle_hash.convert()?,
            row.royalty_basis_points.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn spendable_nft(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Nft<Program>>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, metadata, metadata_updater_puzzle_hash,
            owner_hash, royalty_puzzle_hash, royalty_basis_points
        FROM spendable_coins
        INNER JOIN nfts ON nfts.asset_id = spendable_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = spendable_coins.coin_id
        WHERE asset_hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        NftInfo::new(
            row.launcher_id.convert()?,
            row.metadata.into(),
            row.metadata_updater_puzzle_hash.convert()?,
            row.owner_hash.convert()?,
            row.royalty_puzzle_hash.convert()?,
            row.royalty_basis_points.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn option(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, underlying_coin_hash, underlying_delegated_puzzle_hash
        FROM owned_coins
        INNER JOIN options ON options.asset_id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = owned_coins.coin_id
        WHERE asset_hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(OptionContract::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        OptionInfo::new(
            row.launcher_id.convert()?,
            row.underlying_coin_hash.convert()?,
            row.underlying_delegated_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn spendable_option(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            asset_hash AS launcher_id, underlying_coin_hash, underlying_delegated_puzzle_hash
        FROM spendable_coins
        INNER JOIN options ON options.asset_id = spendable_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = spendable_coins.coin_id
        WHERE asset_hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(OptionContract::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        OptionInfo::new(
            row.launcher_id.convert()?,
            row.underlying_coin_hash.convert()?,
            row.underlying_delegated_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}
