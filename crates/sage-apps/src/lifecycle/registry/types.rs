use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::types::{
    InstalledSageAppStorage, SageAppCommon, SageAppFlags, SageAppSnapshot, SageGrantedPermissions,
    SageRequestedPermissions, UserSageApp, UserSageAppPendingUpdate, UserSageAppSource,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedUserSageApp {
    id: String,
    origin_id: String,
    name: String,
    version: String,
    app_dir: String,
    entry_file: String,
    icon_file: String,
    requested_permissions: SageRequestedPermissions,
    granted_permissions: SageGrantedPermissions,
    capability_flags: SageAppFlags,
    storage: InstalledSageAppStorage,
    active_snapshot: SageAppSnapshot,
    source: UserSageAppSource,
    pending_update: Option<UserSageAppPendingUpdate>,
}

impl From<&UserSageApp> for PersistedUserSageApp {
    fn from(app: &UserSageApp) -> Self {
        Self {
            id: app.common.id.clone(),
            origin_id: app.common.origin_id.clone(),
            name: app.common.name.clone(),
            version: app.common.version.clone(),
            app_dir: app.common.app_dir.clone(),
            entry_file: app.common.entry_file.clone(),
            icon_file: app.common.icon_file.clone(),
            requested_permissions: app.common.requested_permissions.clone(),
            granted_permissions: app.common.granted_permissions.clone(),
            capability_flags: app.common.capability_flags,
            storage: app.common.storage.clone(),
            active_snapshot: app.common.active_snapshot.clone(),
            source: app.source.clone(),
            pending_update: app.pending_update.clone(),
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

    fn try_from(app: PersistedUserSageApp) -> Result<Self, Self::Error> {
        if app.requested_permissions != *app.active_snapshot.manifest.permissions() {
            anyhow::bail!("persisted requested permissions do not match active snapshot manifest");
        }

        let manifest = app.active_snapshot.manifest.clone();

        let common = SageAppCommon::new(
            app.id,
            app.origin_id,
            app.app_dir,
            &manifest,
            app.granted_permissions,
            app.storage,
            app.active_snapshot,
        )?;

        Ok(UserSageApp {
            common,
            source: app.source,
            pending_update: app.pending_update,
        })
    }
}
