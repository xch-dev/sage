use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandler, BridgeTools, ReadyToStopParams, RuntimeAckResult, RustBridgeRequest,
    SetBeforeStopListenerParams, UserBridgeCapability, bridge_result, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct AppLifecycleSetBeforeStopListener;

#[derive(Debug, Clone, Copy)]
pub struct AppLifecycleReadyToStop;

impl BridgeMethodHandler for AppLifecycleSetBeforeStopListener {
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
            listeners.remove(&ctx.app.id());
        }

        bridge_result(RuntimeAckResult { ok: true })
    }
}

impl BridgeMethodHandler for AppLifecycleReadyToStop {
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

        bridge_result(RuntimeAckResult { ok: true })
    }
}
