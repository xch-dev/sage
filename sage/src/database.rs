use chia::{bls::PublicKey, protocol::Bytes32};
use sqlx::{Sqlite, SqliteExecutor, SqlitePool, Transaction};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn tx(&self) -> sqlx::Result<DatabaseTx<'_>> {
        let tx = self.pool.begin().await?;
        Ok(DatabaseTx::new(tx))
    }

    pub async fn insert_derivation(
        &self,
        p2_puzzle_hash: Bytes32,
        index: u32,
        hardened: bool,
        synthetic_key: PublicKey,
    ) -> sqlx::Result<()> {
        insert_derivation(&self.pool, p2_puzzle_hash, index, hardened, synthetic_key).await
    }

    pub async fn derivations(&self) -> sqlx::Result<Vec<Bytes32>> {
        derivations(&self.pool).await
    }

    pub async fn synthetic_key(&self, p2_puzzle_hash: Bytes32) -> sqlx::Result<PublicKey> {
        synthetic_key(&self.pool, p2_puzzle_hash).await
    }
}

pub struct DatabaseTx<'a> {
    tx: Transaction<'a, Sqlite>,
}

impl<'a> DatabaseTx<'a> {
    pub fn new(tx: Transaction<'a, Sqlite>) -> Self {
        Self { tx }
    }

    pub async fn insert_derivation(
        &mut self,
        p2_puzzle_hash: Bytes32,
        index: u32,
        hardened: bool,
        synthetic_key: PublicKey,
    ) -> sqlx::Result<()> {
        insert_derivation(
            &mut *self.tx,
            p2_puzzle_hash,
            index,
            hardened,
            synthetic_key,
        )
        .await
    }

    pub async fn derivations(&mut self) -> sqlx::Result<Vec<Bytes32>> {
        derivations(&mut *self.tx).await
    }

    pub async fn synthetic_key(&mut self, p2_puzzle_hash: Bytes32) -> sqlx::Result<PublicKey> {
        synthetic_key(&mut *self.tx, p2_puzzle_hash).await
    }
}

async fn insert_derivation(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
    index: u32,
    hardened: bool,
    synthetic_key: PublicKey,
) -> sqlx::Result<()> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let synthetic_key = synthetic_key.to_bytes();
    let synthetic_key = synthetic_key.as_slice();
    sqlx::query!(
        "
        INSERT INTO `derivations`
            (`p2_puzzle_hash`, `index`, `hardened`, `synthetic_key`)
        VALUES (?, ?, ?, ?)
        ",
        p2_puzzle_hash,
        index,
        hardened,
        synthetic_key
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn derivations(conn: impl SqliteExecutor<'_>) -> sqlx::Result<Vec<Bytes32>> {
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
        .collect::<sqlx::Result<_>>()
}

async fn synthetic_key(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> sqlx::Result<PublicKey> {
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

fn to_bytes<const N: usize>(slice: &[u8]) -> sqlx::Result<[u8; N]> {
    slice
        .try_into()
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

#[cfg(test)]
mod tests {
    use chia::puzzles::standard::StandardArgs;
    use chia_wallet_sdk::secret_key;

    use super::*;

    #[sqlx::test]
    fn test_derivations(pool: SqlitePool) -> anyhow::Result<()> {
        let db = Database::new(pool);
        let sk = secret_key()?;
        let synthetic_key = sk.public_key();
        let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();
        db.insert_derivation(p2_puzzle_hash, 0, false, synthetic_key)
            .await?;
        assert_eq!(db.derivations().await?, [p2_puzzle_hash]);
        Ok(())
    }
}
