use std::collections::BTreeMap;

use crate::{
    CapabilityDefinition, CapabilityFlags, SystemBridgeCapability, SystemCapabilityDefinition,
    UserBridgeCapability, UserCapabilityDefinition,
};

pub(crate) fn get_user_capability_definition(
    capability: UserBridgeCapability,
) -> UserCapabilityDefinition {
    match capability {
        UserBridgeCapability::StoragePersistentWebview => CapabilityDefinition::new(
            capability,
            "Persistent storage",
            "Allows the app to store data on this device between sessions.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::BridgeSend => CapabilityDefinition::new(
            capability,
            "Bridge messaging",
            "Allows the app to send messages through the Sage bridge. (Only for sandbox tests)",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::AppGetCapabilities => CapabilityDefinition::new(
            capability,
            "Read granted capabilities",
            "Allows the app to read the capabilities currently visible to it.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::AppGetInfo => CapabilityDefinition::new(
            capability,
            "Read app information",
            "Allows the app to read its Sage app identity and permission information.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::AppLifecycleReadyToStop => CapabilityDefinition::new(
            capability,
            "Acknowledge app shutdown",
            "Allows the app to acknowledge that it is ready to stop after a lifecycle request.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::AppLifecycleSetBeforeStopListener => CapabilityDefinition::new(
            capability,
            "Listen before app shutdown",
            "Allows the app to register a before-stop lifecycle listener.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::AppRequestCapabilityGrant => CapabilityDefinition::new(
            capability,
            "Request additional capability",
            "Allows the app to request a capability grant after installation.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::AppRequestNetworkWhitelistGrant => CapabilityDefinition::new(
            capability,
            "Request network access",
            "Allows the app to request access to an additional network target after installation.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::WalletGetKey => CapabilityDefinition::new(
            capability,
            "Read wallet key",
            "Allows the app to read public information about a wallet key.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetSecretKey => CapabilityDefinition::new(
            capability,
            "Read wallet secret key",
            "Allows the app to read wallet secrets, including the mnemonic or private key when available.",
            CapabilityFlags::new(false, true, true, true, true),
        ),
        UserBridgeCapability::WalletSendXch => CapabilityDefinition::new(
            capability,
            "Send XCH",
            "Allows the app to request XCH transactions from your wallet.",
            CapabilityFlags::new(true, false, true, true, true),
        ),
        UserBridgeCapability::WalletSendXchAutoSubmit => CapabilityDefinition::new(
            capability,
            "Automatic XCH send",
            "Allows the app to submit XCH transactions without asking for per-transaction approval.",
            CapabilityFlags::new(false, false, false, true, false),
        ),
        UserBridgeCapability::WalletGetSyncStatus => CapabilityDefinition::new(
            capability,
            "Read sync status",
            "Allows the app to read wallet sync status and current wallet balance summary.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetVersion => CapabilityDefinition::new(
            capability,
            "Read wallet version",
            "Allows the app to read the current Sage wallet version.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetXchUsdPrice => CapabilityDefinition::new(
            capability,
            "Read XCH/USD price",
            "Allows the app to read the current estimated XCH price in USD.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::WalletCheckAddress => CapabilityDefinition::new(
            capability,
            "Check address",
            "Allows the app to validate whether an address belongs to this wallet.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetDerivations => CapabilityDefinition::new(
            capability,
            "Read derivations",
            "Allows the app to read wallet derivation records and addresses.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetSpendableCoinCount => CapabilityDefinition::new(
            capability,
            "Read spendable coin count",
            "Allows the app to read the number of spendable coins in the wallet.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetCoinsByIds => CapabilityDefinition::new(
            capability,
            "Read coins by IDs",
            "Allows the app to read specific wallet coin records by coin ID.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetCoins => CapabilityDefinition::new(
            capability,
            "Read coins",
            "Allows the app to list wallet coins.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetPendingTransactions => CapabilityDefinition::new(
            capability,
            "Read pending transactions",
            "Allows the app to read pending wallet transactions.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetTransaction => CapabilityDefinition::new(
            capability,
            "Read transaction",
            "Allows the app to read a wallet transaction by height.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::WalletGetTransactions => CapabilityDefinition::new(
            capability,
            "Read transactions",
            "Allows the app to list wallet transactions.",
            CapabilityFlags::new(false, false, true, true, true),
        ),
        UserBridgeCapability::EnvironmentThemeGetCurrent => CapabilityDefinition::new(
            capability,
            "Read current theme",
            "Allows the app to read Sage's current theme.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::EnvironmentThemeCssVars => CapabilityDefinition::new(
            capability,
            "Use Sage theme CSS variables",
            "Allows Sage to inject current theme CSS variables into the app runtime.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::EnvironmentThemeListenChanged => CapabilityDefinition::new(
            capability,
            "Observe theme changes",
            "Allows the app to receive events when Sage's theme changes.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
        UserBridgeCapability::EnvironmentGetNetwork => CapabilityDefinition::new(
            capability,
            "Read current network",
            "Allows the app to read Sage's currently active network information.",
            CapabilityFlags::new(false, false, true, false, true),
        ),
    }
}

pub(crate) fn get_system_capability_definition(
    capability: SystemBridgeCapability,
) -> SystemCapabilityDefinition {
    match capability {
        SystemBridgeCapability::RuntimeManagerListRuntimes => CapabilityDefinition::new(
            capability,
            "List app runtimes",
            "Allows the system app to inspect running Sage app runtimes.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerFocusTaskbarRuntime => CapabilityDefinition::new(
            capability,
            "Focus taskbar app runtime",
            "Allows the system app to focus running Sage taskbar app runtime.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerHideRuntime => CapabilityDefinition::new(
            capability,
            "Hide app runtime",
            "Allows the system app to hide running Sage app runtime.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerKillRuntime => CapabilityDefinition::new(
            capability,
            "Kill app runtime",
            "Allows the system app to stop running Sage app runtime.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerGetActiveTaskbarRuntime => CapabilityDefinition::new(
            capability,
            "Get active runtime",
            "Allows the system app to retrieve the currently active Sage app runtime.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerListenRuntimesChanged => CapabilityDefinition::new(
            capability,
            "Observe runtime changes",
            "Allows the system app to receive events when Sage app runtimes change.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerListenActiveTaskbarRuntimeChanged => {
            CapabilityDefinition::new(
                capability,
                "Observe active runtime changes",
                "Allows the system app to receive events when the active Sage app runtime changes.",
                system_app_flags(),
            )
        }
        SystemBridgeCapability::RuntimeManagerHideSelf => CapabilityDefinition::new(
            capability,
            "Hide itself",
            "Allows the system app to hide its own runtime.",
            system_app_flags(),
        ),
        SystemBridgeCapability::RuntimeManagerCloseSelf => CapabilityDefinition::new(
            capability,
            "Close itself",
            "Allows the system app to close its own runtime.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppUpdateRead => CapabilityDefinition::new(
            capability,
            "Read app update review context",
            "Allows the system app to read update information for installed Sage apps.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppUpdateApply => CapabilityDefinition::new(
            capability,
            "Apply app updates",
            "Allows the system app to download and apply approved Sage app updates.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppRegistryListenListedAppsChanged => CapabilityDefinition::new(
            capability,
            "Observe listed apps changes",
            "Allows the system app to receive events when installed/listed Sage apps change.",
            system_app_flags(),
        ),
        SystemBridgeCapability::CapabilityDefinitionsRead => CapabilityDefinition::new(
            capability,
            "Read capability definitions",
            "Allows the system app to read Sage capability definitions.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppPermissionsRead => CapabilityDefinition::new(
            capability,
            "Read app permissions",
            "Allows the system app to read app permissions for review.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppPermissionsApply => CapabilityDefinition::new(
            capability,
            "Apply app permissions",
            "Allows the system app to apply reviewed app permission changes.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppInstallPreview => CapabilityDefinition::new(
            capability,
            "Preview app installs",
            "Allows the system app to preview URL and ZIP app installations.",
            system_app_flags(),
        ),
        SystemBridgeCapability::AppInstallApply => CapabilityDefinition::new(
            capability,
            "Install apps",
            "Allows the system app to install Sage apps after review.",
            system_app_flags(),
        ),
        SystemBridgeCapability::FileSystemSelectFile => CapabilityDefinition::new(
            capability,
            "Select file",
            "Allows the system app to ask the user to select a local file.",
            system_app_flags(),
        ),
        SystemBridgeCapability::BridgeApprovalList => CapabilityDefinition::new(
            capability,
            "List bridge approvals",
            "Allows the system app to list pending bridge approvals.",
            system_app_flags(),
        ),
        SystemBridgeCapability::BridgeApprovalResolve => CapabilityDefinition::new(
            capability,
            "Resolve bridge approval",
            "Allows the system app to resolve a pending bridge approval.",
            system_app_flags(),
        ),
        SystemBridgeCapability::BridgeApprovalListenApprovalsChanged => CapabilityDefinition::new(
            capability,
            "Listen for bridge approval changes",
            "Allows the system app to listen for changes in pending bridge approvals.",
            system_app_flags(),
        ),
        SystemBridgeCapability::DonationGetDetails => CapabilityDefinition::new(
            capability,
            "Get details for donation",
            "Allows the system app to retrieve details to send donation.",
            system_app_flags(),
        ),
        SystemBridgeCapability::SandboxGetState => CapabilityDefinition::new(
            capability,
            "Read sandbox state",
            "Allows the system app to read Sage app sandbox test state.",
            system_app_flags(),
        ),
        SystemBridgeCapability::SandboxRerunTests => CapabilityDefinition::new(
            capability,
            "Re-run sandbox tests",
            "Allows the system app to re-run Sage app sandbox tests.",
            system_app_flags(),
        ),
        SystemBridgeCapability::SandboxListenStateChanged => CapabilityDefinition::new(
            capability,
            "Observe sandbox state changes",
            "Allows the system app to receive events when sandbox test state changes.",
            system_app_flags(),
        ),
        SystemBridgeCapability::WalletListWallets => CapabilityDefinition::new(
            capability,
            "List wallets",
            "Allows the system app to list wallets available in Sage.",
            system_app_flags(),
        ),
    }
}

pub(crate) fn user_registry() -> BTreeMap<UserBridgeCapability, UserCapabilityDefinition> {
    UserBridgeCapability::ALL
        .iter()
        .copied()
        .map(|capability| (capability, get_user_capability_definition(capability)))
        .collect()
}

fn system_app_flags() -> CapabilityFlags {
    CapabilityFlags::new(false, false, true, false, true)
}
