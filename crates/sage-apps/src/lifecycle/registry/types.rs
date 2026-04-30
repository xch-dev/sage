use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{
    InstalledSageAppStorage, SageAppCommon, SageAppIdentity, SageAppSnapshot,
    SageGrantedPermissions, SharedSageApp, UserSageApp, UserSageAppSource,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedUserSageApp {
    identity: SageAppIdentity,
    granted_permissions: SageGrantedPermissions,
    storage: InstalledSageAppStorage,
    active_snapshot: SageAppSnapshot,
    source: UserSageAppSource,
}

impl TryFrom<&SharedSageApp> for PersistedUserSageApp {
    type Error = anyhow::Error;

    fn try_from(app: &SharedSageApp) -> anyhow::Result<Self> {
        app.try_with(|app| {
            let user_app = app
                .as_user()
                .ok_or_else(|| anyhow::anyhow!("not a user app"))?;

            let common = user_app.common();

            Ok(Self {
                identity: common.identity().clone(),
                granted_permissions: common.granted_permissions().clone(),
                storage: common.storage().clone(),
                active_snapshot: common.active_snapshot().clone(),
                source: user_app.source().clone(),
            })
        })
    }
}

impl TryFrom<&UserSageApp> for PersistedUserSageApp {
    type Error = anyhow::Error;

    fn try_from(user_app: &UserSageApp) -> anyhow::Result<Self> {
        let common = user_app.common();

        Ok(Self {
            identity: common.identity().clone(),
            granted_permissions: common.granted_permissions().clone(),
            storage: common.storage().clone(),
            active_snapshot: common.active_snapshot().clone(),
            source: user_app.source().clone(),
        })
    }
}

impl TryFrom<PersistedUserSageApp> for UserSageApp {
    type Error = anyhow::Error;

    fn try_from(persisted: PersistedUserSageApp) -> anyhow::Result<Self> {
        let common = SageAppCommon::new(
            persisted.identity,
            persisted.granted_permissions,
            persisted.storage,
            persisted.active_snapshot,
        )?;

        Ok(UserSageApp::new_installed(common, persisted.source))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedUserSageAppRaw {
    identity: SageAppIdentity,
    granted_permissions: SageGrantedPermissions,
    storage: InstalledSageAppStorage,
    active_snapshot: SageAppSnapshot,
    source: UserSageAppSource,
}

impl<'de> Deserialize<'de> for PersistedUserSageApp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = PersistedUserSageAppRaw::deserialize(deserializer)?;

        Ok(Self {
            identity: raw.identity,
            granted_permissions: raw.granted_permissions,
            storage: raw.storage,
            active_snapshot: raw.active_snapshot,
            source: raw.source,
        })
    }
}
