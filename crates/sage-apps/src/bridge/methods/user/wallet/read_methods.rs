use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::UserBridgeCapability;

use sage_api::{
    CheckAddress, GetCoins, GetCoinsByIds, GetDerivations, GetKey, GetKeys, GetPendingTransactions,
    GetSpendableCoinCount, GetSyncStatus, GetTransaction, GetTransactions, GetVersion,
    GetXchUsdPrice,
};

macro_rules! define_wallet_read_no_params_async_method {
    ($struct_name:ident, $capability:ident, $method_name:expr, $request_ident:ident, $handler:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $struct_name;

        #[async_trait]
        impl BridgeMethod for $struct_name {
            fn name(&self) -> &'static str {
                $method_name
            }

            fn capability(&self) -> BridgeMethodCapability {
                BridgeMethodCapability::user(UserBridgeCapability::$capability)
            }

            fn approval_request(
                &self,
                _ctx: BridgeContext<'_>,
                _request: &RustBridgeRequest,
            ) -> BridgeApprovalRequestResult {
                Ok(None)
            }

            async fn handle(
                &self,
                _ctx: BridgeContext<'_>,
                tools: BridgeTools<'_>,
                _request: &RustBridgeRequest,
            ) -> BridgeHandleResult {
                let sage = tools.app_state.lock().await;

                let result = sage.$handler($request_ident {}).await.map_err(|err| {
                    BridgeMethodHandleError::internal_error(format!(
                        "failed to execute {}: {err}",
                        self.name()
                    ))
                })?;

                Ok(Box::new(result))
            }
        }
    };
}

macro_rules! define_wallet_read_no_params_sync_method {
    ($struct_name:ident, $capability:ident, $method_name:expr, $request_ident:ident, $handler:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $struct_name;

        #[async_trait]
        impl BridgeMethod for $struct_name {
            fn name(&self) -> &'static str {
                $method_name
            }

            fn capability(&self) -> BridgeMethodCapability {
                BridgeMethodCapability::user(UserBridgeCapability::$capability)
            }

            fn approval_request(
                &self,
                _ctx: BridgeContext<'_>,
                _request: &RustBridgeRequest,
            ) -> BridgeApprovalRequestResult {
                Ok(None)
            }

            async fn handle(
                &self,
                _ctx: BridgeContext<'_>,
                tools: BridgeTools<'_>,
                _request: &RustBridgeRequest,
            ) -> BridgeHandleResult {
                let sage = tools.app_state.lock().await;

                let result = sage.$handler($request_ident {}).map_err(|err| {
                    BridgeMethodHandleError::internal_error(format!(
                        "failed to execute {}: {err}",
                        self.name()
                    ))
                })?;

                Ok(Box::new(result))
            }
        }
    };
}

macro_rules! define_wallet_read_params_async_method {
    ($struct_name:ident, $capability:ident, $method_name:expr, $request_ty:ty, $handler:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $struct_name;

        #[async_trait]
        impl BridgeMethod for $struct_name {
            fn name(&self) -> &'static str {
                $method_name
            }

            fn capability(&self) -> BridgeMethodCapability {
                BridgeMethodCapability::user(UserBridgeCapability::$capability)
            }

            fn approval_request(
                &self,
                _ctx: BridgeContext<'_>,
                _request: &RustBridgeRequest,
            ) -> BridgeApprovalRequestResult {
                Ok(None)
            }

            async fn handle(
                &self,
                _ctx: BridgeContext<'_>,
                tools: BridgeTools<'_>,
                request: &RustBridgeRequest,
            ) -> BridgeHandleResult {
                let params: $request_ty = parse_required_params(self, request)?;

                let sage = tools.app_state.lock().await;

                let result = sage.$handler(params).await.map_err(|err| {
                    BridgeMethodHandleError::internal_error(format!(
                        "failed to execute {}: {err}",
                        self.name()
                    ))
                })?;

                Ok(Box::new(result))
            }
        }
    };
}

macro_rules! define_wallet_read_params_sync_method {
    ($struct_name:ident, $capability:ident, $method_name:expr, $request_ty:ty, $handler:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $struct_name;

        #[async_trait]
        impl BridgeMethod for $struct_name {
            fn name(&self) -> &'static str {
                $method_name
            }

            fn capability(&self) -> BridgeMethodCapability {
                BridgeMethodCapability::user(UserBridgeCapability::$capability)
            }

            fn approval_request(
                &self,
                _ctx: BridgeContext<'_>,
                _request: &RustBridgeRequest,
            ) -> BridgeApprovalRequestResult {
                Ok(None)
            }

            async fn handle(
                &self,
                _ctx: BridgeContext<'_>,
                tools: BridgeTools<'_>,
                request: &RustBridgeRequest,
            ) -> BridgeHandleResult {
                let params: $request_ty = parse_required_params(self, request)?;

                let sage = tools.app_state.lock().await;

                let result = sage.$handler(params).map_err(|err| {
                    BridgeMethodHandleError::internal_error(format!(
                        "failed to execute {}: {err}",
                        self.name()
                    ))
                })?;

                Ok(Box::new(result))
            }
        }
    };
}

define_wallet_read_no_params_sync_method!(
    WalletGetKeys,
    WalletGetKeys,
    "wallet.getKeys",
    GetKeys,
    get_keys
);

define_wallet_read_params_sync_method!(
    WalletGetKey,
    WalletGetKey,
    "wallet.getKey",
    GetKey,
    get_key
);

define_wallet_read_no_params_async_method!(
    WalletGetSyncStatus,
    WalletGetSyncStatus,
    "wallet.getSyncStatus",
    GetSyncStatus,
    get_sync_status
);

define_wallet_read_no_params_sync_method!(
    WalletGetVersion,
    WalletGetVersion,
    "wallet.getVersion",
    GetVersion,
    get_version
);

define_wallet_read_no_params_async_method!(
    WalletGetPendingTransactions,
    WalletGetPendingTransactions,
    "wallet.getPendingTransactions",
    GetPendingTransactions,
    get_pending_transactions
);

define_wallet_read_no_params_async_method!(
    WalletGetXchUsdPrice,
    WalletGetXchUsdPrice,
    "wallet.getXchUsdPrice",
    GetXchUsdPrice,
    get_xch_usd_price
);

define_wallet_read_params_async_method!(
    WalletCheckAddress,
    WalletCheckAddress,
    "wallet.checkAddress",
    CheckAddress,
    check_address
);

define_wallet_read_params_async_method!(
    WalletGetDerivations,
    WalletGetDerivations,
    "wallet.getDerivations",
    GetDerivations,
    get_derivations
);

define_wallet_read_params_async_method!(
    WalletGetSpendableCoinCount,
    WalletGetSpendableCoinCount,
    "wallet.getSpendableCoinCount",
    GetSpendableCoinCount,
    get_spendable_coin_count
);

define_wallet_read_params_async_method!(
    WalletGetCoinsByIds,
    WalletGetCoinsByIds,
    "wallet.getCoinsByIds",
    GetCoinsByIds,
    get_coins_by_ids
);

define_wallet_read_params_async_method!(
    WalletGetCoins,
    WalletGetCoins,
    "wallet.getCoins",
    GetCoins,
    get_coins
);

define_wallet_read_params_async_method!(
    WalletGetTransaction,
    WalletGetTransaction,
    "wallet.getTransaction",
    GetTransaction,
    get_transaction
);

define_wallet_read_params_async_method!(
    WalletGetTransactions,
    WalletGetTransactions,
    "wallet.getTransactions",
    GetTransactions,
    get_transactions
);
