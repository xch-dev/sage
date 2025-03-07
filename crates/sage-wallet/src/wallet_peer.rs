use std::net::SocketAddr;

use chia::protocol::{
    Bytes32, CoinSpend, CoinState, CoinStateFilters, Program, RejectStateReason,
    RequestBlockHeader, RespondBlockHeader, RespondPeers, RespondPuzzleState, SpendBundle,
    TransactionAck,
};
use chia_wallet_sdk::client::Peer;

use crate::WalletError;

#[derive(Debug, Clone)]
pub struct WalletPeer {
    peer: Peer,
}

impl WalletPeer {
    pub fn new(peer: Peer) -> Self {
        Self { peer }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.peer.socket_addr()
    }

    pub async fn fetch_coin(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<CoinState, WalletError> {
        let Some(coin_state) = self
            .peer
            .request_coin_state(vec![coin_id], None, genesis_challenge, false)
            .await?
            .map_err(|_| WalletError::PeerMisbehaved)?
            .coin_states
            .into_iter()
            .next()
        else {
            return Err(WalletError::MissingCoin(coin_id));
        };

        Ok(coin_state)
    }

    pub async fn fetch_optional_coin(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<Option<CoinState>, WalletError> {
        Ok(self
            .peer
            .request_coin_state(vec![coin_id], None, genesis_challenge, false)
            .await?
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
        Ok(self
            .peer
            .request_coin_state(coin_ids, None, genesis_challenge, false)
            .await?
            .map_err(|_| WalletError::PeerMisbehaved)?
            .coin_states)
    }

    pub async fn fetch_puzzle_solution(
        &self,
        coin_id: Bytes32,
        spent_height: u32,
    ) -> Result<(Program, Program), WalletError> {
        let response = self
            .peer
            .request_puzzle_and_solution(coin_id, spent_height)
            .await?
            .map_err(|_| WalletError::MissingSpend(coin_id))?;

        Ok((response.puzzle, response.solution))
    }

    pub async fn fetch_coin_spend(
        &self,
        coin_id: Bytes32,
        genesis_challenge: Bytes32,
    ) -> Result<CoinSpend, WalletError> {
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

    pub async fn fetch_child(&self, coin_id: Bytes32) -> Result<CoinState, WalletError> {
        let Some(child) = self.try_fetch_child(coin_id).await? else {
            return Err(WalletError::MissingChild(coin_id));
        };
        Ok(child)
    }

    pub async fn try_fetch_child(
        &self,
        coin_id: Bytes32,
    ) -> Result<Option<CoinState>, WalletError> {
        Ok(self
            .peer
            .request_children(coin_id)
            .await?
            .coin_states
            .into_iter()
            .next())
    }

    pub async fn send_transaction(
        &self,
        spend_bundle: SpendBundle,
    ) -> Result<TransactionAck, WalletError> {
        Ok(self.peer.send_transaction(spend_bundle).await?)
    }

    pub async fn unsubscribe(&self) -> Result<(), WalletError> {
        self.peer.remove_puzzle_subscriptions(None).await?;
        self.peer.remove_coin_subscriptions(None).await?;
        Ok(())
    }

    pub async fn request_peers(&self) -> Result<RespondPeers, WalletError> {
        Ok(self.peer.request_peers().await?)
    }

    pub async fn subscribe_coins(
        &self,
        coin_ids: Vec<Bytes32>,
        previous_height: Option<u32>,
        header_hash: Bytes32,
    ) -> Result<Vec<CoinState>, WalletError> {
        let response = self
            .peer
            .request_coin_state(coin_ids, previous_height, header_hash, true)
            .await?
            .map_err(|error| match error.reason {
                RejectStateReason::ExceededSubscriptionLimit => {
                    WalletError::SubscriptionLimitReached
                }
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
        self.peer
            .request_puzzle_state(puzzle_hashes, previous_height, header_hash, filters, true)
            .await?
            .map_err(|error| match error.reason {
                RejectStateReason::ExceededSubscriptionLimit => {
                    WalletError::SubscriptionLimitReached
                }
                RejectStateReason::Reorg => WalletError::PeerMisbehaved,
            })
    }

    pub async fn unsubscribe_coins(&self, coin_ids: Vec<Bytes32>) -> Result<(), WalletError> {
        self.peer.remove_coin_subscriptions(Some(coin_ids)).await?;
        Ok(())
    }

    pub async fn block_timestamp(&self, height: u32) -> Result<Option<u64>, WalletError> {
        Ok(self
            .peer
            .request_infallible::<RespondBlockHeader, _>(RequestBlockHeader::new(height))
            .await?
            .header_block
            .foliage_transaction_block
            .map(|block| block.timestamp))
    }
}
