use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, Result};

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
}

async fn custody_p2_puzzle_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM p2_puzzles WHERE kind = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
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
