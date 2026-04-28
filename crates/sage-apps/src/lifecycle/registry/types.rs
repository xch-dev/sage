use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::types::{
    InstalledSageAppStorage, SageAppCommon, SageAppSnapshot, SageGrantedPermissions, UserSageApp,
    UserSageAppSource,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedUserSageApp {
    id: String,
    origin_id: String,
    app_dir: String,
    granted_permissions: SageGrantedPermissions,
    storage: InstalledSageAppStorage,
    active_snapshot: SageAppSnapshot,
    source: UserSageAppSource,
}

impl From<&UserSageApp> for PersistedUserSageApp {
    fn from(app: &UserSageApp) -> Self {
        let common = app.common();

        Self {
            id: common.id().to_string(),
            origin_id: common.origin_id().to_string(),
            app_dir: common.app_dir().to_string(),
            granted_permissions: common.granted_permissions().clone(),
            storage: common.storage().clone(),
            active_snapshot: common.active_snapshot().clone(),
            source: app.source().clone(),
        }
    }
}

impl From<UserSageApp> for PersistedUserSageApp {
    fn from(app: UserSageApp) -> Self {
        Self::from(&app)
    }
}

impl TryFrom<PersistedUserSageApp> for UserSageApp {
    type Error = anyhow::Error;

    fn try_from(persisted: PersistedUserSageApp) -> Result<Self, Self::Error> {
        let common = SageAppCommon::new(
            persisted.id,
            persisted.origin_id,
            persisted.app_dir,
            persisted.granted_permissions,
            persisted.storage,
            persisted.active_snapshot,
        )?;

        Ok(UserSageApp::new_installed(common, persisted.source))
    }
}
