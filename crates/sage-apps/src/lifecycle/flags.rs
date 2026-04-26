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

#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::flags::{clear_storage_may_contain_secrets, get_app_flags, mark_storage_may_contain_secrets};
    use crate::types::SageAppFlags;

    #[test]
    fn mark_storage_may_contain_secrets_sets_taint_and_isolation() {
        let flags = SageAppFlags {
            has_secret_access: true,
            has_external_access: false,
            storage_may_contain_secrets: false,
            isolated: true,
        };

        let updated = mark_storage_may_contain_secrets(&flags);
        assert!(updated.storage_may_contain_secrets);
        assert!(updated.isolated);
        assert!(updated.has_secret_access);
        assert!(!updated.has_external_access);
    }
    
    #[test]
    fn clear_storage_may_contain_secrets_preserves_secret_access_isolation_only() {
        let flags = SageAppFlags {
            has_secret_access: true,
            has_external_access: false,
            storage_may_contain_secrets: true,
            isolated: true,
        };

        let updated = clear_storage_may_contain_secrets(&flags);
        assert!(!updated.storage_may_contain_secrets);
        assert!(updated.isolated);
        assert!(updated.has_secret_access);
    }

    #[test]
    fn resolve_capability_flags_sets_expected_flags_for_shared_send_capability() {
        let flags = get_app_flags(&vec![UserBridgeCapability::WalletSendXch], None)
            .expect("expected capability flags to resolve");

        assert!(flags.has_external_access);
        assert!(!flags.has_secret_access);
        assert!(!flags.storage_may_contain_secrets);
        assert!(!flags.isolated);
    }

    #[test]
    fn resolve_capability_flags_rejects_external_access_when_storage_is_tainted() {
        let previous = SageAppFlags {
            has_secret_access: false,
            has_external_access: false,
            storage_may_contain_secrets: true,
            isolated: true,
        };

        let err = get_app_flags(
            &vec![UserBridgeCapability::WalletSendXch],
            Some(&previous),
        )
            .expect_err("expected tainted storage to block externally observable capability");

        assert_eq!(err.to_string(), "STORAGE_TAINTED");
    }
}
