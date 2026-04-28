use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::runtime::{ReadyToStopParams, RuntimeAckResult, SetBeforeStopListenerParams};

#[derive(Debug, Clone, Copy)]
pub struct AppLifecycleSetBeforeStopListener;

#[derive(Debug, Clone, Copy)]
pub struct AppLifecycleReadyToStop;

#[async_trait]
impl BridgeMethod for AppLifecycleSetBeforeStopListener {
    fn name(&self) -> &'static str {
        "app.lifecycle.setBeforeStopListener"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppLifecycleSetBeforeStopListener)
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
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: SetBeforeStopListenerParams = parse_required_params(self, request)?;

        let mut listeners = tools
            .host_state
            .runtime
            .before_stop_listeners_by_app_id
            .lock()
            .await;

        if params.active() {
            listeners.insert(ctx.app.id().to_string());
        } else {
            listeners.remove(ctx.app.id());
        }

        Ok(Box::new(RuntimeAckResult { ok: true }))
    }
}

#[async_trait]
impl BridgeMethod for AppLifecycleReadyToStop {
    fn name(&self) -> &'static str {
        "app.lifecycle.readyToStop"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppLifecycleReadyToStop)
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
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: ReadyToStopParams = parse_required_params(self, request)?;

        let sender = {
            let mut pending = tools.host_state.runtime.pending_stop_ready.lock().await;
            pending.remove(params.request_id())
        };

        if let Some(sender) = sender {
            let _ = sender.send(());
        }

        Ok(Box::new(RuntimeAckResult { ok: true }))
    }
}
