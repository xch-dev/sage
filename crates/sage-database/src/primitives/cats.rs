use chia::{protocol::Bytes32, puzzles::LineageProof};
use chia_wallet_sdk::Cat;
use sqlx::SqliteExecutor;

use crate::{
    to_bytes, to_bytes32, CatCoinRow, CatCoinSql, CatRow, CoinStateRow, CoinStateSql, Database,
    DatabaseTx, FullCatCoinSql, Result,
};

impl Database {
    pub async fn maybe_insert_cat(&self, row: CatRow) -> Result<()> {
        maybe_insert_cat(&self.pool, row).await
    }

    pub async fn update_cat(&self, row: CatRow) -> Result<()> {
        update_cat(&self.pool, row).await
    }

    pub async fn delete_cat(&self, asset_id: Bytes32) -> Result<()> {
        delete_cat(&self.pool, asset_id).await
    }

    pub async fn cats(&self) -> Result<Vec<CatRow>> {
        cats(&self.pool).await
    }

    pub async fn cat(&self, asset_id: Bytes32) -> Result<Option<CatRow>> {
        cat(&self.pool, asset_id).await
    }

    pub async fn unidentified_cat(&self) -> Result<Option<Bytes32>> {
        unidentified_cat(&self.pool).await
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
}

impl<'a> DatabaseTx<'a> {
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

    pub async fn cat_coin_states(&mut self, asset_id: Bytes32) -> Result<Vec<CoinStateRow>> {
        cat_coin_states(&mut *self.tx, asset_id).await
    }
}

async fn maybe_insert_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `cats` (
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.ticker,
        row.description,
        row.icon_url,
        row.visible,
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
            `icon_url`,
            `visible`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.ticker,
        row.description,
        row.icon_url,
        row.visible
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn delete_cat(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<()> {
    let asset_id = asset_id.as_ref();

    sqlx::query!(
        "
        DELETE FROM `cats` WHERE `asset_id` = ?
        ",
        asset_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn cats(conn: impl SqliteExecutor<'_>) -> Result<Vec<CatRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        FROM `cats`
        ORDER BY `name` ASC, `asset_id` ASC
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(CatRow {
                asset_id: to_bytes32(&row.asset_id)?,
                name: row.name,
                ticker: row.ticker,
                description: row.description,
                icon_url: row.icon_url,
                visible: row.visible,
            })
        })
        .collect()
}

async fn cat(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<CatRow>> {
    let asset_id = asset_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        FROM `cats`
        WHERE `asset_id` = ?
        ",
        asset_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        Ok(CatRow {
            asset_id: to_bytes32(&row.asset_id)?,
            name: row.name,
            ticker: row.ticker,
            description: row.description,
            icon_url: row.icon_url,
            visible: row.visible,
        })
    })
    .transpose()
}

async fn unidentified_cat(conn: impl SqliteExecutor<'_>) -> Result<Option<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `asset_id` FROM `cat_coins`
        WHERE `asset_id` NOT IN (SELECT `asset_id` FROM `cats`)
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
        REPLACE INTO `cat_coins` (
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
        FROM `cat_coins`
        INNER JOIN `coin_states` AS cs ON `cat_coins`.`coin_id` = cs.`coin_id`
        LEFT JOIN `transaction_spends` ON cs.`coin_id` = `transaction_spends`.`coin_id`
        WHERE `cat_coins`.`asset_id` = ?
        AND cs.`spent_height` IS NULL
        AND `transaction_spends`.`coin_id` IS NULL
        AND cs.`transaction_id` IS NULL
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(|sql| sql.into_row()).collect()
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

async fn cat_coin_states(
    conn: impl SqliteExecutor<'_>,
    asset_id: Bytes32,
) -> Result<Vec<CoinStateRow>> {
    let asset_id = asset_id.as_ref();

    let rows = sqlx::query_as!(
        CoinStateSql,
        "
        SELECT `parent_coin_id`, `puzzle_hash`, `amount`, `spent_height`, `created_height`, `transaction_id`
        FROM `cat_coins`
        INNER JOIN `coin_states` ON `coin_states`.coin_id = `cat_coins`.coin_id
        WHERE `asset_id` = ?
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter().map(|sql| sql.into_row()).collect()
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

    row.map(|sql| sql.into_row()).transpose()
}
