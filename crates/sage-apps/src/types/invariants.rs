use std::path::PathBuf;

use anyhow::anyhow;

use std::collections::BTreeSet;

use super::app::SageAppFlags;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::capabilities::{CapabilityFlags, get_user_capability_definition};
use crate::lifecycle::{
    MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES, validate_manifest_file_path, validate_sha256_hex,
};
use crate::types::SageAppSnapshot;
use crate::types::manifest::SageAppManifestFile;
use crate::types::network::SageNetworkWhitelistEntry;
use crate::types::normalizers::{normalized_non_empty_string, normalized_optional_string};
use crate::types::permissions::SageRequestedCapabilities;

pub(super) struct NormalizedAppIdentity {
    pub id: String,
    pub origin_id: String,
    pub app_dir: String,
}

pub(super) fn normalize_app_identity(
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

pub(super) fn normalize_optional_manifest_path(
    path: Option<String>,
    label: &str,
) -> anyhow::Result<Option<String>> {
    let path = normalized_optional_string(path);

    if let Some(path) = &path {
        validate_manifest_file_path(path).map_err(|err| anyhow!("{label} is invalid: {err}"))?;
    }

    Ok(path)
}

pub(super) fn validate_manifest_files(files: &[SageAppManifestFile]) -> anyhow::Result<u64> {
    if files.is_empty() {
        anyhow::bail!("manifest files cannot be empty");
    }

    if files.len() > MAX_APP_FILE_COUNT {
        anyhow::bail!(
            "manifest file count {} exceeds limit {}",
            files.len(),
            MAX_APP_FILE_COUNT
        );
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut total: u64 = 0;

    for file in files {
        validate_manifest_file_path(file.path())?;
        validate_sha256_hex(file.sha256())?;

        if !seen.insert(file.path().to_string()) {
            anyhow::bail!("duplicate manifest file path: {}", file.path());
        }

        total = total
            .checked_add(file.size())
            .ok_or_else(|| anyhow!("manifest total size overflow"))?;
    }

    if total > MAX_APP_TOTAL_SIZE_BYTES {
        anyhow::bail!("manifest total size {total} exceeds limit {MAX_APP_TOTAL_SIZE_BYTES}");
    }

    Ok(total)
}

pub(super) fn validate_declared_manifest_asset_exists(
    path: Option<&str>,
    files: &[SageAppManifestFile],
    label: &str,
) -> anyhow::Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    if !files.iter().any(|file| file.path() == path) {
        anyhow::bail!("manifest {label} file is not listed in files: {path}");
    }

    Ok(())
}

pub(super) fn validate_app_flags_policy(flags: SageAppFlags) -> anyhow::Result<()> {
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

pub(super) fn validate_snapshot_entry_and_icon_exist(
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

pub(super) fn validate_permissions_policy(
    capabilities: impl IntoIterator<Item = UserBridgeCapability>,
    network: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    context: &str,
) -> anyhow::Result<()> {
    let capability_flags = capabilities
        .into_iter()
        .fold(CapabilityFlags::EMPTY, |flags, cap| {
            flags.union(get_user_capability_definition(cap).flags())
        });

    let has_secret_access = capability_flags.accesses_sensitive_secret();
    let has_external_access =
        capability_flags.externally_observable() || network.into_iter().next().is_some();

    if has_secret_access && has_external_access {
        anyhow::bail!("{context} cannot include both external access and sensitive secret access");
    }

    Ok(())
}

pub(super) fn validate_requested_capabilities_are_requestable(
    capabilities: &SageRequestedCapabilities,
) -> anyhow::Result<()> {
    for capability in capabilities.required().chain(capabilities.optional()) {
        let definition = get_user_capability_definition(*capability);

        if !definition.flags().requestable_by_app() {
            anyhow::bail!(
                "capability is not requestable by app manifest: {}",
                capability.key()
            );
        }
    }

    Ok(())
}

pub(super) fn build_user_grantable_capability_set(
    requested: &SageRequestedCapabilities,
    capabilities: impl IntoIterator<Item = UserBridgeCapability>,
) -> anyhow::Result<BTreeSet<UserBridgeCapability>> {
    let capabilities = capabilities.into_iter().collect::<BTreeSet<_>>();

    validate_user_granted_capabilities(requested, &capabilities)?;
    validate_required_user_grantable_capabilities_present(requested, &capabilities)?;

    Ok(capabilities)
}

pub(super) fn validate_user_granted_capabilities(
    requested: &SageRequestedCapabilities,
    user_granted: &BTreeSet<UserBridgeCapability>,
) -> anyhow::Result<()> {
    for capability in user_granted {
        if !requested.is_allowed(*capability) {
            anyhow::bail!(
                "granted capability not requested in manifest: {}",
                capability.key()
            );
        }

        let definition = get_user_capability_definition(*capability);

        if !definition.flags().user_grantable() {
            anyhow::bail!(
                "granted capability is not user grantable: {}",
                capability.key()
            );
        }
    }

    Ok(())
}

pub(super) fn validate_required_user_grantable_capabilities_present(
    requested: &SageRequestedCapabilities,
    user_granted: &BTreeSet<UserBridgeCapability>,
) -> anyhow::Result<()> {
    for capability in requested.required() {
        let definition = get_user_capability_definition(*capability);

        if definition.flags().user_grantable() && !user_granted.contains(capability) {
            anyhow::bail!("missing required capability: {}", capability.key());
        }
    }

    Ok(())
}

pub(super) fn split_required_optional_set<T: Ord>(
    required: impl IntoIterator<Item = T>,
    optional: impl IntoIterator<Item = T>,
) -> (BTreeSet<T>, BTreeSet<T>) {
    let required = required.into_iter().collect::<BTreeSet<_>>();

    let optional = optional
        .into_iter()
        .filter(|item| !required.contains(item))
        .collect::<BTreeSet<_>>();

    (required, optional)
}
