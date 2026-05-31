use std::collections::BTreeSet;

use crate::SharedSageApp;

fn csp_source_list(items: &[String]) -> String {
    items.join(" ")
}

pub fn build_app_csp(app: &SharedSageApp, network_id: &str) -> String {
    let mut connect_sources = BTreeSet::from(["'self'".to_string()]);

    app.with(|app| {
        for entry in app
            .granted_permissions()
            .network()
            .effective_whitelist_for_network(network_id)
        {
            connect_sources.insert(entry.as_permission_string());
        }
    });

    let child_src = csp_source_list(&["'none'".to_string()]);
    let connect_src = csp_source_list(&connect_sources.into_iter().collect::<Vec<_>>());
    let default_src = csp_source_list(&["'self'".to_string()]);
    let font_src = csp_source_list(&["'self'".to_string(), "data:".to_string()]);
    let frame_src = csp_source_list(&["'none'".to_string()]);
    let img_src = csp_source_list(&[
        "'self'".to_string(),
        "data:".to_string(),
        "blob:".to_string(),
    ]);
    let manifest_src = csp_source_list(&["'none'".to_string()]);
    let media_src = csp_source_list(&[
        "'self'".to_string(),
        "data:".to_string(),
        "blob:".to_string(),
    ]);
    let object_src = csp_source_list(&["'none'".to_string()]);
    let prefetch_src = csp_source_list(&["'none'".to_string()]);
    let script_src = csp_source_list(&["'self'".to_string(), "'wasm-unsafe-eval'".to_string()]);
    let style_src = csp_source_list(&["'self'".to_string(), "'unsafe-inline'".to_string()]);
    let worker_src = csp_source_list(&["'self'".to_string()]);
    let frame_ancestors = csp_source_list(&["'self'".to_string()]);
    let base_uri = csp_source_list(&["'none'".to_string()]);
    let form_action = csp_source_list(&["'none'".to_string()]);

    format!(
        "child-src {child_src}; \
         connect-src {connect_src}; \
         default-src {default_src}; \
         font-src {font_src}; \
         frame-src {frame_src}; \
         img-src {img_src}; \
         manifest-src {manifest_src}; \
         media-src {media_src}; \
         object-src {object_src}; \
         prefetch-src {prefetch_src}; \
         script-src {script_src}; \
         style-src {style_src}; \
         worker-src {worker_src}; \
         base-uri {base_uri}; \
         form-action {form_action}; \
        frame-ancestors {frame_ancestors};"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{SageAppCommon, SageAppIdentity, SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion, SageAppPackageManifest, SageAppPackageManifestParts, SageAppSnapshot, SageAppStorage, SageAppUrl, SageAppWalletScope, SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions, UserSageApp, UserSageAppSource};

    fn entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn manifest(permissions: SageRequestedPermissions) -> SageAppPackageManifest {
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version: SageAppManifestVersion(0),
            name: "test app".to_string(),
            icon: None,
            sage_version: SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
            version: "1.0.0".to_string(),
            permissions,
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 1).unwrap()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    fn app_with_network_grants() -> (SharedSageApp, TempDir) {
        let shared = entry("https", "shared.example.com");
        let mainnet = entry("https", "mainnet.example.com");
        let testnet = entry("https", "testnet.example.com");

        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [],
                [shared.clone()],
                [
                    (
                        "mainnet".to_string(),
                        SageRequestedNetworkWhitelist::new([], [mainnet.clone()]),
                    ),
                    (
                        "testnet11".to_string(),
                        SageRequestedNetworkWhitelist::new([], [testnet.clone()]),
                    ),
                ],
            )
            .unwrap(),
            SageRequestedCapabilities::empty(),
        )
        .unwrap();

        let granted = SageGrantedPermissions::new(
            &requested,
            [],
            [shared],
            BTreeMap::from([
                ("mainnet".to_string(), BTreeSet::from([mainnet])),
                ("testnet11".to_string(), BTreeSet::from([testnet])),
            ]),
        )
        .unwrap();

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "x").unwrap();
        let snapshot =
            SageAppSnapshot::new("hash", dir.path().to_string_lossy(), manifest(requested))
                .unwrap();
        let common = SageAppCommon::new(
            SageAppIdentity::new("app-id", "origin-id", dir.path().to_string_lossy()).unwrap(),
            granted,
            SageAppStorage::Unmanaged,
            snapshot,
            SageAppWalletScope::AllWallets,
        )
        .unwrap();
        let app = UserSageApp::new_installed(
            common,
            UserSageAppSource::Url {
                app_url: SageAppUrl::parse("https://example.com/app/").unwrap(),
            },
        );

        (SharedSageApp::new(app.into_sage_app()), dir)
    }

    #[test]
    fn csp_connect_src_scopes_network_specific_whitelist_to_active_network() {
        let (app, _dir) = app_with_network_grants();

        let mainnet_csp = build_app_csp(&app, "mainnet");
        assert!(mainnet_csp.contains("https://shared.example.com"));
        assert!(mainnet_csp.contains("https://mainnet.example.com"));
        assert!(!mainnet_csp.contains("https://testnet.example.com"));

        let testnet_csp = build_app_csp(&app, "testnet11");
        assert!(testnet_csp.contains("https://shared.example.com"));
        assert!(testnet_csp.contains("https://testnet.example.com"));
        assert!(!testnet_csp.contains("https://mainnet.example.com"));
    }
}
