use serde::{Deserialize, Serialize};

use crate::{AddressContactsRecord, ContactEntry, ContactKind, ContactRecord, SkippedContact};

/// List contacts in the address book
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "List contacts in this wallet's address book for the current network, sorted by name. Handle contacts include the mapping Sage last verified for them, read from the local store. Nothing is fetched from the network, so resolve a handle before relying on its address."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetContacts {
    /// Only list contacts whose name, note or handle contains this text, ignoring ASCII
    /// case. An address or puzzle hash also matches the contacts which name it
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true, example = "ali"))]
    pub search: Option<String>,
    /// Only list contacts of this kind
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub kind: Option<ContactKind>,
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of contacts to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
}

/// Response with contacts
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetContactsResponse {
    /// The contacts, sorted by name
    pub contacts: Vec<ContactRecord>,
    /// Total number of contacts matching the search
    pub total: u32,
}

/// Get a contact from the address book
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Address Book", description = "Get a single contact by its ID.")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetContact {
    #[cfg_attr(feature = "openapi", schema(example = 1))]
    pub id: u32,
}

/// Response with a contact
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetContactResponse {
    pub contact: ContactRecord,
}

/// Add a contact to the address book
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "Add a named contact for an address on the current network, or for an XCHandles handle written as @handle. A handle isn't looked up when it's added: it's resolved and checked for changes whenever it's used as a recipient. Each recipient can only belong to one contact, and names must be unique ignoring ASCII case."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddContact {
    #[cfg_attr(feature = "openapi", schema(example = "Alice"))]
    pub name: String,
    /// An address on the current network, or a handle written as `@handle`
    #[cfg_attr(feature = "openapi", schema(example = "@alice"))]
    pub recipient: String,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub note: Option<String>,
}

/// Response with the new contact
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddContactResponse {
    pub contact: ContactRecord,
}

/// Rename a contact or change its note
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "Change a contact's name and note. The recipient is the contact's identity and can't be changed; add a new contact instead."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateContact {
    #[cfg_attr(feature = "openapi", schema(example = 1))]
    pub id: u32,
    #[cfg_attr(feature = "openapi", schema(example = "Alice"))]
    pub name: String,
    /// The new note. Leave this out to clear it
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub note: Option<String>,
}

/// Response with the updated contact
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateContactResponse {
    pub contact: ContactRecord,
}

/// Delete a contact from the address book
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "Delete a contact. For a handle contact, the handle's verified mapping, the address it changed from and the address last sent to are kept, so deleting and adding the contact again can't hide a change."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteContact {
    #[cfg_attr(feature = "openapi", schema(example = 1))]
    pub id: u32,
}

/// Response for contact deletion
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteContactResponse {}

/// Look up the contacts for addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "Find the contacts which name each of the given addresses or puzzle hashes: address contacts for it, and handle contacts whose stored mapping is verified, unexpired and resolves to it. A handle contact whose handle changed hands is listed for its new address with `resolution.changed_from` set, and must be shown with a warning. This only uses the local store and never makes network requests. Up to 1000 addresses can be looked up at once."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LookupAddressContacts {
    /// Addresses on the current network, or hex encoded puzzle hashes
    pub addresses: Vec<String>,
}

/// Response with the contacts for each address
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LookupAddressContactsResponse {
    /// One entry per requested address, in the same order
    pub results: Vec<AddressContactsRecord>,
}

/// Export the address book
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "Export every contact for the current network in a portable form, which import_contacts accepts. Only names, recipients and notes are exported. Verification state isn't, since it has to be established again by the wallet importing them."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExportContacts {}

/// Response with the exported address book
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExportContactsResponse {
    /// The network the contacts are for
    #[cfg_attr(feature = "openapi", schema(example = "mainnet"))]
    pub network_id: String,
    /// The contacts, sorted by name
    pub contacts: Vec<ContactEntry>,
}

/// Import contacts into the address book
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Address Book",
        description = "Add contacts exported by export_contacts. Existing contacts are never changed: an entry for a recipient which already has a contact, or with a name another contact already has, is skipped and reported, as is an invalid entry. Handles aren't looked up, so imported handle contacts are verified when they're first used."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportContacts {
    /// The network the contacts were exported from, which must be the current network
    #[cfg_attr(feature = "openapi", schema(example = "mainnet"))]
    pub network_id: String,
    pub contacts: Vec<ContactEntry>,
}

/// Response with the outcome of an import
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Address Book"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportContactsResponse {
    /// The contacts which were added
    pub added: Vec<ContactRecord>,
    /// The entries which weren't added, and why
    pub skipped: Vec<SkippedContact>,
}
