//! XCHandles methods.
//!
//! Resolving a handle (`wallet.resolveHandle`, `wallet.getHandle`) sends it to the XCHandles
//! API, so the app decides what is sent to an outside server. Those methods need
//! `wallet.resolve_handle`, which is externally observable and so can't be granted together
//! with secret access. Reading handles Sage already knows (`wallet.getHandles`,
//! `wallet.lookupAddressHandles`) never leaves the device, and needs `wallet.get_handles`.

use async_trait::async_trait;
use sage::Error;
use sage_api::{
    GetHandle, GetHandles, HandleErrorCode, HandleFilterMode, LookupAddressHandles, ResolveHandle,
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

        let result = sage
            .resolve_handle(params)
            .await
            .map_err(|err| handle_error(self.name(), &err))?;

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

        let result = sage
            .get_handle(params)
            .await
            .map_err(|err| handle_error(self.name(), &err))?;

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
    fn get_handles_cannot_refresh() {
        let mut params = GetHandles {
            filter: HandleFilterMode::Owned,
            include_unverified: false,
            refresh: false,
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
