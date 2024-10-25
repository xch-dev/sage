use chia::{
    protocol::{Bytes32, Coin},
    puzzles::LineageProof,
};
use chia_wallet_sdk::Cat;
use sqlx::SqliteExecutor;

use crate::{
    to_bytes, to_bytes32, to_coin, to_coin_state, to_lineage_proof, CoinStateRow, Database,
    DatabaseTx, Result,
};

#[derive(Debug, Clone, Copy)]
pub struct CatCoin {
    pub coin: Coin,
    pub lineage_proof: LineageProof,
    pub p2_puzzle_hash: Bytes32,
}

impl Database {
    pub async fn spendable_cat_coins(&self, asset_id: Bytes32) -> Result<Vec<CatCoin>> {
        spendable_cat_coins(&self.pool, asset_id).await
    }

    pub async fn cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        cat_balance(&self.pool, asset_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn asset_ids(&self) -> Result<Vec<Bytes32>> {
        asset_ids(&self.pool).await
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
) -> Result<Vec<CatCoin>> {
    let asset_id = asset_id.as_ref();

    let rows = sqlx::query!(
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

    rows.into_iter()
        .map(|row| {
            Ok(CatCoin {
                coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                lineage_proof: to_lineage_proof(
                    &row.parent_parent_coin_id,
                    &row.parent_inner_puzzle_hash,
                    &row.parent_amount,
                )?,
                p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
            })
        })
        .collect()
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

    let rows = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cs.spent_height, cs.created_height, cs.transaction_id
        FROM `coin_states` AS cs
        INNER JOIN `cat_coins` AS cat
        ON cs.coin_id = cat.coin_id
        WHERE cat.asset_id = ?
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(CoinStateRow {
                coin_state: to_coin_state(
                    to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                    row.created_height,
                    row.spent_height,
                )?,
                transaction_id: row.transaction_id.map(|id| to_bytes32(&id)).transpose()?,
            })
        })
        .collect()
}

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id = coin_id.as_ref();

    let row = sqlx::query!(
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

    row.map(|row| {
        Ok(Cat {
            coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
            asset_id: to_bytes32(&row.asset_id)?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
            lineage_proof: Some(to_lineage_proof(
                &row.parent_parent_coin_id,
                &row.parent_inner_puzzle_hash,
                &row.parent_amount,
            )?),
        })
    })
    .transpose()
}

async fn asset_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!("SELECT `asset_id` FROM `cat_coins` GROUP BY `asset_id`")
        .fetch_all(conn)
        .await?;

    rows.into_iter()
        .map(|row| to_bytes32(&row.asset_id))
        .collect()
}
