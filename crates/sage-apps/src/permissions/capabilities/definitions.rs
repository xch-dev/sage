use std::collections::BTreeMap;

use crate::bridge::capabilities::{SystemBridgeCapability, UserBridgeCapability};
use crate::permissions::capabilities::types::{CapabilityFlags, SystemCapabilityDefinition, UserCapabilityDefinition};
use crate::types::{SageAppCapabilityDefinitionView, SageAppCapabilityFlagsView};

pub(crate) fn get_user_capability_definition(
    capability: UserBridgeCapability,
) -> UserCapabilityDefinition {
    match capability {
        UserBridgeCapability::PersistentStorage => UserCapabilityDefinition {
            capability,
            label: "Persistent storage",
            description: "Allows the app to store data on this device between sessions.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: true,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::BridgeSend => UserCapabilityDefinition {
            capability,
            label: "Bridge messaging",
            description: "Allows the app to send messages through the Sage bridge. (Only for sandbox tests)",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::AppGetCapabilities => UserCapabilityDefinition {
            capability,
            label: "Read granted capabilities",
            description: "Allows the app to read the capabilities currently visible to it.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::AppGetInfo => UserCapabilityDefinition {
            capability,
            label: "Read app information",
            description: "Allows the app to read its Sage app identity and permission information.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::AppLifecycleReadyToStop => UserCapabilityDefinition {
            capability,
            label: "Acknowledge app shutdown",
            description: "Allows the app to acknowledge that it is ready to stop after a lifecycle request.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::AppLifecycleSetBeforeStopListener => UserCapabilityDefinition {
            capability,
            label: "Listen before app shutdown",
            description: "Allows the app to register a before-stop lifecycle listener.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::AppRequestCapabilityGrant => UserCapabilityDefinition {
            capability,
            label: "Request additional capability",
            description: "Allows the app to request a capability grant after installation.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::AppRequestNetworkWhitelistGrant => UserCapabilityDefinition {
            capability,
            label: "Request network access",
            description: "Allows the app to request access to an additional network target after installation.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::WalletGetKeys => UserCapabilityDefinition {
            capability,
            label: "List wallet keys",
            description: "Allows the app to list wallet keys configured in Sage.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetKey => UserCapabilityDefinition {
            capability,
            label: "Read wallet key",
            description: "Allows the app to read public information about a wallet key.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetSecretKey => UserCapabilityDefinition {
            capability,
            label: "Read wallet secret key",
            description: "Allows the app to read wallet secrets, including the mnemonic or private key when available.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: true,
                requestable_by_app: true,
                user_grantable: true,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::WalletSendXch => UserCapabilityDefinition {
            capability,
            label: "Send XCH",
            description: "Allows the app to request XCH transactions from your wallet.",
            flags: CapabilityFlags {
                externally_observable: true,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: true,
                shared_with_app: true,
            },
        },

        UserBridgeCapability::WalletSendXchAutoSubmit => UserCapabilityDefinition {
            capability,
            label: "Automatic XCH send",
            description: "Allows the app to submit XCH transactions without asking for per-transaction approval.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: false,
                user_grantable: false,
                shared_with_app: false,
            },
        },
        UserBridgeCapability::WalletGetSyncStatus => UserCapabilityDefinition {
            capability,
            label: "Read sync status",
            description: "Allows the app to read wallet sync status and current wallet balance summary.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetVersion => UserCapabilityDefinition {
            capability,
            label: "Read wallet version",
            description: "Allows the app to read the current Sage wallet version.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletCheckAddress => UserCapabilityDefinition {
            capability,
            label: "Check address",
            description: "Allows the app to validate whether an address belongs to this wallet.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetDerivations => UserCapabilityDefinition {
            capability,
            label: "Read derivations",
            description: "Allows the app to read wallet derivation records and addresses.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetSpendableCoinCount => UserCapabilityDefinition {
            capability,
            label: "Read spendable coin count",
            description: "Allows the app to read the number of spendable coins in the wallet.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetCoinsByIds => UserCapabilityDefinition {
            capability,
            label: "Read coins by IDs",
            description: "Allows the app to read specific wallet coin records by coin ID.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetCoins => UserCapabilityDefinition {
            capability,
            label: "Read coins",
            description: "Allows the app to list wallet coins.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetPendingTransactions => UserCapabilityDefinition {
            capability,
            label: "Read pending transactions",
            description: "Allows the app to read pending wallet transactions.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetTransaction => UserCapabilityDefinition {
            capability,
            label: "Read transaction",
            description: "Allows the app to read a wallet transaction by height.",
            flags: read_wallet_flags(),
        },

        UserBridgeCapability::WalletGetTransactions => UserCapabilityDefinition {
            capability,
            label: "Read transactions",
            description: "Allows the app to list wallet transactions.",
            flags: read_wallet_flags(),
        },
    }
}

pub(crate) fn get_system_capability_definition(
    capability: SystemBridgeCapability,
) -> SystemCapabilityDefinition {
    match capability {
        SystemBridgeCapability::RuntimeManagerListRuntimes => SystemCapabilityDefinition {
            capability,
            label: "List app runtimes",
            description: "Allows the system app to inspect running Sage app runtimes.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        SystemBridgeCapability::RuntimeManagerFocusRuntime => SystemCapabilityDefinition {
            capability,
            label: "Focus app runtimes",
            description: "Allows the system app to focus running Sage app runtimes.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        SystemBridgeCapability::RuntimeManagerHideRuntime => SystemCapabilityDefinition {
            capability,
            label: "Hide app runtimes",
            description: "Allows the system app to hide running Sage app runtimes.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        SystemBridgeCapability::RuntimeManagerKillRuntime => SystemCapabilityDefinition {
            capability,
            label: "Kill app runtimes",
            description: "Allows the system app to stop running Sage app runtimes.",
            flags: CapabilityFlags {
                externally_observable: false,
                accesses_sensitive_secret: false,
                requestable_by_app: true,
                user_grantable: false,
                shared_with_app: true,
            },
        },

        SystemBridgeCapability::RuntimeManagerListenRuntimesChanged => {
            SystemCapabilityDefinition {
                capability,
                label: "Observe runtime changes",
                description: "Allows the system app to receive events when Sage app runtimes change.",
                flags: CapabilityFlags {
                    externally_observable: false,
                    accesses_sensitive_secret: false,
                    requestable_by_app: true,
                    user_grantable: false,
                    shared_with_app: true,
                },
            }
        }
    }
}

pub(crate) fn user_capability_definition_view(
    definition: UserCapabilityDefinition,
) -> SageAppCapabilityDefinitionView {
    SageAppCapabilityDefinitionView {
        key: definition.capability.key().to_string(),
        label: definition.label.to_string(),
        description: definition.description.to_string(),
        flags: SageAppCapabilityFlagsView {
            externally_observable: definition.flags.externally_observable,
            accesses_sensitive_secret: definition.flags.accesses_sensitive_secret,
            requestable_by_app: definition.flags.requestable_by_app,
            user_grantable: definition.flags.user_grantable,
        },
    }
}

pub(crate) fn user_registry() -> BTreeMap<UserBridgeCapability, UserCapabilityDefinition> {
    UserBridgeCapability::ALL
        .iter()
        .copied()
        .map(|capability| {
            let definition = get_user_capability_definition(capability);
            (capability, definition)
        })
        .collect()
}

fn read_wallet_flags() -> CapabilityFlags {
    CapabilityFlags {
        externally_observable: false,
        accesses_sensitive_secret: false,
        requestable_by_app: true,
        user_grantable: true,
        shared_with_app: true,
    }
}
