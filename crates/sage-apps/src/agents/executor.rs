use async_trait::async_trait;
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{AgentTool, AppState, BridgeMethodHandleError, SageAgent, process_agent};

#[async_trait]
pub(crate) trait AgentToolExecutor: Send + Sync + 'static {
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
