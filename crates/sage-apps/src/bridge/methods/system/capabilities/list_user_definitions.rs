use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::{BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::SystemBridgeCapability;
use crate::capabilities::user_registry;
use crate::types::SageAppCapabilityDefinitionView;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CapabilitiesListUserDefinitions;

#[async_trait]
impl BridgeMethod for CapabilitiesListUserDefinitions {
    fn name(&self) -> &'static str {
        "capabilities.listUserDefinitions"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::CapabilityDefinitionsRead)
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
        _tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let definitions = user_registry()
            .into_values()
            .map(Into::into)
            .collect::<Vec<SageAppCapabilityDefinitionView>>();

        Ok(Box::new(definitions))
    }
}
