use chia::protocol::{Bytes32, Coin};
use sqlx::{Row, SqliteExecutor};

use crate::{
    into_row, to_bytes, CoinSortMode, CoinSql, CoinStateRow, CoinStateSql, Database, DatabaseTx,
    Result,
};

impl Database {
    pub async fn spendable_coins(&self) -> Result<Vec<Coin>> {
        spendable_coins(&self.pool).await
    }

    pub async fn balance(&self) -> Result<u128> {
        balance(&self.pool).await
    }

    pub async fn p2_coin_states(
        &self,
        limit: u32,
        offset: u32,
        sort_mode: CoinSortMode,
        ascending: bool,
        include_spent_coins: bool,
    ) -> Result<(Vec<CoinStateRow>, u32)> {
        p2_coin_states(
            &self.pool,
            limit,
            offset,
            sort_mode,
            ascending,
            include_spent_coins,
        )
        .await
    }

    pub async fn created_unspent_p2_coin_states(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_p2_coin_states(&self.pool, limit, offset).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_p2_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_p2_coin(&mut *self.tx, coin_id).await
    }
}

async fn insert_p2_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        UPDATE `coin_states` SET `kind` = 1 WHERE `coin_id` = ?
        ",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn balance(conn: impl SqliteExecutor<'_>) -> Result<u128> {
    let row = sqlx::query!(
        "
        SELECT `coin_states`.`amount` FROM `coin_states` INDEXED BY `coin_kind_spent`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `coin_states`.`spent_height` IS NULL
        AND `transaction_spends`.`coin_id` IS NULL
        AND `kind` = 1
        "
    )
    .fetch_all(conn)
    .await?;

    row.iter()
        .map(|row| Ok(u64::from_be_bytes(to_bytes(&row.amount)?) as u128))
        .sum::<Result<u128>>()
}

async fn spendable_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Coin>> {
    sqlx::query_as!(
        CoinSql,
        "
        SELECT `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount` FROM `coin_states`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        LEFT JOIN `offered_coins` ON `coin_states`.`coin_id` = `offered_coins`.`coin_id`
        LEFT JOIN `offers` ON `offered_coins`.`offer_id` = `offers`.`offer_id`
        WHERE `coin_states`.`spent_height` IS NULL
        AND `transaction_spends`.`coin_id` IS NULL
        AND (`offered_coins`.`coin_id` IS NULL OR `offers`.`status` > 0)
        AND `coin_states`.`transaction_id` IS NULL
        AND `kind` = 1
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn p2_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
    sort_mode: CoinSortMode,
    ascending: bool,
    include_spent_coins: bool,
) -> Result<(Vec<CoinStateRow>, u32)> {
    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT 
            `parent_coin_id`, 
            `puzzle_hash`, 
            `amount`, 
            `spent_height`, 
            `created_height`, 
            `transaction_id`, 
            `kind`, 
            `created_unixtime`, 
            `spent_unixtime`,
            COUNT(*) OVER() as total_count
        FROM `coin_states` 
        WHERE `kind` = 1
        ",
    );

    if !include_spent_coins {
        query.push(" AND `spent_height` IS NULL");
    }

    query.push(" ORDER BY ");

    match sort_mode {
        CoinSortMode::CoinId => {
            query.push("`coin_id`");
        }
        CoinSortMode::Amount => {
            query.push("`amount`");
        }
        CoinSortMode::CreatedHeight => {
            query.push("`created_height`");
        }
        CoinSortMode::SpentHeight => {
            query.push("`spent_height`");
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

    if rows.is_empty() {
        return Ok((vec![], 0));
    }

    let total: u32 = rows.first().unwrap().try_get("total_count")?;
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
        coin_states.push(into_row(sql)?);
    }

    Ok((coin_states, total))
}

async fn created_unspent_p2_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
        FROM `coin_states`
        WHERE `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        AND `kind` = 1
        ORDER BY `created_height`, `coin_states`.`coin_id` LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}
