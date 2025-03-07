use chia::protocol::{Bytes32, CoinState};
use sqlx::Row;
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

    pub async fn full_coin_state(&self, coin_id: Bytes32) -> Result<Option<CoinStateRow>> {
        full_coin_state(&self.pool, coin_id).await
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

    pub async fn get_block_heights(
        &self,
        offset: u32,
        limit: u32,
        asc: bool,
        find_value: Option<String>,
    ) -> Result<(Vec<u32>, u32)> {
        get_block_heights(&self.pool, offset, limit, asc, find_value).await
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

impl DatabaseTx<'_> {
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
        LEFT JOIN `offers` ON `offered_coins`.`offer_id` = `offers`.`offer_id`
        WHERE `coin_states`.`coin_id` = ?
        AND (`offers`.`offer_id` IS NULL OR `offers`.`status` > 0)
        AND `coin_states`.`transaction_id` IS NULL
        AND `transaction_spends`.`transaction_id` IS NULL
        ",
        coin_id
    )
    .fetch_one(conn)
    .await?;

    Ok(row.count > 0)
}

async fn get_block_heights(
    conn: impl SqliteExecutor<'_>,
    offset: u32,
    limit: u32,
    asc: bool,
    find_value: Option<String>,
) -> Result<(Vec<u32>, u32)> {
    let mut query = sqlx::QueryBuilder::new(
        "
        WITH filtered_coins AS (
            SELECT cs.coin_id,  
                cs.kind, 
                cats.ticker,
                cats.name as cat_name,
                dids.name as did_name,
                nfts.name as nft_name,
                cs.created_height as height
            FROM coin_states cs
                LEFT JOIN cat_coins ON cs.coin_id = cat_coins.coin_id
                LEFT JOIN cats ON cat_coins.asset_id = cats.asset_id
                LEFT JOIN did_coins ON cs.coin_id = did_coins.coin_id
                LEFT JOIN dids ON did_coins.coin_id = dids.coin_id
                LEFT JOIN nft_coins ON cs.coin_id = nft_coins.coin_id
                LEFT JOIN nfts ON nft_coins.coin_id = nfts.coin_id
            WHERE cs.created_height IS NOT NULL
            UNION ALL
            SELECT cs.coin_id, 
                cs.kind,
                cats.ticker,
                cats.name as cat_name,
                dids.name as did_name,
                nfts.name as nft_name,
                cs.spent_height as height
            FROM coin_states cs
                LEFT JOIN cat_coins ON cs.coin_id = cat_coins.coin_id
                LEFT JOIN cats ON cat_coins.asset_id = cats.asset_id
                LEFT JOIN did_coins ON cs.coin_id = did_coins.coin_id
                LEFT JOIN dids ON did_coins.coin_id = dids.coin_id
                LEFT JOIN nft_coins ON cs.coin_id = nft_coins.coin_id
                LEFT JOIN nfts ON nft_coins.coin_id = nfts.coin_id
            WHERE cs.spent_height IS NOT NULL
        ),
        filtered_heights AS (
            SELECT DISTINCT height
            FROM filtered_coins
            WHERE 1=1
        ",
    );

    if let Some(value) = &find_value {
        // Check if searching for XCH (matches "x", "xc", or "xch")
        let should_filter_xch = if value.len() <= 3 {
            let value_lower = value.to_lowercase();
            value_lower == "x" || value_lower == "xc" || value_lower == "xch"
        } else {
            false
        };

        query.push(" AND (");

        if should_filter_xch {
            // XCH coins have kind = 1 (standard P2 coins)
            query.push("kind = 1 OR ");
        }

        query
            .push("ticker LIKE ")
            .push_bind(format!("%{value}%"))
            .push(" OR cat_name LIKE ")
            .push_bind(format!("%{value}%"))
            .push(" OR did_name LIKE ")
            .push_bind(format!("%{value}%"))
            .push(" OR nft_name LIKE ")
            .push_bind(format!("%{value}%"))
            .push(")");
    }

    query.push(")");

    // Select both the paginated results and the total count
    query.push(
        "
        SELECT height, COUNT(*) OVER() as total_count
        FROM filtered_heights 
        ORDER BY height ",
    );

    query.push(if asc { "ASC" } else { "DESC" });
    query.push(" LIMIT ? OFFSET ?");

    let built_query = query.build();
    let rows = built_query.bind(limit).bind(offset).fetch_all(conn).await?;

    let mut heights = Vec::with_capacity(rows.len());
    let mut total_count = 0;

    for row in rows {
        if let Some(height) = row.get::<Option<i64>, _>(0) {
            heights.push(height.try_into()?);
        }
        // Get the total count from the first row (it will be the same in all rows)
        if total_count == 0 {
            total_count = row.get::<i64, _>(1).try_into()?;
        }
    }

    Ok((heights, total_count))
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

async fn full_coin_state(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<CoinStateRow>> {
    let coin_id = coin_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`
        FROM `coin_states` WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(sql.into_row()?))
}
