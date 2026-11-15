use async_trait::async_trait;
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    AgentTool, AppState, BridgeMethodHandleError, SageAgent, current_wallet_fingerprint,
    process_agent,
};

#[async_trait]
pub(crate) trait AgentToolExecutor: Send + Sync + 'static {
    /// The wallet logged in to Sage, which every tool acts on.
    async fn current_fingerprint(&self) -> Option<u32>;

    async fn call(
        &self,
        agent: &SageAgent,
        tool: &AgentTool,
        arguments: Value,
    ) -> Result<Value, BridgeMethodHandleError>;
}

pub(crate) struct TauriAgentExecutor {
    app_handle: AppHandle,
}

impl TauriAgentExecutor {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl AgentToolExecutor for TauriAgentExecutor {
    async fn current_fingerprint(&self) -> Option<u32> {
        current_wallet_fingerprint(&self.app_handle.state::<AppState>()).await
    }

    async fn call(
        &self,
        agent: &SageAgent,
        tool: &AgentTool,
        arguments: Value,
    ) -> Result<Value, BridgeMethodHandleError> {
        let app_state = self.app_handle.state::<AppState>();
        process_agent(&self.app_handle, &app_state, agent, tool, arguments).await
    }
}
