use chia_wallet_sdk::prelude::*;
use sage_database::{OfferStatus, VaultState};
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
    /// A tracked vault's state changed, for example because recovery was started, cancelled or
    /// finished, or the vault was melted.
    VaultStateChanged {
        launcher_id: Bytes32,
        state: VaultState,
    },
    CatInfo,
    DidInfo,
    NftData,
    WalletChanged {
        fingerprint: Option<u32>,
    },
    NetworkChanged {
        network_id: String,
    },
}
