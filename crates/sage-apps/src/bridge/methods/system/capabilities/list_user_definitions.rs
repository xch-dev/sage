use async_trait::async_trait;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools, RustBridgeRequest, SageAppCapabilityDefinitionView, SystemBridgeCapability, user_registry};

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
