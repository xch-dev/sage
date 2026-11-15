mod contacts;
mod events;
mod filter_unlocked_coins;
mod get_asset_balance;
mod get_asset_coins;
mod get_public_keys;
mod get_secret_key;
mod get_wallet;
mod handles;
mod read_methods;
mod send_transaction;
mod send_xch;
mod sign_coin_spends;
mod sign_message;

pub(crate) use contacts::*;
pub(crate) use events::*;
pub(crate) use filter_unlocked_coins::*;
pub(crate) use get_asset_balance::*;
pub(crate) use get_asset_coins::*;
pub(crate) use get_public_keys::*;
pub(crate) use get_secret_key::*;
pub(crate) use get_wallet::*;
pub(crate) use handles::*;
pub(crate) use read_methods::*;
pub(crate) use send_transaction::*;
pub(crate) use send_xch::*;
pub(crate) use sign_coin_spends::*;
pub(crate) use sign_message::*;

use sage_api::GetWalletResponse;
use sage_rpc::RpcSession;

use crate::{BridgeContext, BridgeMethodHandleError};

pub(crate) fn current_scoped_wallet(
    ctx: &BridgeContext<'_>,
    sage: &RpcSession<'_>,
) -> Result<(GetWalletResponse, u32), BridgeMethodHandleError> {
    let (response, fingerprint) = sage
        .current_wallet()
        .map_err(|err| BridgeMethodHandleError::new("permission_denied", err.to_string()))?;
    require_scoped_fingerprint(ctx, Some(fingerprint))?;
    Ok((response, fingerprint))
}

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
