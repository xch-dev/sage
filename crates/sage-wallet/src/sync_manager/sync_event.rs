use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncEvent {
    Start(IpAddr),
    Stop,
    Subscribed,
    Derivation,
    CoinState,
    PuzzleBatchSynced,
    DidInfo,
    NftData,
}
