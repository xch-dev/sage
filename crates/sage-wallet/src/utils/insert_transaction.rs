use chia::{
    bls::Signature,
    protocol::{Bytes32, CoinState},
};
use sage_database::DatabaseTx;

use crate::{database::insert_puzzle, Transaction, WalletError};

pub async fn insert_transaction(
    tx: &mut DatabaseTx<'_>,
    transaction_id: Bytes32,
    transaction: Transaction,
    aggregated_signature: Signature,
) -> Result<Vec<Bytes32>, WalletError> {
    let mut subscriptions = Vec::new();

    tx.insert_pending_transaction(transaction_id, aggregated_signature, transaction.fee)
        .await?;

    for (index, input) in transaction.inputs.into_iter().enumerate() {
        tx.insert_transaction_spend(transaction_id, input.coin_spend, index)
            .await?;

        for output in input.outputs {
            let coin_state = CoinState::new(output.coin, None, None);
            let coin_id = output.coin.coin_id();

            if tx.is_p2_puzzle_hash(output.coin.puzzle_hash).await? {
                tx.insert_coin_state(coin_state, true, Some(transaction_id))
                    .await?;
                tx.insert_p2_coin(coin_id).await?;
                continue;
            }

            let Some(p2_puzzle_hash) = output.kind.p2_puzzle_hash() else {
                continue;
            };

            if !tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                continue;
            }

            tx.insert_coin_state(coin_state, true, Some(transaction_id))
                .await?;
            tx.sync_coin(coin_id, Some(p2_puzzle_hash)).await?;

            if output.kind.subscribe() {
                subscriptions.push(coin_id);
            }

            insert_puzzle(tx, coin_state, output.kind, None).await?;
        }
    }

    Ok(subscriptions)
}
