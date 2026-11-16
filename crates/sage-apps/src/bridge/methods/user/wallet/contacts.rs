//! Address book methods.
//!
//! Apps can read the address book (`wallet.getContacts`, `wallet.lookupAddressContacts`) with
//! `wallet.get_contacts`. Contacts never leave the device, but they're personal, so reading
//! them has its own grant. Apps can't change the address book: a contact is trusted when it's
//! picked as a recipient, so only the user adds, renames or deletes them.

use async_trait::async_trait;
use sage::Error;
use sage_api::{
    AddressContactsRecord, ContactKind, ContactRecord, ErrorKind, GetContacts, GetContactsResponse,
    HandleRecord, LookupAddressContacts, LookupAddressContactsResponse,
};
use serde::Serialize;
use specta::Type;

use crate::{
    BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeTools, RustBridgeRequest, UserBridgeCapability,
    parse_required_params, without_history,
};

/// A contact as apps see it. Its row ID is left out: apps can't change contacts, so they've
/// no use for it, and it would hint at how many contacts were ever made. The recipient
/// identifies the contact instead.
#[derive(Debug, Clone, Serialize, Type)]
pub struct WalletContact {
    /// The contact's name. No two contacts on the network have names which look the same,
    /// ignoring case, whitespace and invisible characters like zero-width joiners
    pub name: String,
    /// A private note about the contact
    pub note: Option<String>,
    /// Whether the contact is a raw address or an XCHandles handle
    pub kind: ContactKind,
    /// What to enter as a recipient to send to this contact: the address, or the handle
    /// written as `@handle`. No two contacts have the same recipient
    pub recipient: String,
    /// The handle, without the leading `@`, for handle contacts
    pub handle: Option<String>,
    /// The address for address contacts. For handle contacts, the address the handle was
    /// last verified as resolving to, if it has ever been resolved. Check `resolution` before
    /// relying on it, since it may be outdated, unverified or changed
    pub address: Option<String>,
    /// The stored mapping for handle contacts which have been resolved, including its
    /// verification state and whether it changed, but not the addresses it was used with
    /// before. It is read from the local store, so it is always marked as cached
    pub resolution: Option<HandleRecord>,
    /// Unix timestamp the contact was created at
    pub created_at: u64,
    /// Unix timestamp the contact's name or note was last changed at
    pub updated_at: u64,
}

impl From<ContactRecord> for WalletContact {
    fn from(contact: ContactRecord) -> Self {
        let ContactRecord {
            id: _,
            name,
            note,
            kind,
            recipient,
            handle,
            address,
            resolution,
            created_at,
            updated_at,
        } = contact;

        Self {
            name,
            note,
            kind,
            recipient,
            handle,
            address,
            resolution: resolution.map(|mut record| {
                without_history(&mut record);
                record
            }),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct WalletGetContactsResult {
    /// The contacts, sorted by name
    pub contacts: Vec<WalletContact>,
    /// Total number of contacts matching the search
    pub total: u32,
}

impl From<GetContactsResponse> for WalletGetContactsResult {
    fn from(response: GetContactsResponse) -> Self {
        Self {
            contacts: response.contacts.into_iter().map(Into::into).collect(),
            total: response.total,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct WalletAddressContacts {
    /// The address or puzzle hash, as it was given
    pub address: String,
    /// Contacts for this address, sorted by name. These are address contacts for it, and
    /// handle contacts whose stored mapping is verified, unexpired and resolves to it. A
    /// handle contact whose handle changed hands since it was last used is listed for the
    /// new address, which may belong to someone else; it must be shown with a warning, never
    /// as the contact
    pub contacts: Vec<WalletContact>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct WalletLookupAddressContactsResult {
    /// One entry per requested address, in the same order
    pub results: Vec<WalletAddressContacts>,
}

impl From<LookupAddressContactsResponse> for WalletLookupAddressContactsResult {
    fn from(response: LookupAddressContactsResponse) -> Self {
        Self {
            results: response
                .results
                .into_iter()
                .map(
                    |AddressContactsRecord { address, contacts }| WalletAddressContacts {
                        address,
                        contacts: contacts.into_iter().map(Into::into).collect(),
                    },
                )
                .collect(),
        }
    }
}

/// Maps a failed request to a bridge error, telling a bad request apart from a failure.
fn contact_error(method: &str, error: &Error) -> BridgeMethodHandleError {
    let message = format!("{method} failed: {error}");

    match error.kind() {
        ErrorKind::Api => BridgeMethodHandleError::invalid_request(message),
        _ => BridgeMethodHandleError::internal_error(message),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalletGetContacts;

#[async_trait]
impl BridgeMethod for WalletGetContacts {
    fn name(&self) -> &'static str {
        "wallet.getContacts"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetContacts)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: GetContacts = parse_required_params(self, request)?;
        let sage = tools.app_state.lock().await;

        let result = sage
            .get_contacts(params)
            .await
            .map_err(|err| contact_error(self.name(), &err))?;

        Ok(Box::new(WalletGetContactsResult::from(result)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalletLookupAddressContacts;

#[async_trait]
impl BridgeMethod for WalletLookupAddressContacts {
    fn name(&self) -> &'static str {
        "wallet.lookupAddressContacts"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetContacts)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: LookupAddressContacts = parse_required_params(self, request)?;
        let sage = tools.app_state.lock().await;

        let result = sage
            .lookup_address_contacts(params)
            .await
            .map_err(|err| contact_error(self.name(), &err))?;

        Ok(Box::new(WalletLookupAddressContactsResult::from(result)))
    }
}

#[cfg(test)]
mod tests {
    use sage_api::{HandleStatus, HandleVerification};

    use super::*;

    #[test]
    fn contacts_are_shared_without_their_ids_or_handle_history() {
        let resolution = HandleRecord {
            handle: "alice".to_string(),
            handle_hash: "aa".to_string(),
            address: "txch1new".to_string(),
            status: HandleStatus::Changed,
            previous_address: Some("txch1old".to_string()),
            last_sent_address: Some("txch1paid".to_string()),
            changed_from: Some("txch1old".to_string()),
            cached: true,
            verification: HandleVerification::Verified,
            verification_reason: None,
            owner_launcher_id: "bb".to_string(),
            resolved_launcher_id: "cc".to_string(),
            expiration: 2_000_000_000,
            slot_coin_id: "dd".to_string(),
            slot_confirmation_height: 1,
            resolved_coin_id: "ee".to_string(),
            resolved_confirmation_height: 1,
            updated_height: 1,
            verified_height: 2,
            verified_timestamp: 3,
            last_verified_at: 4,
        };
        let contact = ContactRecord {
            id: 7,
            name: "Alice".to_string(),
            note: None,
            kind: ContactKind::Handle,
            recipient: "@alice".to_string(),
            handle: Some("alice".to_string()),
            address: Some("txch1new".to_string()),
            resolution: Some(resolution),
            created_at: 1,
            updated_at: 2,
        };

        // Whether the handle changed hands is still shown, but not the addresses it was used
        // with, which may come from another wallet.
        let assert_shared = |contact: &serde_json::Value| {
            assert!(contact.get("id").is_none());
            assert_eq!(contact["resolution"]["status"], "changed");
            assert_eq!(contact["resolution"]["address"], "txch1new");
            assert!(contact["resolution"]["previous_address"].is_null());
            assert!(contact["resolution"]["last_sent_address"].is_null());
            assert!(contact["resolution"]["changed_from"].is_null());
        };

        let result = WalletGetContactsResult::from(GetContactsResponse {
            contacts: vec![contact.clone()],
            total: 1,
        });
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["total"], 1);
        assert_eq!(json["contacts"][0]["recipient"], "@alice");
        assert_shared(&json["contacts"][0]);

        let result = WalletLookupAddressContactsResult::from(LookupAddressContactsResponse {
            results: vec![AddressContactsRecord {
                address: "txch1alice".to_string(),
                contacts: vec![contact],
            }],
        });
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["results"][0]["address"], "txch1alice");
        assert_eq!(json["results"][0]["contacts"][0]["name"], "Alice");
        assert_shared(&json["results"][0]["contacts"][0]);
    }

    #[test]
    fn bad_requests_are_told_apart() {
        assert_eq!(
            contact_error("wallet.getContacts", &Error::TooManyAddresses(1)).code,
            "invalid_request"
        );
        assert_eq!(
            contact_error("wallet.getContacts", &Error::NotLoggedIn).code,
            "internal_error"
        );
    }

    #[test]
    fn contact_methods_require_their_capability() {
        for method in [
            &WalletGetContacts as &dyn BridgeMethod,
            &WalletLookupAddressContacts,
        ] {
            assert_eq!(
                method.capability(),
                BridgeMethodCapability::user(UserBridgeCapability::WalletGetContacts)
            );
        }
    }
}
