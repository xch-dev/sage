use serde::{Deserialize, Serialize};

use crate::Amount;

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "Create a new transaction using the action system.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateTransaction {
    /// Pre-selected coins to use in the transaction prior to coin selection
    #[serde(default)]
    pub coin_ids: Vec<String>,
    /// The list of actions to perform in the transaction
    pub actions: Vec<Action>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Action {
    Send(SendAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendAction {
    /// The id of the asset to send
    pub id: Id,
    /// The address to send to, in bech32 format
    pub address: String,
    /// The amount to send, in mojos
    pub amount: Amount,
    /// Optional clawback timestamp (seconds since epoch)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clawback: Option<u64>,
    /// A list of memos (encoded as hex) to include in the transaction
    #[serde(default)]
    pub memos: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FeeAction {
    /// The fee amount, in mojos
    pub amount: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum Id {
    /// The XCH asset
    Xch,
    /// An existing asset by its asset id or launcher id
    Existing(String),
    /// A new asset by its index in the action list
    New(usize),
}
