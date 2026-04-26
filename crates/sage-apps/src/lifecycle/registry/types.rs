use std::convert::TryFrom;
use serde::{Deserialize, Serialize};
use crate::lifecycle::parse_network_permission_target;
use crate::lifecycle::registry::format_network_target;
use crate::types::{InstalledSageAppStorage, SageAppAuthor, SageAppFlags, SageAppCommon, SageAppDonation, SageAppManifestFile, SageAppPackageManifest, SageAppSnapshot, SageGrantedPermissions, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions, UserSageApp, UserSageAppPendingUpdate, UserSageAppSource};

#[derive(Debug, Serialize, Deserialize)]
struct PersistedStringListBucket {
    required: Vec<String>,
    optional: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRequestedNetworkPermissions {
    whitelist: PersistedStringListBucket,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRequestedPermissions {
    network: PersistedRequestedNetworkPermissions,
    capabilities: SageRequestedCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSageAppPackageManifest {
    name: String,
    version: String,
    permissions: PersistedRequestedPermissions,
    files: Vec<SageAppManifestFile>,
    entry: Option<String>,
    icon: Option<String>,
    author: Option<SageAppAuthor>,
    donation: Option<SageAppDonation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSageAppSnapshot {
    manifest_hash: String,
    snapshot_dir: String,
    total_bytes: u64,
    manifest: PersistedSageAppPackageManifest,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedUserSageAppPendingUpdate {
    app_url: String,
    manifest_url: String,
    manifest_hash: String,
    manifest: PersistedSageAppPackageManifest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum PersistedUserSageAppSource {
    Zip,
    Url {
        app_url: String,
        manifest_url: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedUserSageApp {
    id: String,
    origin_id: String,
    name: String,
    version: String,
    app_dir: String,
    entry_file: String,
    icon_file: String,
    requested_permissions: PersistedRequestedPermissions,
    granted_permissions: SageGrantedPermissions,
    capability_flags: SageAppFlags,
    storage: InstalledSageAppStorage,
    active_snapshot: PersistedSageAppSnapshot,
    source: PersistedUserSageAppSource,
    pending_update: Option<PersistedUserSageAppPendingUpdate>,
}

impl From<&SageRequestedPermissions> for PersistedRequestedPermissions {
    fn from(value: &SageRequestedPermissions) -> Self {
        Self {
            network: PersistedRequestedNetworkPermissions {
                whitelist: PersistedStringListBucket {
                    required: value
                        .network
                        .whitelist
                        .required
                        .iter()
                        .map(format_network_target)
                        .collect(),
                    optional: value
                        .network
                        .whitelist
                        .optional
                        .iter()
                        .map(format_network_target)
                        .collect(),
                },
            },
            capabilities: value.capabilities.clone(),
        }
    }
}

impl TryFrom<PersistedRequestedPermissions> for SageRequestedPermissions {
    type Error = anyhow::Error;

    fn try_from(value: PersistedRequestedPermissions) -> Result<Self, Self::Error> {
        let required = value
            .network
            .whitelist
            .required
            .into_iter()
            .map(|entry| parse_network_permission_target(&entry))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| anyhow::anyhow!(
                "failed to parse persisted required network entry: {err}"
            ))?;

        let optional = value
            .network
            .whitelist
            .optional
            .into_iter()
            .map(|entry| parse_network_permission_target(&entry))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| anyhow::anyhow!(
                "failed to parse persisted optional network entry: {err}"
            ))?;

        Ok(SageRequestedPermissions {
            network: SageRequestedNetworkPermissions {
                whitelist: SageRequestedNetworkWhitelist { required, optional },
            },
            capabilities: value.capabilities,
        })
    }
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
            requested_permissions: (&app.common.requested_permissions).into(),
            granted_permissions: app.common.granted_permissions.clone(),
            capability_flags: app.common.capability_flags,
            storage: app.common.storage.clone(),
            active_snapshot: (&app.common.active_snapshot).into(),
            source: (&app.source).into(),
            pending_update: app.pending_update.as_ref().map(Into::into),
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
        Ok(UserSageApp {
            common: SageAppCommon {
                id: app.id,
                origin_id: app.origin_id,
                name: app.name,
                version: app.version,
                app_dir: app.app_dir,
                entry_file: app.entry_file,
                icon_file: app.icon_file,
                requested_permissions: app.requested_permissions.try_into()?,
                granted_permissions: app.granted_permissions,
                capability_flags: app.capability_flags,
                storage: app.storage,
                active_snapshot: app.active_snapshot.try_into()?,
            },
            source: app.source.into(),
            pending_update: app
                .pending_update
                .map(TryInto::try_into)
                .transpose()?,
        })
    }
}

impl TryFrom<PersistedSageAppPackageManifest> for SageAppPackageManifest {
    type Error = anyhow::Error;

    fn try_from(value: PersistedSageAppPackageManifest) -> Result<Self, Self::Error> {
        Ok(SageAppPackageManifest {
            name: value.name,
            version: value.version,
            permissions: value.permissions.try_into()?,
            files: value.files,
            entry: value.entry,
            icon: value.icon,
            author: value.author,
            donation: value.donation,
        })
    }
}

impl From<&SageAppPackageManifest> for PersistedSageAppPackageManifest {
    fn from(value: &SageAppPackageManifest) -> Self {
        PersistedSageAppPackageManifest {
            name: value.name.clone(),
            version: value.version.clone(),
            permissions: (&value.permissions).into(),
            files: value.files.clone(),
            entry: value.entry.clone(),
            icon: value.icon.clone(),
            author: value.author.clone(),
            donation: value.donation.clone(),
        }
    }
}

impl From<&UserSageAppPendingUpdate> for PersistedUserSageAppPendingUpdate {
    fn from(value: &UserSageAppPendingUpdate) -> Self {
        PersistedUserSageAppPendingUpdate {
            app_url: value.app_url.clone(),
            manifest_url: value.manifest_url.clone(),
            manifest_hash: value.manifest_hash.clone(),
            manifest: (&value.manifest).into(),
        }
    }
}

impl From<&UserSageAppSource> for PersistedUserSageAppSource {
    fn from(value: &UserSageAppSource) -> Self {
        match value {
            UserSageAppSource::Zip => PersistedUserSageAppSource::Zip,
            UserSageAppSource::Url {
                app_url,
                manifest_url,
            } => PersistedUserSageAppSource::Url {
                app_url: app_url.clone(),
                manifest_url: manifest_url.clone(),
            },
        }
    }
}

impl From<PersistedUserSageAppSource> for UserSageAppSource {
    fn from(value: PersistedUserSageAppSource) -> Self {
        match value {
            PersistedUserSageAppSource::Zip => UserSageAppSource::Zip,
            PersistedUserSageAppSource::Url {
                app_url,
                manifest_url,
            } => UserSageAppSource::Url {
                app_url,
                manifest_url,
            },
        }
    }
}

impl From<&SageAppSnapshot> for PersistedSageAppSnapshot {
    fn from(value: &SageAppSnapshot) -> Self {
        PersistedSageAppSnapshot {
            manifest_hash: value.manifest_hash.clone(),
            snapshot_dir: value.snapshot_dir.clone(),
            total_bytes: value.total_bytes,
            manifest: (&value.manifest).into(),
        }
    }
}

impl TryFrom<PersistedSageAppSnapshot> for SageAppSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: PersistedSageAppSnapshot) -> Result<Self, Self::Error> {
        Ok(SageAppSnapshot {
            manifest_hash: value.manifest_hash,
            snapshot_dir: value.snapshot_dir,
            total_bytes: value.total_bytes,
            manifest: value.manifest.try_into()?,
        })
    }
}

impl TryFrom<PersistedUserSageAppPendingUpdate> for UserSageAppPendingUpdate {
    type Error = anyhow::Error;

    fn try_from(value: PersistedUserSageAppPendingUpdate) -> Result<Self, Self::Error> {
        Ok(UserSageAppPendingUpdate {
            app_url: value.app_url,
            manifest_url: value.manifest_url,
            manifest_hash: value.manifest_hash,
            manifest: value.manifest.try_into()?,
        })
    }
}
