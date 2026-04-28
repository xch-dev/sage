use serde::{Deserialize, Serialize};
use specta::Type;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::CapabilityFlags;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SageAppFlags {
    pub(super) has_secret_access: bool,
    pub(super) has_external_access: bool,
    pub(super) storage_may_contain_secrets: bool,
    pub(super) isolated: bool,
}

impl SageAppFlags {
    pub fn from_granted_capabilities(
        granted: &[UserBridgeCapability],
        previous_flags: Option<&Self>,
    ) -> anyhow::Result<Self> {
        let granted_capability_flags = CapabilityFlags::from_capabilities(granted);

        Self::new(
            granted_capability_flags.accesses_sensitive_secret(),
            granted_capability_flags.externally_observable(),
            previous_flags.is_some_and(|f| f.storage_may_contain_secrets()),
        )
    }

    pub fn new(
        has_secret_access: bool,
        has_external_access: bool,
        storage_may_contain_secrets: bool,
    ) -> anyhow::Result<Self> {
        if has_external_access && has_secret_access {
            anyhow::bail!(
                "cannot grant externally observable permissions together with sensitive secret access permissions"
            );
        }

        if has_external_access && storage_may_contain_secrets {
            anyhow::bail!("STORAGE_TAINTED");
        }

        Ok(Self {
            has_secret_access,
            has_external_access,
            storage_may_contain_secrets,
            isolated: has_secret_access || storage_may_contain_secrets,
        })
    }

    pub(super) fn mark_storage_may_contain_secrets(&mut self) {
        self.storage_may_contain_secrets = true;
        self.isolated = true;
    }

    pub fn has_secret_access(self) -> bool {
        self.has_secret_access
    }

    pub fn has_external_access(self) -> bool {
        self.has_external_access
    }

    pub fn storage_may_contain_secrets(self) -> bool {
        self.storage_may_contain_secrets
    }

    pub fn isolated(self) -> bool {
        self.isolated
    }
}

#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::types::app::flags::SageAppFlags;

    #[test]
    fn from_granted_capabilities_sets_expected_flags_for_shared_send_capability() {
        let flags = SageAppFlags::from_granted_capabilities(
            &[UserBridgeCapability::WalletSendXch],
            None,
        )
            .unwrap();

        assert!(flags.has_external_access());
        assert!(!flags.has_secret_access());
        assert!(!flags.storage_may_contain_secrets());
        assert!(!flags.isolated());
    }

    #[test]
    fn from_granted_capabilities_rejects_external_access_when_storage_is_tainted() {
        let previous = SageAppFlags::new(false, false, true).unwrap();

        let err = SageAppFlags::from_granted_capabilities(
            &[UserBridgeCapability::WalletSendXch],
            Some(&previous),
        )
            .unwrap_err();

        assert_eq!(err.to_string(), "STORAGE_TAINTED");
    }

    #[test]
    fn new_rejects_external_access_with_secret_access() {
        let err = SageAppFlags::new(true, true, false).unwrap_err();

        assert!(
            err.to_string()
                .contains("externally observable permissions together with sensitive secret access")
        );
    }

    #[test]
    fn new_isolates_secret_access() {
        let flags = SageAppFlags::new(true, false, false).unwrap();

        assert!(flags.has_secret_access());
        assert!(!flags.has_external_access());
        assert!(!flags.storage_may_contain_secrets());
        assert!(flags.isolated());
    }

    #[test]
    fn new_isolates_tainted_storage() {
        let flags = SageAppFlags::new(false, false, true).unwrap();

        assert!(!flags.has_secret_access());
        assert!(!flags.has_external_access());
        assert!(flags.storage_may_contain_secrets());
        assert!(flags.isolated());
    }
}
