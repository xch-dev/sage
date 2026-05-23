use crate::capabilities::get_user_capability_definition;
use crate::capabilities::list::UserBridgeCapability;
use crate::types::{
    SageAppPackageManifest, SageAppUrl, SageGrantedPermissions, SageRequestedPermissions,
    UserSageAppPendingUpdate,
};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppPendingUpdateView {
    app_url: SageAppUrl,
    manifest_hash: String,
    manifest: SageAppPackageManifest,
    decision: UserSageAppPendingUpdateDecisionView,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppPendingUpdateDecisionReviewView {
    required_user_grantable_capabilities: Vec<UserBridgeCapability>,
    required_network_whitelist: Vec<crate::types::SageNetworkWhitelistEntry>,
    required_network_whitelist_by_network:
        std::collections::BTreeMap<String, Vec<crate::types::SageNetworkWhitelistEntry>>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UserSageAppPendingUpdateDecisionView {
    Apply,
    Review(UserSageAppPendingUpdateDecisionReviewView),
}

impl UserSageAppPendingUpdateView {
    pub fn from_pending_update(
        value: &UserSageAppPendingUpdate,
        active_grants: &SageGrantedPermissions,
    ) -> Self {
        Self {
            app_url: value.app_url().clone(),
            manifest_hash: value.manifest_hash().to_string(),
            manifest: value.manifest().clone(),
            decision: UserSageAppPendingUpdateDecisionView::from_pending_update(
                active_grants,
                value.manifest().permissions(),
            ),
        }
    }

    pub fn decision(&self) -> &UserSageAppPendingUpdateDecisionView {
        &self.decision
    }
}

impl UserSageAppPendingUpdateDecisionView {
    pub fn from_pending_update(
        active_grants: &SageGrantedPermissions,
        pending_permissions: &SageRequestedPermissions,
    ) -> Self {
        let required_user_grantable_capabilities = pending_permissions
            .capabilities()
            .required()
            .copied()
            .filter(|capability| {
                get_user_capability_definition(*capability)
                    .flags()
                    .user_grantable()
            })
            .filter(|capability| !active_grants.has_capability(*capability))
            .collect::<Vec<_>>();

        let required_network_whitelist = pending_permissions
            .network()
            .whitelist()
            .required()
            .filter(|entry| !active_grants.network().whitelist().contains(*entry))
            .cloned()
            .collect::<Vec<_>>();

        let required_network_whitelist_by_network = pending_permissions
            .network()
            .whitelist_by_network()
            .iter()
            .filter_map(|(network_id, requested_whitelist)| {
                let granted_entries = active_grants
                    .network()
                    .whitelist_by_network()
                    .get(network_id);

                let missing = requested_whitelist
                    .required()
                    .filter(|entry| {
                        !granted_entries.is_some_and(|entries| entries.contains(*entry))
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                if missing.is_empty() {
                    None
                } else {
                    Some((network_id.clone(), missing))
                }
            })
            .collect::<std::collections::BTreeMap<_, _>>();

        if required_user_grantable_capabilities.is_empty()
            && required_network_whitelist.is_empty()
            && required_network_whitelist_by_network.is_empty()
        {
            Self::Apply
        } else {
            Self::Review(UserSageAppPendingUpdateDecisionReviewView {
                required_user_grantable_capabilities,
                required_network_whitelist,
                required_network_whitelist_by_network,
            })
        }
    }

    pub fn is_review(&self) -> bool {
        matches!(self, Self::Review { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion,
        SageAppPackageManifestParts, SageNetworkWhitelistEntry, SageRequestedCapabilities,
        SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist,
    };
    use std::collections::BTreeMap;

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

    #[test]
    fn pending_update_decision_reviews_new_required_network_specific_entries() {
        let old_permissions = SageRequestedPermissions::empty();
        let active_grants =
            SageGrantedPermissions::new(&old_permissions, [], [], BTreeMap::new()).unwrap();
        let required_entry = entry("https", "mainnet-required.example.com");
        let pending_permissions = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [],
                [],
                [(
                    "mainnet".to_string(),
                    SageRequestedNetworkWhitelist::new([required_entry.clone()], []),
                )],
            )
            .unwrap(),
            SageRequestedCapabilities::empty(),
        )
        .unwrap();

        let pending = UserSageAppPendingUpdate::new(
            SageAppUrl::parse("https://example.com/app/manifest.json").unwrap(),
            "manifest-hash".to_string(),
            manifest(pending_permissions),
        );

        let view = UserSageAppPendingUpdateView::from_pending_update(&pending, &active_grants);

        let UserSageAppPendingUpdateDecisionView::Review(review) = view.decision() else {
            panic!("expected pending update to require review");
        };

        assert!(review.required_user_grantable_capabilities.is_empty());
        assert!(review.required_network_whitelist.is_empty());
        assert_eq!(
            review
                .required_network_whitelist_by_network
                .get("mainnet")
                .unwrap(),
            &vec![required_entry]
        );
    }
}
