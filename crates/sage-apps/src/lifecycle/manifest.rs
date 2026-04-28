use anyhow::{Context, Result as AnyResult};

use crate::types::{MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppPackageManifest};
use crate::utils::bytes_sha256_hex;

pub async fn fetch_url_manifest(
    manifest_url: &SageAppManifestUrl,
) -> AnyResult<(SageAppPackageManifest, String)> {
    let manifest_url = manifest_url.as_str();

    let response = reqwest::get(manifest_url)
        .await
        .with_context(|| format!("failed to GET manifest url {manifest_url}"))?
        .error_for_status()
        .with_context(|| format!("manifest request failed for {manifest_url}"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read manifest response body from {manifest_url}"))?;

    let manifest_hash = bytes_sha256_hex(&bytes);

    let manifest_text = std::str::from_utf8(&bytes)
        .with_context(|| format!("manifest is not valid UTF-8: {manifest_url}"))?;

    let mut deserializer = serde_json::Deserializer::from_str(manifest_text);
    let manifest: SageAppPackageManifest = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|err| {
            anyhow::anyhow!(
                "failed to parse manifest json from {manifest_url} at {}: {}",
                err.path(),
                err.inner()
            )
        })?;

    Ok((manifest, manifest_hash))
}

pub fn read_manifest(package_root: &std::path::Path) -> AnyResult<SageAppPackageManifest> {
    let manifest_path = package_root.join(MANIFEST_FILE_NAME);
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut deserializer = serde_json::Deserializer::from_str(&manifest_text);
    let manifest: SageAppPackageManifest = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|err| {
            anyhow::anyhow!(
                "failed to parse manifest {} at {}: {}",
                manifest_path.display(),
                err.path(),
                err.inner()
            )
        })?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::types::{
        SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
        SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions,
        SageRequestedPermissions,
    };

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile::new(path.to_string(), "a".repeat(64), size).unwrap()
    }

    fn sample_file() -> SageAppManifestFile {
        sample_manifest_file("index.html", 123)
    }

    fn entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn requested_permissions() -> SageRequestedPermissions {
        SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [entry("https", "required.example.com")],
                [entry("wss", "optional.example.com")],
            ),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::WalletSendXch],
                [
                    UserBridgeCapability::PersistentStorage,
                    UserBridgeCapability::WalletGetSecretKey,
                ],
            ),
        )
        .unwrap()
    }

    fn sample_manifest_with(
        entry_file: Option<String>,
        icon_file: Option<String>,
    ) -> SageAppPackageManifest {
        let mut files = vec![sample_manifest_file("index.html", 1)];

        if let Some(entry_file) = &entry_file
            && entry_file != "index.html"
        {
            files.push(sample_manifest_file(entry_file, 1));
        }

        if let Some(icon_file) = &icon_file {
            files.push(sample_manifest_file(icon_file, 1));
        }

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: requested_permissions(),
            files,
            entry: entry_file,
            icon: icon_file,
            author: None,
            donation: None,
        })
        .unwrap()
    }

    #[test]
    fn manifest_rejects_blank_name() {
        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "   ".to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn manifest_rejects_blank_version() {
        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test".to_string(),
            version: "   ".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("version cannot be empty"));
    }

    #[test]
    fn manifest_total_size_is_computed() {
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("dist/index.html", 123)],
            entry: Some("dist/index.html".to_string()),
            icon: None,
            author: None,
            donation: None,
        })
        .unwrap();

        assert_eq!(manifest.total_bytes(), 123);
    }

    #[test]
    fn manifest_entry_file_uses_explicit_entry() {
        let manifest =
            sample_manifest_with(Some("entry.html".to_string()), Some("icon.svg".to_string()));

        assert_eq!(manifest.entry(), "entry.html");
    }

    #[test]
    fn manifest_entry_file_defaults_to_index_html() {
        let manifest = sample_manifest_with(None, Some("icon.svg".to_string()));
        assert_eq!(manifest.entry(), "index.html");
    }

    #[test]
    fn manifest_icon_file_uses_explicit_icon() {
        let manifest =
            sample_manifest_with(Some("entry.html".to_string()), Some("icon.svg".to_string()));

        assert_eq!(manifest.icon().unwrap(), "icon.svg");
    }

    #[test]
    fn manifest_icon_file_defaults_to_none() {
        let manifest = sample_manifest_with(Some("entry.html".to_string()), None);
        assert_eq!(manifest.icon(), None);
    }
}
