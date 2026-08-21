use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::{
    AppsHostState, SageApp, SageAppCompatibility, SageAppCompatibilityStatus,
    SageGrantedPermissions, SharedSageApp, SystemBridgeCapability, SystemRuntimeEvent,
    UserSageAppPendingUpdate, UserSageAppPendingUpdateDecisionView,
    emit_system_runtime_event_to_listeners,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub(crate) enum PendingUpdateStatusView {
    None,
    ReadyToApply {
        #[serde(rename = "manifestHash")]
        #[specta(rename = "manifestHash")]
        manifest_hash: String,
    },
    RequiresReview {
        #[serde(rename = "manifestHash")]
        #[specta(rename = "manifestHash")]
        manifest_hash: String,
    },
    RequiresNewerSage {
        #[serde(rename = "manifestHash")]
        #[specta(rename = "manifestHash")]
        manifest_hash: String,
        #[serde(rename = "currentVersion")]
        #[specta(rename = "currentVersion")]
        current_version: String,
        #[serde(rename = "minimumVersion")]
        #[specta(rename = "minimumVersion")]
        minimum_version: String,
    },
    UntestedNewerSage {
        #[serde(rename = "manifestHash")]
        #[specta(rename = "manifestHash")]
        manifest_hash: String,
        #[serde(rename = "currentVersion")]
        #[specta(rename = "currentVersion")]
        current_version: String,
        #[serde(rename = "testedMaxVersion")]
        #[specta(rename = "testedMaxVersion")]
        tested_max_version: String,
    },
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingUpdateChangedEvent {
    pub app_id: String,
    pub status: PendingUpdateStatusView,
}

impl SystemRuntimeEvent for PendingUpdateChangedEvent {
    const TYPE: &'static str = "appUpdate.pendingUpdateChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability = SystemBridgeCapability::AppUpdateRead;
}

pub(crate) async fn emit_pending_update_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    shared_sage_app: &SharedSageApp,
) {
    let Some((app_id, status)) = shared_sage_app.with(|sage_app| match sage_app {
        SageApp::User(user_app) => {
            let status = pending_update_status_view(
                app_handle,
                user_app.pending_update(),
                user_app.common().granted_permissions(),
            );

            Some((user_app.common().id().to_string(), status))
        }
        SageApp::System(_) => None,
    }) else {
        return;
    };

    emit_system_runtime_event_to_listeners(
        app_handle,
        apps_state,
        PendingUpdateChangedEvent { app_id, status },
    )
    .await;
}

fn pending_update_status_view(
    app_handle: &AppHandle,
    pending_update: Option<&UserSageAppPendingUpdate>,
    granted_permissions: &SageGrantedPermissions,
) -> PendingUpdateStatusView {
    let Some(pending) = pending_update else {
        return PendingUpdateStatusView::None;
    };

    let manifest_hash = pending.manifest_hash().to_string();
    let compatibility =
        SageAppCompatibility::for_app(app_handle, pending.manifest().sage_version());

    match compatibility.status() {
        SageAppCompatibilityStatus::RequiresNewerSage { minimum_version } => {
            PendingUpdateStatusView::RequiresNewerSage {
                manifest_hash,
                current_version: app_handle.package_info().version.to_string(),
                minimum_version: minimum_version.clone(),
            }
        }
        SageAppCompatibilityStatus::Invalid { .. } => {
            PendingUpdateStatusView::RequiresReview { manifest_hash }
        }
        SageAppCompatibilityStatus::UntestedNewerSage { tested_max_version } => {
            PendingUpdateStatusView::UntestedNewerSage {
                manifest_hash,
                current_version: app_handle.package_info().version.to_string(),
                tested_max_version: tested_max_version.clone(),
            }
        }
        SageAppCompatibilityStatus::Compatible => {
            if UserSageAppPendingUpdateDecisionView::from_pending_update(
                granted_permissions,
                pending.manifest().permissions(),
            )
            .is_review()
            {
                PendingUpdateStatusView::RequiresReview { manifest_hash }
            } else {
                PendingUpdateStatusView::ReadyToApply { manifest_hash }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PendingUpdateStatusView;

    #[test]
    fn pending_update_manifest_hash_is_serialized_as_camel_case() {
        for status in [
            PendingUpdateStatusView::ReadyToApply {
                manifest_hash: "hash".to_string(),
            },
            PendingUpdateStatusView::RequiresReview {
                manifest_hash: "hash".to_string(),
            },
            PendingUpdateStatusView::RequiresNewerSage {
                manifest_hash: "hash".to_string(),
                current_version: "0.13.0".to_string(),
                minimum_version: "0.14.0".to_string(),
            },
            PendingUpdateStatusView::UntestedNewerSage {
                manifest_hash: "hash".to_string(),
                current_version: "0.14.0".to_string(),
                tested_max_version: "0.13.0".to_string(),
            },
        ] {
            let value = serde_json::to_value(status).unwrap();

            assert_eq!(value["manifestHash"], "hash");
            assert!(value.get("manifest_hash").is_none());
        }
    }
}
