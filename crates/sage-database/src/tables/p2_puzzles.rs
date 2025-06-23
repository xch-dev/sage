use chia::{bls::PublicKey, protocol::Bytes32};
use chia_wallet_sdk::driver::{OptionType, OptionUnderlying};
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum P2PuzzleKind {
    PublicKey,
    Clawback,
    OptionUnderlying,
}

#[derive(Debug, Clone, Copy)]
pub enum P2Puzzle {
    PublicKey(PublicKey),
    Clawback(Clawback),
    OptionUnderlying(OptionUnderlyingWithKey),
}

#[derive(Debug, Clone, Copy)]
pub struct Clawback {
    pub public_key: PublicKey,
    pub sender_puzzle_hash: Bytes32,
    pub receiver_puzzle_hash: Bytes32,
    pub seconds: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionUnderlyingWithKey {
    pub public_key: PublicKey,
    pub option: OptionUnderlying,
}

#[derive(Debug, Clone, Copy)]
pub struct Derivation {
    pub derivation_index: u32,
    pub is_hardened: bool,
}

impl Database {
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
            P2PuzzleKind::PublicKey => Ok(P2Puzzle::PublicKey(
                public_key(&self.pool, puzzle_hash).await?,
            )),
            P2PuzzleKind::Clawback => {
                Ok(P2Puzzle::Clawback(clawback(&self.pool, puzzle_hash).await?))
            }
            P2PuzzleKind::OptionUnderlying => Ok(P2Puzzle::OptionUnderlying(
                option_underlying(&self.pool, puzzle_hash).await?,
            )),
        }
    }

    pub async fn derivation(&self, public_key: PublicKey) -> Result<Option<Derivation>> {
        derivation(&self.pool, public_key).await
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
        2 => P2PuzzleKind::OptionUnderlying,
        _ => return Err(DatabaseError::InvalidEnumVariant),
    })
}

async fn public_key(conn: impl SqliteExecutor<'_>, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    query!(
        "
        SELECT key
        FROM p2_puzzles
        INNER JOIN public_keys ON public_keys.p2_puzzle_id = p2_puzzles.id
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?
    .key
    .convert()
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
            WHERE (hash = clawbacks.sender_puzzle_hash AND unixepoch() < clawbacks.expiration_seconds)
            OR (hash = clawbacks.receiver_puzzle_hash AND unixepoch() >= clawbacks.expiration_seconds)
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

async fn option_underlying(
    conn: impl SqliteExecutor<'_>,
    p2_puzzle_hash: Bytes32,
) -> Result<OptionUnderlyingWithKey> {
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();

    let row = query!(
        "
        SELECT
            key, expiration_seconds, creator_puzzle_hash,
            option_asset.hash AS launcher_id,
            strike_asset.hash AS strike_asset_hash,
            strike_asset.id AS strike_asset_id,
            strike_hidden_puzzle_hash, strike_settlement_puzzle_hash,
            coins.amount AS underlying_amount, strike_amount
        FROM p2_puzzles
        INNER JOIN p2_options ON p2_options.p2_puzzle_id = p2_puzzles.id
        INNER JOIN assets AS option_asset ON option_asset.id = p2_options.option_asset_id
        INNER JOIN options ON options.asset_id = option_asset.id
        INNER JOIN assets AS strike_asset ON strike_asset.id = options.strike_asset_id
        INNER JOIN public_keys ON public_keys.p2_puzzle_id IN (
            SELECT id FROM p2_puzzles
            WHERE hash = options.creator_puzzle_hash
            AND unixepoch() >= options.expiration_seconds
        )
        INNER JOIN coins ON coins.hash = options.underlying_coin_hash
        WHERE p2_puzzles.hash = ?
        ",
        p2_puzzle_hash
    )
    .fetch_one(conn)
    .await?;

    let Some(strike_asset_id) = row.strike_asset_id else {
        return Err(DatabaseError::IncompleteStrikeAssetInfo);
    };

    Ok(OptionUnderlyingWithKey {
        public_key: row.key.convert()?,
        option: OptionUnderlying {
            launcher_id: row.launcher_id.convert()?,
            creator_puzzle_hash: row.creator_puzzle_hash.convert()?,
            seconds: row.expiration_seconds.convert()?,
            amount: row.underlying_amount.convert()?,
            strike_type: if strike_asset_id == 0 {
                OptionType::Xch {
                    amount: row.strike_amount.convert()?,
                }
            } else if let Some(settlement_puzzle_hash) =
                row.strike_settlement_puzzle_hash.convert()?
            {
                OptionType::Nft {
                    launcher_id: row.strike_asset_hash.convert()?,
                    settlement_puzzle_hash,
                    amount: row.strike_amount.convert()?,
                }
            } else if let Some(hidden_puzzle_hash) = row.strike_hidden_puzzle_hash.convert()? {
                OptionType::RevocableCat {
                    asset_id: row.strike_asset_hash.convert()?,
                    hidden_puzzle_hash,
                    amount: row.strike_amount.convert()?,
                }
            } else {
                OptionType::Cat {
                    asset_id: row.strike_asset_hash.convert()?,
                    amount: row.strike_amount.convert()?,
                }
            },
        },
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
