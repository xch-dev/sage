mod get_key;
pub mod get_secret_key;
pub mod read_methods;
pub mod send_xch;

pub use get_key::WalletGetKey;
pub use get_secret_key::WalletGetSecretKey;
pub use read_methods::{
    WalletCheckAddress, WalletGetCoins, WalletGetCoinsByIds, WalletGetDerivations,
    WalletGetPendingTransactions, WalletGetSpendableCoinCount, WalletGetSyncStatus,
    WalletGetTransaction, WalletGetTransactions, WalletGetVersion,
};
pub use send_xch::WalletSendXch;

fn require_scoped_fingerprint(
    ctx: &crate::bridge::methods::BridgeContext<'_>,
    fingerprint: Option<u32>,
) -> Result<u32, crate::bridge::methods::shared::BridgeMethodHandleError> {
    let fingerprint = fingerprint.ok_or_else(|| {
        crate::bridge::methods::shared::BridgeMethodHandleError::invalid_request(
            "wallet fingerprint is required for apps",
        )
    })?;

    if !ctx.app.is_wallet_in_scope(fingerprint) {
        return Err(crate::bridge::methods::shared::BridgeMethodHandleError::invalid_request(
            format!("wallet fingerprint not in app wallet scope: {fingerprint}"),
        ));
    }

    Ok(fingerprint)
}
