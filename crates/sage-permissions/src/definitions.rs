use crate::{WalletCapability, WalletCapabilityDefinition};

pub fn wallet_capability_definition(capability: WalletCapability) -> WalletCapabilityDefinition {
    match capability {
        WalletCapability::WalletGetWallet => WalletCapabilityDefinition {
            label: "Read wallet",
            description: "Allows the caller to read public information about the selected wallet.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetSecretKey => WalletCapabilityDefinition {
            label: "Read wallet secret key",
            description: "Allows the caller to read wallet secrets, including the mnemonic or private key when available.",
            externally_observable: false,
            accesses_sensitive_secret: true,
        },
        WalletCapability::WalletSendXch => WalletCapabilityDefinition {
            label: "Send XCH",
            description: "Allows the caller to request XCH transactions from your wallet.",
            externally_observable: true,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletSendXchAutoSubmit => WalletCapabilityDefinition {
            label: "Automatic XCH send",
            description: "Allows the caller to submit XCH transactions without asking for per-transaction approval.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetSyncStatus => WalletCapabilityDefinition {
            label: "Read sync status",
            description: "Allows the caller to read wallet sync status and current wallet balance summary.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetVersion => WalletCapabilityDefinition {
            label: "Read wallet version",
            description: "Allows the caller to read the current Sage wallet version.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetXchUsdPrice => WalletCapabilityDefinition {
            label: "Read XCH/USD price",
            description: "Allows the caller to read the current estimated XCH price in USD.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletCheckAddress => WalletCapabilityDefinition {
            label: "Check address",
            description: "Allows the caller to validate whether an address belongs to this wallet.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletResolveHandle => WalletCapabilityDefinition {
            label: "Resolve XCHandles",
            description: "Allows the caller to look up XCHandles handles, which sends them to the XCHandles API, and verify them against the blockchain.",
            externally_observable: true,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetHandles => WalletCapabilityDefinition {
            label: "Read known XCHandles",
            description: "Allows the caller to read the XCHandles handles Sage has already verified on this network, in any wallet, including which ones this wallet owns and which handles point to an address.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetContacts => WalletCapabilityDefinition {
            label: "Read address book",
            description: "Allows the caller to read the contacts in your address book, which every wallet on this network shares, including their names, notes, addresses and handles.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletFilterUnlockedCoins => WalletCapabilityDefinition {
            label: "Filter unlocked wallet coins",
            description: "Allows the caller to check which supplied coin IDs are currently spendable.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetAssetCoins => WalletCapabilityDefinition {
            label: "Read wallet asset coins",
            description: "Allows the caller to list spendable XCH, CAT, DID, or NFT coins.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetAssetBalance => WalletCapabilityDefinition {
            label: "Read wallet asset balance",
            description: "Allows the caller to read confirmed and spendable balances for XCH, CAT, DID, or NFT assets.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletSignCoinSpends => WalletCapabilityDefinition {
            label: "Sign wallet coin spends",
            description: "Allows the caller to request signatures for custom coin spends after per-request approval.",
            externally_observable: true,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletSignMessage => WalletCapabilityDefinition {
            label: "Sign wallet messages",
            description: "Allows the caller to request a message signature after per-request approval.",
            externally_observable: true,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletSendTransaction => WalletCapabilityDefinition {
            label: "Broadcast wallet transactions",
            description: "Allows the caller to submit an already signed spend bundle to the Chia network.",
            externally_observable: true,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetPublicKeys => WalletCapabilityDefinition {
            label: "Read wallet public keys",
            description: "Allows the caller to read public keys derived by the active wallet.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetDerivations => WalletCapabilityDefinition {
            label: "Read derivations",
            description: "Allows the caller to read wallet derivation records and addresses.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetSpendableCoinCount => WalletCapabilityDefinition {
            label: "Read spendable coin count",
            description: "Allows the caller to read the number of spendable coins in the wallet.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetCoinsByIds => WalletCapabilityDefinition {
            label: "Read coins by IDs",
            description: "Allows the caller to read specific wallet coin records by coin ID.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetCoins => WalletCapabilityDefinition {
            label: "Read coins",
            description: "Allows the caller to list wallet coins.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetPendingTransactions => WalletCapabilityDefinition {
            label: "Read pending transactions",
            description: "Allows the caller to read pending wallet transactions.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetTransaction => WalletCapabilityDefinition {
            label: "Read transaction",
            description: "Allows the caller to read a wallet transaction by height.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
        WalletCapability::WalletGetTransactions => WalletCapabilityDefinition {
            label: "Read transactions",
            description: "Allows the caller to list wallet transactions.",
            externally_observable: false,
            accesses_sensitive_secret: false,
        },
    }
}
