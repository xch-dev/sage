use serde::{Deserialize, Serialize};

use crate::{
    Amount, FeePolicy, OfferRecord, OfferRecordStatus, OfferSummary, SpendBundleJson,
    TransactionSummary,
};

use super::TransactionResponse;

/// Create a new offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Create a new offer for peer-to-peer trading of assets."
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
    /// Optional fee policy for requested CAT assets
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub fee_policy: Option<FeePolicy>,
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
}

/// Accept an offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Accept and complete an offer created by another party."
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
        description = "Combine multiple offers into a single compound offer."
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
        description = "View the details of an offer without accepting it."
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
}

/// Import an offer
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Import an offer file from an external source."
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
        description = "List all offers created by or available to this wallet."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffers {}

/// Response with list of offers
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Offers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOffersResponse {
    /// List of offers
    pub offers: Vec<OfferRecord>,
}

/// Get offers for a specific asset
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Get all offers that involve a specific asset."
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
        description = "Get detailed information about a specific offer by ID."
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
        description = "Delete an offer from the wallet (doesn't cancel on-chain)."
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
        description = "Cancel an offer by spending the offered coins on-chain.",
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
}

pub type CancelOfferResponse = TransactionResponse;

/// Cancel multiple offers
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Offers",
        description = "Cancel multiple offers in a single transaction.",
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
}

pub type CancelOffersResponse = TransactionResponse;

fn yes() -> bool {
    true
}
