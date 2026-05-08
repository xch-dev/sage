use crate::types::{SageAppPackageManifest, SageAppUrl, UserSageAppPendingUpdate};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppPendingUpdateView {
    app_url: SageAppUrl,
    manifest_hash: String,
    manifest: SageAppPackageManifest,
}

impl From<&UserSageAppPendingUpdate> for UserSageAppPendingUpdateView {
    fn from(value: &UserSageAppPendingUpdate) -> Self {
        Self {
            app_url: value.app_url().clone(),
            manifest_hash: value.manifest_hash().to_string(),
            manifest: value.manifest().clone(),
        }
    }
}
