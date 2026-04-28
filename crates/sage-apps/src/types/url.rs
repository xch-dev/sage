use anyhow::{Context, Result as AnyResult};

pub const MANIFEST_FILE_NAME: &str = "sage-manifest.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SageAppUrl(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SageAppManifestUrl(String);

impl SageAppUrl {
    pub fn parse(value: impl AsRef<str>) -> AnyResult<Self> {
        let value = value.as_ref();

        reqwest::Url::parse(value).with_context(|| format!("invalid app url: {value}"))?;

        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl SageAppManifestUrl {
    pub fn derive_from_app_url(app_url: &SageAppUrl) -> AnyResult<Self> {
        let base = reqwest::Url::parse(app_url.as_str())
            .with_context(|| format!("invalid app url: {}", app_url.as_str()))?;

        let manifest_url = base.join(MANIFEST_FILE_NAME).with_context(|| {
            format!(
                "failed to derive manifest url from app url: {}",
                app_url.as_str()
            )
        })?;

        Ok(Self(manifest_url.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}
