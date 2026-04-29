use serde::{Deserialize, Serialize};
use specta::Type;
use crate::types::app::view::permission::SageGrantedPermissionsView;
use crate::types::app::view::snapshot::SageAppSnapshotView;
use crate::types::{SageAppCommon, SageAppIdentity};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppIdentityView {
    id: String,
    origin_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCommonView {
    identity: SageAppIdentityView,
    granted_permissions: SageGrantedPermissionsView,
    active_snapshot: SageAppSnapshotView,
}

impl From<&SageAppCommon> for SageAppCommonView {
    fn from(common: &SageAppCommon) -> Self {
        Self {
            identity: common.identity().into(),
            active_snapshot: common.active_snapshot().into(),
            granted_permissions: common.granted_permissions().into(),
        }
    }
}

impl From<&SageAppIdentity> for SageAppIdentityView {
    fn from(value: &SageAppIdentity) -> Self {
        Self {
            id: value.id().to_string(),
            origin_id: value.origin_id().to_string()
        }
    }
}
