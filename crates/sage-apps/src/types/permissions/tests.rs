use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::capabilities::UserBridgeCapability;
use crate::types::{SageNetworkWhitelistEntry, SageRequestedNetworkWhitelist};

fn network_entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
    SageNetworkWhitelistEntry::new(scheme, host).unwrap()
}

fn requested_permissions() -> SageRequestedPermissions {
    SageRequestedPermissions::new(
        SageRequestedNetworkPermissions::new(
            [network_entry("https", "required.example.com")],
            [network_entry("wss", "optional.example.com")],
            [(
                "mainnet".to_string(),
                SageRequestedNetworkWhitelist::new(
                    [network_entry("https", "mainnet-required.example.com")],
                    [network_entry("https", "mainnet-optional.example.com")],
                ),
            )],
        )
        .unwrap(),
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
        [UserBridgeCapability::WalletGetCoins],
        [],
        BTreeMap::new(),
    )
    .unwrap_err();

    assert!(err.to_string().contains("not requested in manifest"));
    assert!(
        err.to_string()
            .contains(UserBridgeCapability::WalletGetCoins.key())
    );
}

#[test]
fn with_capability_added_rejects_unrequested_capability() {
    let requested = requested_permissions();
    let granted = SageGrantedPermissions::new(&requested, [], [], BTreeMap::new()).unwrap();

    let err = granted
        .with_capability_added(&requested, UserBridgeCapability::WalletGetCoins)
        .unwrap_err();

    assert!(err.to_string().contains("not requested in manifest"));
    assert!(
        err.to_string()
            .contains(UserBridgeCapability::WalletGetCoins.key())
    );
}

#[test]
fn with_network_whitelist_entry_added_rejects_unrequested_entry() {
    let requested = requested_permissions();
    let granted = SageGrantedPermissions::new(&requested, [], [], BTreeMap::new()).unwrap();

    let entry = network_entry("https", "evil.example.com");

    let err = granted
        .with_network_whitelist_entry_added(&requested, entry)
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("granted shared network whitelist entry not requested in manifest")
    );
}

#[test]
fn with_network_whitelist_entry_for_network_added_rejects_unrequested_network() {
    let requested = requested_permissions();
    let granted = SageGrantedPermissions::new(&requested, [], [], BTreeMap::new()).unwrap();

    let err = granted
        .with_network_whitelist_entry_for_network_added(
            &requested,
            "testnet11",
            network_entry("https", "mainnet-optional.example.com"),
        )
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("granted network-specific whitelist entry for unrequested network")
    );
}

#[test]
fn with_network_whitelist_entry_for_network_added_rejects_unrequested_entry() {
    let requested = requested_permissions();
    let granted = SageGrantedPermissions::new(&requested, [], [], BTreeMap::new()).unwrap();

    let err = granted
        .with_network_whitelist_entry_for_network_added(
            &requested,
            "mainnet",
            network_entry("https", "evil.example.com"),
        )
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("granted network-specific whitelist entry not requested in manifest")
    );
}

#[test]
fn effective_whitelist_for_network_merges_shared_and_network_specific_entries() {
    let requested = requested_permissions();

    let granted = SageGrantedPermissions::new(
        &requested,
        [],
        [network_entry("https", "required.example.com")],
        BTreeMap::from([(
            "mainnet".to_string(),
            BTreeSet::from([network_entry("https", "mainnet-required.example.com")]),
        )]),
    )
    .unwrap();

    let effective = granted
        .network()
        .effective_whitelist_for_network("mainnet")
        .into_iter()
        .collect::<BTreeSet<_>>();

    let expected = BTreeSet::from([
        network_entry("https", "required.example.com"),
        network_entry("https", "mainnet-required.example.com"),
    ]);

    assert_eq!(effective, expected);
}

#[test]
fn requested_permissions_reject_secret_capability_with_required_network() {
    let err = SageRequestedPermissions::new(
        SageRequestedNetworkPermissions::new(
            [network_entry("https", "required.example.com")],
            [],
            [],
        )
        .unwrap(),
        SageRequestedCapabilities::new([UserBridgeCapability::WalletGetSecretKey], []),
    )
    .unwrap_err();

    assert!(
        err.to_string()
            .contains("cannot include both external access and sensitive secret access"),
        "unexpected error: {err}"
    );
}

#[test]
fn granted_permissions_reject_secret_capability_with_optional_network_grant() {
    let requested = SageRequestedPermissions::new(
        SageRequestedNetworkPermissions::new(
            [],
            [network_entry("https", "optional.example.com")],
            [],
        )
        .unwrap(),
        SageRequestedCapabilities::new([], [UserBridgeCapability::WalletGetSecretKey]),
    )
    .unwrap();

    let err = SageGrantedPermissions::new(
        &requested,
        [UserBridgeCapability::WalletGetSecretKey],
        [network_entry("https", "optional.example.com")],
        BTreeMap::new(),
    )
    .unwrap_err();

    assert!(
        err.to_string()
            .contains("cannot include both external access and sensitive secret access"),
        "unexpected error: {err}"
    );
}

#[test]
fn granted_permissions_reject_secret_capability_with_external_capability() {
    let requested = SageRequestedPermissions::new(
        SageRequestedNetworkPermissions::empty(),
        SageRequestedCapabilities::new(
            [],
            [
                UserBridgeCapability::WalletGetSecretKey,
                UserBridgeCapability::WalletSendXch,
            ],
        ),
    )
    .unwrap();

    let err = SageGrantedPermissions::new(
        &requested,
        [
            UserBridgeCapability::WalletGetSecretKey,
            UserBridgeCapability::WalletSendXch,
        ],
        [],
        BTreeMap::new(),
    )
    .unwrap_err();

    assert!(
        err.to_string()
            .contains("cannot include both external access and sensitive secret access"),
        "unexpected error: {err}"
    );
}
