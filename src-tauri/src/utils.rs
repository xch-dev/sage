use chia::protocol::Bytes32;
use sage_database::{DatabaseError, DatabaseTx};

pub async fn fetch_puzzle_hash(tx: &mut DatabaseTx<'_>) -> Result<Option<Bytes32>, DatabaseError> {
    let next_index = tx.derivation_index(false).await?;

    if next_index == 0 {
        return Ok(None);
    }

    let max = next_index - 1;
    let max_used = tx.max_used_derivation_index(false).await?;
    let mut index = max_used.map_or(0, |i| i + 1);
    if index > max {
        index = max;
    }
    let puzzle_hash = tx.p2_puzzle_hash(index, false).await?;

    Ok(Some(puzzle_hash))
}
