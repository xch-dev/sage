use chia::protocol::{Bytes32, CoinState};
use sage_database::DatabaseTx;

use crate::WalletError;

pub async fn upsert_coin(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    transaction_id: Option<Bytes32>,
) -> Result<(), WalletError> {
    let coin_id = coin_state.coin.coin_id();

    // Check if the coin is plain XCH, rather than an asset that wraps the p2 puzzle hash.
    let is_p2 = tx.is_p2_puzzle_hash(coin_state.coin.puzzle_hash).await?;

    // If the coin is XCH, there's no reason to sync the puzzle.
    tx.insert_coin_state(coin_state, is_p2, transaction_id)
        .await?;

    // If the coin already existed, instead of replacing it we will just update it.
    tx.update_coin_state(
        coin_id,
        coin_state.created_height,
        coin_state.spent_height,
        transaction_id,
    )
    .await?;

    // This allows querying for XCH coins without joining on the derivations table.
    if is_p2 {
        tx.insert_p2_coin(coin_id).await?;
    }

    if coin_state.spent_height.is_some() {
        spend_coin(tx, coin_id).await?;
    }

    Ok(())
}

pub async fn spend_coin(tx: &mut DatabaseTx<'_>, coin_id: Bytes32) -> Result<(), WalletError> {
    if let Some(transaction_id) = tx.transaction_for_spent_coin(coin_id).await? {
        tx.remove_transaction(transaction_id).await?;
    }

    if let Some(launcher_id) = tx.nft_launcher_id(coin_id).await? {
        tx.delete_nft(launcher_id).await?;
    }

    Ok(())
}
