use std::collections::HashMap;

use chia_wallet_sdk::{clvm_utils::tree_hash_from_bytes, prelude::*};
use tracing::warn;

use crate::{WalletError, WalletPeer};

/// The coins an offer creates when it's taken, and the spends in the offer that create them.
///
/// A settlement coin's id only depends on its parent, the settlement puzzle hash, and the amount,
/// so another offer which spends the same coin for the same amount creates the same settlement
/// coin. Its existence alone doesn't mean this offer was taken.
#[derive(Debug, Clone)]
pub struct OfferSettlement {
    coins: Vec<Coin>,
    parent_spends: HashMap<Bytes32, CoinSpend>,
}

impl OfferSettlement {
    pub fn new(offer: &Offer) -> Self {
        let coins = offer.offered_coins().flatten();

        let parent_spends = offer
            .spend_bundle()
            .coin_spends
            .iter()
            .filter(|coin_spend| {
                coins
                    .iter()
                    .any(|coin| coin.parent_coin_info == coin_spend.coin.coin_id())
            })
            .map(|coin_spend| (coin_spend.coin.coin_id(), coin_spend.clone()))
            .collect();

        Self {
            coins,
            parent_spends,
        }
    }

    pub fn coin_ids(&self) -> impl Iterator<Item = Bytes32> + '_ {
        self.coins.iter().map(Coin::coin_id)
    }

    /// Whether the offer was taken. Taking an offer creates all of its settlement coins at once,
    /// by spending their parents the way the offer does, so each parent has to have been spent
    /// with the offer's solution. The parent's puzzle is fixed by its puzzle hash, so only the
    /// solution can differ.
    ///
    /// The coin states should include those of the settlement coins, and this only asks the peer
    /// for the parent spends if all of them were created.
    pub async fn is_settled(
        &self,
        peer: &WalletPeer,
        coin_states: &HashMap<Bytes32, CoinState>,
    ) -> Result<bool, WalletError> {
        if self.coins.is_empty() {
            return Ok(false);
        }

        // A coin is created in the block its parent is spent in.
        let mut parent_heights = HashMap::new();

        for coin in &self.coins {
            let Some(created_height) = coin_states
                .get(&coin.coin_id())
                .and_then(|coin_state| coin_state.created_height)
            else {
                return Ok(false);
            };

            parent_heights.insert(coin.parent_coin_info, created_height);
        }

        for (parent_coin_id, spent_height) in parent_heights {
            let Some(coin_spend) = self.parent_spends.get(&parent_coin_id) else {
                return Ok(false);
            };

            let (_puzzle, solution) = peer
                .fetch_puzzle_solution(parent_coin_id, spent_height)
                .await?;

            if !same_program(&solution, &coin_spend.solution) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Like [`Self::is_settled`], but tries each peer in turn until one answers.
    pub async fn is_settled_on_peers(
        &self,
        peers: &[WalletPeer],
        coin_states: &HashMap<Bytes32, CoinState>,
    ) -> Result<bool, WalletError> {
        let mut last_error = WalletError::NoPeers;

        for peer in peers {
            match self.is_settled(peer, coin_states).await {
                Ok(is_settled) => return Ok(is_settled),
                Err(error) => {
                    warn!(
                        "Failed to check offer settlement with peer {}: {error}",
                        peer.socket_addr()
                    );
                    last_error = error;
                }
            }
        }

        Err(last_error)
    }
}

/// Compares programs by tree hash, since the same program can be serialized in more than one way.
fn same_program(a: &Program, b: &Program) -> bool {
    match (tree_hash_from_bytes(a), tree_hash_from_bytes(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
