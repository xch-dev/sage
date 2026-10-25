use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::{
    AppsHostState, UserBridgeCapability, UserRuntimeEvent,
    emit_user_runtime_event_to_wallet_listeners,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SelectedWalletChangedEvent {
    pub fingerprint: u32,
}

impl UserRuntimeEvent for SelectedWalletChangedEvent {
    const TYPE: &'static str = "wallet.selectedWallet.changed";
    const REQUIRED_CAPABILITY: UserBridgeCapability =
        UserBridgeCapability::WalletListenSelectedWalletChanged;
}

pub(crate) async fn emit_selected_wallet_changed_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    fingerprint: u32,
) {
    emit_user_runtime_event_to_wallet_listeners(
        app_handle,
        apps_state,
        fingerprint,
        SelectedWalletChangedEvent { fingerprint },
    )
    .await;
}
