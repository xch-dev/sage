use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

use crate::{
    SageNetworkWhitelistEntry, SageRequestedNetworkWhitelist, UserBridgeCapability,
    get_user_capability_definition, split_required_optional_set, validate_network_id,
    validate_permissions_policy, validate_requested_capabilities_are_requestable,
};

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageRequestedNetworkPermissions {
    whitelist: SageRequestedNetworkWhitelist,
    whitelist_by_network: BTreeMap<String, SageRequestedNetworkWhitelist>,
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

#[derive(Debug, Deserialize, Default)]
struct RawNetworkWhitelistBucket {
    #[serde(default)]
    required: Vec<SageNetworkWhitelistEntry>,

    #[serde(default)]
    optional: Vec<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawRequestedNetworkPermissions {
    #[serde(default)]
    whitelist: RawNetworkWhitelistBucket,

    #[serde(default)]
    whitelist_by_network: BTreeMap<String, RawNetworkWhitelistBucket>,
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
        let raw = RawRequestedPermissions::deserialize(deserializer)?;

        let whitelist_by_network =
            raw.network
                .whitelist_by_network
                .into_iter()
                .map(|(network_id, bucket)| {
                    (
                        network_id,
                        SageRequestedNetworkWhitelist::new(bucket.required, bucket.optional),
                    )
                });

        let network = SageRequestedNetworkPermissions::new(
            raw.network.whitelist.required,
            raw.network.whitelist.optional,
            whitelist_by_network,
        )
        .map_err(serde::de::Error::custom)?;

        SageRequestedPermissions::new(network, raw.capabilities.unwrap_or_default())
            .map_err(serde::de::Error::custom)
    }
}

impl SageRequestedPermissions {
    pub fn new(
        network: SageRequestedNetworkPermissions,
        capabilities: SageRequestedCapabilities,
    ) -> anyhow::Result<Self> {
        validate_requested_capabilities_are_requestable(&capabilities)?;

        let required_network = network
            .whitelist()
            .required()
            .cloned()
            .chain(
                network
                    .whitelist_by_network()
                    .values()
                    .flat_map(|whitelist| whitelist.required().cloned()),
            )
            .collect::<Vec<_>>();

        validate_permissions_policy(
            capabilities.required().copied(),
            required_network,
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
        whitelist_by_network: impl IntoIterator<Item = (String, SageRequestedNetworkWhitelist)>,
    ) -> anyhow::Result<Self> {
        let whitelist = SageRequestedNetworkWhitelist::new(required, optional);

        let mut by_network = BTreeMap::new();

        for (network_id, whitelist) in whitelist_by_network {
            let network_id = network_id.trim().to_string();

            validate_network_id(&network_id)?;

            by_network.insert(network_id, whitelist);
        }

        Ok(Self {
            whitelist,
            whitelist_by_network: by_network,
        })
    }

    pub fn empty() -> Self {
        Self::new([], [], []).expect("empty requested network permissions should be valid")
    }

    pub fn whitelist(&self) -> &SageRequestedNetworkWhitelist {
        &self.whitelist
    }

    pub fn whitelist_by_network(&self) -> &BTreeMap<String, SageRequestedNetworkWhitelist> {
        &self.whitelist_by_network
    }

    pub fn all_whitelist_entries(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.whitelist
            .required()
            .chain(self.whitelist.optional())
            .chain(
                self.whitelist_by_network
                    .values()
                    .flat_map(|whitelist| whitelist.required().chain(whitelist.optional())),
            )
    }
}
