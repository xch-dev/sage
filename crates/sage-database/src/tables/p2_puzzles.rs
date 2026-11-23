use chia_wallet_sdk::{
    driver::{
        Clawback as ClawbackV1, ClawbackV2, OptionType, OptionUnderlying, P2ConditionsOrSingleton,
        P2ConditionsOrSingletonVersion, P2Singleton,
    },
    prelude::*,
    types::puzzles::P2DelegatedConditionsArgs,
};
use sqlx::{SqliteExecutor, query};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum P2PuzzleKind {
    PublicKey,
    Clawback,
    Option,
    Arbor,
    /// A puzzle hash that's watched, whose puzzle reveal is unknown.
    External,
    /// A vault address, spent by a message from the vault singleton.
    P2Singleton,
    /// A coin locked to a fixed set of conditions, which the vault can also spend.
    P2ConditionsOrSingleton,
}

#[derive(Debug, Clone, Copy)]
pub enum P2Puzzle {
    PublicKey(PublicKey),
    Clawback(Clawback),
    Option(Underlying),
    Arbor(PublicKey),
    /// A watched puzzle hash. Its coins can't be spent, since the puzzle reveal is unknown.
    External,
    P2Singleton(P2Singleton),
    P2ConditionsOrSingleton(ConditionsOrSingleton),
}

#[derive(Debug, Clone, Copy)]
pub struct ConditionsOrSingleton {
    pub p2: P2ConditionsOrSingleton,
    /// The version of the fixed path, which is only known once its conditions are revealed.
    pub version: Option<P2ConditionsOrSingletonVersion>,
}

#[derive(Debug, Clone, Copy)]
pub struct Clawback {
    pub public_key: Option<PublicKey>,
    pub sender_puzzle_hash: Bytes32,
    pub receiver_puzzle_hash: Bytes32,
    pub seconds: u64,
    pub version: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Underlying {
    pub public_key: PublicKey,
    pub launcher_id: Bytes32,
    pub creator_puzzle_hash: Bytes32,
    pub seconds: u64,
    pub amount: u64,
    pub strike_type: OptionType,
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
                    return Err(DatabaseError::PublicKeyNotFound);
                };

                Ok(P2Puzzle::PublicKey(key))
            }
            P2PuzzleKind::Clawback => {
                Ok(P2Puzzle::Clawback(clawback(&self.pool, puzzle_hash).await?))
            }
            P2PuzzleKind::Option => {
                let launcher_id = underlying_launcher_id(&self.pool, puzzle_hash).await?;
                let underlying = self
                    .option_underlying(launcher_id)
                    .await?
                    .ok_or(DatabaseError::OptionUnderlyingNotFound)?;

                let Some(key) = public_key(&self.pool, underlying.creator_puzzle_hash).await?
                else {
                    return Err(DatabaseError::PublicKeyNotFound);
                };

                Ok(P2Puzzle::Option(Underlying {
                    public_key: key,
                    launcher_id,
                    creator_puzzle_hash: underlying.creator_puzzle_hash,
                    seconds: underlying.seconds,
                    amount: underlying.amount,
                    strike_type: underlying.strike_type,
                }))
            }
            P2PuzzleKind::Arbor => {
                let Some(key) = arbor_key(&self.pool, puzzle_hash).await? else {
                    return Err(DatabaseError::PublicKeyNotFound);
                };

                Ok(P2Puzzle::Arbor(key))
            }
            P2PuzzleKind::External => Ok(P2Puzzle::External),
            P2PuzzleKind::P2Singleton => Ok(P2Puzzle::P2Singleton(
                p2_singleton(&self.pool, puzzle_hash).await?,
            )),
            P2PuzzleKind::P2ConditionsOrSingleton => Ok(P2Puzzle::P2ConditionsOrSingleton(
                p2_conditions_or_singleton(&self.pool, puzzle_hash).await?,
            )),
        }
    }

    /// The fixed conditions of a P2 conditions or singleton puzzle, if they've been revealed.
    pub async fn p2_conditions_or_singleton_conditions(
        &self,
        p2_puzzle_hash: Bytes32,
    ) -> Result<Option<Program>> {
        p2_conditions_or_singleton_conditions(&self.pool, p2_puzzle_hash).await
    }

    /// The P2 singleton puzzle hashes of a vault, ordered by nonce.
    pub async fn p2_singleton_puzzle_hashes(&self, launcher_id: Bytes32) -> Result<Vec<Bytes32>> {
        p2_singleton_puzzle_hashes(&self.pool, launcher_id).await
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

    pub async fn max_derivation_index(&self, is_hardened: bool) -> Result<Option<u32>> {
        max_derivation_index(&self.pool, is_hardened).await
    }

    pub async fn external_p2_puzzle_hashes(&self) -> Result<Vec<Bytes32>> {
        external_p2_puzzle_hashes(&self.pool).await
    }

    /// Custody puzzle hashes that haven't been synced from the start of the chain to the end yet.
    pub async fn unsynced_custody_p2_puzzle_hashes(&self) -> Result<Vec<Bytes32>> {
        unsynced_custody_p2_puzzle_hashes(&self.pool).await
    }

    /// Marks puzzle hashes as synced from the start of the chain to the end.
    pub async fn mark_p2_puzzles_synced(&self, p2_puzzle_hashes: &[Bytes32]) -> Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        for &p2_puzzle_hash in p2_puzzle_hashes {
            mark_p2_puzzle_synced(&mut *tx, p2_puzzle_hash).await?;
        }

        tx.commit().await?;

        Ok(())
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

    pub async fn insert_clawback_v1_p2_puzzle(&mut self, clawback: ClawbackV1) -> Result<()> {
        insert_clawback_v1_p2_puzzle(&mut *self.tx, clawback).await
    }

    pub async fn insert_option_p2_puzzle(&mut self, underlying: OptionUnderlying) -> Result<()> {
        insert_option_p2_puzzle(&mut *self.tx, underlying).await
    }

    pub async fn insert_arbor_p2_puzzle(&mut self, key: PublicKey) -> Result<()> {
        insert_arbor_p2_puzzle(&mut *self.tx, key).await
    }

    pub async fn external_p2_puzzle_hashes(&mut self) -> Result<Vec<Bytes32>> {
        external_p2_puzzle_hashes(&mut *self.tx).await
    }

    /// Watches a puzzle hash, so that its coins count as the wallet's own.
    pub async fn insert_external_p2_puzzle(&mut self, p2_puzzle_hash: Bytes32) -> Result<()> {
        insert_external_p2_puzzle(&mut *self.tx, p2_puzzle_hash).await
    }

    /// Stops watching a puzzle hash, and deletes the coins that were synced for it. They're
    /// synced again from scratch if it's watched again.
    pub async fn delete_external_p2_puzzle(&mut self, p2_puzzle_hash: Bytes32) -> Result<()> {
        delete_external_p2_puzzle(&mut *self.tx, p2_puzzle_hash).await
    }

    /// The vault address with the puzzle hash, if it's one of ours.
    pub async fn p2_singleton(&mut self, p2_puzzle_hash: Bytes32) -> Result<Option<P2Singleton>> {
        optional_p2_singleton(&mut *self.tx, p2_puzzle_hash).await
    }

    /// The highest nonce of a vault's addresses, and the highest nonce of an address that has
    /// received a coin, if any.
    pub async fn p2_singleton_nonces(
        &mut self,
        launcher_id: Bytes32,
    ) -> Result<(Option<u32>, Option<u32>)> {
        p2_singleton_nonces(&mut *self.tx, launcher_id).await
    }

    /// Inserts a vault address. It isn't marked as synced, so it's synced from the start of the
    /// chain.
    pub async fn insert_p2_singleton_p2_puzzle(&mut self, p2_singleton: P2Singleton) -> Result<()> {
        insert_p2_singleton_p2_puzzle(&mut *self.tx, p2_singleton).await
    }

    /// Inserts a P2 conditions or singleton puzzle. If its fixed conditions are passed, they're
    /// stored along with their version, including for a puzzle that already exists.
    pub async fn insert_p2_conditions_or_singleton_p2_puzzle(
        &mut self,
        p2: P2ConditionsOrSingleton,
        conditions: Option<(P2ConditionsOrSingletonVersion, Program)>,
    ) -> Result<()> {
        insert_p2_conditions_or_singleton_p2_puzzle(&mut *self.tx, p2, conditions).await
    }
}

async fn custody_p2_puzzle_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM p2_puzzles WHERE kind IN (0, 3, 4, 5)")
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
        "SELECT COUNT(*) AS count FROM p2_puzzles WHERE hash = ? AND kind IN (0, 3, 4, 5)",
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

async fn max_derivation_index(
    conn: impl SqliteExecutor<'_>,
    is_hardened: bool,
) -> Result<Option<u32>> {
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

    row.derivation_index.convert()
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
    let version: i64 = 2;

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 1);

        INSERT OR IGNORE INTO clawbacks (p2_puzzle_id, sender_puzzle_hash, receiver_puzzle_hash, expiration_seconds, version)
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?, ?, ?, ?);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        sender_puzzle_hash,
        receiver_puzzle_hash,
        seconds,
        version
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_clawback_v1_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    clawback: ClawbackV1,
) -> Result<()> {
    let p2_puzzle_hash = clawback.tree_hash().to_vec();
    let sender_puzzle_hash = clawback.sender_puzzle_hash.as_ref();
    let receiver_puzzle_hash = clawback.receiver_puzzle_hash.as_ref();
    let seconds: i64 = clawback.timelock.try_into()?;
    let version: i64 = 1;

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 1);

        INSERT OR IGNORE INTO clawbacks (p2_puzzle_id, sender_puzzle_hash, receiver_puzzle_hash, expiration_seconds, version)
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?, ?, ?, ?);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        sender_puzzle_hash,
        receiver_puzzle_hash,
        seconds,
        version,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_option_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    underlying: OptionUnderlying,
) -> Result<()> {
    let asset_hash = underlying.launcher_id.as_ref();
    let p2_puzzle_hash = underlying.tree_hash().to_vec();
    let creator_puzzle_hash = underlying.creator_puzzle_hash.as_ref();
    let seconds: i64 = underlying.seconds.try_into()?;

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 2);

        INSERT OR IGNORE INTO p2_options (p2_puzzle_id, option_asset_id, creator_puzzle_hash, expiration_seconds)
        VALUES (
            (SELECT id FROM p2_puzzles WHERE hash = ?),
            (SELECT id FROM assets WHERE hash = ?),
            ?,
            ?
        );
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        asset_hash,
        creator_puzzle_hash,
        seconds,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_arbor_p2_puzzle(conn: impl SqliteExecutor<'_>, key: PublicKey) -> Result<()> {
    let p2_puzzle_hash = P2DelegatedConditionsArgs::new(key)
        .curry_tree_hash()
        .to_vec();
    let key = key.to_bytes();
    let key = key.as_ref();

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 3);

        INSERT OR IGNORE INTO p2_arbor (p2_puzzle_id, key)
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        key,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn external_p2_puzzle_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM p2_puzzles WHERE kind = 4")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
}

async fn unsynced_custody_p2_puzzle_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM p2_puzzles WHERE kind IN (0, 3, 4, 5) AND is_synced = FALSE")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
}

async fn mark_p2_puzzle_synced(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<()> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    query!(
        "UPDATE p2_puzzles SET is_synced = TRUE WHERE hash = ?",
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_external_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<()> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    query!(
        "INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 4)",
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn delete_external_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<()> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    query!(
        "
        DELETE FROM coins WHERE p2_puzzle_id = (
            SELECT id FROM p2_puzzles WHERE hash = ? AND kind = 4
        );

        DELETE FROM p2_puzzles WHERE hash = ? AND kind = 4;
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_p2_singleton_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    p2_singleton: P2Singleton,
) -> Result<()> {
    let p2_puzzle_hash = p2_singleton.tree_hash().to_vec();
    let launcher_id = p2_singleton.launcher_id.as_ref();
    let nonce: i64 = p2_singleton.nonce.try_into()?;

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 5);

        INSERT OR IGNORE INTO p2_singletons (p2_puzzle_id, launcher_id, nonce)
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?, ?);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        launcher_id,
        nonce,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_p2_conditions_or_singleton_p2_puzzle(
    conn: impl SqliteExecutor<'_>,
    p2: P2ConditionsOrSingleton,
    conditions: Option<(P2ConditionsOrSingletonVersion, Program)>,
) -> Result<()> {
    let p2_puzzle_hash = p2.tree_hash().to_vec();
    let launcher_id = p2.launcher_id.as_ref();
    let nonce: i64 = p2.nonce.try_into()?;
    let fixed_conditions_hash = p2.fixed_conditions_hash.as_ref();
    let version = conditions.as_ref().map(|(version, _)| match version {
        P2ConditionsOrSingletonVersion::Legacy => 0,
        P2ConditionsOrSingletonVersion::FixedPuzzleMember => 1,
    });
    let conditions = conditions
        .as_ref()
        .map(|(_, conditions)| conditions.as_slice());

    query!(
        "
        INSERT OR IGNORE INTO p2_puzzles (hash, kind) VALUES (?, 6);

        INSERT INTO p2_conditions_or_singletons (
            p2_puzzle_id, launcher_id, nonce, fixed_conditions_hash, version, conditions
        )
        VALUES ((SELECT id FROM p2_puzzles WHERE hash = ?), ?, ?, ?, ?, ?)
        ON CONFLICT(p2_puzzle_id) DO UPDATE SET
            version = COALESCE(excluded.version, version),
            conditions = COALESCE(excluded.conditions, conditions);
        ",
        p2_puzzle_hash,
        p2_puzzle_hash,
        launcher_id,
        nonce,
        fixed_conditions_hash,
        version,
        conditions,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn p2_singleton(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<P2Singleton> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!(
        "
        SELECT launcher_id, nonce
        FROM p2_puzzles
        INNER JOIN p2_singletons ON p2_singletons.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?;

    Ok(P2Singleton::new(
        row.launcher_id.convert()?,
        row.nonce.try_into()?,
    ))
}

async fn optional_p2_singleton(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<Option<P2Singleton>> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    query!(
        "
        SELECT launcher_id, nonce
        FROM p2_puzzles
        INNER JOIN p2_singletons ON p2_singletons.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_optional(conn)
    .await?
    .map(|row| {
        Ok(P2Singleton::new(
            row.launcher_id.convert()?,
            row.nonce.try_into()?,
        ))
    })
    .transpose()
}

async fn p2_singleton_nonces(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<(Option<u32>, Option<u32>)> {
    let launcher_id = launcher_id.as_ref();

    let row = query!(
        "
        SELECT
            MAX(p2_singletons.nonce) AS \"max_nonce?: i64\",
            MAX(CASE WHEN EXISTS (
                SELECT 1 FROM coins WHERE coins.p2_puzzle_id = p2_singletons.p2_puzzle_id
            ) THEN p2_singletons.nonce END) AS \"max_used_nonce?: i64\"
        FROM p2_singletons
        WHERE p2_singletons.launcher_id = ?
        ",
        launcher_id
    )
    .fetch_one(conn)
    .await?;

    Ok((row.max_nonce.convert()?, row.max_used_nonce.convert()?))
}

async fn p2_singleton_puzzle_hashes(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Vec<Bytes32>> {
    let launcher_id = launcher_id.as_ref();

    query!(
        "
        SELECT hash
        FROM p2_puzzles
        INNER JOIN p2_singletons ON p2_singletons.p2_puzzle_id = p2_puzzles.id
        WHERE p2_singletons.launcher_id = ?
        ORDER BY p2_singletons.nonce ASC
        ",
        launcher_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.hash.convert())
    .collect()
}

async fn p2_conditions_or_singleton(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<ConditionsOrSingleton> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!(
        "
        SELECT launcher_id, nonce, fixed_conditions_hash, version
        FROM p2_puzzles
        INNER JOIN p2_conditions_or_singletons
            ON p2_conditions_or_singletons.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?;

    Ok(ConditionsOrSingleton {
        p2: P2ConditionsOrSingleton::new(
            row.launcher_id.convert()?,
            row.nonce.try_into()?,
            row.fixed_conditions_hash.convert()?,
        ),
        version: row
            .version
            .map(|version| match version {
                0 => Ok(P2ConditionsOrSingletonVersion::Legacy),
                1 => Ok(P2ConditionsOrSingletonVersion::FixedPuzzleMember),
                _ => Err(DatabaseError::InvalidEnumVariant),
            })
            .transpose()?,
    })
}

async fn p2_conditions_or_singleton_conditions(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<Option<Program>> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    Ok(query!(
        "
        SELECT conditions
        FROM p2_puzzles
        INNER JOIN p2_conditions_or_singletons
            ON p2_conditions_or_singletons.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_optional(conn)
    .await?
    .and_then(|row| row.conditions)
    .map(Program::from))
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
        2 => P2PuzzleKind::Option,
        3 => P2PuzzleKind::Arbor,
        4 => P2PuzzleKind::External,
        5 => P2PuzzleKind::P2Singleton,
        6 => P2PuzzleKind::P2ConditionsOrSingleton,
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

    // V2: key follows absolute custody switch. V1: always sender key (claim loads receiver separately).
    let row = query!(
        "
        SELECT key AS 'key?', sender_puzzle_hash, receiver_puzzle_hash, expiration_seconds, version
        FROM p2_puzzles
        INNER JOIN clawbacks ON clawbacks.p2_puzzle_id = p2_puzzles.id
        LEFT JOIN public_keys ON public_keys.p2_puzzle_id IN (
            SELECT id FROM p2_puzzles
            WHERE (
                clawbacks.version = 2
                AND (
                    (hash = clawbacks.sender_puzzle_hash AND unixepoch() < clawbacks.expiration_seconds)
                    OR (hash = clawbacks.receiver_puzzle_hash AND unixepoch() >= clawbacks.expiration_seconds)
                )
            )
            OR (
                clawbacks.version = 1
                AND hash = clawbacks.sender_puzzle_hash
            )
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
        version: row.version.convert()?,
    })
}

async fn underlying_launcher_id(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<Bytes32> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    query!(
        "
        SELECT assets.hash AS launcher_id
        FROM p2_puzzles
        INNER JOIN p2_options ON p2_options.p2_puzzle_id = p2_puzzles.id
        INNER JOIN options ON options.asset_id = p2_options.option_asset_id
        INNER JOIN assets ON assets.id = options.asset_id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?
    .launcher_id
    .convert()
}

async fn arbor_key(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<Option<PublicKey>> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!(
        "
        SELECT key
        FROM p2_puzzles
        INNER JOIN p2_arbor ON p2_arbor.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| row.key.convert()).transpose()
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
