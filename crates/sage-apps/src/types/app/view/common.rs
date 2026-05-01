use serde::{Serialize};
use specta::Type;
use crate::types::app::view::permission::SageGrantedPermissionsView;
use crate::types::app::view::snapshot::SageAppSnapshotView;
use crate::types::{SageAppCommon, SageAppIdentity};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppIdentityView {
    id: String,
    origin_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCommonView {
    identity: SageAppIdentityView,
    granted_permissions: SageGrantedPermissionsView,
    active_snapshot: SageAppSnapshotView,
    icon: Option<SageAppIconView>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppIconView {
    mime: String,
    bytes: Vec<u8>,
}

impl From<&SageAppCommon> for SageAppCommonView {
    fn from(common: &SageAppCommon) -> Self {
        Self {
            identity: common.identity().into(),
            active_snapshot: common.active_snapshot().into(),
            granted_permissions: common.granted_permissions().into(),
            icon: read_common_icon(common),
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

fn read_common_icon(common: &SageAppCommon) -> Option<SageAppIconView> {
    let icon_path = common.active_snapshot().manifest().icon()?;
    let file_path = common
        .active_snapshot()
        .resolve_file_path(icon_path)
        .ok()?;

    let bytes = std::fs::read(&file_path).ok()?;
    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Some(SageAppIconView { mime, bytes })
}
