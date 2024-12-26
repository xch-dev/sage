use std::cmp::Reverse;

use chia::protocol::{Bytes32, CoinState};
use sqlx::SqliteExecutor;

use crate::{
    into_row, to_bytes32, CoinKind, CoinStateRow, CoinStateSql, Database, DatabaseTx, IntoRow,
    Result,
};

impl Database {
    pub async fn unsynced_coin_states(&self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coin_states(&self.pool, limit).await
    }

    pub async fn coin_state(&self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&self.pool, coin_id).await
    }

    pub async fn unspent_nft_coin_ids(&self) -> Result<Vec<Bytes32>> {
        unspent_nft_coin_ids(&self.pool).await
    }

    pub async fn unspent_did_coin_ids(&self) -> Result<Vec<Bytes32>> {
        unspent_did_coin_ids(&self.pool).await
    }

    pub async fn unspent_cat_coin_ids(&self) -> Result<Vec<Bytes32>> {
        unspent_cat_coin_ids(&self.pool).await
    }

    pub async fn delete_coin_state(&self, coin_id: Bytes32) -> Result<()> {
        delete_coin_state(&self.pool, coin_id).await
    }

    pub async fn total_coin_count(&self) -> Result<u32> {
        total_coin_count(&self.pool).await
    }

    pub async fn synced_coin_count(&self) -> Result<u32> {
        synced_coin_count(&self.pool).await
    }

    pub async fn sync_coin(
        &self,
        coin_id: Bytes32,
        hint: Option<Bytes32>,
        kind: CoinKind,
    ) -> Result<()> {
        sync_coin(&self.pool, coin_id, hint, kind).await
    }

    pub async fn is_coin_locked(&self, coin_id: Bytes32) -> Result<bool> {
        is_coin_locked(&self.pool, coin_id).await
    }

    pub async fn get_block_heights(&self) -> Result<Vec<u32>> {
        get_block_heights(&self.pool).await
    }

    pub async fn get_coin_states_by_created_height(
        &self,
        height: u32,
    ) -> Result<Vec<CoinStateRow>> {
        get_coin_states_by_created_height(&self.pool, height).await
    }

    pub async fn get_coin_states_by_spent_height(&self, height: u32) -> Result<Vec<CoinStateRow>> {
        get_coin_states_by_spent_height(&self.pool, height).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_coin_state(
        &mut self,
        coin_state: CoinState,
        synced: bool,
        transaction_id: Option<Bytes32>,
    ) -> Result<()> {
        insert_coin_state(&mut *self.tx, coin_state, synced, transaction_id).await
    }

    pub async fn update_coin_state(
        &mut self,
        coin_id: Bytes32,
        created_height: Option<u32>,
        spent_height: Option<u32>,
        transaction_id: Option<Bytes32>,
    ) -> Result<()> {
        update_coin_state(
            &mut *self.tx,
            coin_id,
            created_height,
            spent_height,
            transaction_id,
        )
        .await
    }

    pub async fn sync_coin(
        &mut self,
        coin_id: Bytes32,
        hint: Option<Bytes32>,
        kind: CoinKind,
    ) -> Result<()> {
        sync_coin(&mut *self.tx, coin_id, hint, kind).await
    }

    pub async fn unsync_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        unsync_coin(&mut *self.tx, coin_id).await
    }

    pub async fn remove_coin_transaction_id(&mut self, coin_id: Bytes32) -> Result<()> {
        remove_coin_transaction_id(&mut *self.tx, coin_id).await
    }

    pub async fn is_p2_coin(&mut self, coin_id: Bytes32) -> Result<Option<bool>> {
        is_p2_coin(&mut *self.tx, coin_id).await
    }
}

async fn insert_coin_state(
    conn: impl SqliteExecutor<'_>,
    coin_state: CoinState,
    synced: bool,
    transaction_id: Option<Bytes32>,
) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();
    let coin_id_ref = coin_id.as_ref();
    let parent_coin_id = coin_state.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_state.coin.puzzle_hash.as_ref();
    let amount = coin_state.coin.amount.to_be_bytes();
    let amount_ref = amount.as_ref();
    let transaction_id = transaction_id.as_deref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `coin_states` (
            `coin_id`,
            `parent_coin_id`,
            `puzzle_hash`,
            `amount`,
            `created_height`,
            `spent_height`,
            `synced`,
            `transaction_id`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id_ref,
        parent_coin_id,
        puzzle_hash,
        amount_ref,
        coin_state.created_height,
        coin_state.spent_height,
        synced,
        transaction_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn update_coin_state(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    created_height: Option<u32>,
    spent_height: Option<u32>,
    transaction_id: Option<Bytes32>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let transaction_id = transaction_id.as_deref();

    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `created_height` = ?, `spent_height` = ?, `transaction_id` = ?
        WHERE `coin_id` = ?
        ",
        created_height,
        spent_height,
        transaction_id,
        coin_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn delete_coin_state(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    sqlx::query!(
        "
        DELETE FROM `coin_states`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn unsynced_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: usize,
) -> Result<Vec<CoinState>> {
    let limit: i64 = limit.try_into()?;
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`
        FROM `coin_states`
        WHERE `synced` = 0 AND `created_height` IS NOT NULL
        ORDER BY `spent_height` ASC
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|sql| sql.into_row().map(|row| row.coin_state))
        .collect()
}

async fn sync_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    hint: Option<Bytes32>,
    kind: CoinKind,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let kind = kind as u32;
    let hint = hint.as_deref();

    sqlx::query!(
        "
        UPDATE `coin_states` SET `synced` = 1, `hint` = ?, `kind` = ? WHERE `coin_id` = ?
        ",
        hint,
        kind,
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn unsync_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "UPDATE `coin_states` SET `synced` = 0 WHERE `coin_id` = ?",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn remove_coin_transaction_id(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `transaction_id` = NULL
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn total_coin_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `total`
        FROM `coin_states`
        "
    )
    .fetch_one(conn)
    .await?;
    Ok(row.total.try_into()?)
}

async fn synced_coin_count(conn: impl SqliteExecutor<'_>) -> Result<u32> {
    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `synced`
        FROM `coin_states`
        WHERE `synced` = 1
        "
    )
    .fetch_one(conn)
    .await?;
    Ok(row.synced.try_into()?)
}

async fn coin_state(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<CoinState>> {
    let coin_id = coin_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`
        FROM `coin_states`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(sql.into_row()?.coin_state))
}

async fn unspent_nft_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `coin_states`.`coin_id`
        FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `nft_coins` ON `coin_states`.`coin_id` = `nft_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}

async fn unspent_did_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `coin_states`.`coin_id`
        FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `did_coins` ON `coin_states`.`coin_id` = `did_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}

async fn unspent_cat_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `coin_states`.`coin_id`
        FROM `coin_states` INDEXED BY `coin_spent`
        INNER JOIN `cat_coins` ON `coin_states`.`coin_id` = `cat_coins`.`coin_id`
        WHERE `spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.coin_id))
        .collect()
}

async fn is_p2_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<bool>> {
    let coin_id = coin_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT `kind`
        FROM `coin_states`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|row| row.kind == 1))
}

async fn is_coin_locked(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<bool> {
    let coin_id = coin_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT COUNT(*) AS `count`
        FROM `coin_states`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        LEFT JOIN `offered_coins` ON `coin_states`.`coin_id` = `offered_coins`.`coin_id`
        WHERE `coin_states`.`coin_id` = ?
        AND `offer_id` IS NULL
        AND `coin_states`.`transaction_id` IS NULL
        AND `transaction_spends`.`transaction_id` IS NULL
        ",
        coin_id
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count > 0)
}

async fn get_block_heights(conn: impl SqliteExecutor<'_>) -> Result<Vec<u32>> {
    let rows = sqlx::query!(
        "
        SELECT DISTINCT height FROM (
            SELECT created_height as height FROM coin_states INDEXED BY `coin_created`
            WHERE created_height IS NOT NULL
            UNION ALL
            SELECT spent_height as height FROM coin_states INDEXED BY `coin_spent`
            WHERE spent_height IS NOT NULL
        )
        GROUP BY height
        "
    )
    .fetch_all(conn)
    .await?;

    let mut heights = Vec::with_capacity(rows.len());

    for row in rows {
        if let Some(height) = row.height {
            heights.push(height.try_into()?);
        }
    }

    heights.sort_by_key(|height| Reverse(*height));

    Ok(heights)
}

async fn get_coin_states_by_created_height(
    conn: impl SqliteExecutor<'_>,
    height: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`
        FROM `coin_states` INDEXED BY `coin_created`
        WHERE `created_height` = ?
        ",
        height
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn get_coin_states_by_spent_height(
    conn: impl SqliteExecutor<'_>,
    height: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`
        FROM `coin_states` INDEXED BY `coin_spent`
        WHERE `spent_height` = ?
        ",
        height
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}
