use serde::{Deserialize, Serialize};

use crate::TokenRecord;

/// Resynchronize a `CAT` token's metadata from an external source
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Tokens",
        description = "Fetch a CAT's name, ticker, description and icon from Dexie again, replacing what the wallet has stored."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResyncCat {
    /// The asset ID of the `CAT` token to resynchronize
    #[cfg_attr(
        feature = "openapi",
        schema(example = "a628c1c2c6fcb74d53746157e438e108eab5c0bb3e5c80ff9b1910b3e4832913")
    )]
    pub asset_id: String,
}

/// Response after resynchronizing a `CAT` token
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Tokens"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResyncCatResponse {}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "System",
        description = "Get the current price of XCH in USD, from CoinGecko."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetXchUsdPrice {}

#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetXchUsdPriceResponse {
    pub usd: f64,
}

/// Update a `CAT` token's metadata and visibility
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Tokens",
        description = "Change how a CAT is shown in the wallet: its name, ticker, description, icon and visibility. Nothing on chain changes."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateCat {
    /// The token record containing updated metadata
    pub record: TokenRecord,
}

/// Response after updating a `CAT` token
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Tokens"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateCatResponse {}

/// Update an option's visibility settings
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "Show or hide an option in the wallet. Nothing on chain changes."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateOption {
    /// The option ID to update
    #[cfg_attr(feature = "openapi", schema(example = "0x..."))]
    pub option_id: String,
    /// Whether the option should be visible in the UI
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub visible: bool,
}

/// Response after updating an option
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Options"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateOptionResponse {}

/// Update a `DID`'s name and visibility settings
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "DIDs",
        description = "Change a DID's name and visibility in the wallet. Nothing on chain changes."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateDid {
    /// The `DID` ID to update
    #[cfg_attr(feature = "openapi", schema(example = "did:chia:..."))]
    pub did_id: String,
    /// Optional new name for the `DID`
    #[cfg_attr(feature = "openapi", schema(example = "My Identity"))]
    pub name: Option<String>,
    /// Whether the `DID` should be visible in the UI
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub visible: bool,
}

/// Response after updating a `DID`
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "DIDs"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateDidResponse {}

/// Update an `NFT`'s visibility settings
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Show or hide an NFT in the wallet. Nothing on chain changes."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateNft {
    /// The `NFT` ID to update
    #[cfg_attr(feature = "openapi", schema(example = "nft1..."))]
    pub nft_id: String,
    /// Whether the `NFT` should be visible in the UI
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub visible: bool,
}

/// Response after updating an `NFT`
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateNftResponse {}

/// Update an `NFT` collection's visibility settings
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Show or hide an NFT collection in the wallet. Nothing on chain changes."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateNftCollection {
    /// The collection ID to update
    #[cfg_attr(feature = "openapi", schema(example = "col1..."))]
    pub collection_id: String,
    /// Whether the collection should be visible in the UI
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub visible: bool,
}

/// Response after updating an `NFT` collection
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateNftCollectionResponse {}

/// Re-download an `NFT`'s data and metadata from its URIs
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Delete an NFT's downloaded data, metadata and license, and queue them to be downloaded again from its URIs."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RedownloadNft {
    /// The `NFT` ID to re-download
    #[cfg_attr(feature = "openapi", schema(example = "nft1..."))]
    pub nft_id: String,
}

/// Response after re-downloading an `NFT`
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RedownloadNftResponse {}

/// Increase the derivation index to generate more addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Addresses",
        description = "Derive the active wallet's addresses up to the given index, and start syncing them. Hardened addresses need the wallet's secret key. Not supported by watch wallets."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IncreaseDerivationIndex {
    /// Whether to derive hardened addresses (defaults to true if not specified)
    #[serde(default)]
    pub hardened: Option<bool>,
    /// Whether to derive unhardened addresses (defaults to true if not specified)
    #[serde(default)]
    pub unhardened: Option<bool>,
    /// The target derivation index to increase to
    #[cfg_attr(feature = "openapi", schema(example = 100))]
    pub index: u32,
}

/// Response after increasing the derivation index
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Addresses"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IncreaseDerivationIndexResponse {}
