use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

use crate::types::split_required_optional_set;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq, PartialOrd, Ord)]
pub struct SageNetworkWhitelistEntry {
    scheme: String,
    host: String,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
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

        if !Self::is_csp_safe_host(&host) {
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

    fn is_csp_safe_host(host: &str) -> bool {
        !host.is_empty()
            && host.chars().all(|c| {
                c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '*' | ':' | '[' | ']')
            })
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
        let (required, optional) = split_required_optional_set(required, optional);
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

#[cfg(test)]
mod tests {
    use super::SageNetworkWhitelistEntry;

    #[test]
    fn network_entry_accepts_csp_safe_hosts() {
        for host in [
            "example.com",
            "*.example.com",
            "localhost:4173",
            "127.0.0.1:4173",
            "[::1]:4173",
        ] {
            SageNetworkWhitelistEntry::new("https", host)
                .unwrap_or_else(|err| panic!("expected {host} to be accepted: {err}"));
        }
    }

    #[test]
    fn network_entry_rejects_csp_separators_and_controls() {
        for host in [
            "example.com/script.js",
            "example.com?x=1",
            "example.com#frag",
            "example.com; script-src 'unsafe-inline'",
            "example.com,https://evil.example",
            "example.com\tfoo",
            "example.com\nfoo",
            "example.com'foo",
            "example.com\"foo",
            "example.com`foo",
        ] {
            assert!(
                SageNetworkWhitelistEntry::new("https", host).is_err(),
                "expected {host:?} to be rejected"
            );
        }
    }
}
