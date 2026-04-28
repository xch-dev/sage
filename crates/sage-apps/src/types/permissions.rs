use crate::bridge::capabilities::{
    SharedCapabilitiesExt, SystemBridgeCapability, UserBridgeCapability,
};
use crate::capabilities::{CapabilityDefinition, CapabilityFlags, get_user_capability_definition};
use crate::types::invariants::{
    build_user_grantable_capability_set, split_required_optional_set, validate_permissions_policy,
    validate_requested_capabilities_are_requestable,
};
use crate::types::network::{SageNetworkWhitelistEntry, SageRequestedNetworkWhitelist};
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedNetworkPermissions {
    whitelist: SageRequestedNetworkWhitelist,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedCapabilities {
    required: BTreeSet<UserBridgeCapability>,
    optional: BTreeSet<UserBridgeCapability>,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedPermissions {
    network: SageRequestedNetworkPermissions,
    capabilities: SageRequestedCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedNetworkPermissions {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedPermissions {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedSystemPermissions {
    capabilities: Vec<SystemBridgeCapability>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityFlagsView {
    externally_observable: bool,
    accesses_sensitive_secret: bool,
    requestable_by_app: bool,
    user_grantable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityDefinitionView {
    key: String,
    label: String,
    description: String,
    flags: SageAppCapabilityFlagsView,
}

#[derive(Debug, Deserialize, Default)]
struct RawNetworkWhitelistBucket {
    #[serde(default)]
    required: Vec<SageNetworkWhitelistEntry>,

    #[serde(default)]
    optional: Vec<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct RawRequestedNetworkPermissions {
    #[serde(default)]
    whitelist: RawNetworkWhitelistBucket,
}

#[derive(Debug, Deserialize, Default)]
struct RawRequestedPermissions {
    #[serde(default)]
    network: RawRequestedNetworkPermissions,

    #[serde(default)]
    capabilities: Option<SageRequestedCapabilities>,
}

#[derive(Debug, Deserialize, Default)]
struct RawRequestedCapabilities {
    #[serde(default)]
    required: Vec<UserBridgeCapability>,

    #[serde(default)]
    optional: Vec<UserBridgeCapability>,
}

impl<'de> Deserialize<'de> for SageRequestedCapabilities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawRequestedCapabilities::deserialize(deserializer)?;
        Ok(Self::new(raw.required, raw.optional))
    }
}

impl<'de> Deserialize<'de> for SageRequestedPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <RawRequestedPermissions as Deserialize>::deserialize(deserializer)?;

        let required_network = raw.network.whitelist.required;
        let optional_network = raw.network.whitelist.optional;

        SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(required_network, optional_network),
            raw.capabilities.unwrap_or_default(),
        )
        .map_err(serde::de::Error::custom)
    }
}

impl From<CapabilityFlags> for SageAppCapabilityFlagsView {
    fn from(flags: CapabilityFlags) -> Self {
        Self {
            externally_observable: flags.externally_observable(),
            accesses_sensitive_secret: flags.accesses_sensitive_secret(),
            requestable_by_app: flags.requestable_by_app(),
            user_grantable: flags.user_grantable(),
        }
    }
}

impl From<CapabilityDefinition<UserBridgeCapability>> for SageAppCapabilityDefinitionView {
    fn from(definition: CapabilityDefinition<UserBridgeCapability>) -> Self {
        Self {
            key: definition.capability().key().to_string(),
            label: definition.label().to_string(),
            description: definition.description().to_string(),
            flags: definition.flags().into(),
        }
    }
}

impl SageGrantedSystemPermissions {
    pub fn new(capabilities: impl IntoIterator<Item = SystemBridgeCapability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    pub fn capabilities(&self) -> &[SystemBridgeCapability] {
        &self.capabilities
    }
}

impl SageGrantedNetworkPermissions {
    pub fn new(
        requested: &SageRequestedNetworkPermissions,
        whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> anyhow::Result<Self> {
        let whitelist = whitelist.into_iter().collect::<BTreeSet<_>>();

        for entry in &whitelist {
            if !requested.whitelist.is_allowed(entry) {
                anyhow::bail!(
                    "granted network whitelist entry not requested in manifest: {}",
                    entry.as_permission_string()
                );
            }
        }

        Ok(Self { whitelist })
    }

    pub fn whitelist(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.whitelist.iter()
    }

    pub fn contains(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.whitelist.contains(entry)
    }

    pub fn into_vec(self) -> Vec<SageNetworkWhitelistEntry> {
        self.whitelist.into_iter().collect()
    }
}

impl SageGrantedPermissions {
    pub fn new(
        requested: &SageRequestedPermissions,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> anyhow::Result<Self> {
        let capabilities =
            build_user_grantable_capability_set(&requested.capabilities, capabilities)?;

        let network = SageGrantedNetworkPermissions::new(&requested.network, network_whitelist)?;

        let effective_capabilities = requested
            .capabilities
            .resolve_effective_grants(capabilities.iter().copied())?;

        validate_permissions_policy(
            effective_capabilities,
            network.whitelist().cloned(),
            "granted permissions",
        )?;

        Ok(Self {
            capabilities,
            network,
        })
    }

    pub fn from_requested_and_granted(
        requested: &SageRequestedPermissions,
        granted: SageGrantedPermissions,
    ) -> anyhow::Result<Self> {
        Self::new(requested, granted.capabilities, granted.network.whitelist)
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.capabilities.iter()
    }

    pub fn capabilities_vec(&self) -> Vec<UserBridgeCapability> {
        self.capabilities.iter().copied().collect()
    }

    pub fn network(&self) -> &SageGrantedNetworkPermissions {
        &self.network
    }

    pub fn shared_capabilities(&self) -> Vec<UserBridgeCapability> {
        self.capabilities().copied().shared()
    }

    pub fn for_builtin_requested(requested: &SageRequestedPermissions) -> anyhow::Result<Self> {
        Self::new(
            requested,
            requested.capabilities.user_grantable(),
            requested.network.whitelist.required().cloned(),
        )
    }

    #[cfg(test)]
    pub fn new_unchecked(
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
            network: SageGrantedNetworkPermissions {
                whitelist: network_whitelist.into_iter().collect(),
            },
        }
    }
}

impl SageRequestedPermissions {
    pub fn new(
        network: SageRequestedNetworkPermissions,
        capabilities: SageRequestedCapabilities,
    ) -> anyhow::Result<Self> {
        validate_requested_capabilities_are_requestable(&capabilities)?;

        validate_permissions_policy(
            capabilities.required().copied(),
            network.whitelist().required().cloned(),
            "required requested permissions",
        )?;

        Ok(Self {
            network,
            capabilities,
        })
    }

    pub fn empty() -> Self {
        Self {
            network: SageRequestedNetworkPermissions::empty(),
            capabilities: SageRequestedCapabilities::empty(),
        }
    }

    pub fn network(&self) -> &SageRequestedNetworkPermissions {
        &self.network
    }
    pub fn capabilities(&self) -> &SageRequestedCapabilities {
        &self.capabilities
    }
}

impl SageRequestedCapabilities {
    pub fn new(
        required: impl IntoIterator<Item = UserBridgeCapability>,
        optional: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> Self {
        let (required, optional) = split_required_optional_set(required, optional);
        Self { required, optional }
    }

    pub fn empty() -> Self {
        Self::new([], [])
    }

    pub fn required(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.required.iter()
    }

    pub fn optional(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.optional.iter()
    }

    pub fn is_required(&self, cap: UserBridgeCapability) -> bool {
        self.required.contains(&cap)
    }

    pub fn is_optional(&self, cap: UserBridgeCapability) -> bool {
        self.optional.contains(&cap)
    }

    pub fn is_allowed(&self, cap: UserBridgeCapability) -> bool {
        self.is_required(cap) || self.is_optional(cap)
    }

    pub fn user_grantable(&self) -> Vec<UserBridgeCapability> {
        self.required()
            .chain(self.optional())
            .copied()
            .filter(|cap| {
                get_user_capability_definition(*cap)
                    .flags()
                    .user_grantable()
            })
            .collect()
    }

    pub fn resolve_effective_grants(
        &self,
        user_granted: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> anyhow::Result<Vec<UserBridgeCapability>> {
        let user_granted = self.build_user_grants(user_granted)?;

        let mut effective = user_granted;

        for capability in self.required().chain(self.optional()) {
            let definition = get_user_capability_definition(*capability);

            if !definition.flags().user_grantable() {
                effective.insert(*capability);
            }
        }

        Ok(effective.into_iter().collect())
    }

    fn build_user_grants(
        &self,
        user_granted: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> anyhow::Result<BTreeSet<UserBridgeCapability>> {
        build_user_grantable_capability_set(self, user_granted)
    }
}

impl SageRequestedNetworkPermissions {
    pub fn new(
        required: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        optional: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        Self {
            whitelist: SageRequestedNetworkWhitelist::new(required, optional),
        }
    }

    pub fn empty() -> Self {
        Self::new([], [])
    }

    pub fn whitelist(&self) -> &SageRequestedNetworkWhitelist {
        &self.whitelist
    }
}
