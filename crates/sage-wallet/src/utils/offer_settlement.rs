use std::collections::HashMap;

use chia_wallet_sdk::{
    clvm_utils::{TreeHash, tree_hash_from_bytes},
    prelude::*,
};
use tracing::warn;

use crate::{MAX_PEER_ATTEMPTS, WalletError, WalletPeer};

/// What taking an offer leaves on-chain: its settlement coins, created by the maker's spends.
///
/// A settlement coin's id only depends on its parent, the settlement puzzle hash, and the amount,
/// so another offer which spends the same coin for the same amount creates the same settlement
/// coin. The offer's terms, such as what it requests and when it expires, are often only in the
/// conditions of one of the maker's spends, which may not create a settlement coin. So the offer
/// was only taken if every one of the maker's spends was confirmed as it is in the offer.
#[derive(Debug, Clone)]
pub struct OfferSettlement {
    settlement_coin_ids: Vec<Bytes32>,
    /// The tree hash of each of the maker's solutions, by coin id. A coin's puzzle is fixed by its
    /// puzzle hash, so only the solution can differ.
    solution_hashes: HashMap<Bytes32, Option<TreeHash>>,
}

impl OfferSettlement {
    pub fn new(offer: &Offer) -> Self {
        Self {
            settlement_coin_ids: offer
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

    /// The coins whose states [`Self::is_settled`] needs: the settlement coins, and the coins of
    /// the maker's spends.
    pub fn coin_ids(&self) -> impl Iterator<Item = Bytes32> + '_ {
        self.settlement_coin_ids
            .iter()
            .chain(self.solution_hashes.keys())
            .copied()
    }

    /// Whether the offer was taken, given the coin states of [`Self::coin_ids`] from the same
    /// peer. The peer is only asked for the maker's spends if they're all confirmed in the block
    /// that created the settlement coins.
    pub async fn is_settled(
        &self,
        peer: &WalletPeer,
        coin_states: &HashMap<Bytes32, CoinState>,
    ) -> Result<bool, WalletError> {
        // Taking the offer spends all of the maker's coins in one block, creating every
        // settlement coin in it.
        let mut height = None;

        for coin_id in &self.settlement_coin_ids {
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

        // Coins another offer's spends didn't include, or that were spent in another block, can't
        // have been spent by taking this offer, and have no spend to look up at this height.
        if self.solution_hashes.keys().any(|coin_id| {
            coin_states
                .get(coin_id)
                .is_none_or(|coin_state| coin_state.spent_height != Some(height))
        }) {
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

    /// Like [`Self::is_settled`], but tries each peer in turn until one answers. The coin states
    /// are still the ones the caller fetched, so a peer that's behind them reports an error for a
    /// missing spend rather than that the offer wasn't taken.
    pub async fn is_settled_on_peers(
        &self,
        peers: &[WalletPeer],
        coin_states: &HashMap<Bytes32, CoinState>,
    ) -> Result<bool, WalletError> {
        let mut last_error = WalletError::NoPeers;

        for peer in peers.iter().take(MAX_PEER_ATTEMPTS) {
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
