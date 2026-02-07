use serde::{Deserialize, Serialize};

use crate::{Amount, NftUriKind, TransactionResponse};

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
    pub selected_coin_ids: Vec<String>,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Send(SendAction),
    MintNft(MintNftAction),
    UpdateNft(UpdateNftAction),
    Fee(FeeAction),
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
pub struct MintNftAction {
    /// The parent asset id of the minted NFT
    pub parent_id: Id,
    /// Edition number
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub edition_number: Option<u32>,
    /// Total editions
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub edition_total: Option<u32>,
    /// Data hash
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub data_hash: Option<String>,
    /// Data URIs
    #[serde(default)]
    pub data_uris: Vec<String>,
    /// Metadata hash
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub metadata_hash: Option<String>,
    /// Metadata URIs
    #[serde(default)]
    pub metadata_uris: Vec<String>,
    /// License hash
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub license_hash: Option<String>,
    /// License URIs
    #[serde(default)]
    pub license_uris: Vec<String>,
    /// Royalty payment address
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub royalty_address: Option<String>,
    /// Royalty percentage in ten-thousandths (e.g., 300 = 3%)
    #[serde(default)]
    pub royalty_ten_thousandths: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateNftAction {
    /// The id of the NFT to update
    pub id: Id,
    /// A list of URLs to add
    #[serde(default)]
    pub new_uris: Vec<NewNftUri>,
    /// An optional transfer to perform on the NFT
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub transfer: Option<NftTransfer>,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Id {
    /// The XCH asset
    Xch,
    /// An existing asset by its asset id or launcher id
    Existing { asset_id: String },
    /// A new asset by its index in the action list
    New { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NewNftUri {
    /// The type of URI
    pub kind: NftUriKind,
    /// The URI to add
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftTransfer {
    /// The DID to assign as the owner
    pub did_id: Option<Id>,
}

pub type CreateTransactionResponse = TransactionResponse;
