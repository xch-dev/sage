use chia::protocol::Bytes32;
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
    CoinsUpdated {
        coin_ids: Vec<Bytes32>,
    },
    TransactionUpdated {
        transaction_id: Bytes32,
    },
    TransactionConfirmed {
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
