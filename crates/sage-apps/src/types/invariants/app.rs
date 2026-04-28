use std::path::PathBuf;

use crate::types::app::SageAppFlags;
use crate::types::normalizers::normalized_non_empty_string;
use crate::types::{SageAppPackageManifest, SageAppSnapshot, SageGrantedPermissions};

pub struct NormalizedAppIdentity {
    pub id: String,
    pub origin_id: String,
    pub app_dir: String,
}

pub fn normalize_app_identity(
    id: String,
    origin_id: String,
    app_dir: String,
) -> anyhow::Result<NormalizedAppIdentity> {
    Ok(NormalizedAppIdentity {
        id: normalized_non_empty_string(id, "app id")?,
        origin_id: normalized_non_empty_string(origin_id, "app origin id")?,
        app_dir: normalized_non_empty_string(app_dir, "app directory")?,
    })
}

pub fn validate_app_flags_policy(flags: SageAppFlags) -> anyhow::Result<()> {
    if flags.has_external_access() && flags.has_secret_access() {
        anyhow::bail!(
            "app permissions cannot include both external access and sensitive secret access"
        );
    }

    if flags.has_external_access() && flags.storage_may_contain_secrets() {
        anyhow::bail!(
            "app permissions cannot include external access while storage may contain secrets"
        );
    }

    Ok(())
}

pub fn resolve_app_capability_flags(
    manifest: &SageAppPackageManifest,
    granted_permissions: &SageGrantedPermissions,
    previous_flags: Option<&SageAppFlags>,
) -> anyhow::Result<SageAppFlags> {
    let effective_capabilities = manifest
        .permissions()
        .capabilities()
        .resolve_effective_grants(granted_permissions.capabilities().copied())?;

    let capability_flags =
        SageAppFlags::from_granted_capabilities(&effective_capabilities, previous_flags)?;

    validate_app_flags_policy(capability_flags)?;

    Ok(capability_flags)
}

pub fn validate_snapshot_entry_and_icon_exist(
    snapshot: &SageAppSnapshot,
    entry_file: &str,
    icon_file: Option<&str>,
    label: &str,
) -> anyhow::Result<()> {
    let entry_file = snapshot.file_path(entry_file);

    if !entry_file.is_file() {
        anyhow::bail!(
            "{label} entry file does not exist: {}",
            entry_file.display()
        );
    }

    if let Some(icon_file) = icon_file {
        let icon_file: PathBuf = snapshot.file_path(icon_file);

        if !icon_file.is_file() {
            anyhow::bail!("{label} icon file does not exist: {}", icon_file.display());
        }
    }

    Ok(())
}
