use std::collections::HashMap;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::Bytes32,
};
use chia_wallet_sdk::{
    driver::ParsedOffer,
    types::{run_puzzle, Condition},
};
use clvmr::{Allocator, NodePtr};
use sage_wallet::WalletPeer;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct OfferExpiration {
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct CoinCreation {
    pub previous_height: u32,
    pub previous_timestamp: u64,
}

pub async fn lookup_coin_creation(
    peer: &WalletPeer,
    coin_ids: Vec<Bytes32>,
    genesis_challenge: Bytes32,
) -> Result<HashMap<Bytes32, CoinCreation>> {
    let coin_states_list = peer
        .fetch_coins(coin_ids.clone(), genesis_challenge)
        .await?;

    let mut coin_states = HashMap::new();

    for coin_state in coin_states_list {
        coin_states.insert(coin_state.coin.coin_id(), coin_state);
    }

    let mut coin_creation = HashMap::new();

    for coin_id in coin_ids {
        let coin_state = coin_states
            .get(&coin_id)
            .ok_or(Error::MissingCoin(coin_id))?;

        let previous_height = coin_state
            .created_height
            .map_or(0, |height| height.saturating_sub(1));

        let previous_timestamp = peer.block_timestamp(previous_height).await?.unwrap_or(0);

        coin_creation.insert(
            coin_id,
            CoinCreation {
                previous_height,
                previous_timestamp,
            },
        );
    }

    Ok(coin_creation)
}

pub fn offer_expiration(
    allocator: &mut Allocator,
    offer: &ParsedOffer,
    coin_creation: &HashMap<Bytes32, CoinCreation>,
) -> Result<OfferExpiration> {
    let mut expiration_height = None::<u32>;
    let mut expiration_timestamp = None::<u64>;

    for coin_spend in &offer.coin_spends {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(allocator)?;
        let solution = coin_spend.solution.to_clvm(allocator)?;
        let output = run_puzzle(allocator, puzzle, solution)?;
        let conditions = Vec::<Condition<NodePtr>>::from_clvm(allocator, output)?;

        for condition in conditions {
            match condition {
                Condition::AssertBeforeHeightAbsolute(cond) => {
                    expiration_height = if let Some(old_height) = expiration_height {
                        Some(old_height.min(cond.height))
                    } else {
                        Some(cond.height)
                    };
                }
                Condition::AssertBeforeSecondsAbsolute(cond) => {
                    expiration_timestamp = if let Some(old_timestamp) = expiration_timestamp {
                        Some(old_timestamp.min(cond.seconds))
                    } else {
                        Some(cond.seconds)
                    };
                }
                Condition::AssertBeforeHeightRelative(cond) => {
                    let previous_height = coin_creation
                        .get(&coin_spend.coin.coin_id())
                        .map_or(0, |state| state.previous_height);

                    let height = previous_height + cond.height;

                    expiration_height = if let Some(old_height) = expiration_height {
                        Some(old_height.min(height))
                    } else {
                        Some(height)
                    };
                }
                Condition::AssertBeforeSecondsRelative(cond) => {
                    let previous_timestamp = coin_creation
                        .get(&coin_spend.coin.coin_id())
                        .map_or(0, |state| state.previous_timestamp);

                    let timestamp = previous_timestamp + cond.seconds;

                    expiration_timestamp = if let Some(old_timestamp) = expiration_timestamp {
                        Some(old_timestamp.min(timestamp))
                    } else {
                        Some(timestamp)
                    };
                }
                _ => {}
            }
        }
    }

    Ok(OfferExpiration {
        expiration_height,
        expiration_timestamp,
    })
}
