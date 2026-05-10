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
