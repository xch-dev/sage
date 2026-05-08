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
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UserSageAppPendingUpdateDecisionView {
    Apply,
    Review {
        required_user_grantable_capabilities: Vec<UserBridgeCapability>,
    },
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

        if required_user_grantable_capabilities.is_empty() {
            Self::Apply
        } else {
            Self::Review {
                required_user_grantable_capabilities,
            }
        }
    }

    pub fn is_review(&self) -> bool {
        matches!(self, Self::Review { .. })
    }
}
