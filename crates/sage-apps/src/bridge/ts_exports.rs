use sage_api::{
    CheckAddress, CheckAddressResponse, GetCoins, GetCoinsByIds, GetCoinsByIdsResponse,
    GetCoinsResponse, GetDerivations, GetDerivationsResponse, GetKeyResponse,
    GetPendingTransactions, GetPendingTransactionsResponse, GetSecretKeyResponse,
    GetSpendableCoinCount, GetSpendableCoinCountResponse, GetSyncStatus, GetSyncStatusResponse,
    GetTransaction, GetTransactionResponse, GetTransactions, GetTransactionsResponse, GetVersion,
    GetVersionResponse, GetXchUsdPriceResponse, TransactionResponse,
};
use specta::TypeCollection;
use specta_typescript::{BigIntExportBehavior, Typescript};

use crate::{
    AppGetInfoResult, AppInstallInstallResult, AppInstallInstallUrlParams,
    AppInstallInstallZipParams, AppInstallPreviewUrlParams, AppInstallPreviewZipParams,
    AppPermissionsApplyPermissionsParams, AppPermissionsApplyPermissionsResult,
    AppPermissionsGetReviewContextParams, AppPermissionsReviewContext, AppUpdateApplyUpdateParams,
    AppUpdateApplyUpdateResult, AppUpdateGetReviewContextParams, AppUpdateReviewContext,
    BeforeStopEvent, BridgeApprovalsChangedEvent, BridgePingResult, BridgeSendResult,
    DonationDetails, DonationGetDetailsParams, EnvironmentGetNetworkResult,
    EnvironmentOpenExternalUrlParams, EnvironmentOpenExternalUrlResult,
    EnvironmentThemeChangedEvent, EnvironmentThemeGetCurrentResult, FileSystemSelectFileParams,
    FileSystemSelectFileResult, GrantedCapabilitiesChangeEvent, GrantedNetworkWhitelistChangeEvent,
    ListedAppsChangedEvent, PendingBridgeApprovalView, PendingUpdateChangedEvent,
    ReadyToStopParams, RequestCapabilityGrantParams, RequestCapabilityGrantResult,
    RequestNetworkWhitelistGrantParams, RequestNetworkWhitelistGrantResult,
    RequestPermissionGrantsParams, RequestPermissionGrantsResult, ResolveBridgeApprovalArgs,
    RuntimeAckResult, RuntimeManagerActiveTaskbarRuntimeChangedEvent,
    RuntimeManagerRuntimesChangedEvent, RuntimeTargetParams, RustBridgeInvokeResult,
    SageAppCapabilityDefinitionView, SageAppWalletScope, SageNetworkPermissionInfo,
    SandboxStateChangedEvent, SandboxStateView, SelectedWalletChangedEvent,
    SetBeforeStopListenerParams, SystemKillRuntimeResult, SystemWalletView,
    WalletFilterUnlockedCoinsParams, WalletFilterUnlockedCoinsResult, WalletGetAssetBalanceParams,
    WalletGetAssetBalanceResult, WalletGetAssetCoinsParams, WalletGetAssetCoinsResult,
    WalletGetPublicKeysParams, WalletGetPublicKeysResult, WalletListWalletsResult,
    WalletSendTransactionParams, WalletSendTransactionResult, WalletSendXchParams,
    WalletSignCoinSpendsParams, WalletSignCoinSpendsResult, WalletSignMessageParams,
    WalletSignMessageResult,
};

pub fn export_user_bridge_typescript() -> Result<String, String> {
    let mut types = TypeCollection::default();

    types.register::<RustBridgeInvokeResult>();

    types.register::<BridgePingResult>();
    types.register::<BridgeSendResult>();
    types.register::<SageNetworkPermissionInfo>();
    types.register::<AppGetInfoResult>();
    types.register::<WalletSendXchParams>();
    types.register::<TransactionResponse>();
    types.register::<RequestCapabilityGrantParams>();
    types.register::<RequestCapabilityGrantResult>();
    types.register::<RequestNetworkWhitelistGrantParams>();
    types.register::<RequestNetworkWhitelistGrantResult>();
    types.register::<RequestPermissionGrantsParams>();
    types.register::<RequestPermissionGrantsResult>();
    types.register::<GrantedCapabilitiesChangeEvent>();
    types.register::<GrantedNetworkWhitelistChangeEvent>();
    types.register::<BeforeStopEvent>();
    types.register::<SetBeforeStopListenerParams>();
    types.register::<ReadyToStopParams>();
    types.register::<RuntimeAckResult>();
    types.register::<GetKeyResponse>();
    types.register::<GetXchUsdPriceResponse>();
    types.register::<GetSecretKeyResponse>();
    types.register::<WalletGetPublicKeysParams>();
    types.register::<WalletGetPublicKeysResult>();
    types.register::<WalletFilterUnlockedCoinsParams>();
    types.register::<WalletFilterUnlockedCoinsResult>();
    types.register::<WalletGetAssetCoinsParams>();
    types.register::<WalletGetAssetCoinsResult>();
    types.register::<WalletGetAssetBalanceParams>();
    types.register::<WalletGetAssetBalanceResult>();
    types.register::<WalletSignCoinSpendsParams>();
    types.register::<WalletSignCoinSpendsResult>();
    types.register::<WalletSignMessageParams>();
    types.register::<WalletSignMessageResult>();
    types.register::<WalletSendTransactionParams>();
    types.register::<WalletSendTransactionResult>();
    types.register::<GetSyncStatus>();
    types.register::<GetSyncStatusResponse>();
    types.register::<GetVersion>();
    types.register::<GetVersionResponse>();
    types.register::<GetPendingTransactions>();
    types.register::<GetPendingTransactionsResponse>();
    types.register::<CheckAddress>();
    types.register::<CheckAddressResponse>();
    types.register::<GetDerivations>();
    types.register::<GetDerivationsResponse>();
    types.register::<GetSpendableCoinCount>();
    types.register::<GetSpendableCoinCountResponse>();
    types.register::<GetCoinsByIds>();
    types.register::<GetCoinsByIdsResponse>();
    types.register::<GetCoins>();
    types.register::<GetCoinsResponse>();
    types.register::<GetTransaction>();
    types.register::<GetTransactionResponse>();
    types.register::<GetTransactions>();
    types.register::<GetTransactionsResponse>();
    types.register::<SelectedWalletChangedEvent>();
    types.register::<EnvironmentThemeGetCurrentResult>();
    types.register::<EnvironmentThemeChangedEvent>();
    types.register::<EnvironmentGetNetworkResult>();
    types.register::<EnvironmentOpenExternalUrlParams>();
    types.register::<EnvironmentOpenExternalUrlResult>();

    Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .export(&types)
        .map_err(|err| format!("failed to export user bridge TS types: {err}"))
}

pub fn export_system_bridge_typescript() -> Result<String, String> {
    let mut types = TypeCollection::default();

    types.register::<RuntimeTargetParams>();
    types.register::<SystemKillRuntimeResult>();
    types.register::<RuntimeManagerRuntimesChangedEvent>();
    types.register::<RuntimeManagerActiveTaskbarRuntimeChangedEvent>();

    types.register::<AppInstallPreviewUrlParams>();
    types.register::<AppInstallPreviewZipParams>();
    types.register::<AppInstallInstallUrlParams>();
    types.register::<AppInstallInstallZipParams>();
    types.register::<AppInstallInstallResult>();

    types.register::<AppUpdateGetReviewContextParams>();
    types.register::<AppUpdateReviewContext>();
    types.register::<AppUpdateApplyUpdateParams>();
    types.register::<AppUpdateApplyUpdateResult>();
    types.register::<PendingUpdateChangedEvent>();

    types.register::<ListedAppsChangedEvent>();

    types.register::<SageAppCapabilityDefinitionView>();
    types.register::<AppPermissionsGetReviewContextParams>();
    types.register::<AppPermissionsReviewContext>();
    types.register::<AppPermissionsApplyPermissionsParams>();
    types.register::<AppPermissionsApplyPermissionsResult>();

    types.register::<FileSystemSelectFileParams>();
    types.register::<FileSystemSelectFileResult>();

    types.register::<ResolveBridgeApprovalArgs>();
    types.register::<PendingBridgeApprovalView>();
    types.register::<BridgeApprovalsChangedEvent>();

    types.register::<DonationGetDetailsParams>();
    types.register::<DonationDetails>();

    types.register::<SandboxStateChangedEvent>();
    types.register::<SandboxStateView>();

    types.register::<SageAppWalletScope>();
    types.register::<SystemWalletView>();
    types.register::<WalletListWalletsResult>();

    Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .export(&types)
        .map_err(|err| format!("failed to export system bridge TS types: {err}"))
}
