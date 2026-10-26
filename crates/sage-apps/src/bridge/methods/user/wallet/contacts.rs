//! Address book methods.
//!
//! Apps can read the address book (`wallet.getContacts`, `wallet.lookupAddressContacts`) with
//! `wallet.get_contacts`. Contacts never leave the device, but they're personal, so reading
//! them has its own grant. Apps can't change the address book: a contact is trusted when it's
//! picked as a recipient, so only the user adds, renames or deletes them.

use async_trait::async_trait;
use sage::Error;
use sage_api::{ErrorKind, GetContacts, LookupAddressContacts};

use crate::{
    BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeTools, RustBridgeRequest, UserBridgeCapability,
    parse_required_params,
};

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

        Ok(Box::new(result))
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

        Ok(Box::new(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
