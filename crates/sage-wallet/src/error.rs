use std::{num::TryFromIntError, time::SystemTimeError};

use chia_wallet_sdk::{
    client::ClientError,
    clvm_traits::{FromClvmError, ToClvmError},
    clvmr::error::EvalErr,
    prelude::*,
    signer::SignerError,
    utils::CoinSelectionError,
};
use sage_assets::UriError;
use sage_database::{CoinKind, DatabaseError};
use thiserror::Error;
use tokio::{task::JoinError, time::error::Elapsed};

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("Signer error: {0}")]
    Signer(#[from] SignerError),

    #[error("Coin selection error: {0}")]
    CoinSelection(#[from] CoinSelectionError),

    /// The wallet's spendable coins of an asset can't cover the amount a transaction needs.
    #[error("{}", insufficient_coins_message(*asset, *needed, *available, *error))]
    InsufficientCoins {
        asset: CoinSelectionAsset,
        needed: u64,
        available: u64,
        error: CoinSelectionError,
    },

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("URI error: {0}")]
    Uri(#[from] UriError),

    #[error("Timeout exceeded")]
    Elapsed(#[from] Elapsed),

    #[error("Coinset request failed: {0}")]
    Coinset(String),

    #[error("Missing spend with id {0}")]
    MissingSpend(Bytes32),

    #[error("Missing child of id {0}")]
    MissingChild(Bytes32),

    #[error("Peer misbehaved")]
    PeerMisbehaved,

    #[error("No peers are connected")]
    NoPeers,

    #[error("Subscription limit reached")]
    SubscriptionLimitReached,

    #[error("System time error: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Join error: {0}")]
    Join(#[from] JoinError),

    #[error("CLVM encoding error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("CLVM decoding error: {0}")]
    FromClvm(#[from] FromClvmError),

    #[error("CLVM error: {0}")]
    Clvm(#[from] EvalErr),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("The transaction's amounts add up to more than the maximum amount")]
    AmountOverflow,

    #[error("Insufficient derivations")]
    InsufficientDerivations,

    #[error("This kind of wallet doesn't derive addresses")]
    DerivationsNotSupported,

    #[error("Watch-only wallets can't create or sign transactions")]
    WatchOnly,

    #[error("Spending from a vault isn't supported yet")]
    VaultSpendNotSupported,

    #[error("Missing secret key")]
    UnknownPublicKey,

    #[error(
        "Refusing to sign an AGG_SIG_UNSAFE condition with one of this wallet's keys, since the signature could be reused elsewhere"
    )]
    AggSigUnsafe,

    #[error("Missing XCH coin with id {0}")]
    MissingXchCoin(Bytes32),

    #[error("Missing CAT coin with id {0}")]
    MissingCatCoin(Bytes32),

    #[error("Missing DID coin with id {0}")]
    MissingDidCoin(Bytes32),

    #[error("Missing NFT coin with id {0}")]
    MissingNftCoin(Bytes32),

    #[error("Missing coin with id {0}")]
    MissingCoin(Bytes32),

    #[error("Missing option contract coin with id {0}")]
    MissingOptionCoin(Bytes32),

    #[error("Missing DID with id {0}. It may have been spent recently. Please try again later.")]
    MissingDid(Bytes32),

    #[error("Missing NFT with id {0}. It may have been spent recently. Please try again later.")]
    MissingNft(Bytes32),

    #[error(
        "Missing option contract with id {0}. It may have been spent recently. Please try again later."
    )]
    MissingOption(Bytes32),

    #[error("Missing asset with id {0}")]
    MissingAsset(Bytes32),

    #[error("Uncancellable offer")]
    UncancellableOffer,

    #[error("Cannot import offer that doesn't belong to this wallet")]
    CannotImportOffer,

    #[error("The transaction splitting the offer's coins didn't create the expected coins")]
    MissingPreSplitCoin,

    #[error(
        "NFT with id {0} doesn't use the default metadata updater, so URIs can't be appended to it"
    )]
    NftMetadataNotAppendOnly(Bytes32),

    #[error("Option contract with id {0} wasn't minted by this wallet")]
    NotOptionCreator(Bytes32),

    #[error("Cancelling an option requires a coin of its strike asset, of any amount")]
    MissingStrikeCoin,

    #[error("Cancelling an option requires an XCH coin, of any amount")]
    MissingMeltCoin,

    #[error("NFT option contracts are not supported")]
    NftOptionNotSupported,

    #[error("Unsupported underlying coin kind: {0:?}")]
    UnsupportedUnderlyingCoinKind(CoinKind),

    #[error("Unsupported clawback coin kind: {0:?}")]
    UnsupportedClawbackCoinKind(CoinKind),

    #[error("Cannot find clawback info for coin with id {0}")]
    MissingClawbackInfo(Bytes32),

    #[error("Clawback V1 coin {0} is not yet claimable (relative timelock)")]
    ClawbackNotYetClaimable(Bytes32),

    #[error(
        "Clawback version mismatch for coin {coin_id}: expected version {expected}, found {actual}"
    )]
    ClawbackVersionMismatch {
        coin_id: Bytes32,
        expected: u8,
        actual: u8,
    },

    #[error("Cannot fulfill clawback spend (wrong role, version path, or time window)")]
    ClawbackSpendNotAllowed,

    #[error("Clawback V1 receiver spend requires claim_clawback")]
    ClawbackClaimRequired,

    #[error("Clawback coin {0} is already spent or pending in the mempool")]
    ClawbackAlreadySpentOrPending(Bytes32),

    #[error("Clawback coin {0} has not been confirmed yet")]
    ClawbackUnconfirmed(Bytes32),

    #[error("Try from int error: {0}")]
    TryFromInt(#[from] TryFromIntError),
}

/// The asset that coin selection ran short of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinSelectionAsset {
    Xch,
    Cat(Bytes32),
}

fn insufficient_coins_message(
    asset: CoinSelectionAsset,
    needed: u64,
    available: u64,
    error: CoinSelectionError,
) -> String {
    let (name, decimals) = match asset {
        CoinSelectionAsset::Xch => ("XCH".to_string(), 12),
        CoinSelectionAsset::Cat(asset_id) => (format!("CAT {asset_id}"), 3),
    };
    let amount = |mojos| format_amount(mojos, decimals);

    let message = match error {
        CoinSelectionError::ExceededMaxCoins => {
            return format!(
                "Paying {} {name} would take more than 500 coins. Combine some of your {name} coins first, then try again.",
                amount(needed)
            );
        }
        CoinSelectionError::NoSpendableCoins | CoinSelectionError::InsufficientBalance(_)
            if needed == 0 =>
        {
            let article = if asset == CoinSelectionAsset::Xch {
                "an"
            } else {
                "a"
            };
            format!(
                "This needs {article} {name} coin to spend, but there are no spendable {name} coins."
            )
        }
        CoinSelectionError::NoSpendableCoins | CoinSelectionError::InsufficientBalance(_) => {
            format!(
                "Not enough spendable {name}: this needs {} {name}, but only {} {name} is spendable.",
                amount(needed),
                amount(available)
            )
        }
    };

    if asset == CoinSelectionAsset::Xch {
        format!(
            "{message} The network fee is always paid in XCH, even when nothing else in the transaction is."
        )
    } else {
        message
    }
}

/// Formats an amount of mojos in whole units, without trailing zeros.
fn format_amount(mojos: u64, decimals: u32) -> String {
    let scale = 10u64.pow(decimals);
    let whole = mojos / scale;
    let fraction = mojos % scale;

    if fraction == 0 {
        return whole.to_string();
    }

    let fraction = format!("{fraction:0width$}", width = decimals as usize);
    format!("{whole}.{}", fraction.trim_end_matches('0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(asset: CoinSelectionAsset, needed: u64, available: u64) -> String {
        WalletError::InsufficientCoins {
            asset,
            needed,
            available,
            error: CoinSelectionError::InsufficientBalance(available),
        }
        .to_string()
    }

    #[test]
    fn test_xch_shortfall_mentions_the_network_fee() {
        assert_eq!(
            message(CoinSelectionAsset::Xch, 10_000_000, 0),
            "Not enough spendable XCH: this needs 0.00001 XCH, but only 0 XCH is spendable. \
             The network fee is always paid in XCH, even when nothing else in the transaction is."
        );
    }

    #[test]
    fn test_cat_shortfall_names_the_asset() {
        let asset_id = Bytes32::new([1; 32]);

        assert_eq!(
            message(CoinSelectionAsset::Cat(asset_id), 110_000, 1_500),
            format!(
                "Not enough spendable CAT {asset_id}: this needs 110 CAT {asset_id}, but only 1.5 \
                 CAT {asset_id} is spendable."
            )
        );
    }

    #[test]
    fn test_any_coin_with_none_spendable() {
        let error = WalletError::InsufficientCoins {
            asset: CoinSelectionAsset::Xch,
            needed: 0,
            available: 0,
            error: CoinSelectionError::NoSpendableCoins,
        };

        assert!(
            error.to_string().starts_with(
                "This needs an XCH coin to spend, but there are no spendable XCH coins."
            )
        );
    }

    #[test]
    fn test_too_many_coins() {
        let error = WalletError::InsufficientCoins {
            asset: CoinSelectionAsset::Xch,
            needed: 1_500_000_000_000,
            available: 2_000_000_000_000,
            error: CoinSelectionError::ExceededMaxCoins,
        };

        assert_eq!(
            error.to_string(),
            "Paying 1.5 XCH would take more than 500 coins. Combine some of your XCH coins first, \
             then try again."
        );
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(0, 12), "0");
        assert_eq!(format_amount(1, 12), "0.000000000001");
        assert_eq!(format_amount(1_000_000_000_000, 12), "1");
        assert_eq!(format_amount(1_234_000, 3), "1234");
        assert_eq!(format_amount(1_230, 3), "1.23");
    }
}
