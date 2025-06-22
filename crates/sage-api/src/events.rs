use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type, tauri_specta::Event))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncEvent {
    Start {
        ip: String,
    },
    Stop,
    Subscribed,
    Derivation,
    CoinState,
    TransactionFailed {
        transaction_id: String,
        error: Option<String>,
    },
    PuzzleBatchSynced,
    CatInfo,
    DidInfo,
    NftData,
}
