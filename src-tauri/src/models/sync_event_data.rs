use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncEventData {
    Start { ip: String },
    Stop,
    Subscribed,
    CoinUpdate,
}
