use chia::{
    bls::PublicKey,
    protocol::{Bytes32, Coin, CoinState},
    puzzles::LineageProof,
};
use chia_wallet_sdk::Cat;
use sqlx::{Sqlite, SqliteExecutor, SqlitePool, Transaction};

use crate::error::{DatabaseError, Result};

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn tx(&self) -> Result<DatabaseTx<'_>> {
        let tx = self.pool.begin().await?;
        Ok(DatabaseTx::new(tx))
    }

    pub async fn insert_peak(&self, height: u32, header_hash: Bytes32) -> Result<()> {
        insert_peak(&self.pool, height, header_hash).await
    }

    pub async fn delete_peak(&self, height: u32) -> Result<()> {
        delete_peak(&self.pool, height).await
    }

    pub async fn latest_peak(&self) -> Result<Option<(u32, Bytes32)>> {
        latest_peak(&self.pool).await
    }

    pub async fn insert_derivation(
        &self,
        p2_puzzle_hash: Bytes32,
        index: u32,
        hardened: bool,
        synthetic_key: PublicKey,
    ) -> Result<()> {
        insert_derivation(&self.pool, p2_puzzle_hash, index, hardened, synthetic_key).await
    }

    pub async fn derivation_index(&self, hardened: bool) -> Result<u32> {
        derivation_index(&self.pool, hardened).await
    }

    pub async fn max_used_derivation_index(&self) -> Result<Option<u32>> {
        max_used_derivation_index(&self.pool).await
    }

    pub async fn p2_puzzle_hashes(&self) -> Result<Vec<Bytes32>> {
        p2_puzzle_hashes(&self.pool).await
    }

    pub async fn synthetic_key(&self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
        synthetic_key(&self.pool, p2_puzzle_hash).await
    }

    pub async fn try_insert_coin_state(&self, coin_state: CoinState) -> Result<()> {
        try_insert_coin_state(&self.pool, coin_state).await
    }

    pub async fn remove_coin_state(&self, coin_id: Bytes32) -> Result<()> {
        remove_coin_state(&self.pool, coin_id).await
    }

    pub async fn unsynced_coin_states(&self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coin_states(&self.pool, limit).await
    }

    pub async fn mark_coin_synced(&self, coin_id: Bytes32) -> Result<()> {
        mark_coin_synced(&self.pool, coin_id).await
    }

    pub async fn total_coin_count(&self) -> Result<u32> {
        total_coin_count(&self.pool).await
    }

    pub async fn synced_coin_count(&self) -> Result<u32> {
        synced_coin_count(&self.pool).await
    }

    pub async fn coin_state(&self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&self.pool, coin_id).await
    }

    pub async fn insert_cat_info(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_info(&self.pool, coin_id, lineage_proof, p2_puzzle_hash, asset_id).await
    }

    pub async fn cat_info(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_info(&self.pool, coin_id).await
    }
}

#[derive(Debug)]
pub struct DatabaseTx<'a> {
    tx: Transaction<'a, Sqlite>,
}

impl<'a> DatabaseTx<'a> {
    pub fn new(tx: Transaction<'a, Sqlite>) -> Self {
        Self { tx }
    }

    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }

    pub async fn rollback(self) -> Result<()> {
        Ok(self.tx.rollback().await?)
    }

    pub async fn insert_peak(&mut self, height: u32, header_hash: Bytes32) -> Result<()> {
        insert_peak(&mut *self.tx, height, header_hash).await
    }

    pub async fn delete_peak(&mut self, height: u32) -> Result<()> {
        delete_peak(&mut *self.tx, height).await
    }

    pub async fn latest_peak(&mut self) -> Result<Option<(u32, Bytes32)>> {
        latest_peak(&mut *self.tx).await
    }

    pub async fn insert_derivation(
        &mut self,
        p2_puzzle_hash: Bytes32,
        index: u32,
        hardened: bool,
        synthetic_key: PublicKey,
    ) -> Result<()> {
        insert_derivation(
            &mut *self.tx,
            p2_puzzle_hash,
            index,
            hardened,
            synthetic_key,
        )
        .await
    }

    pub async fn derivation_index(&mut self, hardened: bool) -> Result<u32> {
        derivation_index(&mut *self.tx, hardened).await
    }

    pub async fn max_used_derivation_index(&mut self) -> Result<Option<u32>> {
        max_used_derivation_index(&mut *self.tx).await
    }

    pub async fn p2_puzzle_hashes(&mut self) -> Result<Vec<Bytes32>> {
        p2_puzzle_hashes(&mut *self.tx).await
    }

    pub async fn synthetic_key(&mut self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
        synthetic_key(&mut *self.tx, p2_puzzle_hash).await
    }

    pub async fn try_insert_coin_state(&mut self, coin_state: CoinState) -> Result<()> {
        try_insert_coin_state(&mut *self.tx, coin_state).await
    }

    pub async fn remove_coin_state(&mut self, coin_id: Bytes32) -> Result<()> {
        remove_coin_state(&mut *self.tx, coin_id).await
    }

    pub async fn unsynced_coin_states(&mut self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coin_states(&mut *self.tx, limit).await
    }

    pub async fn mark_coin_synced(&mut self, coin_id: Bytes32) -> Result<()> {
        mark_coin_synced(&mut *self.tx, coin_id).await
    }

    pub async fn total_coin_count(&mut self) -> Result<u32> {
        total_coin_count(&mut *self.tx).await
    }

    pub async fn synced_coin_count(&mut self) -> Result<u32> {
        synced_coin_count(&mut *self.tx).await
    }

    pub async fn coin_state(&mut self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&mut *self.tx, coin_id).await
    }

    pub async fn insert_cat_info(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_info(
            &mut *self.tx,
            coin_id,
            lineage_proof,
            p2_puzzle_hash,
            asset_id,
        )
        .await
    }

    pub async fn cat_info(&mut self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_info(&mut *self.tx, coin_id).await
    }
}

async fn insert_peak(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    header_hash: Bytes32,
) -> Result<()> {
    let header_hash = header_hash.as_ref();
    sqlx::query!(
        "
        INSERT INTO `peaks` (`height`, `header_hash`)
        VALUES (?, ?)
        ",
        height,
        header_hash
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn delete_peak(conn: impl SqliteExecutor<'_>, height: u32) -> Result<()> {
    sqlx::query!(
        "
        DELETE FROM `peaks`
        WHERE `height` = ?
        ",
        height
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn latest_peak(conn: impl SqliteExecutor<'_>) -> Result<Option<(u32, Bytes32)>> {
    sqlx::query!(
        "
        SELECT `height`, `header_hash`
        FROM `peaks`
        ORDER BY `height` DESC
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok((
            row.height
                .try_into()
                .map_err(|_| DatabaseError::PrecisionLost)?,
            Bytes32::new(to_bytes(&row.header_hash).unwrap()),
        ))
    })
    .transpose()
}

async fn insert_derivation(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
    index: u32,
    hardened: bool,
    synthetic_key: PublicKey,
) -> Result<()> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let synthetic_key = synthetic_key.to_bytes();
    let synthetic_key_ref = synthetic_key.as_ref();
    sqlx::query!(
        "
        INSERT INTO `derivations` (`p2_puzzle_hash`, `index`, `hardened`, `synthetic_key`)
        VALUES (?, ?, ?, ?)
        ",
        p2_puzzle_hash,
        index,
        hardened,
        synthetic_key_ref
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn derivation_index(conn: impl SqliteExecutor<'_>, hardened: bool) -> Result<u32> {
    sqlx::query!(
        "
        SELECT MAX(`index`) AS `max_index`
        FROM `derivations`
        WHERE `hardened` = ?
        ",
        hardened
    )
    .fetch_one(conn)
    .await?
    .max_index
    .map_or(0, |index| index + 1)
    .try_into()
    .map_err(|_| DatabaseError::PrecisionLost)
}

async fn max_used_derivation_index(conn: impl SqliteExecutor<'_>) -> Result<Option<u32>> {
    let row = sqlx::query!(
        "
        SELECT MAX(`index`) AS `max_index`
        FROM `derivations`
        WHERE EXISTS (SELECT * FROM `coin_states` WHERE `puzzle_hash` = `p2_puzzle_hash` OR `hint` = `p2_puzzle_hash`)
        "
    )
    .fetch_one(conn)
    .await?;
    row.max_index
        .map(|index| index.try_into().map_err(|_| DatabaseError::PrecisionLost))
        .transpose()
}

async fn p2_puzzle_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `p2_puzzle_hash`
        FROM `derivations`
        ORDER BY `index` ASC, `hardened` ASC
        "
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|row| Ok(Bytes32::new(to_bytes(&row.p2_puzzle_hash)?)))
        .collect::<Result<_>>()
}

async fn synthetic_key(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<PublicKey> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let row = sqlx::query!(
        "
        SELECT `synthetic_key`
        FROM `derivations`
        WHERE `p2_puzzle_hash` = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?;
    let bytes = row.synthetic_key.as_slice();
    Ok(PublicKey::from_bytes(&to_bytes(bytes)?).unwrap())
}

async fn try_insert_coin_state(conn: impl SqliteExecutor<'_>, coin_state: CoinState) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();
    let coin_id_ref = coin_id.as_ref();
    let parent_coin_id = coin_state.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_state.coin.puzzle_hash.as_ref();
    let amount = coin_state.coin.amount.to_be_bytes();
    let amount_ref = amount.as_ref();
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
            `hint`
        )
        VALUES (
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            EXISTS (SELECT * FROM `derivations` WHERE `p2_puzzle_hash` = ?),
            NULL
        )
        ",
        coin_id_ref,
        parent_coin_id,
        puzzle_hash,
        amount_ref,
        coin_state.created_height,
        coin_state.spent_height,
        puzzle_hash
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn remove_coin_state(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
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
    let limit: i64 = limit.try_into().map_err(|_| DatabaseError::PrecisionLost)?;
    let rows = sqlx::query!(
        "
        SELECT *
        FROM `coin_states`
        WHERE `synced` = 0 AND `created_height` IS NOT NULL
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|row| {
            to_coin_state(
                to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                row.created_height,
                row.spent_height,
            )
        })
        .collect()
}

async fn mark_coin_synced(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();
    sqlx::query!(
        "
        UPDATE `coin_states`
        SET `synced` = 1
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
    row.total
        .try_into()
        .map_err(|_| DatabaseError::PrecisionLost)
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
    row.synced
        .try_into()
        .map_err(|_| DatabaseError::PrecisionLost)
}

async fn coin_state(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<CoinState>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT *
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

    Ok(Some(to_coin_state(
        to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        row.created_height,
        row.spent_height,
    )?))
}

async fn insert_cat_info(
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
        INSERT INTO `cat_info` (
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

async fn cat_info(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cat.parent_parent_coin_id, cat.parent_inner_puzzle_hash, cat.parent_amount,
            cat.p2_puzzle_hash, cat.asset_id
        FROM `coin_states` AS cs
        INNER JOIN cat_info AS cat
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
        p2_puzzle_hash: Bytes32::new(to_bytes(&row.p2_puzzle_hash).unwrap()),
        asset_id: Bytes32::new(to_bytes(&row.asset_id).unwrap()),
    }))
}

fn to_coin_state(
    coin: Coin,
    created_height: Option<i64>,
    spent_height: Option<i64>,
) -> Result<CoinState> {
    Ok(CoinState {
        coin,
        spent_height: spent_height
            .map(|height| height.try_into().map_err(|_| DatabaseError::PrecisionLost))
            .transpose()?,
        created_height: created_height
            .map(|height| height.try_into().map_err(|_| DatabaseError::PrecisionLost))
            .transpose()?,
    })
}

fn to_coin(parent_coin_id: &[u8], puzzle_hash: &[u8], amount: &[u8]) -> Result<Coin> {
    Ok(Coin {
        parent_coin_info: Bytes32::new(to_bytes(parent_coin_id)?),
        puzzle_hash: Bytes32::new(to_bytes(puzzle_hash)?),
        amount: u64::from_be_bytes(to_bytes(amount)?),
    })
}

fn to_lineage_proof(
    parent_parent_coin_id: &[u8],
    parent_inner_puzzle_hash: &[u8],
    parent_amount: &[u8],
) -> Result<LineageProof> {
    Ok(LineageProof {
        parent_parent_coin_info: Bytes32::new(to_bytes(parent_parent_coin_id)?),
        parent_inner_puzzle_hash: Bytes32::new(to_bytes(parent_inner_puzzle_hash)?),
        parent_amount: u64::from_be_bytes(to_bytes(parent_amount)?),
    })
}

fn to_bytes<const N: usize>(slice: &[u8]) -> Result<[u8; N]> {
    slice
        .try_into()
        .map_err(|_| DatabaseError::InvalidLength(slice.len(), N))
}
