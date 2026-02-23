use chia_wallet_sdk::prelude::*;
use sage_database::OfferStatus;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncEvent {
    Start(IpAddr),
    Stop,
    Subscribed,
    DerivationIndex {
        next_index: u32,
    },
    CoinsUpdated,
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
    CatInfo,
    DidInfo,
    NftData,
}
