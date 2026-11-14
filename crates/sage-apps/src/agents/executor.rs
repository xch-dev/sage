use async_trait::async_trait;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    AgentCallError, AgentTool, AppState, RustBridgeRequest, RustBridgeResponse, SageAgent,
    process_agent,
};

#[async_trait]
pub(crate) trait AgentToolExecutor: Send + Sync + 'static {
    async fn call(
        &self,
        agent: &SageAgent,
        tool: &AgentTool,
        arguments: Value,
    ) -> Result<Value, AgentCallError>;
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
    ) -> Result<Value, AgentCallError> {
        let app_state = self.app_handle.state::<AppState>();

        let request = RustBridgeRequest {
            bridge_version: Some("v1".to_string()),
            id: Uuid::new_v4().to_string(),
            method: tool.bridge_method.to_string(),
            params_json: Some(arguments.to_string()),
        };

        let response = process_agent(&self.app_handle, &app_state, agent, &request).await;

        response_to_result(response)
    }
}

fn response_to_result(response: RustBridgeResponse) -> Result<Value, AgentCallError> {
    match response {
        RustBridgeResponse::Success(success) => {
            serde_json::from_str(&success.result_json).map_err(|err| AgentCallError {
                code: "internal_error",
                message: format!("failed to decode bridge result: {err}"),
            })
        }
        RustBridgeResponse::Error(error) => Err(map_error(&error.error.code, error.error.message)),
    }
}

fn map_error(code: &str, message: String) -> AgentCallError {
    let known = match code {
        "permission_denied" => Some("permission_denied"),
        "no_wallet" => Some("no_wallet"),
        "wallet_not_in_scope" => Some("wallet_not_in_scope"),
        "watch_only" => Some("watch_only"),
        "approval_unsupported" => Some("approval_unsupported"),
        "method_not_found" => Some("method_not_found"),
        "invalid_request" => Some("invalid_request"),
        "internal_error" => Some("internal_error"),
        _ => None,
    };

    match known {
        Some(code) => AgentCallError { code, message },
        None => AgentCallError {
            code: "internal_error",
            message: format!("{code}: {message}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn success_maps_to_value() {
        let response = RustBridgeResponse::success("1", &json!({ "a": 1 }));
        assert_eq!(response_to_result(response).unwrap(), json!({ "a": 1 }));
    }

    #[test]
    fn known_error_code_is_preserved() {
        let response = RustBridgeResponse::error("1", "watch_only", "nope");
        let err = response_to_result(response).unwrap_err();
        assert_eq!(err.code, "watch_only");
        assert_eq!(err.message, "nope");
    }

    #[test]
    fn unknown_error_code_falls_back_and_keeps_text() {
        let response = RustBridgeResponse::error("1", "weird_code", "boom");
        let err = response_to_result(response).unwrap_err();
        assert_eq!(err.code, "internal_error");
        assert_eq!(err.message, "weird_code: boom");
    }
}
