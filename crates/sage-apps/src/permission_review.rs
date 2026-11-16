//! Read-only projection of installed app grants for the host's shared permissions view.
use serde::Serialize;
use specta::Type;
use tauri::{State, command};

use crate::{
    AppsHostState, ListedSageApp, Result, SageAppWalletScope, SageGrantedNetworkPermissionsView,
    SageRequestedCapabilities, UserBridgeCapability, get_user_capability_definition,
};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReviewedAppCapability {
    key: String,
    label: String,
    description: String,
    wallet: bool,
    automatic: bool,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppPermissionReview {
    id: String,
    name: String,
    wallet_scope: SageAppWalletScope,
    capabilities: Vec<ReviewedAppCapability>,
    network: SageGrantedNetworkPermissionsView,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppPermissionReviews {
    apps: Vec<AppPermissionReview>,
    unavailable_app_ids: Vec<String>,
}

fn review_capabilities(
    requested: &SageRequestedCapabilities,
    granted: impl IntoIterator<Item = UserBridgeCapability>,
) -> Vec<ReviewedAppCapability> {
    requested
        .resolve_effective_grants(granted)
        .into_iter()
        .map(|capability| {
            let definition = get_user_capability_definition(capability);
            ReviewedAppCapability {
                key: capability.key().into(),
                label: definition.label().into(),
                description: definition.description().into(),
                wallet: capability.wallet_capability().is_some(),
                automatic: !definition.flags().user_grantable(),
            }
        })
        .collect()
}

#[command]
#[specta::specta]
pub async fn apps_review_permissions(
    state: State<'_, AppsHostState>,
) -> Result<AppPermissionReviews> {
    let installed =
        state.db.list_installed_apps().await.map_err(|err| {
            std::io::Error::other(format!("failed to read app permissions: {err}"))
        })?;
    let mut reviews = AppPermissionReviews {
        apps: Vec::new(),
        unavailable_app_ids: Vec::new(),
    };
    for entry in installed {
        match entry {
            ListedSageApp::User(app) => {
                let common = app.common();
                reviews.apps.push(AppPermissionReview {
                    id: common.id().to_string(),
                    name: common.name().to_string(),
                    wallet_scope: common.wallet_scope().clone(),
                    capabilities: review_capabilities(
                        common.requested_permissions().capabilities(),
                        common.granted_permissions().capabilities().copied(),
                    ),
                    network: common.granted_permissions().network().into(),
                });
            }
            ListedSageApp::Corrupted(app) => reviews.unavailable_app_ids.push(app.id().into()),
            // Built-in system components are trusted host UI, not installed callers.
            ListedSageApp::System(_) => {}
        }
    }
    reviews
        .apps
        .sort_by_cached_key(|app| app.name.to_lowercase());
    Ok(reviews)
}

#[cfg(test)]
mod tests {
    use super::*;
    use UserBridgeCapability as Cap;

    #[test]
    fn review_matches_effective_grants_without_showing_ungranted_optional_permissions() {
        let requested = SageRequestedCapabilities::new(
            [Cap::AppGetInfo],
            [Cap::WalletGetCoins, Cap::WalletGetSecretKey],
        );
        let review = review_capabilities(&requested, [Cap::WalletGetCoins]);
        assert_eq!(review.len(), 2);
        let automatic = review.iter().find(|c| c.key == "app.get_info").unwrap();
        assert!(automatic.automatic && !automatic.wallet);
        let wallet = review.iter().find(|c| c.key == "wallet.get_coins").unwrap();
        assert!(wallet.wallet && !wallet.automatic);
        assert!(!review.iter().any(|c| c.key == "wallet.get_secret_key"));
    }
}
