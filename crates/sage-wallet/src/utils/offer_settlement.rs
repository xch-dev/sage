use std::collections::HashMap;

use chia_wallet_sdk::{
    clvm_utils::{TreeHash, tree_hash_from_bytes},
    prelude::*,
};
use tracing::warn;

use crate::{WalletError, WalletPeer};

/// What taking an offer leaves on-chain: its settlement coins, created by the maker's spends.
///
/// A settlement coin's id only depends on its parent, the settlement puzzle hash, and the amount,
/// so another offer which spends the same coin for the same amount creates the same settlement
/// coin. The offer's terms, such as what it requests and when it expires, are often only in the
/// conditions of one of the maker's spends, which may not create a settlement coin. So the offer
/// was only taken if every one of the maker's spends was confirmed as it is in the offer.
#[derive(Debug, Clone)]
pub struct OfferSettlement {
    coin_ids: Vec<Bytes32>,
    /// The tree hash of each of the maker's solutions, by coin id. A coin's puzzle is fixed by its
    /// puzzle hash, so only the solution can differ.
    solution_hashes: HashMap<Bytes32, Option<TreeHash>>,
}

impl OfferSettlement {
    pub fn new(offer: &Offer) -> Self {
        Self {
            coin_ids: offer
                .offered_coins()
                .flatten()
                .iter()
                .map(Coin::coin_id)
                .collect(),
            solution_hashes: offer
                .spend_bundle()
                .coin_spends
                .iter()
                .map(|coin_spend| {
                    (
                        coin_spend.coin.coin_id(),
                        tree_hash_from_bytes(&coin_spend.solution).ok(),
                    )
                })
                .collect(),
        }
    }

    pub fn coin_ids(&self) -> impl Iterator<Item = Bytes32> + '_ {
        self.coin_ids.iter().copied()
    }

    /// Whether the offer was taken. The coin states should include those of the settlement coins,
    /// and the peer is only asked about the maker's spends if all of them were created.
    pub async fn is_settled(
        &self,
        peer: &WalletPeer,
        coin_states: &HashMap<Bytes32, CoinState>,
        genesis_challenge: Bytes32,
    ) -> Result<bool, WalletError> {
        // Taking the offer spends all of the maker's coins in one block, creating every
        // settlement coin in it.
        let mut height = None;

        for coin_id in &self.coin_ids {
            let Some(created_height) = coin_states
                .get(coin_id)
                .and_then(|coin_state| coin_state.created_height)
            else {
                return Ok(false);
            };

            if height.is_some_and(|height| height != created_height) {
                return Ok(false);
            }

            height = Some(created_height);
        }

        let Some(height) = height else {
            return Ok(false);
        };

        // Coins the other spends didn't include, or spent in another block, can't have been spent
        // by taking this offer, and there's no spend to look up for them at this height.
        let spent_coins = peer
            .fetch_coins(
                self.solution_hashes.keys().copied().collect(),
                genesis_challenge,
            )
            .await?;

        if spent_coins.len() != self.solution_hashes.len()
            || spent_coins
                .iter()
                .any(|coin_state| coin_state.spent_height != Some(height))
        {
            return Ok(false);
        }

        for (&coin_id, &solution_hash) in &self.solution_hashes {
            let Some(solution_hash) = solution_hash else {
                return Ok(false);
            };

            let (_puzzle, solution) = peer.fetch_puzzle_solution(coin_id, height).await?;

            if tree_hash_from_bytes(&solution).ok() != Some(solution_hash) {
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
        genesis_challenge: Bytes32,
    ) -> Result<bool, WalletError> {
        let mut last_error = WalletError::NoPeers;

        for peer in peers {
            match self.is_settled(peer, coin_states, genesis_challenge).await {
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
