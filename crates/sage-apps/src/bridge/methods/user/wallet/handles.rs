//! XCHandles methods.
//!
//! Resolving a handle (`wallet.resolveHandle`, `wallet.getHandle`) sends it to the XCHandles
//! API, so the app decides what is sent to an outside server. Those methods need
//! `wallet.resolve_handle`, which is externally observable and so can't be granted together
//! with secret access. Reading handles Sage already knows (`wallet.getHandles`,
//! `wallet.lookupAddressHandles`) never leaves the device, and needs `wallet.get_handles`.
//!
//! Handle history (the address a handle last resolved to, the address last sent to, and any
//! change from it) is shared by every wallet on the network, so it's only returned by the
//! methods which need `wallet.get_handles`. Resolving a handle returns what the lookup found,
//! and whether it changed hands, but not the addresses it was used with before.

use async_trait::async_trait;
use sage::Error;
use sage_api::{
    GetHandle, GetHandles, HandleErrorCode, HandleFilterMode, HandleRecord, LookupAddressHandles,
    ResolveHandle,
};

use crate::{
    BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeTools, RustBridgeRequest, UserBridgeCapability,
    parse_required_params,
};

/// Maps a failed request to a bridge error, giving each kind of `XCHandles` failure its own
/// code so that apps can tell them apart.
fn handle_error(method: &str, error: &Error) -> BridgeMethodHandleError {
    let code = match error.handle_error_code() {
        Some(HandleErrorCode::InvalidHandle) => "invalid_handle",
        Some(HandleErrorCode::UnsupportedNetwork) => "unsupported_network",
        Some(HandleErrorCode::NotFound) => "handle_not_found",
        Some(HandleErrorCode::Expired) => "handle_expired",
        Some(HandleErrorCode::Unavailable) => "handle_unavailable",
        Some(HandleErrorCode::VerificationFailed) => "handle_verification_failed",
        None => "internal_error",
    };

    BridgeMethodHandleError::new(code, format!("{method} failed: {error}"))
}

/// Removes how a handle was used before from a record returned by a lookup or with a
/// contact, since that's shared by every wallet on the network, and needs
/// `wallet.get_handles` to read.
pub(crate) fn without_history(record: &mut HandleRecord) {
    record.previous_address = None;
    record.last_sent_address = None;
    record.changed_from = None;
}

#[derive(Debug, Clone, Copy)]
pub struct WalletResolveHandle;

#[async_trait]
impl BridgeMethod for WalletResolveHandle {
    fn name(&self) -> &'static str {
        "wallet.resolveHandle"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletResolveHandle)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: ResolveHandle = parse_required_params(self, request)?;
        let sage = tools.app_state.lock().await;

        let mut result = sage
            .resolve_handle(params)
            .await
            .map_err(|err| handle_error(self.name(), &err))?;
        without_history(&mut result.handle);

        Ok(Box::new(result))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalletGetHandle;

#[async_trait]
impl BridgeMethod for WalletGetHandle {
    fn name(&self) -> &'static str {
        "wallet.getHandle"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletResolveHandle)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: GetHandle = parse_required_params(self, request)?;
        let sage = tools.app_state.lock().await;

        let mut result = sage
            .get_handle(params)
            .await
            .map_err(|err| handle_error(self.name(), &err))?;
        without_history(&mut result.handle);

        Ok(Box::new(result))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalletGetHandles;

/// Refreshing looks up handles over the network, which `wallet.get_handles` doesn't allow.
/// Listing the handles saved as contacts would reveal the address book, which needs
/// `wallet.get_contacts`.
fn validate_get_handles(params: &GetHandles) -> Result<(), BridgeMethodHandleError> {
    if params.refresh {
        return Err(BridgeMethodHandleError::invalid_request(
            "wallet.getHandles can't refresh handles; use wallet.resolveHandle instead",
        ));
    }

    if params.filter == HandleFilterMode::Contacts {
        return Err(BridgeMethodHandleError::invalid_request(
            "wallet.getHandles can't list contacts; use wallet.getContacts instead",
        ));
    }

    Ok(())
}

#[async_trait]
impl BridgeMethod for WalletGetHandles {
    fn name(&self) -> &'static str {
        "wallet.getHandles"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetHandles)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: GetHandles = parse_required_params(self, request)?;
        validate_get_handles(&params)?;

        let sage = tools.app_state.lock().await;

        let result = sage
            .get_handles(params)
            .await
            .map_err(|err| handle_error(self.name(), &err))?;

        Ok(Box::new(result))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WalletLookupAddressHandles;

#[async_trait]
impl BridgeMethod for WalletLookupAddressHandles {
    fn name(&self) -> &'static str {
        "wallet.lookupAddressHandles"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetHandles)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: LookupAddressHandles = parse_required_params(self, request)?;
        let sage = tools.app_state.lock().await;

        let result = sage
            .lookup_address_handles(params)
            .await
            .map_err(|err| handle_error(self.name(), &err))?;

        Ok(Box::new(result))
    }
}

#[cfg(test)]
mod tests {
    use sage_api::{HandleStatus, HandleVerification};
    use sage_wallet::{HandleError, XchandlesApiError, XchandlesError};

    use super::*;

    #[test]
    fn handle_methods_require_their_capabilities() {
        for method in [&WalletResolveHandle as &dyn BridgeMethod, &WalletGetHandle] {
            assert_eq!(
                method.capability(),
                BridgeMethodCapability::user(UserBridgeCapability::WalletResolveHandle)
            );
        }

        for method in [
            &WalletGetHandles as &dyn BridgeMethod,
            &WalletLookupAddressHandles,
        ] {
            assert_eq!(
                method.capability(),
                BridgeMethodCapability::user(UserBridgeCapability::WalletGetHandles)
            );
        }
    }

    #[test]
    fn lookups_leave_out_handle_history() {
        let mut record = HandleRecord {
            handle: "alice".to_string(),
            handle_hash: "aa".to_string(),
            address: "txch1new".to_string(),
            status: HandleStatus::Changed,
            previous_address: Some("txch1old".to_string()),
            last_sent_address: Some("txch1paid".to_string()),
            changed_from: Some("txch1old".to_string()),
            cached: false,
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

        without_history(&mut record);

        // Whether the handle changed hands is still shown, but not the addresses it was
        // used with, which may come from another wallet.
        assert_eq!(record.status, HandleStatus::Changed);
        assert_eq!(record.address, "txch1new");
        assert_eq!(record.previous_address, None);
        assert_eq!(record.last_sent_address, None);
        assert_eq!(record.changed_from, None);
    }

    #[test]
    fn get_handles_cannot_refresh() {
        let mut params = GetHandles {
            filter: HandleFilterMode::Owned,
            include_unverified: false,
            refresh: false,
            search: None,
            offset: 0,
            limit: 10,
        };
        assert!(validate_get_handles(&params).is_ok());

        params.refresh = true;
        assert_eq!(
            validate_get_handles(&params).unwrap_err().code,
            "invalid_request"
        );

        params.refresh = false;
        params.filter = HandleFilterMode::Contacts;
        assert_eq!(
            validate_get_handles(&params).unwrap_err().code,
            "invalid_request"
        );
    }

    #[test]
    fn handle_errors_have_distinct_codes() {
        let code = |error: XchandlesError| handle_error("wallet.resolveHandle", &error.into()).code;

        assert_eq!(code(HandleError::InvalidCharacter.into()), "invalid_handle");
        assert_eq!(
            code(XchandlesError::UnsupportedNetwork),
            "unsupported_network"
        );
        assert_eq!(
            code(XchandlesApiError::HandleNotFound.into()),
            "handle_not_found"
        );
        assert_eq!(
            code(XchandlesApiError::HandleExpired.into()),
            "handle_expired"
        );
        assert_eq!(
            code(XchandlesApiError::IndexStale.into()),
            "handle_unavailable"
        );
        assert_eq!(
            code(XchandlesError::InvalidLineage),
            "handle_verification_failed"
        );
        assert_eq!(
            handle_error("wallet.getHandles", &Error::NotLoggedIn).code,
            "internal_error"
        );
    }
}
