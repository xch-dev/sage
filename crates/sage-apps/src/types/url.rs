use std::fmt;

use anyhow::{Context, Result as AnyResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use specta::Type;
use url::Url;

use crate::types::invariants::normalize_app_url;
use crate::utils::slugify_app_name;

pub const MANIFEST_FILE_NAME: &str = "sage-manifest.json";

#[derive(Debug, Clone, PartialEq, Eq, Type)]
pub struct SageAppUrl(Url);

#[derive(Debug, Clone, PartialEq, Eq, Type)]
pub struct SageAppManifestUrl(Url);

impl SageAppUrl {
    pub fn parse(value: impl AsRef<str>) -> AnyResult<Self> {
        let value = value.as_ref();
        let url = Url::parse(value).with_context(|| format!("invalid app url: {value}"))?;
        Ok(Self(normalize_app_url(url)?))
    }

    pub fn manifest_url(&self) -> SageAppManifestUrl {
        SageAppManifestUrl::derive_from_app_url(self)
    }

    pub fn slug(&self) -> String {
        let host = self.0.host_str().unwrap_or("app");

        slugify_app_name(host)
    }

    pub fn join(&self, relative_path: &str) -> AnyResult<String> {
        Ok(self.0.join(relative_path)?.to_string())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    pub fn into_string(self) -> String {
        self.0.to_string()
    }
}

impl SageAppManifestUrl {
    fn derive_from_app_url(app_url: &SageAppUrl) -> Self {
        let url = app_url
            .0
            .join(MANIFEST_FILE_NAME)
            .expect("valid app url + static manifest path must always join");

        Self(url)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl Serialize for SageAppUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SageAppUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for SageAppUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for SageAppManifestUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
