mod filter_unlocked_coins;
mod get_key;
mod get_public_keys;
mod get_secret_key;
mod read_methods;
mod send_xch;

pub(crate) use filter_unlocked_coins::*;
pub(crate) use get_key::*;
pub(crate) use get_public_keys::*;
pub(crate) use get_secret_key::*;
pub(crate) use read_methods::*;
pub(crate) use send_xch::*;

use crate::{BridgeContext, BridgeMethodHandleError};

pub(crate) fn require_scoped_fingerprint(
    ctx: &BridgeContext<'_>,
    fingerprint: Option<u32>,
) -> Result<u32, BridgeMethodHandleError> {
    let fingerprint = fingerprint.ok_or_else(|| {
        BridgeMethodHandleError::invalid_request("wallet fingerprint is required for apps")
    })?;

    if !ctx.app.is_wallet_in_scope(fingerprint) {
        return Err(BridgeMethodHandleError::invalid_request(format!(
            "wallet fingerprint not in app wallet scope: {fingerprint}"
        )));
    }

    Ok(fingerprint)
}
