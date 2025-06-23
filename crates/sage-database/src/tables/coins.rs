use chia::{
    protocol::{Bytes32, Coin, CoinState},
    puzzles::LineageProof,
};
use chia_wallet_sdk::driver::{Cat, CatInfo};
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, Result};

impl Database {
    pub async fn unsynced_coins(&self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coins(&self.pool, limit).await
    }

    pub async fn xch_balance(&self) -> Result<u128> {
        xch_balance(&self.pool).await
    }

    pub async fn cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        cat_balance(&self.pool, asset_id).await
    }

    pub async fn spendable_xch_balance(&self) -> Result<u128> {
        spendable_xch_balance(&self.pool).await
    }

    pub async fn spendable_cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        spendable_cat_balance(&self.pool, asset_id).await
    }

    pub async fn spendable_xch_coins(&self) -> Result<Vec<Coin>> {
        spendable_xch_coins(&self.pool).await
    }

    pub async fn spendable_cat_coins(&self, asset_id: Bytes32) -> Result<Vec<Cat>> {
        spendable_cat_coins(&self.pool, asset_id).await
    }
}

async fn unsynced_coins(conn: impl SqliteExecutor<'_>, limit: usize) -> Result<Vec<CoinState>> {
    let limit = i64::try_from(limit)?;

    query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount, created_height, spent_height
        FROM coins
        WHERE asset_id IS NULL
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(CoinState::new(
            Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            row.spent_height.convert()?,
            row.created_height.convert()?,
        ))
    })
    .collect()
}

async fn xch_balance(conn: impl SqliteExecutor<'_>) -> Result<u128> {
    query!("SELECT amount FROM owned_coins WHERE asset_id = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| -> Result<u128> { row.amount.convert() })
        .sum()
}

async fn cat_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT amount FROM owned_coins
        INNER JOIN assets ON assets.id = asset_id
        WHERE assets.hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| -> Result<u128> { row.amount.convert() })
    .sum()
}

async fn spendable_xch_balance(conn: impl SqliteExecutor<'_>) -> Result<u128> {
    query!("SELECT amount FROM spendable_coins WHERE asset_id = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| -> Result<u128> { row.amount.convert() })
        .sum()
}

async fn spendable_cat_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT amount FROM spendable_coins
        INNER JOIN assets ON assets.id = asset_id
        WHERE assets.hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| -> Result<u128> { row.amount.convert() })
    .sum()
}

async fn spendable_xch_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Coin>> {
    query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount FROM spendable_coins
        WHERE asset_id = 0
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ))
    })
    .collect()
}

async fn spendable_cat_coins(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Vec<Cat>> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, hidden_puzzle_hash, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount
        FROM spendable_coins
        INNER JOIN assets ON assets.id = asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE assets.hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Cat::new(
            Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            Some(LineageProof {
                parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
                parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
                parent_amount: row.parent_amount.convert()?,
            }),
            CatInfo::new(
                asset_id,
                row.hidden_puzzle_hash.convert()?,
                row.p2_puzzle_hash.convert()?,
            ),
        ))
    })
    .collect()
}
