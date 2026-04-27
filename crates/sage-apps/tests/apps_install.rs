mod common;

use common::sample_manifest_file;
use sage_apps::bridge::capabilities::UserBridgeCapability;
use sage_apps::lifecycle::install::url::normalize_app_url;
use sage_apps::lifecycle::manifest::{manifest_entry_file, manifest_icon_file};
use sage_apps::types::{
    SageAppPackageManifest, SageAppPackageManifestParts, SageNetworkWhitelistEntry,
    SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions,
};

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
            [UserBridgeCapability::PersistentStorage],
        ),
    )
    .unwrap()
}

fn sample_manifest_with(
    entry_file: Option<String>,
    icon_file: Option<String>,
) -> SageAppPackageManifest {
    SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: "Test App".to_string(),
        version: "1.0.0".to_string(),
        permissions: requested_permissions(),
        files: vec![sample_manifest_file("index.html", 1)],
        entry: entry_file,
        icon: icon_file,
        author: None,
        donation: None,
    })
    .unwrap()
}

fn sample_manifest() -> SageAppPackageManifest {
    sample_manifest_with(Some("entry.html".to_string()), Some("icon.svg".to_string()))
}

#[test]
fn normalize_app_url_keeps_https_and_adds_trailing_slash() {
    let out = normalize_app_url("https://example.com/app").unwrap();
    assert_eq!(out, "https://example.com/app/");
}

#[test]
fn normalize_app_url_strips_query_and_fragment() {
    let out = normalize_app_url("https://example.com/app?x=1#frag").unwrap();
    assert_eq!(out, "https://example.com/app/");
}

#[test]
fn normalize_app_url_allows_localhost_http() {
    let out = normalize_app_url("http://localhost:4173").unwrap();
    assert_eq!(out, "http://localhost:4173/");
}

#[test]
fn normalize_app_url_allows_loopback_http() {
    assert_eq!(
        normalize_app_url("http://127.0.0.1:4173").unwrap(),
        "http://127.0.0.1:4173/"
    );
}

#[test]
fn normalize_app_url_rejects_non_local_http() {
    let err = normalize_app_url("http://example.com/app")
        .unwrap_err()
        .to_string();
    assert!(err.contains("requires HTTPS"));
}

#[test]
fn normalize_app_url_rejects_unsupported_scheme() {
    let err = normalize_app_url("ftp://example.com/app")
        .unwrap_err()
        .to_string();
    assert!(err.contains("unsupported app URL scheme"));
}

#[test]
fn manifest_entry_file_uses_explicit_entry() {
    let manifest = sample_manifest();
    assert_eq!(manifest_entry_file(&manifest), "entry.html");
}

#[test]
fn manifest_entry_file_defaults_to_index_html() {
    let manifest = sample_manifest_with(None, Some("icon.svg".to_string()));
    assert_eq!(manifest_entry_file(&manifest), "index.html");
}

#[test]
fn manifest_icon_file_uses_explicit_icon() {
    let manifest = sample_manifest();
    assert_eq!(manifest_icon_file(&manifest), "icon.svg");
}

#[test]
fn manifest_icon_file_defaults_to_icon_png() {
    let manifest = sample_manifest_with(Some("entry.html".to_string()), None);
    assert_eq!(manifest_icon_file(&manifest), "icon.png");
}
