use sage_api::{ErrorKind, HandleErrorCode, RecipientError};
use serde::Serialize;
use serde_json::Value;

/// An error in the same shape as the desktop app's command errors, so that the frontend handles
/// it the same way.
#[derive(Debug, Clone, Serialize)]
pub struct BridgeError {
    pub kind: ErrorKind,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle_error_code: Option<HandleErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<RecipientError>>,
}

impl BridgeError {
    pub fn api(reason: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Api,
            reason: reason.into(),
            handle_error_code: None,
            recipients: None,
        }
    }

    pub fn internal(reason: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: reason.into(),
            handle_error_code: None,
            recipients: None,
        }
    }

    pub fn invalid_request(error: &serde_json::Error) -> Self {
        Self::api(format!("Invalid request: {error}"))
    }

    pub fn to_value(value: impl Serialize) -> Result<Value, Self> {
        serde_json::to_value(value)
            .map_err(|error| Self::internal(format!("Failed to serialize response: {error}")))
    }
}

impl From<sage::Error> for BridgeError {
    fn from(error: sage::Error) -> Self {
        Self {
            kind: error.kind(),
            reason: error.to_string(),
            handle_error_code: error.handle_error_code(),
            recipients: error.recipient_errors().map(<[RecipientError]>::to_vec),
        }
    }
}
