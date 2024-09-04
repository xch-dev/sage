use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncEvent {
    Start { ip: String },
    Stop,
    Subscribed,
    CoinUpdate,
    PuzzleUpdate,
    CatUpdate,
    NftUpdate,
}
