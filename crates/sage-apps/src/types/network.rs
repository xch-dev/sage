use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq, PartialOrd, Ord)]
pub struct SageNetworkWhitelistEntry {
    scheme: String,
    host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedNetworkWhitelist {
    required: BTreeSet<SageNetworkWhitelistEntry>,
    optional: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawSageNetworkWhitelistEntry {
    String(String),
    Object { scheme: String, host: String },
}

impl<'de> Deserialize<'de> for SageNetworkWhitelistEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match RawSageNetworkWhitelistEntry::deserialize(deserializer)? {
            RawSageNetworkWhitelistEntry::String(value) => {
                value.parse().map_err(serde::de::Error::custom)
            }
            RawSageNetworkWhitelistEntry::Object { scheme, host } => {
                Self::new(scheme, host).map_err(serde::de::Error::custom)
            }
        }
    }
}

impl SageNetworkWhitelistEntry {
    pub fn new(scheme: impl Into<String>, host: impl Into<String>) -> anyhow::Result<Self> {
        let scheme = scheme.into().trim().to_ascii_lowercase();
        let host = host.into().trim().to_ascii_lowercase();

        if !Self::is_allowed_scheme(&scheme) {
            anyhow::bail!("invalid scheme '{scheme}', only https and wss allowed");
        }

        if host.is_empty()
            || host.contains('/')
            || host.contains('?')
            || host.contains('#')
            || host.contains(' ')
        {
            anyhow::bail!("invalid host in network entry: {scheme}://{host}");
        }

        Ok(Self { scheme, host })
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn as_permission_string(&self) -> String {
        format!("{}://{}", self.scheme, self.host)
    }

    fn is_allowed_scheme(s: &str) -> bool {
        matches!(s, "https" | "wss")
    }

    #[cfg(test)]
    pub fn new_unchecked(scheme: &str, host: &str) -> Self {
        Self::new(scheme, host).unwrap()
    }
}

impl std::str::FromStr for SageNetworkWhitelistEntry {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        let (scheme, host) = value
            .split_once("://")
            .ok_or_else(|| anyhow::anyhow!("invalid network entry, missing scheme: {value}"))?;

        Self::new(scheme, host)
    }
}

impl SageRequestedNetworkWhitelist {
    pub fn new(
        required: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        optional: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        let required = required.into_iter().collect::<BTreeSet<_>>();

        let optional = optional
            .into_iter()
            .filter(|entry| !required.contains(entry))
            .collect::<BTreeSet<_>>();

        Self { required, optional }
    }

    pub fn empty() -> Self {
        Self::new([], [])
    }

    pub fn required(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.required.iter()
    }

    pub fn optional(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.optional.iter()
    }

    pub fn is_required(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.required.contains(entry)
    }

    pub fn is_optional(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.optional.contains(entry)
    }

    pub fn is_allowed(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.is_required(entry) || self.is_optional(entry)
    }
}
