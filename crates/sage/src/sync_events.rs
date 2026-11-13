use std::{collections::HashSet, future, time::Duration};

use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;
use tokio::time::{Instant, sleep_until};

/// How long a burst of refresh events is collected before being sent.
pub const REFRESH_DEBOUNCE: Duration = Duration::from_millis(500);

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
enum RefreshEvent {
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
    fn from_api_event(event: &ApiEvent) -> Option<Self> {
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

    fn into_api_event(self) -> ApiEvent {
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

/// Collapses bursts of refresh events into one of each kind, sent [`REFRESH_DEBOUNCE`] after the
/// first, so that the frontend doesn't refetch the same data many times during a sync.
#[derive(Debug, Default)]
pub struct RefreshDebouncer {
    pending: HashSet<RefreshEvent>,
    flush_at: Option<Instant>,
}

impl RefreshDebouncer {
    /// Returns the event if it must be sent now, because it carries a payload or is a one-off
    /// state change. Refresh events are held until [`Self::flushed`] returns them.
    pub fn push(&mut self, event: ApiEvent) -> Option<ApiEvent> {
        let Some(refresh) = RefreshEvent::from_api_event(&event) else {
            return Some(event);
        };

        self.pending.insert(refresh);
        self.flush_at
            .get_or_insert_with(|| Instant::now() + REFRESH_DEBOUNCE);

        None
    }

    /// Waits until the held refresh events are due and returns them, in no particular order. This
    /// never finishes while nothing is held, and it's cancel-safe, so it can be used in `select!`.
    pub async fn flushed(&mut self) -> Vec<ApiEvent> {
        match self.flush_at {
            Some(deadline) => sleep_until(deadline).await,
            None => future::pending().await,
        }

        self.flush_at = None;
        self.pending
            .drain()
            .map(RefreshEvent::into_api_event)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chia_wallet_sdk::prelude::Bytes32;
    use sage_database::OfferStatus;
    use serde_json::{Value, json};
    use tokio::time::timeout;

    use super::*;

    fn to_json(event: &ApiEvent) -> Value {
        serde_json::to_value(event).expect("event isn't serializable")
    }

    #[test]
    fn api_sync_event_converts_every_event() {
        let id = Bytes32::default();
        let cases = [
            (
                SyncEvent::Start(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                Some(json!({ "type": "start", "ip": "127.0.0.1" })),
            ),
            (SyncEvent::Stop, Some(json!({ "type": "stop" }))),
            (SyncEvent::Subscribed, Some(json!({ "type": "subscribed" }))),
            (
                SyncEvent::DerivationIndex { next_index: 5 },
                Some(json!({ "type": "derivation" })),
            ),
            (
                SyncEvent::CoinsUpdated,
                Some(json!({ "type": "coin_state" })),
            ),
            (
                SyncEvent::TransactionUpdated { transaction_id: id },
                Some(json!({ "type": "coin_state" })),
            ),
            (
                SyncEvent::TransactionFailed {
                    transaction_id: id,
                    error: Some("rejected".to_string()),
                },
                Some(json!({
                    "type": "transaction_failed",
                    "transaction_id": id.to_string(),
                    "error": "rejected",
                })),
            ),
            (
                SyncEvent::OfferUpdated {
                    offer_id: id,
                    status: OfferStatus::Active,
                },
                Some(json!({ "type": "coin_state" })),
            ),
            (
                SyncEvent::PuzzleBatchSynced,
                Some(json!({ "type": "puzzle_batch_synced" })),
            ),
            (SyncEvent::CatInfo, Some(json!({ "type": "cat_info" }))),
            (SyncEvent::DidInfo, Some(json!({ "type": "did_info" }))),
            (SyncEvent::NftData, Some(json!({ "type": "nft_data" }))),
            (
                SyncEvent::WalletChanged {
                    fingerprint: Some(1),
                },
                None,
            ),
            (
                SyncEvent::NetworkChanged {
                    network_id: "testnet11".to_string(),
                },
                None,
            ),
        ];

        for (event, expected) in cases {
            let description = format!("{event:?}");
            let converted = api_sync_event(event).map(|event| to_json(&event));
            assert_eq!(converted, expected, "{description}");
        }
    }

    #[test]
    fn refresh_events_round_trip() {
        for refresh in [
            RefreshEvent::Derivation,
            RefreshEvent::CoinState,
            RefreshEvent::PuzzleBatchSynced,
            RefreshEvent::CatInfo,
            RefreshEvent::DidInfo,
            RefreshEvent::NftData,
        ] {
            assert_eq!(
                RefreshEvent::from_api_event(&refresh.into_api_event()),
                Some(refresh)
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn debouncer_collapses_refresh_bursts() {
        let mut debouncer = RefreshDebouncer::default();

        // Nothing is held, so there's nothing to wait for.
        assert!(
            timeout(Duration::from_mins(1), debouncer.flushed())
                .await
                .is_err()
        );

        // Events that aren't refreshes are sent at once.
        let stop = debouncer.push(ApiEvent::Stop);
        assert_eq!(stop.as_ref().map(to_json), Some(json!({ "type": "stop" })));

        let start = Instant::now();

        assert!(debouncer.push(ApiEvent::CoinState).is_none());
        assert!(debouncer.push(ApiEvent::CatInfo).is_none());
        assert!(debouncer.push(ApiEvent::CoinState).is_none());

        let flushed: HashSet<String> = debouncer
            .flushed()
            .await
            .iter()
            .map(|event| to_json(event).to_string())
            .collect();
        let expected: HashSet<String> = [
            json!({ "type": "coin_state" }),
            json!({ "type": "cat_info" }),
        ]
        .iter()
        .map(Value::to_string)
        .collect();

        assert_eq!(flushed, expected);
        assert_eq!(start.elapsed(), REFRESH_DEBOUNCE);

        // Once flushed, it starts over.
        assert!(
            timeout(Duration::from_mins(1), debouncer.flushed())
                .await
                .is_err()
        );
        assert!(debouncer.push(ApiEvent::NftData).is_none());
        assert_eq!(debouncer.flushed().await.len(), 1);
    }
}
