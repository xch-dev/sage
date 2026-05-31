mod get_key;
mod get_secret_key;
mod read_methods;
mod send_xch;

pub(crate) use get_key::*;
pub(crate) use get_secret_key::*;
pub(crate) use read_methods::*;
pub(crate) use send_xch::*;

pub(crate) fn require_scoped_fingerprint(
    ctx: &crate::bridge::BridgeContext<'_>,
    fingerprint: Option<u32>,
) -> Result<u32, crate::bridge::BridgeMethodHandleError> {
    let fingerprint = fingerprint.ok_or_else(|| {
        crate::bridge::BridgeMethodHandleError::invalid_request(
            "wallet fingerprint is required for apps",
        )
    })?;

    if !ctx.app.is_wallet_in_scope(fingerprint) {
        return Err(crate::bridge::BridgeMethodHandleError::invalid_request(
            format!("wallet fingerprint not in app wallet scope: {fingerprint}"),
        ));
    }

    Ok(fingerprint)
}
