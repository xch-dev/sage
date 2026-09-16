use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools,
    PermissionGrantCapabilityApproval, PermissionGrantNetworkTarget, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, SageNetworkWhitelistEntry, UserBridgeCapability,
    get_user_capability_definition, grant_permissions, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct AppRequestPermissionGrants;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestPermissionGrantsParams {
    #[serde(default)]
    pub capabilities: Vec<UserBridgeCapability>,

    #[serde(default)]
    pub network_whitelist: Vec<PermissionGrantNetworkTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionGrantsResult {
    pub granted: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub already_granted: Option<bool>,

    pub capabilities: Vec<UserBridgeCapability>,
    pub network_whitelist: Vec<PermissionGrantNetworkTarget>,
    pub full_granted_capabilities: Vec<UserBridgeCapability>,
    pub full_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
    pub full_granted_network_whitelist_by_network: BTreeMap<String, Vec<SageNetworkWhitelistEntry>>,
}

fn ensure_capability_requestable_by_app(
    capability: UserBridgeCapability,
) -> Result<(), BridgeMethodHandleError> {
    let definition = get_user_capability_definition(capability);

    if !definition.flags().requestable_by_app() {
        return Err(BridgeMethodHandleError::invalid_request(format!(
            "capability cannot be requested by app: {}",
            capability.key()
        )));
    }

    Ok(())
}

fn normalized_params(
    params: RequestPermissionGrantsParams,
) -> Result<RequestPermissionGrantsParams, BridgeMethodHandleError> {
    let capabilities = params
        .capabilities
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let network_whitelist = params
        .network_whitelist
        .into_iter()
        .map(|mut target| {
            if let Some(network_id) = &target.network_id {
                let network_id = network_id.trim();
                if network_id.is_empty() {
                    return Err(BridgeMethodHandleError::invalid_request(
                        "network id cannot be empty",
                    ));
                }
                target.network_id = Some(network_id.to_string());
            }
            Ok(target)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if capabilities.is_empty() && network_whitelist.is_empty() {
        return Err(BridgeMethodHandleError::invalid_request(
            "permission grant request must include at least one capability or network target",
        ));
    }

    for capability in &capabilities {
        ensure_capability_requestable_by_app(*capability)?;
    }

    Ok(RequestPermissionGrantsParams {
        capabilities,
        network_whitelist,
    })
}

fn validate_grants_allowed_by_manifest(
    ctx: &BridgeContext<'_>,
    params: &RequestPermissionGrantsParams,
) -> Result<(), BridgeMethodHandleError> {
    ctx.app
        .try_with(|app| {
            let requested = app.common().requested_permissions();
            let mut granted = app.common().granted_permissions().clone();

            for capability in &params.capabilities {
                granted = granted.with_capability_added(requested, *capability)?;
            }

            for target in &params.network_whitelist {
                granted = match &target.network_id {
                    Some(network_id) => granted.with_network_whitelist_entry_for_network_added(
                        requested,
                        network_id,
                        target.entry.clone(),
                    )?,
                    None => granted
                        .with_network_whitelist_entry_added(requested, target.entry.clone())?,
                };
            }

            Ok::<_, anyhow::Error>(())
        })
        .map_err(|err| {
            BridgeMethodHandleError::invalid_request(format!(
                "requested permissions cannot be granted: {err}"
            ))
        })
}

fn missing_grants(
    ctx: &BridgeContext<'_>,
    params: &RequestPermissionGrantsParams,
) -> (
    Vec<PermissionGrantCapabilityApproval>,
    Vec<PermissionGrantNetworkTarget>,
) {
    ctx.app.with(|app| {
        let granted = app.granted_permissions();
        let capabilities = params
            .capabilities
            .iter()
            .filter(|capability| !granted.has_capability(**capability))
            .map(|capability| PermissionGrantCapabilityApproval {
                capability: *capability,
                definition: get_user_capability_definition(*capability).into(),
            })
            .collect();

        let network_whitelist = params
            .network_whitelist
            .iter()
            .filter(|target| match target.network_id.as_deref() {
                Some(network_id) => granted
                    .network()
                    .whitelist_by_network()
                    .get(network_id)
                    .is_none_or(|entries| !entries.contains(&target.entry)),
                None => !granted.network().whitelist().contains(&target.entry),
            })
            .cloned()
            .collect();

        (capabilities, network_whitelist)
    })
}

#[async_trait]
impl BridgeMethod for AppRequestPermissionGrants {
    fn name(&self) -> &'static str {
        "app.requestPermissionGrants"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppRequestPermissionGrants)
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params = normalized_params(parse_required_params(self, request)?)?;
        validate_grants_allowed_by_manifest(&ctx, &params)?;
        let (capabilities, network_whitelist) = missing_grants(&ctx, &params);

        if capabilities.is_empty() && network_whitelist.is_empty() {
            return Ok(None);
        }

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::PermissionGrants {
                capabilities,
                network_whitelist,
            },
        }))
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params = normalized_params(parse_required_params(self, request)?)?;
        let network_entries = params
            .network_whitelist
            .iter()
            .map(|target| (target.network_id.clone(), target.entry.clone()))
            .collect::<Vec<_>>();

        let update = grant_permissions(
            tools.app_handle,
            tools.host_state,
            &ctx.app.id(),
            &params.capabilities,
            &network_entries,
        )
        .await
        .map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to grant requested permissions: {err}"
            ))
        })?;

        let change = update.change();
        let already_granted = change.capabilities().is_empty()
            && change.network_whitelist().is_empty()
            && change.network_whitelist_by_network().is_empty();

        Ok(Box::new(RequestPermissionGrantsResult {
            granted: true,
            already_granted: already_granted.then_some(true),
            capabilities: params.capabilities,
            network_whitelist: params.network_whitelist,
            full_granted_capabilities: change.capabilities().full.clone(),
            full_granted_network_whitelist: change.network_whitelist().full.clone(),
            full_granted_network_whitelist_by_network: change
                .network_whitelist_by_network()
                .full
                .clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(host: &str) -> PermissionGrantNetworkTarget {
        PermissionGrantNetworkTarget {
            entry: SageNetworkWhitelistEntry::new("https", host).unwrap(),
            network_id: None,
        }
    }

    #[test]
    fn rejects_empty_batch() {
        let err = normalized_params(RequestPermissionGrantsParams {
            capabilities: vec![],
            network_whitelist: vec![],
        })
        .expect_err("empty batches must be rejected");

        assert!(format!("{err:?}").contains("at least one"));
    }

    #[test]
    fn accepts_mixed_batch_and_removes_duplicates() {
        let params = normalized_params(RequestPermissionGrantsParams {
            capabilities: vec![
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::WalletSendXch,
            ],
            network_whitelist: vec![target("api.example.com"), target("api.example.com")],
        })
        .expect("mixed batch should be valid");

        assert_eq!(params.capabilities.len(), 1);
        assert_eq!(params.network_whitelist.len(), 1);
    }

    #[test]
    fn rejects_non_requestable_capability_in_batch() {
        let err = normalized_params(RequestPermissionGrantsParams {
            capabilities: vec![UserBridgeCapability::WalletSendXchAutoSubmit],
            network_whitelist: vec![],
        })
        .expect_err("auto-submit send must not be requestable by running apps");

        assert!(format!("{err:?}").contains("wallet.send_xch_auto_submit"));
    }

    #[test]
    fn normalizes_network_ids_and_rejects_blank_ones() {
        let params = normalized_params(RequestPermissionGrantsParams {
            capabilities: vec![],
            network_whitelist: vec![PermissionGrantNetworkTarget {
                entry: SageNetworkWhitelistEntry::new("https", "api.example.com").unwrap(),
                network_id: Some(" mainnet ".into()),
            }],
        })
        .expect("trimmed network id should be valid");

        assert_eq!(
            params.network_whitelist[0].network_id.as_deref(),
            Some("mainnet")
        );

        let err = normalized_params(RequestPermissionGrantsParams {
            capabilities: vec![],
            network_whitelist: vec![PermissionGrantNetworkTarget {
                entry: SageNetworkWhitelistEntry::new("https", "api.example.com").unwrap(),
                network_id: Some("  ".into()),
            }],
        })
        .expect_err("blank network id must be rejected");

        assert!(format!("{err:?}").contains("network id cannot be empty"));
    }
}
