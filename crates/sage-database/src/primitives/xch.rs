use chia::protocol::{Bytes32, Coin};
use sqlx::SqliteExecutor;

use crate::{
    into_row, to_bytes, CoinSql, CoinStateRow, CoinStateSql, Database, DatabaseTx, Result,
};

impl Database {
    pub async fn spendable_coins(&self) -> Result<Vec<Coin>> {
        spendable_coins(&self.pool).await
    }

    pub async fn balance(&self) -> Result<u128> {
        balance(&self.pool).await
    }

    pub async fn p2_coin_states(&self) -> Result<Vec<CoinStateRow>> {
        p2_coin_states(&self.pool).await
    }

    pub async fn created_unspent_p2_coin_states(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_p2_coin_states(&self.pool, limit, offset).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_p2_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_p2_coin(&mut *self.tx, coin_id).await
    }
}

async fn insert_p2_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `p2_coins` (`coin_id`) VALUES (?)
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
        SELECT `coin_states`.`amount` FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `p2_coins` ON `coin_states`.`coin_id` = `p2_coins`.`coin_id`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `coin_states`.`spent_height` IS NULL
        AND `transaction_spends`.`coin_id` IS NULL
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
        INNER JOIN `p2_coins` ON `coin_states`.`coin_id` = `p2_coins`.`coin_id`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        LEFT JOIN `offered_coins` ON `coin_states`.`coin_id` = `offered_coins`.`coin_id`
        LEFT JOIN `offers` ON `offered_coins`.`offer_id` = `offers`.`offer_id`
        WHERE `coin_states`.`spent_height` IS NULL
        AND `transaction_spends`.`coin_id` IS NULL
        AND (`offered_coins`.`coin_id` IS NULL OR `offers`.`status` > 0)
        AND `coin_states`.`transaction_id` IS NULL
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn p2_coin_states(conn: impl SqliteExecutor<'_>) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`
        FROM `coin_states`
        INNER JOIN `p2_coins` ON `coin_states`.`coin_id` = `p2_coins`.`coin_id`
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn created_unspent_p2_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`
        FROM `coin_states`
        INNER JOIN `p2_coins` ON `coin_states`.`coin_id` = `p2_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        ORDER BY `created_height`, `coin_states`.`coin_id` LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}
