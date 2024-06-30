use chia::{
    bls::PublicKey,
    protocol::{Bytes32, Coin, CoinState},
};
use sqlx::{Sqlite, SqliteExecutor, SqlitePool, Transaction};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),

    #[error("precision lost in cast")]
    PrecisionLoss,
}

type Result<T> = std::result::Result<T, DatabaseError>;

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

    pub async fn insert_derivation(
        &self,
        p2_puzzle_hash: Bytes32,
        index: u32,
        hardened: bool,
        synthetic_key: PublicKey,
    ) -> Result<()> {
        insert_derivation(&self.pool, p2_puzzle_hash, index, hardened, synthetic_key).await
    }

    pub async fn derivations(&self) -> Result<Vec<Bytes32>> {
        derivations(&self.pool).await
    }

    pub async fn synthetic_key(&self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
        synthetic_key(&self.pool, p2_puzzle_hash).await
    }

    pub async fn insert_coin_state(&self, coin_state: CoinState) -> Result<()> {
        insert_coin_state(&self.pool, coin_state).await
    }

    pub async fn coin_state(&self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&self.pool, coin_id).await
    }
}

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

    pub async fn derivations(&mut self) -> Result<Vec<Bytes32>> {
        derivations(&mut *self.tx).await
    }

    pub async fn synthetic_key(&mut self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
        synthetic_key(&mut *self.tx, p2_puzzle_hash).await
    }

    pub async fn insert_coin_state(&mut self, coin_state: CoinState) -> Result<()> {
        insert_coin_state(&mut *self.tx, coin_state).await
    }

    pub async fn coin_state(&mut self, coin_id: Bytes32) -> Result<Option<CoinState>> {
        coin_state(&mut *self.tx, coin_id).await
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

async fn derivations(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `p2_puzzle_hash`
        FROM `derivations`
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

async fn insert_coin_state(conn: impl SqliteExecutor<'_>, coin_state: CoinState) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();
    let coin_id_ref = coin_id.as_ref();
    let parent_coin_id = coin_state.coin.parent_coin_info.as_ref();
    let puzzle_hash = coin_state.coin.puzzle_hash.as_ref();
    let amount = coin_state.coin.amount.to_be_bytes();
    let amount_ref = amount.as_ref();
    sqlx::query!(
        "
        INSERT INTO `coin_states` (`coin_id`, `parent_coin_id`, `puzzle_hash`, `amount`)
        VALUES (?, ?, ?, ?)
        ",
        coin_id_ref,
        parent_coin_id,
        puzzle_hash,
        amount_ref
    )
    .execute(conn)
    .await?;
    Ok(())
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

    Ok(Some(CoinState {
        coin: Coin {
            parent_coin_info: Bytes32::new(to_bytes(&row.parent_coin_id)?),
            puzzle_hash: Bytes32::new(to_bytes(&row.puzzle_hash)?),
            amount: u64::from_be_bytes(to_bytes(&row.amount)?),
        },
        spent_height: row
            .spent_height
            .map(|height| height.try_into().map_err(|_| DatabaseError::PrecisionLoss))
            .transpose()?,
        created_height: row
            .created_height
            .map(|height| height.try_into().map_err(|_| DatabaseError::PrecisionLoss))
            .transpose()?,
    }))
}

fn to_bytes<const N: usize>(slice: &[u8]) -> Result<[u8; N]> {
    slice
        .try_into()
        .map_err(|_| DatabaseError::InvalidLength(slice.len(), N))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chia::{
        bls::DerivableKey,
        puzzles::{standard::StandardArgs, DeriveSynthetic},
    };
    use chia_wallet_sdk::secret_key;

    use super::*;

    #[sqlx::test]
    fn test_derivations(pool: SqlitePool) -> anyhow::Result<()> {
        sqlx::migrate!("../migrations").run(&pool).await?;

        let db = Database::new(pool);
        let sk = secret_key()?;
        let synthetic_key = sk.public_key().derive_synthetic();
        let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();

        db.insert_derivation(p2_puzzle_hash, 0, false, synthetic_key)
            .await?;
        assert_eq!(db.derivations().await?, [p2_puzzle_hash]);
        assert_eq!(db.synthetic_key(p2_puzzle_hash).await?, synthetic_key);

        Ok(())
    }

    #[sqlx::test]
    fn test_transaction(pool: SqlitePool) -> anyhow::Result<()> {
        sqlx::migrate!("../migrations").run(&pool).await?;

        let db = Database::new(pool);
        let sk = secret_key()?;
        let root_pk = sk.public_key();

        let mut puzzle_hashes = HashSet::new();
        let mut tx = db.tx().await?;

        for i in 0..100 {
            let synthetic_key = root_pk.derive_unhardened(i).derive_synthetic();
            let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();
            puzzle_hashes.insert(p2_puzzle_hash);
            tx.insert_derivation(p2_puzzle_hash, i, false, synthetic_key)
                .await?;
        }

        tx.commit().await?;

        let rows: HashSet<Bytes32> = db.derivations().await?.into_iter().collect();
        assert_eq!(rows, puzzle_hashes);

        Ok(())
    }
}
