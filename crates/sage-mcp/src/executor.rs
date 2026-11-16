use async_trait::async_trait;
use sage_auth::{Agent, AuthError};
use serde_json::Value;

use crate::AgentTool;

/// A failure reported to the agent as a tool error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
}

impl ToolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// The refusal for any authorization decision other than `Allowed`.
    pub fn approval_unsupported() -> Self {
        Self::new(
            "permission_denied",
            "this action needs approval, which isn't supported",
        )
    }
}

impl From<AuthError> for ToolError {
    fn from(err: AuthError) -> Self {
        Self {
            code: err.code().to_string(),
            message: err.to_string(),
        }
    }
}

#[async_trait]
pub trait AgentToolExecutor: Send + Sync + 'static {
    /// The wallet logged in to Sage, which every tool acts on.
    async fn current_fingerprint(&self) -> Option<u32>;

    async fn call(
        &self,
        agent: &Agent,
        tool: &AgentTool,
        arguments: Value,
    ) -> Result<Value, ToolError>;
}
