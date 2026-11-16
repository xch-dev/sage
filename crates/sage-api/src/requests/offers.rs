use serde::{Deserialize, Serialize};

use crate::{
    Amount, HandleConfirmation, OfferHandleNft, OfferRecord, OfferRecordStatus, OfferSummary,
    SpendBundleJson, TransactionSummary,
};

use super::TransactionResponse;

/// Create a new offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Create an offer of assets from the active wallet in exchange for requested assets, optionally with an expiration. Its coins are locked while the offer is active."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MakeOffer {
    /// Assets requested in the offer
    pub requested_assets: Vec<OfferAmount>,
    /// Assets offered in exchange
    pub offered_assets: Vec<OfferAmount>,
    /// Transaction fee
    pub fee: Amount,
    /// Optional receive address
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub receive_address: Option<String>,
    /// Optional expiration timestamp
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub expires_at_second: Option<u64>,
    /// Whether to automatically import the offer
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub auto_import: bool,
    /// Optional specific coin IDs to use for the offer instead of auto-selecting
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub coin_ids: Option<Vec<String>>,
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
    /// If set, the XCH and CAT coins the offer would spend are first split into coins of the
    /// exact amounts it offers, in a transaction paying this fee, so the rest of the balance
    /// isn't locked up by the offer. The offer is always imported, and stays pending until
    /// that transaction is confirmed, so it shouldn't be shared before then. If the coins are
    /// already the exact amounts, nothing is split.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub pre_split_fee: Option<Amount>,
}

/// Asset amount in an offer
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OfferAmount {
    /// Optional asset ID (null for XCH)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
    /// Optional hidden puzzle hash for privacy
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub hidden_puzzle_hash: Option<String>,
    /// Amount of the asset
    pub amount: Amount,
}

/// Response with created offer
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MakeOfferResponse {
    /// Offer string (bech32 encoded)
    pub offer: String,
    /// Offer ID
    pub offer_id: String,
    /// The ID of the transaction splitting the offer's coins, if they had to be split. The
    /// offer is pending until it's confirmed.
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub pre_split_transaction_id: Option<String>,
}

/// Accept an offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Take an offer, signing the wallet's side and combining it with the offer. Offers of a handle NFT with a pending transfer, or one that can't be checked, are refused."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TakeOffer {
    /// Offer string to accept
    pub offer: String,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response with accepted offer details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TakeOfferResponse {
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Spend bundle
    pub spend_bundle: SpendBundleJson,
    /// Transaction ID
    pub transaction_id: String,
}

/// Combine multiple offers
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Combine several offers into one offer, which is taken all at once."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CombineOffers {
    /// Offer strings to combine
    pub offers: Vec<String>,
}

/// Response with combined offer
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CombineOffersResponse {
    /// Combined offer string
    pub offer: String,
}

/// View an offer without accepting
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Summarize an offer and its status without importing or taking it, and check any XCHandles handle NFTs it offers."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ViewOffer {
    /// Offer string to view
    pub offer: String,
}

/// Response with offer details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ViewOfferResponse {
    /// Offer summary
    pub offer: OfferSummary,
    /// Offer status
    pub status: OfferRecordStatus,
    /// Offered NFTs which claim to be XCHandles handles, checked against the registry.
    /// An offer with a pending handle transfer, or a handle which couldn't be checked,
    /// can't be taken.
    #[serde(default)]
    pub handle_nfts: Vec<OfferHandleNft>,
}

/// Import an offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Save an offer to the wallet's offer list, so its status is tracked. Returns its offer ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportOffer {
    /// Offer string to import
    pub offer: String,
}

/// Response with imported offer ID
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportOfferResponse {
    /// ID of the imported offer
    pub offer_id: String,
}

/// List all offers
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "List the offers saved in the wallet, made or imported, with filtering, searching, sorting and pagination."
    )
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffers {
    /// Starting offset for pagination
    #[serde(default)]
    pub offset: Option<u32>,
    /// Maximum number of offers to return (all when omitted)
    #[serde(default)]
    pub limit: Option<u32>,
    /// Only return offers with this status
    #[serde(default)]
    pub status: Option<OfferRecordStatus>,
    /// Search by asset name, ticker, asset/offer id, or NFT/DID/option address
    #[serde(default)]
    pub find_value: Option<String>,
    /// Which side of the offer the search applies to (defaults to any)
    #[serde(default)]
    pub find_side: Option<OfferFindSide>,
    /// Sort column (defaults to creation time)
    #[serde(default)]
    pub sort_mode: Option<OfferSortMode>,
    /// Sort ascending (defaults to false, newest first)
    #[serde(default)]
    pub ascending: bool,
}

/// Which side of an offer a search matches
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum OfferFindSide {
    #[default]
    Any,
    Offered,
    Requested,
}

/// Offer list sort column
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum OfferSortMode {
    #[default]
    Created,
    Expiration,
}

/// Response with list of offers
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffersResponse {
    /// List of offers
    pub offers: Vec<OfferRecord>,
    /// Total number of offers matching the filters, before paging
    pub total: u32,
}

/// Get offers for a specific asset
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "List the active offers saved in the wallet that include an asset, given a CAT asset ID or an NFT or option ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffersForAsset {
    /// Asset ID to filter by
    pub asset_id: String,
}

/// Response with offers for asset
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffersForAssetResponse {
    /// List of offers involving the asset
    pub offers: Vec<OfferRecord>,
}

/// Get a specific offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Get an offer saved in the wallet by its offer ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffer {
    /// Offer ID
    pub offer_id: String,
}

/// Response with offer details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOfferResponse {
    /// Offer details
    pub offer: OfferRecord,
}

/// Delete an offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Remove an offer from the wallet, which unlocks its coins. The offer isn't cancelled, so anyone with a copy can still take it."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteOffer {
    /// Offer ID to delete
    pub offer_id: String,
}

/// Response for offer deletion
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteOfferResponse {}

/// Cancel an offer on-chain
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Cancel an offer on chain by spending its coins back to the wallet, so it can no longer be taken.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CancelOffer {
    /// Offer ID to cancel
    pub offer_id: String,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
    /// Optional XCH coin IDs to pay the fee with. Each must be selectable: confirmed, unspent, and not pending or locked in an offer
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub coin_ids: Option<Vec<String>>,
}

pub type CancelOfferResponse = TransactionResponse;

/// Cancel multiple offers
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Cancel several offers on chain in one transaction. The fee is paid once per offer.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CancelOffers {
    /// Offer IDs to cancel
    pub offer_ids: Vec<String>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
    /// Optional XCH coin IDs to pay the fee with. Each must be selectable: confirmed, unspent, and not pending or locked in an offer
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub coin_ids: Option<Vec<String>>,
}

pub type CancelOffersResponse = TransactionResponse;

fn yes() -> bool {
    true
}
