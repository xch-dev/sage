use serde::{Deserialize, Serialize};

/// A handle mapping which was verified against the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HandleRecord {
    /// The handle, without the leading `@`
    #[cfg_attr(feature = "openapi", schema(example = "alice"))]
    pub handle: String,
    /// The hash the registry stores the handle under
    pub handle_hash: String,
    /// The address the handle resolves to
    pub address: String,
    /// Whether the resolved address is new since it was last verified
    pub status: HandleStatus,
    /// The address the handle resolved to before it changed, while the change is
    /// unacknowledged
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub previous_address: Option<String>,
    /// The address this handle last resolved to when it was sent to
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub last_sent_address: Option<String>,
    /// The address a send to this handle has to confirm the new address against, if any.
    /// This is the previous address while a change is unacknowledged, or otherwise the
    /// address last sent to if the handle now resolves somewhere else.
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub changed_from: Option<String>,
    /// Whether the mapping was verified against the chain before this request, rather
    /// than by it. Resolutions are only cached when the handle could not be verified right
    /// now, while mappings listed from the store are always cached
    pub cached: bool,
    /// Whether the stored mapping can currently be relied on
    pub verification: HandleVerification,
    /// Why the mapping is no longer verified, if it isn't
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub verification_reason: Option<String>,
    /// Launcher ID of the singleton which owns the handle
    pub owner_launcher_id: String,
    /// Launcher ID of the singleton the handle resolves to
    pub resolved_launcher_id: String,
    /// Unix timestamp after which the handle expires
    pub expiration: u64,
    /// The handle's registry slot coin
    pub slot_coin_id: String,
    /// Height the slot coin was confirmed at
    pub slot_confirmation_height: u32,
    /// The resolved singleton coin the address was derived from
    pub resolved_coin_id: String,
    /// Height the resolved singleton coin was confirmed at
    pub resolved_confirmation_height: u32,
    /// The latest height at which the slot or the resolved singleton changed, so the
    /// latest height at which the address this handle resolves to could have changed. The
    /// slot is also recreated by activity on neighboring handles, so this can be later
    /// than the last time the handle itself was updated.
    pub updated_height: u32,
    /// Height of the peak transaction block the mapping was verified at
    pub verified_height: u32,
    /// Timestamp of the peak transaction block the mapping was verified at
    pub verified_timestamp: u64,
    /// Local Unix timestamp of the verification
    pub last_verified_at: u64,
}

/// Whether a verified handle mapping has changed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum HandleStatus {
    /// The handle resolves to the same address as when it was first verified, or the
    /// change has been acknowledged
    Verified,
    /// The handle now resolves to a different address than it did previously
    Changed,
}

/// Whether a stored handle mapping can currently be relied on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum HandleVerification {
    /// The values were verified against the chain by the most recent lookup, within the
    /// last day
    Verified,
    /// The values were verified by the most recent lookup, but more than a day ago, so the
    /// handle may have changed since. Resolve it again before relying on it
    Outdated,
    /// The most recent lookup was rejected, or showed the handle is no longer registered,
    /// so the values are only the last ones that were verified
    Unverified,
    /// The values must be verified again before they are used, for example after a reorg
    Stale,
    /// The handle has expired since it was last verified
    Expired,
}

/// The registry slot a handle was verified against
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HandleSlotRecord {
    /// Launcher ID of the registry singleton the handle is registered in
    pub registry_launcher_id: String,
    /// The slot's counter, which increases whenever the handle's slot is updated
    pub counter: u64,
    /// Hash of the neighboring handle before this one in the registry
    pub left_handle_hash: String,
    /// Hash of the neighboring handle after this one in the registry
    pub right_handle_hash: String,
    /// The registry coin which created the slot
    pub parent_id: String,
    /// The parent of the registry coin which created the slot
    pub parent_parent_id: String,
    /// Inner puzzle hash of the registry coin which created the slot
    pub parent_inner_puzzle_hash: String,
    /// The API's indexed peak height when the handle was last fetched
    pub indexed_height: u32,
}

/// Whether a handle has a pending transfer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum PendingTransferStatus {
    /// The API reports no pending transfer. The API is the only source of pending
    /// transfers, so this can't be proven on chain
    None,
    /// The API reports a pending transfer, and its update slot was found on chain
    Verified,
    /// The API reports a pending transfer, but no matching update slot from the registry
    /// was found on chain. It may already have been executed, or the claim may be false
    Unverified,
    /// Pending transfers couldn't be checked right now
    Unavailable,
}

/// A transfer of a handle to a new owner or resolved singleton, which has been initiated
/// but not executed yet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PendingTransferRecord {
    /// Launcher ID of the singleton which will own the handle
    pub new_owner_launcher_id: String,
    /// Launcher ID of the singleton the handle will resolve to
    pub new_resolved_launcher_id: String,
    /// Height from which the transfer can be executed
    pub minimum_execution_height: u32,
    /// The owner coin which initiated the transfer
    pub initiator_coin_id: String,
    /// The coin the API expects to execute the transfer. Not verified
    pub executor_coin_id: String,
    /// Height the API reports the transfer was initiated at. Not verified
    pub update_confirmation_height: u32,
    /// The registry's update slot coin which records the transfer, if it was verified
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub update_slot_coin_id: Option<String>,
}

/// Which known handles to list
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum HandleFilterMode {
    /// Every handle Sage knows about
    #[default]
    All,
    /// Handles whose owner NFT or DID is in this wallet
    Owned,
    /// Handles saved in the address book
    Contacts,
}

/// The known handles which resolve to an address
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressHandlesRecord {
    /// The address or puzzle hash, as it was given
    pub address: String,
    /// Unexpired handles which resolve to it, sorted by handle. Only mappings which are
    /// verified or outdated are included, and outdated ones should be resolved again before
    /// they're relied on
    pub handles: Vec<HandleRecord>,
}

/// Why an XCHandles request failed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum HandleErrorCode {
    /// The handle isn't syntactically valid
    InvalidHandle,
    /// XCHandles isn't deployed on the current network
    UnsupportedNetwork,
    /// The handle isn't registered
    NotFound,
    /// The handle has expired
    Expired,
    /// The API or peers were unavailable or behind, so the handle couldn't be verified
    /// right now. Retrying later may succeed
    Unavailable,
    /// The API's answer failed verification against the chain
    VerificationFailed,
}

/// Confirms that a handle recipient was reviewed while it resolved to an address.
///
/// A transaction to a handle is rejected if the handle no longer resolves to the confirmed
/// address. Confirming the address the handle resolves to now also acknowledges a change
/// from the address it resolved to before, or from the address last sent to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HandleConfirmation {
    /// The handle, with or without a leading `@`
    #[cfg_attr(feature = "openapi", schema(example = "@alice"))]
    pub handle: String,
    /// The address the handle was reviewed as resolving to
    pub address: String,
}

/// Why a recipient couldn't be used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum RecipientErrorKind {
    /// The recipient isn't a valid address or handle for this network
    Invalid,
    /// XCHandles isn't supported on this network
    UnsupportedNetwork,
    /// The handle isn't registered, or has expired
    NotRegistered,
    /// The handle's registration couldn't be verified against the chain
    Unverified,
    /// The handle couldn't be verified right now, and there is no recent verified mapping
    Unavailable,
    /// The handle resolves to a different address than the one last sent to or confirmed,
    /// and the new address must be confirmed before it can be used
    Changed,
}

/// A recipient which couldn't be used, so the transaction wasn't created
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RecipientError {
    /// The recipient as it was given
    pub recipient: String,
    pub kind: RecipientErrorKind,
    pub reason: String,
    /// The handle, without the leading `@`, if the recipient is a handle
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub handle: Option<String>,
    /// The verified address the handle resolves to now
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub address: Option<String>,
    /// The address the handle resolved to before it changed. Confirm `address` with a
    /// [`HandleConfirmation`] to use the new address.
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub previous_address: Option<String>,
}

/// An NFT offered in an offer whose metadata names an XCHandles handle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OfferHandleNft {
    /// The offered NFT
    pub nft_id: String,
    /// The handle named by the NFT's metadata, without the leading `@`. The owner can set
    /// this to anything, so it only says which handle to check.
    pub handle: String,
    pub status: OfferHandleStatus,
    /// Why the handle couldn't be checked, or what is pending
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub reason: Option<String>,
}

/// What the registry says about a handle NFT in an offer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum OfferHandleStatus {
    /// The registry confirms the handle belongs to the NFT, and no update is pending
    Verified,
    /// An update to the handle's owner or resolved singleton is pending, so the seller
    /// could take the handle back after the trade. The offer can't be taken.
    PendingTransfer,
    /// The registry doesn't associate the handle with this NFT, or the handle isn't
    /// registered, so the NFT isn't the handle it claims to be
    NotAssociated,
    /// The handle couldn't be checked right now. The offer can't be taken until it is.
    Unavailable,
}
