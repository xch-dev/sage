use crate::AppsHostState;
use crate::bridge::emit_system_runtime_event_to_listeners;
use crate::bridge::event_emit::SystemRuntimeEvent;
use crate::capabilities::list::SystemBridgeCapability;
use crate::lifecycle::{list_installed_apps_internal};
use crate::types::ListedSageAppView;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListedAppsChangedEvent {
    pub apps: Vec<ListedSageAppView>,
}

impl SystemRuntimeEvent for ListedAppsChangedEvent {
    const TYPE: &'static str = "appRegistry.listedAppsChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability =
        SystemBridgeCapability::AppRegistryListenListedAppsChanged;
}

pub(crate) async fn emit_listed_apps_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let Ok(apps) = list_installed_apps_internal(&apps_state.db).await else {
        return;
    };

    let apps = apps
        .iter()
        .map(Into::into)
        .collect::<Vec<ListedSageAppView>>();

    emit_system_runtime_event_to_listeners(app_handle, apps_state, ListedAppsChangedEvent { apps })
        .await;
}
