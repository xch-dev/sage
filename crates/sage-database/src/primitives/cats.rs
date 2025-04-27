use chia::{protocol::Bytes32, puzzles::LineageProof};
use chia_wallet_sdk::driver::Cat;
use sqlx::{Row, SqliteExecutor};

use crate::{
    into_row, to_bytes, to_bytes32, CatCoinRow, CatCoinSql, CatRow, CatSql, CoinStateRow,
    CoinStateSql, Database, DatabaseTx, EnhancedCoinStateRow, FullCatCoinSql, Result,
};

#[derive(Debug, Clone, Copy)]
pub enum CoinSortMode {
    CoinId,
    Amount,
    CreatedHeight,
    SpentHeight,
}

impl Default for CoinSortMode {
    fn default() -> Self {
        Self::CreatedHeight
    }
}

impl Database {
    pub async fn insert_cat(&self, row: CatRow) -> Result<()> {
        insert_cat(&self.pool, row).await
    }

    pub async fn update_cat(&self, row: CatRow) -> Result<()> {
        update_cat(&self.pool, row).await
    }

    pub async fn cats_by_name(&self) -> Result<Vec<CatRow>> {
        cats_by_name(&self.pool).await
    }

    pub async fn cat(&self, asset_id: Bytes32) -> Result<Option<CatRow>> {
        cat(&self.pool, asset_id).await
    }

    pub async fn unfetched_cat(&self) -> Result<Option<Bytes32>> {
        unfetched_cat(&self.pool).await
    }

    pub async fn spendable_cat_coins(&self, asset_id: Bytes32) -> Result<Vec<CatCoinRow>> {
        spendable_cat_coins(&self.pool, asset_id).await
    }

    pub async fn cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        cat_balance(&self.pool, asset_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn refetch_cat(&self, asset_id: Bytes32) -> Result<()> {
        refetch_cat(&self.pool, asset_id).await
    }

    pub async fn spendable_cat_coin_count(&self, asset_id: Bytes32) -> Result<u32> {
        spendable_cat_coin_count(&self.pool, asset_id).await
    }

    pub async fn coin_states_by_ids(
        &self,
        coin_ids: &[String],
    ) -> Result<Vec<EnhancedCoinStateRow>> {
        coin_states_by_ids(&self.pool, coin_ids).await
    }

    pub async fn cat_coin_states(
        &self,
        asset_id: Bytes32,
        limit: u32,
        offset: u32,
        sort_mode: CoinSortMode,
        ascending: bool,
        include_spent_coins: bool,
    ) -> Result<(Vec<EnhancedCoinStateRow>, u32)> {
        cat_coin_states(
            &self.pool,
            asset_id,
            limit,
            offset,
            sort_mode,
            ascending,
            include_spent_coins,
        )
        .await
    }

    pub async fn created_unspent_cat_coin_states(
        &self,
        asset_id: Bytes32,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_cat_coin_states(&self.pool, asset_id, limit, offset).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_cat(&mut self, row: CatRow) -> Result<()> {
        insert_cat(&mut *self.tx, row).await
    }

    pub async fn insert_cat_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(
            &mut *self.tx,
            coin_id,
            lineage_proof,
            p2_puzzle_hash,
            asset_id,
        )
        .await
    }
}

async fn insert_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `cats` (
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon`,
            `visible`,
            `fetched`
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.ticker,
        row.description,
        row.icon,
        row.visible,
        row.fetched
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn update_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `cats` (
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon`,
            `visible`,
            `fetched`
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.ticker,
        row.description,
        row.icon,
        row.visible,
        row.fetched
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn cats_by_name(conn: impl SqliteExecutor<'_>) -> Result<Vec<CatRow>> {
    let rows = sqlx::query_as!(
        CatSql,
        "
        SELECT `asset_id`, `name`, `ticker`, `description`, `icon`, `visible`, `fetched`
        FROM `cats` INDEXED BY `cat_name`
        ORDER BY `visible` DESC, `is_named` DESC, `name` ASC, `asset_id` ASC
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn cat(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<CatRow>> {
    let asset_id = asset_id.as_ref();

    let row = sqlx::query_as!(
        CatSql,
        "
        SELECT `asset_id`, `name`, `ticker`, `description`, `icon`, `visible`, `fetched`
        FROM `cats`
        WHERE `asset_id` = ?
        ",
        asset_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(into_row).transpose()
}

async fn unfetched_cat(conn: impl SqliteExecutor<'_>) -> Result<Option<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `asset_id` FROM `cats` WHERE `fetched` = 0
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?;
    rows.map(|row| to_bytes32(&row.asset_id)).transpose()
}

async fn insert_cat_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    p2_puzzle_hash: Bytes32,
    asset_id: Bytes32,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let asset_id = asset_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `cat_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `p2_puzzle_hash`,
            `asset_id`
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        p2_puzzle_hash,
        asset_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn spendable_cat_coins(
    conn: impl SqliteExecutor<'_>,
    asset_id: Bytes32,
) -> Result<Vec<CatCoinRow>> {
    let asset_id = asset_id.as_ref();

    let rows = sqlx::query_as!(
        CatCoinSql,
        "
        SELECT
            cs.`parent_coin_id`, cs.`puzzle_hash`, cs.`amount`, `p2_puzzle_hash`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`
        FROM `cat_coins` INDEXED BY `cat_asset_id`
        INNER JOIN `coin_states` AS cs ON `cat_coins`.`coin_id` = cs.`coin_id`
        LEFT JOIN `transaction_spends` ON cs.`coin_id` = `transaction_spends`.`coin_id`
        LEFT JOIN `offered_coins` ON cs.`coin_id` = `offered_coins`.`coin_id`
        LEFT JOIN `offers` ON `offered_coins`.`offer_id` = `offers`.`offer_id`
        WHERE `cat_coins`.`asset_id` = ?
        AND cs.`spent_height` IS NULL
        AND `transaction_spends`.`coin_id` IS NULL
        AND (`offered_coins`.`coin_id` IS NULL OR `offers`.`status` > 0)
        AND cs.`transaction_id` IS NULL
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn cat_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id = asset_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT `coin_states`.`amount` FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `cat_coins` ON `coin_states`.`coin_id` = `cat_coins`.`coin_id`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `coin_states`.`spent_height` IS NULL
        AND `cat_coins`.`asset_id` = ?
        AND `transaction_spends`.`coin_id` IS NULL
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    row.iter()
        .map(|row| Ok(u64::from_be_bytes(to_bytes(&row.amount)?) as u128))
        .sum::<Result<u128>>()
}

async fn spendable_cat_coin_count(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u32> {
    let asset_id = asset_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT COUNT(*) as count
        FROM coin_states
        INNER JOIN `cat_coins` ON `coin_states`.`coin_id` = `cat_coins`.`coin_id`
        LEFT JOIN transaction_spends ON transaction_spends.coin_id = coin_states.coin_id
        WHERE 1=1 
        AND `cat_coins`.`asset_id` = ?
        AND created_height IS NOT NULL
        AND spent_height IS NULL
        AND coin_states.transaction_id IS NULL
        AND transaction_spends.coin_id IS NULL
        ",
        asset_id
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count.try_into()?)
}

async fn coin_states_by_ids(
    conn: impl SqliteExecutor<'_>,
    coin_ids: &[String],
) -> Result<Vec<EnhancedCoinStateRow>> {
    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`, `spent_height`, `created_height`, 
               `coin_states`.`transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`,
               `offered_coins`.offer_id, `transaction_spends`.transaction_id as spend_transaction_id
        FROM `coin_states` 
        LEFT JOIN `offered_coins` ON `coin_states`.coin_id = `offered_coins`.coin_id
        LEFT JOIN `transaction_spends` ON `coin_states`.coin_id = `transaction_spends`.coin_id
        WHERE 1=1 
        AND coin_states.coin_id IN (",
    );

    let mut separated = query.separated(", ");
    for coin_id in coin_ids {
        separated.push(format!("X'{coin_id}'"));
    }
    separated.push_unseparated(")");
    let rows = query.build().fetch_all(conn).await?;

    if rows.is_empty() {
        return Ok(vec![]);
    }

    let mut coin_states = Vec::with_capacity(rows.len());
    for row in rows {
        let sql = CoinStateSql {
            parent_coin_id: row.try_get("parent_coin_id")?,
            puzzle_hash: row.try_get("puzzle_hash")?,
            amount: row.try_get("amount")?,
            spent_height: row.try_get("spent_height")?,
            created_height: row.try_get("created_height")?,
            transaction_id: row.try_get("transaction_id")?,
            kind: row.try_get("kind")?,
            created_unixtime: row.try_get("created_unixtime")?,
            spent_unixtime: row.try_get("spent_unixtime")?,
        };

        let coin_state_row = into_row(sql)?;

        let mut enhanced_row = EnhancedCoinStateRow::from(coin_state_row);

        enhanced_row.offer_id = row.try_get("offer_id").ok();
        enhanced_row.spend_transaction_id = row.try_get("spend_transaction_id").ok();

        coin_states.push(enhanced_row);
    }

    Ok(coin_states)
}

async fn cat_coin_states(
    conn: impl SqliteExecutor<'_> + Clone,
    asset_id: Bytes32,
    limit: u32,
    offset: u32,
    sort_mode: CoinSortMode,
    ascending: bool,
    include_spent_coins: bool,
) -> Result<(Vec<EnhancedCoinStateRow>, u32)> {
    let asset_id = asset_id.as_ref();

    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`, `spent_height`, `created_height`, 
               `coin_states`.`transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`,
               `offered_coins`.offer_id, `transaction_spends`.transaction_id as spend_transaction_id,
                COUNT(*) OVER() as total_count
        FROM `cat_coins` INDEXED BY `cat_asset_id`
        INNER JOIN `coin_states` ON `coin_states`.coin_id = `cat_coins`.coin_id
        LEFT JOIN `offered_coins` ON `coin_states`.coin_id = `offered_coins`.coin_id
        LEFT JOIN `transaction_spends` ON `coin_states`.coin_id = `transaction_spends`.coin_id
        WHERE `asset_id` = ",
    );
    query.push_bind(asset_id);

    if !include_spent_coins {
        query.push(" AND `spent_height` IS NULL");
    }

    query.push(" ORDER BY ");

    match sort_mode {
        CoinSortMode::CoinId => {
            query.push("`coin_states`.`coin_id`");
        }
        CoinSortMode::Amount => {
            query.push("`coin_states`.`amount`");
        }
        CoinSortMode::CreatedHeight => {
            query.push("`coin_states`.`created_height`");
        }
        CoinSortMode::SpentHeight => {
            query.push("`coin_states`.`spent_height`");
        }
    }

    if ascending {
        query.push(" ASC");
    } else {
        query.push(" DESC");
    }

    query.push(" LIMIT ");
    query.push_bind(limit);
    query.push(" OFFSET ");
    query.push_bind(offset);

    let rows = query.build().fetch_all(conn).await?;

    let Some(first_row) = rows.first() else {
        return Ok((vec![], 0));
    };

    let total: u32 = first_row.try_get("total_count")?;

    let mut coin_states = Vec::with_capacity(rows.len());

    for row in rows {
        let sql = CoinStateSql {
            parent_coin_id: row.try_get("parent_coin_id")?,
            puzzle_hash: row.try_get("puzzle_hash")?,
            amount: row.try_get("amount")?,
            spent_height: row.try_get("spent_height")?,
            created_height: row.try_get("created_height")?,
            transaction_id: row.try_get("transaction_id")?,
            kind: row.try_get("kind")?,
            created_unixtime: row.try_get("created_unixtime")?,
            spent_unixtime: row.try_get("spent_unixtime")?,
        };

        let coin_state_row = into_row(sql)?;

        let mut enhanced_row = EnhancedCoinStateRow::from(coin_state_row);

        enhanced_row.offer_id = row.try_get("offer_id").ok();
        enhanced_row.spend_transaction_id = row.try_get("spend_transaction_id").ok();

        coin_states.push(enhanced_row);
    }

    Ok((coin_states, total))
}

async fn created_unspent_cat_coin_states(
    conn: impl SqliteExecutor<'_>,
    asset_id: Bytes32,
    limit: u32,
    offset: u32,
) -> Result<Vec<CoinStateRow>> {
    let asset_id = asset_id.as_ref();

    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
        FROM `coin_states`
        INNER JOIN `cat_coins` ON `coin_states`.coin_id = `cat_coins`.coin_id
        WHERE `asset_id` = ?
        AND `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        ORDER BY `created_height`, `coin_states`.`coin_id` LIMIT ? OFFSET ?
        ",
        asset_id,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id = coin_id.as_ref();

    let row = sqlx::query_as!(
        FullCatCoinSql,
        "
        SELECT
            `parent_coin_id`, `puzzle_hash`, `amount`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`,
            `asset_id`, `p2_puzzle_hash`
        FROM `coin_states`
        INNER JOIN `cat_coins` ON `coin_states`.`coin_id` = `cat_coins`.`coin_id`
        WHERE `coin_states`.`coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(into_row).transpose()
}

async fn refetch_cat(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<()> {
    let asset_id = asset_id.as_ref();

    sqlx::query!(
        "
        UPDATE `cats` SET `fetched` = 0 WHERE `asset_id` = ?
        ",
        asset_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
