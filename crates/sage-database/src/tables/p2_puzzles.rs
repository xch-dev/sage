use chia::{bls::PublicKey, clvm_utils::ToTreeHash, protocol::Bytes32};
use chia_wallet_sdk::driver::ClawbackV2;
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum P2PuzzleKind {
    PublicKey,
    Clawback,
}

#[derive(Debug, Clone, Copy)]
pub enum P2Puzzle {
    PublicKey(PublicKey),
    Clawback(Clawback),
}

#[derive(Debug, Clone, Copy)]
pub struct Clawback {
    pub public_key: PublicKey,
    pub sender_puzzle_hash: Bytes32,
    pub receiver_puzzle_hash: Bytes32,
    pub seconds: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Derivation {
    pub derivation_index: u32,
    pub is_hardened: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DerivationRow {
    pub p2_puzzle_hash: Bytes32,
    pub index: u32,
    pub hardened: bool,
    pub synthetic_key: PublicKey,
}

impl Database {
    pub async fn public_key(&self, p2_puzzle_hash: Bytes32) -> Result<Option<PublicKey>> {
        public_key(&self.pool, p2_puzzle_hash).await
    }

    pub async fn custody_p2_puzzle_hashes(&self) -> Result<Vec<Bytes32>> {
        custody_p2_puzzle_hashes(&self.pool).await
    }

    pub async fn is_custody_p2_puzzle_hash(&self, puzzle_hash: Bytes32) -> Result<bool> {
        is_custody_p2_puzzle_hash(&self.pool, puzzle_hash).await
    }

    pub async fn is_p2_puzzle_hash(&self, puzzle_hash: Bytes32) -> Result<bool> {
        is_p2_puzzle_hash(&self.pool, puzzle_hash).await
    }

    pub async fn p2_puzzle(&self, puzzle_hash: Bytes32) -> Result<P2Puzzle> {
        match p2_puzzle_kind(&self.pool, puzzle_hash).await? {
            P2PuzzleKind::PublicKey => {
                let Some(key) = public_key(&self.pool, puzzle_hash).await? else {
                    return Err(DatabaseError::InvalidEnumVariant);
                };
                Ok(P2Puzzle::PublicKey(key))
            }
            P2PuzzleKind::Clawback => {
                Ok(P2Puzzle::Clawback(clawback(&self.pool, puzzle_hash).await?))
            }
        }
    }

    pub async fn derivation(&self, public_key: PublicKey) -> Result<Option<Derivation>> {
        derivation(&self.pool, public_key).await
    }

    pub async fn derivations(
        &self,
        is_hardened: bool,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<DerivationRow>, u32)> {
        derivations(&self.pool, is_hardened, limit, offset).await
    }

    pub async fn max_derivation_index(&self, is_hardened: bool) -> Result<u32> {
        max_derivation_index(&self.pool, is_hardened).await
    }
}

impl DatabaseTx<'_> {
    pub async fn custody_p2_puzzle_hash(
        &mut self,
        derivation_index: u32,
        is_hardened: bool,
    ) -> Result<Bytes32> {
        custody_p2_puzzle_hash(&mut *self.tx, derivation_index, is_hardened).await
    }

    pub async fn is_custody_p2_puzzle_hash(&mut self, puzzle_hash: Bytes32) -> Result<bool> {
        is_custody_p2_puzzle_hash(&mut *self.tx, puzzle_hash).await
    }

    pub async fn is_p2_puzzle_hash(&mut self, puzzle_hash: Bytes32) -> Result<bool> {
        is_p2_puzzle_hash(&mut *self.tx, puzzle_hash).await
    }

    pub async fn derivation_index(&mut self, is_hardened: bool) -> Result<u32> {
        derivation_index(&mut *self.tx, is_hardened).await
    }

    pub async fn unused_derivation_index(&mut self, is_hardened: bool) -> Result<u32> {
        unused_derivation_index(&mut *self.tx, is_hardened).await
    }

    pub async fn insert_custody_p2_puzzle(
        &mut self,
        p2_puzzle_hash: Bytes32,
        key: PublicKey,
        derivation: Derivation,
    ) -> Result<()> {
        insert_custody_p2_puzzle(&mut *self.tx, p2_puzzle_hash, key, derivation).await
    }

    pub async fn insert_clawback_p2_puzzle(&mut self, clawback: ClawbackV2) -> Result<()> {
        insert_clawback_p2_puzzle(&mut *self.tx, clawback).await
    }
}

async fn custody_p2_puzzle_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM p2_puzzles WHERE kind = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
}

async fn custody_p2_puzzle_hash(
    conn: impl SqliteExecutor<'_>,
    derivation_index: u32,
    is_hardened: bool,
) -> Result<Bytes32> {
    query!(
        "
        SELECT hash FROM p2_puzzles
        INNER JOIN public_keys ON public_keys.p2_puzzle_id = p2_puzzles.id
        WHERE public_keys.derivation_index = ? AND public_keys.is_hardened = ?
        ",
        derivation_index,
        is_hardened
    )
    .fetch_one(conn)
    .await?
    .hash
    .convert()
}

async fn is_custody_p2_puzzle_hash(
    conn: impl SqliteExecutor<'_>,
    puzzle_hash: Bytes32,
) -> Result<bool> {
    let puzzle_hash = puzzle_hash.as_ref();

    Ok(query!(
        "SELECT COUNT(*) AS count FROM p2_puzzles WHERE hash = ? AND kind = 0",
        puzzle_hash
    )
    .fetch_one(conn)
    .await?
    .count
        > 0)
}

async fn is_p2_puzzle_hash(conn: impl SqliteExecutor<'_>, puzzle_hash: Bytes32) -> Result<bool> {
    let puzzle_hash = puzzle_hash.as_ref();

    Ok(query!(
        "SELECT COUNT(*) AS count FROM p2_puzzles WHERE hash = ?",
        puzzle_hash
    )
    .fetch_one(conn)
    .await?
    .count
        > 0)
}

async fn derivation_index(conn: impl SqliteExecutor<'_>, is_hardened: bool) -> Result<u32> {
    query!(
        "
        SELECT COALESCE(MAX(derivation_index) + 1, 0) AS derivation_index
        FROM public_keys
        WHERE is_hardened = ?
        ",
        is_hardened
    )
    .fetch_one(conn)
    .await?
    .derivation_index
    .convert()
}

async fn max_derivation_index(conn: impl SqliteExecutor<'_>, is_hardened: bool) -> Result<u32> {
    let row = query!(
        "
        SELECT MAX(derivation_index) AS derivation_index 
        FROM public_keys 
        WHERE is_hardened = ?
        ",
        is_hardened
    )
    .fetch_one(conn)
    .await?;

    Ok(row
        .derivation_index
        .map_or(0, |idx| idx.try_into().unwrap_or(0)))
}

async fn derivations(
    conn: impl SqliteExecutor<'_>,
    is_hardened: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<DerivationRow>, u32)> {
    let rows = query!(
        "
        SELECT
            p2_puzzles.hash AS p2_puzzle_hash,
            public_keys.derivation_index,
            public_keys.is_hardened,
            public_keys.key AS synthetic_key,
            COUNT(*) OVER() AS total
        FROM p2_puzzles
        INNER JOIN public_keys ON public_keys.p2_puzzle_id = p2_puzzles.id
        WHERE public_keys.is_hardened = ?
        ORDER BY public_keys.derivation_index ASC
        LIMIT ? OFFSET ?
        ",
        is_hardened,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?;

    let total_count = rows.first().map_or(Ok(0), |row| row.total.try_into())?;

    let derivations = rows
        .into_iter()
        .map(|row| {
            Ok(DerivationRow {
                p2_puzzle_hash: row.p2_puzzle_hash.convert()?,
                index: row.derivation_index.convert()?,
                hardened: row.is_hardened,
                synthetic_key: row.synthetic_key.convert()?,
            })
        })
        .collect::<Result<Vec<DerivationRow>>>()?;

    Ok((derivations, total_count))
}

async fn unused_derivation_index(conn: impl SqliteExecutor<'_>, is_hardened: bool) -> Result<u32> {
    query!(
        "
        SELECT COALESCE(MAX(derivation_index) + 1, 0) AS derivation_index
        FROM public_keys
        INNER JOIN coins ON coins.p2_puzzle_id = public_keys.p2_puzzle_id
        WHERE is_hardened = ?
        ",
        is_hardened
    )
    .fetch_one(conn)
    .await?
    .derivation_index
    .convert()
}

async fn insert_custody_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
    key: PublicKey,
    derivation: Derivation,
) -> Result<()> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let key = key.to_bytes();
    let key = key.as_ref();

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 0);

        INSERT OR IGNORE INTO public_keys (p2_puzzle_id, is_hardened, derivation_index, key)
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?, ?, ?);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        derivation.is_hardened,
        derivation.derivation_index,
        key,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_clawback_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    clawback: ClawbackV2,
) -> Result<()> {
    let p2_puzzle_hash = clawback.tree_hash().to_vec();
    let sender_puzzle_hash = clawback.sender_puzzle_hash.as_ref();
    let receiver_puzzle_hash = clawback.receiver_puzzle_hash.as_ref();
    let seconds: i64 = clawback.seconds.try_into()?;

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 1);

        INSERT OR IGNORE INTO clawbacks (p2_puzzle_id, sender_puzzle_hash, receiver_puzzle_hash, expiration_seconds)
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?, ?, ?);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        sender_puzzle_hash,
        receiver_puzzle_hash,
        seconds,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn p2_puzzle_kind(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<P2PuzzleKind> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!("SELECT kind FROM p2_puzzles WHERE hash = ?", p2_puzzle_hash)
        .fetch_one(conn)
        .await?;

    Ok(match row.kind {
        0 => P2PuzzleKind::PublicKey,
        1 => P2PuzzleKind::Clawback,
        _ => return Err(DatabaseError::InvalidEnumVariant),
    })
}

async fn public_key(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<Option<PublicKey>> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!(
        "
        SELECT key
        FROM p2_puzzles
        INNER JOIN public_keys ON public_keys.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| row.key.convert()).transpose()
}

async fn clawback(conn: impl SqliteExecutor<'_>, p2_puzzle_hash: Bytes32) -> Result<Clawback> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!(
        "
        SELECT key, sender_puzzle_hash, receiver_puzzle_hash, expiration_seconds
        FROM p2_puzzles
        INNER JOIN clawbacks ON clawbacks.p2_puzzle_id = p2_puzzles.id
        INNER JOIN public_keys ON public_keys.p2_puzzle_id IN (
            SELECT id FROM p2_puzzles
            WHERE (hash = sender_puzzle_hash AND unixepoch() < expiration_seconds)
            OR (hash = receiver_puzzle_hash AND unixepoch() >= expiration_seconds)
            LIMIT 1
        )
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?;

    Ok(Clawback {
        public_key: row.key.convert()?,
        sender_puzzle_hash: row.sender_puzzle_hash.convert()?,
        receiver_puzzle_hash: row.receiver_puzzle_hash.convert()?,
        seconds: row.expiration_seconds.convert()?,
    })
}

async fn derivation(
    conn: impl SqliteExecutor<'_>,
    public_key: PublicKey,
) -> Result<Option<Derivation>> {
    let public_key = public_key.to_bytes();
    let public_key = public_key.as_ref();

    let Some(row) = query!(
        "SELECT derivation_index, is_hardened FROM public_keys WHERE key = ?",
        public_key
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Derivation {
        derivation_index: row.derivation_index.convert()?,
        is_hardened: row.is_hardened,
    }))
}
