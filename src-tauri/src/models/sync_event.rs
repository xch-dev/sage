use chia::protocol::Bytes32;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncEvent {
    Start {
        ip: String,
    },
    Stop,
    Subscribed,
    CoinUpdate {
        #[serde_as(as = "Vec<Hex>")]
        coin_ids: Vec<Bytes32>,
    },
}

impl From<sage_wallet::SyncEvent> for SyncEvent {
    fn from(value: sage_wallet::SyncEvent) -> Self {
        #[allow(clippy::enum_glob_use)]
        use sage_wallet::SyncEvent::*;

        match value {
            Start(ip) => Self::Start { ip: ip.to_string() },
            Stop => Self::Stop,
            Subscribed => Self::Subscribed,
            CoinUpdate(coin_ids) => Self::CoinUpdate { coin_ids },
        }
    }
}
