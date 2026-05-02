use crate::capabilities::list::{SystemBridgeCapability, UserBridgeCapability};
use crate::capabilities::definitions::get_user_capability_definition;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct CapabilityFlags {
    externally_observable: bool,
    accesses_sensitive_secret: bool,
    requestable_by_app: bool,
    user_grantable: bool,
    shared_with_app: bool,
}

impl CapabilityFlags {
    pub const EMPTY: Self = Self {
        externally_observable: false,
        accesses_sensitive_secret: false,
        requestable_by_app: false,
        user_grantable: false,
        shared_with_app: false,
    };

    #[allow(clippy::fn_params_excessive_bools)]
    pub const fn new(
        externally_observable: bool,
        accesses_sensitive_secret: bool,
        requestable_by_app: bool,
        user_grantable: bool,
        shared_with_app: bool,
    ) -> Self {
        Self {
            externally_observable,
            accesses_sensitive_secret,
            requestable_by_app,
            user_grantable,
            shared_with_app,
        }
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            externally_observable: self.externally_observable || other.externally_observable,
            accesses_sensitive_secret: self.accesses_sensitive_secret
                || other.accesses_sensitive_secret,
            requestable_by_app: self.requestable_by_app || other.requestable_by_app,
            user_grantable: self.user_grantable || other.user_grantable,
            shared_with_app: self.shared_with_app || other.shared_with_app,
        }
    }

    pub fn from_capabilities(capabilities: &[UserBridgeCapability]) -> Self {
        capabilities.iter().fold(Self::EMPTY, |flags, capability| {
            let def = get_user_capability_definition(*capability);
            flags.union(def.flags)
        })
    }

    pub fn externally_observable(self) -> bool {
        self.externally_observable
    }
    pub fn accesses_sensitive_secret(self) -> bool {
        self.accesses_sensitive_secret
    }
    pub fn requestable_by_app(self) -> bool {
        self.requestable_by_app
    }
    pub fn user_grantable(self) -> bool {
        self.user_grantable
    }
    pub fn shared_with_app(self) -> bool {
        self.shared_with_app
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CapabilityDefinition<C> {
    capability: C,
    label: &'static str,
    description: &'static str,
    flags: CapabilityFlags,
}

impl<C: Copy> CapabilityDefinition<C> {
    pub const fn new(
        capability: C,
        label: &'static str,
        description: &'static str,
        flags: CapabilityFlags,
    ) -> Self {
        Self {
            capability,
            label,
            description,
            flags,
        }
    }

    pub fn capability(&self) -> C {
        self.capability
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn flags(&self) -> CapabilityFlags {
        self.flags
    }
}

pub type UserCapabilityDefinition = CapabilityDefinition<UserBridgeCapability>;
pub type SystemCapabilityDefinition = CapabilityDefinition<SystemBridgeCapability>;
