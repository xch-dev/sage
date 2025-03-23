use chia::{
    protocol::{Bytes32, Program},
    puzzles::LineageProof,
};
use chia_wallet_sdk::driver::{Did, DidInfo};
use sqlx::SqliteExecutor;

use crate::{
    into_row, CoinStateRow, CoinStateSql, Database, DatabaseTx, DidCoinInfo, DidCoinInfoSql,
    DidRow, DidSql, FullDidCoinSql, IntoRow, Result,
};

impl Database {
    pub async fn insert_did(&self, row: DidRow) -> Result<()> {
        insert_did(&self.pool, row).await
    }

    pub async fn dids_by_name(&self) -> Result<Vec<DidRow>> {
        dids_by_name(&self.pool).await
    }

    pub async fn did_row(&self, launcher_id: Bytes32) -> Result<Option<DidRow>> {
        did_row(&self.pool, launcher_id).await
    }

    pub async fn did_coin_info(&self, coin_id: Bytes32) -> Result<Option<DidCoinInfo>> {
        did_coin_info(&self.pool, coin_id).await
    }

    pub async fn spendable_did(&self, did_id: Bytes32) -> Result<Option<Did<Program>>> {
        spendable_did(&self.pool, did_id).await
    }

    pub async fn did_name(&self, launcher_id: Bytes32) -> Result<Option<String>> {
        did_name(&self.pool, launcher_id).await
    }

    pub async fn set_future_did_name(&self, launcher_id: Bytes32, name: String) -> Result<()> {
        set_future_did_name(&self.pool, launcher_id, name).await
    }

    pub async fn created_unspent_did_coin_states(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_did_coin_states(&self.pool, limit, offset).await
    }

    pub async fn created_unspent_did_coin_state(
        &self,
        launcher_id: Bytes32,
    ) -> Result<Vec<CoinStateRow>> {
        created_unspent_did_coin_state(&self.pool, launcher_id).await
    }

    pub async fn did_by_coin_id(&self, coin_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_by_coin_id(&self.pool, coin_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_did(&mut self, row: DidRow) -> Result<()> {
        insert_did(&mut *self.tx, row).await
    }

    pub async fn insert_did_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        did_info: DidInfo<Program>,
    ) -> Result<()> {
        insert_did_coin(&mut *self.tx, coin_id, lineage_proof, did_info).await
    }

    pub async fn delete_future_did_name(&mut self, launcher_id: Bytes32) -> Result<()> {
        delete_future_did_name(&mut *self.tx, launcher_id).await
    }

    pub async fn get_future_did_name(&mut self, launcher_id: Bytes32) -> Result<Option<String>> {
        get_future_did_name(&mut *self.tx, launcher_id).await
    }

    pub async fn did_row(&mut self, launcher_id: Bytes32) -> Result<Option<DidRow>> {
        did_row(&mut *self.tx, launcher_id).await
    }

    pub async fn did_row_by_coin(&mut self, coin_id: Bytes32) -> Result<Option<DidRow>> {
        did_row_by_coin(&mut *self.tx, coin_id).await
    }

    pub async fn set_did_not_owned(&mut self, coin_id: Bytes32) -> Result<()> {
        set_did_not_owned(&mut *self.tx, coin_id).await
    }

    pub async fn set_did_created_height(
        &mut self,
        coin_id: Bytes32,
        height: Option<u32>,
    ) -> Result<()> {
        set_did_created_height(&mut *self.tx, coin_id, height).await
    }
}

async fn insert_did(conn: impl SqliteExecutor<'_>, row: DidRow) -> Result<()> {
    let launcher_id = row.launcher_id.as_ref();
    let coin_id = row.coin_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `dids` (
            `launcher_id`,
            `coin_id`,
            `name`,
            `is_owned`,
            `visible`,
            `created_height`
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        launcher_id,
        coin_id,
        row.name,
        row.is_owned,
        row.visible,
        row.created_height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_did_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    did_info: DidInfo<Program>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = did_info.launcher_id.as_ref();
    let recovery_list_hash = did_info.recovery_list_hash.as_deref();
    let num_verifications_required = did_info.num_verifications_required.to_be_bytes();
    let num_verifications_required = num_verifications_required.as_ref();
    let metadata = did_info.metadata.as_ref();
    let p2_puzzle_hash = did_info.p2_puzzle_hash.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `did_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `launcher_id`,
            `recovery_list_hash`,
            `num_verifications_required`,
            `metadata`,
            `p2_puzzle_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        launcher_id,
        recovery_list_hash,
        num_verifications_required,
        metadata,
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn dids_by_name(conn: impl SqliteExecutor<'_>) -> Result<Vec<DidRow>> {
    sqlx::query_as!(
        DidSql,
        "
        SELECT `launcher_id`, `coin_id`, `name`, `is_owned`, `visible`, `created_height`
        FROM `dids` INDEXED BY `did_name`
        WHERE `is_owned` = 1
        ORDER BY `visible` DESC, `is_pending` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
}

async fn did_row(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<DidRow>> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query_as!(
        DidSql,
        "
        SELECT `launcher_id`, `coin_id`, `name`, `is_owned`, `visible`, `created_height`
        FROM `dids`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    .map(into_row)
    .transpose()
}

async fn did_row_by_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<DidRow>> {
    let coin_id = coin_id.as_ref();

    sqlx::query_as!(
        DidSql,
        "
        SELECT `launcher_id`, `coin_id`, `name`, `is_owned`, `visible`, `created_height`
        FROM `dids`
        WHERE `coin_id` = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    .map(into_row)
    .transpose()
}

async fn set_did_not_owned(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "UPDATE `dids` SET `is_owned` = 0 WHERE `coin_id` = ?",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn set_did_created_height(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    height: Option<u32>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "UPDATE `dids` SET `created_height` = ? WHERE `coin_id` = ?",
        height,
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn did_coin_info(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<DidCoinInfo>> {
    let coin_id = coin_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        DidCoinInfoSql,
        "
        SELECT
            `did_coins`.`coin_id`, `amount`, `p2_puzzle_hash`,
            `recovery_list_hash`, `created_height`, `transaction_id`
        FROM `did_coins`
        INNER JOIN `coin_states` ON `coin_states`.coin_id = `did_coins`.coin_id
        WHERE `did_coins`.`coin_id` = ?
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

async fn spendable_did(
    conn: impl SqliteExecutor<'_>,
    did_id: Bytes32,
) -> Result<Option<Did<Program>>> {
    let did_id = did_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        FullDidCoinSql,
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did ON cs.coin_id = did.coin_id
        LEFT JOIN `transaction_spends` ON cs.coin_id = transaction_spends.coin_id
        WHERE did.launcher_id = ?
        AND cs.spent_height IS NULL
        AND cs.created_height IS NOT NULL
        AND cs.transaction_id IS NULL
        AND transaction_spends.transaction_id IS NULL
        ",
        did_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(sql.into_row()?))
}

async fn did_name(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<String>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT `name`
        FROM `dids`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(row.name)
}

async fn set_future_did_name(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
    name: String,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `future_did_names` (`launcher_id`, `name`)
        VALUES (?, ?)
        ",
        launcher_id,
        name
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn get_future_did_name(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<String>> {
    let launcher_id = launcher_id.as_ref();

    Ok(sqlx::query!(
        "
        SELECT `name` FROM `future_did_names`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.name))
}

async fn delete_future_did_name(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<()> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query!(
        "
        DELETE FROM `future_did_names` WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn created_unspent_did_coin_states(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<CoinStateRow>> {
    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
        FROM `coin_states`
        INNER JOIN `did_coins` ON `coin_states`.coin_id = `did_coins`.coin_id
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

async fn created_unspent_did_coin_state(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Vec<CoinStateRow>> {
    let launcher_id = launcher_id.as_ref();

    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`, `kind`, `created_unixtime`, `spent_unixtime`
        FROM `did_coins`
        INNER JOIN `coin_states` ON `coin_states`.coin_id = `did_coins`.coin_id
        WHERE `launcher_id` = ?
        AND `spent_height` IS NULL
        AND `created_height` IS NOT NULL
        ",
        launcher_id,
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(into_row).collect()
}

async fn did_by_coin_id(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<Did<Program>>> {
    let coin_id = coin_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        FullDidCoinSql,
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did ON cs.coin_id = did.coin_id
        WHERE cs.coin_id = ?
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
