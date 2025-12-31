use chia::protocol::{Bytes32, CoinState};
use sage_database::OfferStatus;
use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoinStateInfo {
    pub coin_id: Bytes32,
    pub puzzle_hash: Bytes32,
    pub amount: u64,
    pub created_height: Option<u32>,
    pub spent_height: Option<u32>,
}

impl CoinStateInfo {
    pub fn from_coin_state(cs: &CoinState) -> Self {
        Self {
            coin_id: cs.coin.coin_id(),
            puzzle_hash: cs.coin.puzzle_hash,
            amount: cs.coin.amount,
            created_height: cs.created_height,
            spent_height: cs.spent_height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncEvent {
    Start(IpAddr),
    Stop,
    Subscribed,
    DerivationIndex {
        next_index: u32,
    },
    CoinsUpdated {
        coin_states: Vec<CoinStateInfo>,
    },
    TransactionUpdated {
        transaction_id: Bytes32,
    },
    TransactionFailed {
        transaction_id: Bytes32,
        error: Option<String>,
    },
    OfferUpdated {
        offer_id: Bytes32,
        status: OfferStatus,
    },
    PuzzleBatchSynced,
    CatInfo {
        asset_ids: Vec<Bytes32>,
    },
    DidInfo {
        launcher_id: Bytes32,
    },
    NftData {
        launcher_ids: Vec<Bytes32>,
    },
    WebhooksChanged,
    WebhookInvoked,
}
