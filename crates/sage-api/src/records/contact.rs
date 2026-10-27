use serde::{Deserialize, Serialize};

use crate::HandleRecord;

/// A named entry in the address book
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ContactRecord {
    /// Stable identifier of the contact
    #[cfg_attr(feature = "openapi", schema(example = 1))]
    pub id: u32,
    /// The contact's name. No two contacts in a wallet have names which look the same,
    /// ignoring case, whitespace and invisible characters like zero-width joiners
    #[cfg_attr(feature = "openapi", schema(example = "Alice"))]
    pub name: String,
    /// A private note about the contact
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub note: Option<String>,
    /// Whether the contact is a raw address or an XCHandles handle
    pub kind: ContactKind,
    /// What to enter as a recipient to send to this contact: the address, or the handle
    /// written as `@handle`. Handles are resolved and checked for changes when they're used
    #[cfg_attr(feature = "openapi", schema(example = "@alice"))]
    pub recipient: String,
    /// The handle, without the leading `@`, for handle contacts
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub handle: Option<String>,
    /// The address for address contacts. For handle contacts, the address the handle was
    /// last verified as resolving to, if it has ever been resolved. Check `resolution` before
    /// relying on it, since it may be outdated, unverified or changed
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub address: Option<String>,
    /// The stored mapping for handle contacts which have been resolved, including its
    /// verification state, the address it changed from and the address last sent to. It is
    /// read from the local store, so it is always marked as cached
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub resolution: Option<HandleRecord>,
    /// Unix timestamp the contact was created at
    pub created_at: u64,
    /// Unix timestamp the contact's name or note was last changed at
    pub updated_at: u64,
}

/// What a contact names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ContactKind {
    /// A raw address
    Address,
    /// An XCHandles handle, which is resolved to an address when it's used
    Handle,
}

/// The contacts which name an address
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddressContactsRecord {
    /// The address or puzzle hash, as it was given
    pub address: String,
    /// Contacts for this address, sorted by name. These are address contacts for it, and
    /// handle contacts whose stored mapping is verified, unexpired and resolves to it. Handle
    /// contacts whose mapping is outdated should be resolved again before they're relied on.
    /// A handle contact whose handle changed hands since it was last used, so that
    /// `resolution.changed_from` is set, is listed for the new address, which may belong to
    /// someone else; it must be shown with a warning, never as the contact
    pub contacts: Vec<ContactRecord>,
}

/// A contact in a portable form, for export and import
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ContactEntry {
    #[cfg_attr(feature = "openapi", schema(example = "Alice"))]
    pub name: String,
    /// An address on the current network, or a handle written as `@handle`
    #[cfg_attr(feature = "openapi", schema(example = "@alice"))]
    pub recipient: String,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub note: Option<String>,
}

/// An imported contact which wasn't added
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SkippedContact {
    /// The entry as it was given
    pub entry: ContactEntry,
    pub reason: ContactSkipReason,
    /// The existing contact it conflicts with, if any
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub existing_id: Option<u32>,
}

/// Why an imported contact wasn't added
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ContactSkipReason {
    /// The name or recipient isn't valid on this network
    Invalid,
    /// A contact for the same recipient already exists. It's kept unchanged
    DuplicateRecipient,
    /// A contact for a different recipient already has this name
    DuplicateName,
}
