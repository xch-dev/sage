use chia::{protocol::Bytes32, puzzles::LineageProof};
use chia_wallet_sdk::driver::{OptionContract, OptionInfo};
use sqlx::SqliteExecutor;

use crate::{into_row, Database, DatabaseTx, IntoRow, OptionCoinSql, OptionRow, OptionSql, Result};

#[derive(sqlx::FromRow)]
struct OptionSearchRow {
    #[sqlx(flatten)]
    option: OptionSql,
    total_count: i32,
}

impl Database {
    pub async fn option_by_coin_id(&self, coin_id: Bytes32) -> Result<Option<OptionContract>> {
        option_by_coin_id(&self.pool, coin_id).await
    }

    pub async fn options(
        &self,
        include_hidden: bool,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<OptionRow>, u32)> {
        options(&self.pool, include_hidden, limit, offset).await
    }

    pub async fn option(&self, launcher_id: Bytes32) -> Result<Option<OptionContract>> {
        option(&self.pool, launcher_id).await
    }

    pub async fn option_row(&self, launcher_id: Bytes32) -> Result<Option<OptionRow>> {
        option_row(&self.pool, launcher_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_option_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        info: OptionInfo,
    ) -> Result<()> {
        insert_option_coin(&mut *self.tx, coin_id, lineage_proof, info).await
    }

    pub async fn option_row(&mut self, launcher_id: Bytes32) -> Result<Option<OptionRow>> {
        option_row(&mut *self.tx, launcher_id).await
    }

    pub async fn insert_option(&mut self, row: OptionRow) -> Result<()> {
        insert_option(&mut *self.tx, row).await
    }
}

async fn insert_option_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    info: OptionInfo,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = info.launcher_id.as_ref();
    let underlying_coin_id = info.underlying_coin_id.as_ref();
    let underlying_delegated_puzzle_hash = info.underlying_delegated_puzzle_hash.as_ref();
    let p2_puzzle_hash = info.p2_puzzle_hash.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `option_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `launcher_id`,
            `underlying_coin_id`,
            `underlying_delegated_puzzle_hash`,
            `p2_puzzle_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        launcher_id,
        underlying_coin_id,
        underlying_delegated_puzzle_hash,
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn option_row(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<OptionRow>> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query_as!(
        OptionSql,
        "
        SELECT * FROM `options` WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    .map(into_row)
    .transpose()
}

async fn insert_option(conn: impl SqliteExecutor<'_>, row: OptionRow) -> Result<()> {
    let launcher_id = row.launcher_id.as_ref();
    let coin_id = row.coin_id.as_ref();

    sqlx::query!(
        "REPLACE INTO `options` (
            `launcher_id`,
            `coin_id`,
            `visible`,
            `is_owned`,
            `created_height`
        ) VALUES (?, ?, ?, ?, ?)",
        launcher_id,
        coin_id,
        row.visible,
        row.is_owned,
        row.created_height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn option_by_coin_id(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let coin_id = coin_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        OptionCoinSql,
        "
        SELECT
            `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`,
            `launcher_id`, `underlying_coin_id`, `underlying_delegated_puzzle_hash`, `p2_puzzle_hash`
        FROM `option_coins`
        INNER JOIN `coin_states` INDEXED BY `coin_height` ON `option_coins`.`coin_id` = `coin_states`.`coin_id`
        WHERE `coin_states`.`coin_id` = ?
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

async fn options(
    conn: impl SqliteExecutor<'_>,
    include_hidden: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<OptionRow>, u32)> {
    let mut query = sqlx::QueryBuilder::new(
        "SELECT launcher_id, 
            coin_id, 
            visible, 
            is_owned, 
            created_height, 
            is_pending,
            COUNT(*) OVER() as total_count	
        FROM options
        WHERE 1=1 
        AND is_owned = 1
        ",
    );

    // Add visibility condition if not including hidden options
    if !include_hidden {
        query.push(" AND visible = 1");
    }

    // Add ORDER BY clause based on sort_mode
    query.push(" ORDER BY ");

    // Add visible DESC to sort order if including hidden options
    if include_hidden {
        query.push("visible DESC, ");
    }

    query.push("is_pending DESC, created_height DESC, launcher_id ASC LIMIT ? OFFSET ?");

    let query = query.build_query_as::<OptionSearchRow>();

    // Bind limit and offset
    let query = query.bind(limit).bind(offset);

    let rows = query.fetch_all(conn).await?;
    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.total_count.try_into())?;
    let options = rows
        .into_iter()
        .map(|row| into_row(row.option))
        .collect::<Result<Vec<_>>>()?;

    Ok((options, total_count))
}

async fn option(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let launcher_id = launcher_id.as_ref();

    let Some(sql) = sqlx::query_as!(
        OptionCoinSql,
        "
        SELECT
            `coin_states`.`parent_coin_id`, `coin_states`.`puzzle_hash`, `coin_states`.`amount`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`,
            `launcher_id`, `underlying_coin_id`, `underlying_delegated_puzzle_hash`, `p2_puzzle_hash`
        FROM `option_coins` INDEXED BY `option_launcher_id`
        INNER JOIN `coin_states` ON `option_coins`.`coin_id` = `coin_states`.`coin_id`
        LEFT JOIN `transaction_spends` ON `coin_states`.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `launcher_id` = ?
        AND `spent_height` IS NULL
        AND `transaction_spends`.`transaction_id` IS NULL
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(sql.into_row()?))
}
