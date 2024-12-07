use chia::{bls::PublicKey, protocol::Bytes32};
use sqlx::SqliteExecutor;

use crate::{
    into_row, to_bytes, to_bytes32, Database, DatabaseTx, DerivationRow, DerivationSql, Result,
};

impl Database {
    pub async fn unhardened_derivations(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DerivationRow>> {
        unhardened_derivations(&self.pool, limit, offset).await
    }

    pub async fn p2_puzzle_hashes(&self) -> Result<Vec<Bytes32>> {
        p2_puzzle_hashes(&self.pool).await
    }

    pub async fn synthetic_key(&self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
        synthetic_key(&self.pool, p2_puzzle_hash).await
    }

    pub async fn synthetic_key_index(&self, synthetic_key: PublicKey) -> Result<Option<u32>> {
        synthetic_key_index(&self.pool, synthetic_key).await
    }

    pub async fn is_p2_puzzle_hash(&self, p2_puzzle_hash: Bytes32) -> Result<bool> {
        is_p2_puzzle_hash(&self.pool, p2_puzzle_hash).await
    }
}

impl<'a> DatabaseTx<'a> {
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

    pub async fn max_used_derivation_index(&mut self, hardened: bool) -> Result<Option<u32>> {
        max_used_derivation_index(&mut *self.tx, hardened).await
    }

    pub async fn synthetic_key(&mut self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
        synthetic_key(&mut *self.tx, p2_puzzle_hash).await
    }

    pub async fn p2_puzzle_hash(&mut self, index: u32, hardened: bool) -> Result<Bytes32> {
        p2_puzzle_hash(&mut *self.tx, index, hardened).await
    }

    pub async fn is_p2_puzzle_hash(&mut self, p2_puzzle_hash: Bytes32) -> Result<bool> {
        is_p2_puzzle_hash(&mut *self.tx, p2_puzzle_hash).await
    }
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
        INSERT OR IGNORE INTO `derivations` (`p2_puzzle_hash`, `index`, `hardened`, `synthetic_key`)
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
    Ok(sqlx::query!(
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
    .try_into()?)
}

async fn max_used_derivation_index(
    conn: impl SqliteExecutor<'_>,
    hardened: bool,
) -> Result<Option<u32>> {
    let row = sqlx::query!(
        "
        SELECT MAX(`index`) AS `max_index`
        FROM `derivations`
        WHERE EXISTS (
            SELECT 1 FROM `coin_states`
            WHERE `puzzle_hash` = `p2_puzzle_hash`
            OR `hint` = `p2_puzzle_hash`
        )
        AND `hardened` = ?
        ",
        hardened
    )
    .fetch_one(conn)
    .await?;
    Ok(row.max_index.map(TryInto::try_into).transpose()?)
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
        .map(|row| to_bytes32(&row.p2_puzzle_hash))
        .collect::<Result<_>>()
}

async fn unhardened_derivations(
    conn: impl SqliteExecutor<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<DerivationRow>> {
    sqlx::query_as!(
        DerivationSql,
        "
        SELECT * FROM `derivations`
        WHERE `hardened` = 0
        ORDER BY `index` ASC
        LIMIT ? OFFSET ?
        ",
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(into_row)
    .collect()
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
    Ok(PublicKey::from_bytes(&to_bytes(bytes)?)?)
}

async fn synthetic_key_index(
    conn: impl SqliteExecutor<'_>,
    synthetic_key: PublicKey,
) -> Result<Option<u32>> {
    let synthetic_key = synthetic_key.to_bytes();
    let synthetic_key_ref = synthetic_key.as_ref();
    Ok(sqlx::query!(
        "
        SELECT `index`
        FROM `derivations`
        WHERE `synthetic_key` = ?
        AND `hardened` = 0
        ",
        synthetic_key_ref
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.index.try_into())
    .transpose()?)
}

async fn p2_puzzle_hash(
    conn: impl SqliteExecutor<'_>,
    index: u32,
    hardened: bool,
) -> Result<Bytes32> {
    let row = sqlx::query!(
        "
        SELECT `p2_puzzle_hash`
        FROM `derivations`
        WHERE `index` = ?
        AND `hardened` = ?
        ",
        index,
        hardened
    )
    .fetch_one(conn)
    .await?;
    to_bytes32(&row.p2_puzzle_hash)
}

async fn is_p2_puzzle_hash(conn: impl SqliteExecutor<'_>, p2_puzzle_hash: Bytes32) -> Result<bool> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    Ok(sqlx::query!(
        "
        SELECT COUNT(*) AS `count` FROM `derivations` WHERE `p2_puzzle_hash` = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?
    .count
        > 0)
}
