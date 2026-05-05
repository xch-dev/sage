use crate::bridge::methods::system::{AppInstallInstallResult, AppInstallInstallUrlParams, AppInstallInstallZipParams, AppInstallPreviewUrlParams, AppInstallPreviewZipParams, AppPermissionsApplyPermissionsParams, AppPermissionsApplyPermissionsResult, AppPermissionsGetReviewContextParams, AppPermissionsReviewContext, AppUpdateApplyUpdateParams, AppUpdateApplyUpdateResult, AppUpdateGetReviewContextParams, AppUpdateReviewContext, BridgeApprovalsChangedEvent, FileSystemSelectFileParams, FileSystemSelectFileResult, PendingBridgeApprovalView, RuntimeManagerActiveTaskbarRuntimeChangedEvent, RuntimeManagerRuntimesChangedEvent};
use crate::bridge::methods::user::app::get_info::{AppGetInfoResult, SageNetworkPermissionInfo};
use crate::bridge::methods::user::app::request_capability_grant::{
    RequestCapabilityGrantParams, RequestCapabilityGrantResult,
};
use crate::bridge::methods::user::app::request_network_whitelist_grant::{
    RequestNetworkWhitelistGrantParams, RequestNetworkWhitelistGrantResult,
};
use crate::bridge::methods::user::app::{
    GrantedCapabilitiesChangeEvent, GrantedNetworkWhitelistChangeEvent,
};
use crate::bridge::methods::user::bridge::ping::BridgePingResult;
use crate::bridge::methods::user::bridge::send::BridgeSendResult;
use crate::bridge::methods::user::wallet::send_xch::WalletSendXchParams;
use crate::runtime::stop::SystemKillRuntimeResult;
use crate::runtime::{ReadyToStopParams, RuntimeAckResult, RuntimeTargetParams, SetBeforeStopListenerParams};
use sage_api::{
    CheckAddress, CheckAddressResponse, GetCoins, GetCoinsByIds, GetCoinsByIdsResponse,
    GetCoinsResponse, GetDerivations, GetDerivationsResponse, GetKey, GetKeyResponse, GetKeys,
    GetKeysResponse, GetPendingTransactions, GetPendingTransactionsResponse, GetSecretKey,
    GetSecretKeyResponse, GetSpendableCoinCount, GetSpendableCoinCountResponse, GetSyncStatus,
    GetSyncStatusResponse, GetTransaction, GetTransactionResponse, GetTransactions,
    GetTransactionsResponse, GetVersion, GetVersionResponse, TransactionResponse,
};
use specta::TypeCollection;
use specta_typescript::{BigIntExportBehavior, Typescript};
use crate::bridge::methods::user::app::events::BeforeStopEvent;
use crate::bridge::methods::user::environment::{EnvironmentThemeChangedEvent, EnvironmentThemeGetCurrentResult};
use crate::bridge::{ResolveBridgeApprovalArgs, RustBridgeInvokeResult};
use crate::types::SageAppCapabilityDefinitionView;

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
    types.register::<GrantedCapabilitiesChangeEvent>();
    types.register::<GrantedNetworkWhitelistChangeEvent>();
    types.register::<BeforeStopEvent>();
    types.register::<SetBeforeStopListenerParams>();
    types.register::<ReadyToStopParams>();
    types.register::<RuntimeAckResult>();
    types.register::<GetKeys>();
    types.register::<GetKeysResponse>();
    types.register::<GetKey>();
    types.register::<GetKeyResponse>();
    types.register::<GetSecretKey>();
    types.register::<GetSecretKeyResponse>();
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
    types.register::<EnvironmentThemeGetCurrentResult>();
    types.register::<EnvironmentThemeChangedEvent>();

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

    Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .export(&types)
        .map_err(|err| format!("failed to export system bridge TS types: {err}"))
}
