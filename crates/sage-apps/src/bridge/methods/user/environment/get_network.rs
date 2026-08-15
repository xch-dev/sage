use async_trait::async_trait;
use sage_api::{GetNetwork, NetworkKind};
use serde::Serialize;
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentGetNetworkResult {
    pub name: String,
    pub network_id: String,
    pub kind: NetworkKind,
    pub ticker: String,
    pub prefix: String,
    pub precision: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvironmentGetNetwork;

#[async_trait]
impl BridgeMethod for EnvironmentGetNetwork {
    fn name(&self) -> &'static str {
        "environment.getNetwork"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::EnvironmentGetNetwork)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let current = tools
            .app_state
            .lock()
            .await
            .get_network(GetNetwork {})
            .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(EnvironmentGetNetworkResult {
            name: current.network.name.clone(),
            network_id: current.network.network_id(),
            kind: current.kind,
            ticker: current.network.ticker.clone(),
            prefix: current.network.prefix(),
            precision: current.network.precision,
        }))
    }
}
