use chia::protocol::{Bytes32, CoinState};
use sqlx::SqliteExecutor;
use sqlx::{sqlite::SqliteRow, Row};

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

    pub async fn get_transaction_coins(
        &self,
        offset: u32,
        limit: u32,
        asc: bool,
        find_value: Option<String>,
    ) -> Result<(Vec<SqliteRow>, u32)> {
        get_transaction_coins(&self.pool, offset, limit, asc, find_value).await
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

    pub async fn get_are_coins_spendable(&self, coin_ids: &[String]) -> Result<bool> {
        get_are_coins_spendable(&self.pool, coin_ids).await
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
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
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
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
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

async fn get_transaction_coins(
    conn: impl SqliteExecutor<'_>,
    offset: u32,
    limit: u32,
    asc: bool,
    find_value: Option<String>,
) -> Result<(Vec<SqliteRow>, u32)> {
    let mut query = sqlx::QueryBuilder::new(
        "
        WITH coin_states_with_heights AS (
            SELECT 
                cs.coin_id,
                cs.created_height as height
            FROM coin_states cs
            WHERE cs.created_height IS NOT NULL
            
            UNION ALL
            
            SELECT 
                cs.coin_id,
                cs.spent_height as height
            FROM coin_states cs
            WHERE cs.spent_height IS NOT NULL
        ),
        joined_coin_states AS (
            SELECT 
                DISTINCT h.height
            FROM coin_states_with_heights h
                LEFT JOIN cat_coins ON h.coin_id = cat_coins.coin_id
                LEFT JOIN cats ON cat_coins.asset_id = cats.asset_id
                LEFT JOIN did_coins ON h.coin_id = did_coins.coin_id
                LEFT JOIN dids ON did_coins.coin_id = dids.coin_id
                LEFT JOIN nft_coins ON h.coin_id = nft_coins.coin_id
                LEFT JOIN nfts ON nft_coins.coin_id = nfts.coin_id
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

        if is_valid_asset_id(value) {
            query.push("cats.asset_id = X'").push(value).push("' OR ");
        } else if is_valid_address(value, "nft") {
            if let Some(puzzle_hash) = puzzle_hash_from_address(value) {
                query
                    .push("nfts.launcher_id = X'")
                    .push(puzzle_hash)
                    .push("' OR ");
            }
        } else if is_valid_address(value, "did:chia:") {
            if let Some(puzzle_hash) = puzzle_hash_from_address(value) {
                query
                    .push("dids.launcher_id = X'")
                    .push(puzzle_hash)
                    .push("' OR ");
            }
        }

        if is_valid_height(value) {
            query.push("height = ").push(value).push(" OR ");
        }

        query
            .push("ticker LIKE ")
            .push_bind(format!("%{value}%"))
            .push(" OR cats.name LIKE ")
            .push_bind(format!("%{value}%"))
            .push(" OR dids.name LIKE ")
            .push_bind(format!("%{value}%"))
            .push(" OR nfts.name LIKE ")
            .push_bind(format!("%{value}%"))
            .push(")");
    }
    query.push(
        "
        ),
        paged_heights AS (
            SELECT 
                height,
                COUNT(*) OVER() as total_count
            FROM joined_coin_states
            ORDER BY height ",
    );
    query.push(if asc { "ASC" } else { "DESC" });
    query.push(" LIMIT ? OFFSET ? )");

    query.push("
        SELECT 
            paged_heights.total_count,
            'created' AS action_type,
            cs.created_height AS height,
            created_unixtime AS unixtime, 
            cs.coin_id,  
            cs.kind, 
            amount,
            COALESCE (cat_coins.p2_puzzle_hash, did_coins.p2_puzzle_hash, nft_coins.p2_puzzle_hash, puzzle_hash) AS p2_puzzle_hash,
            COALESCE (cats.name, nfts.name, dids.name, NULL) AS name,
            COALESCE (cats.asset_id, nfts.launcher_id, dids.launcher_id, NULL) AS item_id,
            nft_coins.metadata AS nft_metadata,
            cats.ticker,
            cats.icon AS cat_icon_url,
            nft_thumbnails.icon AS nft_icon,
            (SELECT COUNT(*)
                FROM derivations d 
                WHERE d.p2_puzzle_hash = COALESCE(cat_coins.p2_puzzle_hash, did_coins.p2_puzzle_hash, nft_coins.p2_puzzle_hash, cs.puzzle_hash)) AS derivation_count
        FROM coin_states cs
            INNER JOIN paged_heights ON cs.created_height = paged_heights.height
            LEFT JOIN cat_coins ON cs.coin_id = cat_coins.coin_id
            LEFT JOIN cats ON cat_coins.asset_id = cats.asset_id
            LEFT JOIN did_coins ON cs.coin_id = did_coins.coin_id
            LEFT JOIN dids ON did_coins.coin_id = dids.coin_id
            LEFT JOIN nft_coins ON cs.coin_id = nft_coins.coin_id
            LEFT JOIN nfts ON nft_coins.coin_id = nfts.coin_id
	        LEFT JOIN nft_thumbnails ON nft_coins.data_hash = nft_thumbnails.hash

        UNION ALL 

        SELECT 
            paged_heights.total_count,
            'spent' AS action_type,
            cs.spent_height AS height,
            spent_unixtime AS unixtime, 
            cs.coin_id,  
            cs.kind, 
            amount,
            COALESCE (cat_coins.p2_puzzle_hash, did_coins.p2_puzzle_hash, nft_coins.p2_puzzle_hash, puzzle_hash) AS p2_puzzle_hash,
            COALESCE (cats.name, nfts.name, dids.name, NULL) AS name,
            COALESCE (cats.asset_id, nfts.launcher_id, dids.launcher_id, NULL) AS item_id,
            nft_coins.metadata AS nft_metadata,
            cats.ticker,
            cats.icon AS cat_icon_url,
            nft_thumbnails.icon AS nft_icon,
            (SELECT COUNT(*)
                FROM derivations d 
                WHERE d.p2_puzzle_hash = COALESCE(cat_coins.p2_puzzle_hash, did_coins.p2_puzzle_hash, nft_coins.p2_puzzle_hash, cs.puzzle_hash)) AS derivation_count
        FROM coin_states cs
            INNER JOIN paged_heights ON cs.spent_height = paged_heights.height
            LEFT JOIN cat_coins ON cs.coin_id = cat_coins.coin_id
            LEFT JOIN cats ON cat_coins.asset_id = cats.asset_id
            LEFT JOIN did_coins ON cs.coin_id = did_coins.coin_id
            LEFT JOIN dids ON did_coins.coin_id = dids.coin_id
            LEFT JOIN nft_coins ON cs.coin_id = nft_coins.coin_id
            LEFT JOIN nfts ON nft_coins.coin_id = nfts.coin_id
            LEFT JOIN nft_thumbnails ON nft_coins.data_hash = nft_thumbnails.hash");
    let built_query = query.build();
    let rows = built_query.bind(limit).bind(offset).fetch_all(conn).await?;

    let Some(first_row) = rows.first() else {
        return Ok((vec![], 0));
    };

    let total: u32 = first_row.try_get("total_count")?;

    Ok((rows, total))
}

async fn get_coin_states_by_created_height(
    conn: impl SqliteExecutor<'_>,
    height: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
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
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
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
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `created_height`, `spent_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
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

/// Checks if the provided string is a valid asset ID (64 character hex string)
pub fn is_valid_asset_id(asset_id: &str) -> bool {
    asset_id.len() == 64 && asset_id.chars().all(|c| c.is_ascii_hexdigit())
}

/// Checks if a given address is valid for the specified prefix
pub fn is_valid_address(address: &str, prefix: &str) -> bool {
    if let Ok(decoded) = chia_wallet_sdk::utils::Address::decode(address) {
        // Check if the prefix matches and the puzzle hash is valid (32 bytes)
        decoded.prefix == prefix && decoded.puzzle_hash.as_ref().len() == 32
    } else {
        false
    }
}

/// Extracts puzzle hash from an address
fn puzzle_hash_from_address(address: &str) -> Option<String> {
    chia_wallet_sdk::utils::Address::decode(address)
        .map(|decoded| hex::encode(decoded.puzzle_hash.as_ref()))
        .ok()
}

/// Checks if a string is a valid block height (non-negative integer)
pub fn is_valid_height(height_str: &str) -> bool {
    height_str.parse::<u32>().is_ok()
}

async fn get_are_coins_spendable(
    conn: impl SqliteExecutor<'_>,
    coin_ids: &[String],
) -> Result<bool> {
    if coin_ids.is_empty() {
        return Ok(false);
    }

    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT COUNT(*)
        FROM coin_states
        LEFT JOIN transaction_spends ON transaction_spends.coin_id = coin_states.coin_id
        WHERE 1=1 
        AND created_height IS NOT NULL
        AND spent_height IS NULL
        AND coin_states.transaction_id IS NULL
        AND transaction_spends.coin_id IS NULL
        AND coin_states.coin_id IN (",
    );

    let mut separated = query.separated(", ");
    for coin_id in coin_ids {
        separated.push(format!("X'{coin_id}'"));
    }
    separated.push_unseparated(")");

    let count: i64 = query.build().fetch_one(conn).await?.get(0);

    #[allow(clippy::cast_possible_wrap)]
    Ok(count == coin_ids.len() as i64)
}
