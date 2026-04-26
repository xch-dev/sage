use anyhow::{anyhow, Result};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::CapabilityFlags;
use crate::types::SageAppFlags;

pub fn get_app_flags(
    granted: &[UserBridgeCapability],
    previous_flags: Option<&SageAppFlags>,
) -> Result<SageAppFlags> {
    let granted_capability_flags = CapabilityFlags::from_capabilities(granted);

    let previous_storage_may_contain_secrets = previous_flags
        .map(|flags| flags.storage_may_contain_secrets)
        .unwrap_or(false);

    let has_secret_access = granted_capability_flags.accesses_sensitive_secret;
    let has_external_access = granted_capability_flags.externally_observable;
    let storage_may_contain_secrets = previous_storage_may_contain_secrets;

    if has_external_access && has_secret_access {
        return Err(anyhow!(
            "cannot grant externally observable permissions together with sensitive secret access permissions"
        ));
    }

    if has_external_access && storage_may_contain_secrets {
        return Err(anyhow!("STORAGE_TAINTED"));
    }

    Ok(SageAppFlags {
        has_secret_access,
        has_external_access,
        storage_may_contain_secrets,
        isolated: has_secret_access || storage_may_contain_secrets,
    })
}

pub fn mark_storage_may_contain_secrets(
    flags: &SageAppFlags,
) -> SageAppFlags {
    SageAppFlags {
        has_secret_access: flags.has_secret_access,
        has_external_access: flags.has_external_access,
        storage_may_contain_secrets: true,
        isolated: true,
    }
}

pub fn clear_storage_may_contain_secrets(
    flags: &SageAppFlags,
) -> SageAppFlags {
    SageAppFlags {
        has_secret_access: flags.has_secret_access,
        has_external_access: flags.has_external_access,
        storage_may_contain_secrets: false,
        isolated: flags.has_secret_access,
    }
}
