use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;

/// Converts a wallet's sync event into the `sync-event` payload the frontend listens for, or
/// returns `None` for events the frontend doesn't need.
pub fn api_sync_event(event: SyncEvent) -> Option<ApiEvent> {
    Some(match event {
        SyncEvent::Start(ip) => ApiEvent::Start { ip: ip.to_string() },
        SyncEvent::Stop => ApiEvent::Stop,
        SyncEvent::Subscribed => ApiEvent::Subscribed,
        SyncEvent::DerivationIndex { .. } => ApiEvent::Derivation,
        SyncEvent::TransactionFailed {
            transaction_id,
            error,
        } => ApiEvent::TransactionFailed {
            transaction_id: transaction_id.to_string(),
            error,
        },
        // TODO: New event?
        SyncEvent::CoinsUpdated
        | SyncEvent::TransactionUpdated { .. }
        | SyncEvent::OfferUpdated { .. } => ApiEvent::CoinState,
        SyncEvent::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
        SyncEvent::CatInfo => ApiEvent::CatInfo,
        SyncEvent::DidInfo => ApiEvent::DidInfo,
        SyncEvent::NftData => ApiEvent::NftData,
        SyncEvent::WalletChanged { .. } | SyncEvent::NetworkChanged { .. } => return None,
    })
}

/// Refresh events, which tell the frontend "this kind of data changed, refetch".
///
/// A burst of these can be collapsed into one of each kind before being sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefreshEvent {
    Derivation,
    CoinState,
    PuzzleBatchSynced,
    CatInfo,
    DidInfo,
    NftData,
}

impl RefreshEvent {
    /// Returns `None` for events that must be delivered as they happen, either
    /// because they carry a payload or because they are one-off state changes.
    pub fn from_api_event(event: &ApiEvent) -> Option<Self> {
        match event {
            ApiEvent::Derivation => Some(Self::Derivation),
            ApiEvent::CoinState => Some(Self::CoinState),
            ApiEvent::PuzzleBatchSynced => Some(Self::PuzzleBatchSynced),
            ApiEvent::CatInfo => Some(Self::CatInfo),
            ApiEvent::DidInfo => Some(Self::DidInfo),
            ApiEvent::NftData => Some(Self::NftData),
            ApiEvent::Start { .. }
            | ApiEvent::Stop
            | ApiEvent::Subscribed
            | ApiEvent::TransactionFailed { .. } => None,
        }
    }

    pub fn into_api_event(self) -> ApiEvent {
        match self {
            Self::Derivation => ApiEvent::Derivation,
            Self::CoinState => ApiEvent::CoinState,
            Self::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
            Self::CatInfo => ApiEvent::CatInfo,
            Self::DidInfo => ApiEvent::DidInfo,
            Self::NftData => ApiEvent::NftData,
        }
    }
}
