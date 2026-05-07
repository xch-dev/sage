use crate::capabilities::list::{
    SharedCapabilitiesExt, SystemBridgeCapability, UserBridgeCapability,
};
use crate::capabilities::{get_user_capability_definition};
use crate::types::invariants::{
    build_user_grantable_capability_set, split_required_optional_set, validate_permissions_policy,
    validate_requested_capabilities_are_requestable,
};
use crate::types::network::{SageNetworkWhitelistEntry, SageRequestedNetworkWhitelist};
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct SageGrantedNetworkPermissions {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct SageGrantedPermissions {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Type)]
pub struct SageGrantedSystemPermissions {
    capabilities: Vec<SystemBridgeCapability>,
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

    pub fn whitelist(&self) -> &BTreeSet<SageNetworkWhitelistEntry> {
        &self.whitelist
    }

    pub fn whitelist_iter(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.whitelist.iter()
    }
}

impl SageGrantedPermissions {
    pub fn new(
        requested: &SageRequestedPermissions,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> anyhow::Result<Self> {
        Self::new_with_extra_granted_capabilities(
            requested,
            capabilities,
            std::iter::empty(),
            network_whitelist,
        )
    }

    pub(crate) fn new_with_extra_granted_capabilities(
        requested: &SageRequestedPermissions,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        extra_granted_capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> anyhow::Result<Self> {
        let mut capabilities =
            build_user_grantable_capability_set(&requested.capabilities, capabilities)?;

        for capability in extra_granted_capabilities {
            let definition = get_user_capability_definition(capability);

            if !definition.flags().user_grantable() {
                anyhow::bail!(
                    "extra granted capability is not user grantable: {}",
                    capability.key()
                );
            }

            if definition.flags().requestable_by_app() {
                anyhow::bail!(
                    "extra granted capability must not be app-manifest requestable: {}",
                    capability.key()
                );
            }

            capabilities.insert(capability);
        }

        let network = SageGrantedNetworkPermissions::new(
            &requested.network,
            network_whitelist,
        )?;

        let effective_capabilities = requested
            .capabilities
            .resolve_effective_grants(capabilities.iter().copied());

        validate_permissions_policy(
            effective_capabilities,
            network.whitelist_iter().cloned(),
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

    pub fn with_capability_added(
        &self,
        requested: &SageRequestedPermissions,
        capability: UserBridgeCapability,
    ) -> anyhow::Result<Self> {
        Self::new(
            requested,
            self.capabilities.iter().copied().chain([capability]),
            self.network.whitelist_iter().cloned(),
        )
    }

    pub fn with_network_whitelist_entry_added(
        &self,
        requested: &SageRequestedPermissions,
        entry: SageNetworkWhitelistEntry,
    ) -> anyhow::Result<Self> {
        Self::new(
            requested,
            self.capabilities.iter().copied(),
            self.network.whitelist_iter().cloned().chain([entry]),
        )
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.capabilities.iter()
    }
    pub fn has_capability(&self, capability: UserBridgeCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn capabilities_vec(&self) -> Vec<UserBridgeCapability> {
        self.capabilities.iter().copied().collect()
    }

    pub fn network(&self) -> &SageGrantedNetworkPermissions {
        &self.network
    }

    pub fn network_whitelist_vec(&self) -> Vec<SageNetworkWhitelistEntry> {
        self.network.whitelist_iter().cloned().collect()
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

    pub fn all(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.required().chain(self.optional())
    }

    pub fn contains(&self, capability: UserBridgeCapability) -> bool {
        self.is_allowed(capability)
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
        granted: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> Vec<UserBridgeCapability> {
        let mut effective = granted.into_iter().collect::<BTreeSet<_>>();

        for capability in self.required() {
            let definition = get_user_capability_definition(*capability);

            if !definition.flags().user_grantable() {
                effective.insert(*capability);
            }
        }

        effective.into_iter().collect()
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

impl<'de> Deserialize<'de> for SageGrantedPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawSageGrantedPermissions {
            #[serde(default)]
            capabilities: BTreeSet<UserBridgeCapability>,

            #[serde(default)]
            network: SageGrantedNetworkPermissions,
        }

        let raw = RawSageGrantedPermissions::deserialize(deserializer)?;

        validate_permissions_policy(
            raw.capabilities.iter().copied(),
            raw.network.whitelist_iter().cloned(),
            "granted permissions",
        )
            .map_err(serde::de::Error::custom)?;

        Ok(Self {
            capabilities: raw.capabilities,
            network: raw.network,
        })
    }
}

impl<'de> Deserialize<'de> for SageGrantedNetworkPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            #[serde(default)]
            whitelist: BTreeSet<SageNetworkWhitelistEntry>,
        }

        let raw = Raw::deserialize(deserializer)?;

        Ok(Self {
            whitelist: raw.whitelist,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network_entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn requested_permissions() -> SageRequestedPermissions {
        SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [network_entry("https", "required.example.com")],
                [network_entry("wss", "optional.example.com")],
            ),
            SageRequestedCapabilities::new(
                [],
                [
                    UserBridgeCapability::WalletSendXch,
                    UserBridgeCapability::StoragePersistentWebview,
                ],
            ),
        )
        .unwrap()
    }

    #[test]
    fn granted_permissions_reject_unrequested_capability() {
        let requested = requested_permissions();

        let err = SageGrantedPermissions::new(
            &requested,
            [UserBridgeCapability::WalletSendXchAutoSubmit],
            [],
        )
        .unwrap_err();

        assert!(err.to_string().contains("not requested in manifest"));
        assert!(
            err.to_string()
                .contains(UserBridgeCapability::WalletSendXchAutoSubmit.key())
        );
    }

    #[test]
    fn with_capability_added_rejects_unrequested_capability() {
        let requested = requested_permissions();
        let granted = SageGrantedPermissions::new(&requested, [], []).unwrap();

        let err = granted
            .with_capability_added(&requested, UserBridgeCapability::WalletSendXchAutoSubmit)
            .unwrap_err();

        assert!(err.to_string().contains("not requested in manifest"));
        assert!(
            err.to_string()
                .contains(UserBridgeCapability::WalletSendXchAutoSubmit.key())
        );
    }

    #[test]
    fn with_network_whitelist_entry_added_rejects_unrequested_entry() {
        let requested = requested_permissions();
        let granted = SageGrantedPermissions::new(&requested, [], []).unwrap();

        let entry = network_entry("https", "evil.example.com");

        let err = granted
            .with_network_whitelist_entry_added(&requested, entry)
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("granted network whitelist entry not requested in manifest")
        );
    }
}
