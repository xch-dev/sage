use chia::{protocol::Bytes32, puzzles::LineageProof};
use chia_wallet_sdk::Cat;
use sqlx::SqliteExecutor;

use crate::{error::Result, to_bytes32, to_coin, to_lineage_proof, Database, DatabaseTx};

impl Database {
    pub async fn insert_cat_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(&self.pool, coin_id, lineage_proof, p2_puzzle_hash, asset_id).await
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

    pub async fn cat_coin(&mut self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&mut *self.tx, coin_id).await
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
        INSERT INTO `cat_coins` (
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

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cat.parent_parent_coin_id, cat.parent_inner_puzzle_hash, cat.parent_amount,
            cat.p2_puzzle_hash, cat.asset_id
        FROM `coin_states` AS cs
        INNER JOIN `cat_coins` AS cat
        ON cs.coin_id = cat.coin_id
        WHERE cs.coin_id = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Cat {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        lineage_proof: Some(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        asset_id: to_bytes32(&row.asset_id)?,
    }))
}
