use anyhow::{anyhow, Result};
use crate::bridge::capabilities::{SystemBridgeCapability, UserBridgeCapability};
use crate::permissions::capabilities::definitions::get_user_capability_definition;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CapabilityFlags {
    pub externally_observable: bool,
    pub accesses_sensitive_secret: bool,
    pub requestable_by_app: bool,
    pub user_grantable: bool,
    pub shared_with_app: bool,
}

impl CapabilityFlags {
    pub fn union(self, other: Self) -> Self {
        Self {
            externally_observable: self.externally_observable || other.externally_observable,
            accesses_sensitive_secret: self.accesses_sensitive_secret || other.accesses_sensitive_secret,
            requestable_by_app: self.requestable_by_app || other.requestable_by_app,
            user_grantable: self.user_grantable || other.user_grantable,
            shared_with_app: self.shared_with_app || other.shared_with_app,
        }
    }

    pub fn from_capabilities(
        capabilities: &[UserBridgeCapability],
    ) -> Result<Self> {
        let (first, rest) = capabilities
            .split_first()
            .ok_or_else(|| anyhow!("cannot derive capability flags from empty capability list"))?;

        let first_def = get_user_capability_definition(*first);

        rest.iter().try_fold(first_def.flags, |flags, capability| {
            let def = get_user_capability_definition(*capability);

            Ok(flags.union(def.flags))
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CapabilityDefinition<C> {
    pub capability: C,
    pub label: &'static str,
    pub description: &'static str,
    pub flags: CapabilityFlags,
}

pub type UserCapabilityDefinition = CapabilityDefinition<UserBridgeCapability>;
pub type SystemCapabilityDefinition = CapabilityDefinition<SystemBridgeCapability>;
