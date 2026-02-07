use std::{collections::HashMap, net::SocketAddr, time::Duration};

use chia_wallet_sdk::{
    chia::protocol::{
        CoinStateFilters, RejectStateReason, RequestBlockHeader, RespondBlockHeader, RespondPeers,
        RespondPuzzleState, TransactionAck,
    },
    prelude::*,
};
use tokio::time::timeout;

use crate::WalletError;

#[derive(Debug, Clone)]
pub struct WalletPeer {
    peer: Peer,
    pending_coin_states: HashMap<Bytes32, CoinState>,
    pending_coin_spends: HashMap<Bytes32, CoinSpend>,
}

impl WalletPeer {
    pub fn new(peer: Peer) -> Self {
        Self {
            peer,
            pending_coin_states: HashMap::new(),
            pending_coin_spends: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_pending(
        &self,
        pending_coin_states: HashMap<Bytes32, CoinState>,
        pending_coin_spends: HashMap<Bytes32, CoinSpend>,
    ) -> Self {
        Self {
            peer: self.peer.clone(),
            pending_coin_states,
            pending_coin_spends,
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.peer.socket_addr()
    }

    pub async fn fetch_coin(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<CoinState, WalletError> {
        if let Some(coin_state) = self.pending_coin_states.get(&coin_id) {
            return Ok(*coin_state);
        }

        let Some(coin_state) = timeout(
            Duration::from_secs(5),
            self.peer
                .request_coin_state(vec![coin_id], None, genesis_challenge, false),
        )
        .await??
        .map_err(|_| WalletError::PeerMisbehaved)?
        .coin_states
        .into_iter()
        .next() else {
            return Err(WalletError::MissingCoin(coin_id));
        };

        Ok(coin_state)
    }

    pub async fn fetch_optional_coin(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<Option<CoinState>, WalletError> {
        if let Some(coin_state) = self.pending_coin_states.get(&coin_id) {
            return Ok(Some(*coin_state));
        }

        Ok(timeout(
            Duration::from_secs(5),
            self.peer
                .request_coin_state(vec![coin_id], None, genesis_challenge, false),
        )
        .await??
        .map_err(|_| WalletError::PeerMisbehaved)?
        .coin_states
        .into_iter()
        .next())
    }

    pub async fn fetch_coins(
        &self,
        coin_ids: Vec<Bytes32>,
        genesis_challenge: Bytes32,
    ) -> Result<Vec<CoinState>, WalletError> {
        let mut coin_states = HashMap::new();
        let mut unknown_coin_ids = Vec::new();

        for &coin_id in &coin_ids {
            if let Some(coin_state) = self.pending_coin_states.get(&coin_id) {
                coin_states.insert(coin_id, *coin_state);
            } else {
                unknown_coin_ids.push(coin_id);
            }
        }

        let peer_coins = timeout(
            Duration::from_secs(10),
            self.peer
                .request_coin_state(unknown_coin_ids, None, genesis_challenge, false),
        )
        .await??
        .map_err(|_| WalletError::PeerMisbehaved)?
        .coin_states;

        for coin_state in peer_coins {
            coin_states.insert(coin_state.coin.coin_id(), coin_state);
        }

        let mut all_coins = Vec::new();

        for coin_id in coin_ids {
            if let Some(coin_state) = coin_states.get(&coin_id) {
                all_coins.push(*coin_state);
            }
        }

        Ok(all_coins)
    }

    pub async fn fetch_puzzle_solution(
        &self,
        coin_id: Bytes32,
        spent_height: u32,
    ) -> Result<(Program, Program), WalletError> {
        if let Some(coin_spend) = self.pending_coin_spends.get(&coin_id) {
            return Ok((
                coin_spend.puzzle_reveal.clone(),
                coin_spend.solution.clone(),
            ));
        }

        let response = timeout(
            Duration::from_secs(15),
            self.peer.request_puzzle_and_solution(coin_id, spent_height),
        )
        .await??
        .map_err(|_| WalletError::MissingSpend(coin_id))?;

        Ok((response.puzzle, response.solution))
    }

    pub async fn fetch_coin_spend(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<CoinSpend, WalletError> {
        if let Some(coin_spend) = self.pending_coin_spends.get(&coin_id) {
            return Ok(coin_spend.clone());
        }

        let coin_state = self.fetch_coin(coin_id, genesis_challenge).await?;
        let spent_height = coin_state.spent_height.ok_or(WalletError::PeerMisbehaved)?;
        let (puzzle_reveal, solution) = self.fetch_puzzle_solution(coin_id, spent_height).await?;
        Ok(CoinSpend::new(coin_state.coin, puzzle_reveal, solution))
    }

    pub async fn fetch_optional_coin_spend(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<Option<CoinSpend>, WalletError> {
        if let Some(coin_spend) = self.pending_coin_spends.get(&coin_id) {
            return Ok(Some(coin_spend.clone()));
        }

        let Some(coin_state) = self.fetch_optional_coin(coin_id, genesis_challenge).await? else {
            return Ok(None);
        };

        let spent_height = coin_state.spent_height.ok_or(WalletError::PeerMisbehaved)?;
        let (puzzle_reveal, solution) = self.fetch_puzzle_solution(coin_id, spent_height).await?;

        Ok(Some(CoinSpend::new(
            coin_state.coin,
            puzzle_reveal,
            solution,
        )))
    }

    pub async fn try_fetch_singleton_child(
        &self,
        coin_id: Bytes32,
    ) -> Result<Option<CoinState>, WalletError> {
        if let Some(child) = self
            .pending_coin_states
            .values()
            .find(|state| state.coin.parent_coin_info == coin_id && state.coin.amount % 2 == 1)
        {
            return Ok(Some(*child));
        }

        Ok(
            timeout(Duration::from_secs(5), self.peer.request_children(coin_id))
                .await??
                .coin_states
                .into_iter()
                .find(|child| child.coin.amount % 2 == 1),
        )
    }

    pub async fn send_transaction(
        &self,
        spend_bundle: SpendBundle,
    ) -> Result<TransactionAck, WalletError> {
        Ok(timeout(
            Duration::from_secs(15),
            self.peer.send_transaction(spend_bundle),
        )
        .await??)
    }

    pub async fn unsubscribe(&self) -> Result<(), WalletError> {
        timeout(
            Duration::from_secs(10),
            self.peer.remove_puzzle_subscriptions(None),
        )
        .await??;
        timeout(
            Duration::from_secs(10),
            self.peer.remove_coin_subscriptions(None),
        )
        .await??;

        Ok(())
    }

    pub async fn request_peers(&self) -> Result<RespondPeers, WalletError> {
        Ok(timeout(Duration::from_secs(15), self.peer.request_peers())
            .await
            .unwrap_or_else(|_| Ok(RespondPeers::new(Vec::new())))?)
    }

    pub async fn subscribe_coins(
        &self,
        coin_ids: Vec<Bytes32>,
        previous_height: Option<u32>,
        header_hash: Bytes32,
    ) -> Result<Vec<CoinState>, WalletError> {
        let response = timeout(
            Duration::from_secs(15),
            self.peer
                .request_coin_state(coin_ids, previous_height, header_hash, true),
        )
        .await??
        .map_err(|error| match error.reason {
            RejectStateReason::ExceededSubscriptionLimit => WalletError::SubscriptionLimitReached,
            RejectStateReason::Reorg => WalletError::PeerMisbehaved,
        })?;

        Ok(response.coin_states)
    }

    pub async fn subscribe_puzzles(
        &self,
        puzzle_hashes: Vec<Bytes32>,
        previous_height: Option<u32>,
        header_hash: Bytes32,
        filters: CoinStateFilters,
    ) -> Result<RespondPuzzleState, WalletError> {
        timeout(
            Duration::from_secs(45),
            self.peer.request_puzzle_state(
                puzzle_hashes,
                previous_height,
                header_hash,
                filters,
                true,
            ),
        )
        .await??
        .map_err(|error| match error.reason {
            RejectStateReason::ExceededSubscriptionLimit => WalletError::SubscriptionLimitReached,
            RejectStateReason::Reorg => WalletError::PeerMisbehaved,
        })
    }

    pub async fn unsubscribe_coins(&self, coin_ids: Vec<Bytes32>) -> Result<(), WalletError> {
        timeout(
            Duration::from_secs(10),
            self.peer.remove_coin_subscriptions(Some(coin_ids)),
        )
        .await??;

        Ok(())
    }

    pub async fn block_timestamp(&self, height: u32) -> Result<(Bytes32, u64), WalletError> {
        let header_block = timeout(
            Duration::from_secs(5),
            self.peer
                .request_infallible::<RespondBlockHeader, _>(RequestBlockHeader::new(height)),
        )
        .await??
        .header_block;

        let timestamp = header_block
            .foliage_transaction_block
            .as_ref()
            .map(|block| block.timestamp);

        Ok((
            header_block.header_hash(),
            timestamp.ok_or(WalletError::PeerMisbehaved)?,
        ))
    }
}
