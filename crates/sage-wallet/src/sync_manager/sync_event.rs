use std::net::IpAddr;

use chia::protocol::{Bytes32, CoinState};
use sage_database::OfferStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncEvent {
    Start(IpAddr),
    Stop,
    Subscribed,
    DerivationIndex {
        next_index: u32,
    },
    CoinsUpdated {
        coin_states: Vec<CoinState>,
    },
    TransactionEnded {
        transaction_id: Bytes32,
        success: bool,
    },
    OfferUpdated {
        offer_id: Bytes32,
        status: OfferStatus,
    },
    PuzzleBatchSynced,
    CatInfo,
    DidInfo,
    NftData,
}
