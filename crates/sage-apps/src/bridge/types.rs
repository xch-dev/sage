use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::methods::user::wallet::send_xch::WalletSendXchParams;
use crate::types::{SageApp, SageAppCapabilityDefinitionView, SageNetworkWhitelistEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone)]
pub struct PendingBridgeApproval {
    pub app_id: String,
    pub app_webview_label: String,
    pub request: RustBridgeRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RustBridgeRequest {
    pub channel: String,
    pub bridge_version: Option<String>,
    pub id: String,
    pub method: String,
    pub params_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RustBridgeErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RustBridgeSuccessResponse {
    pub channel: String,
    pub bridge_version: String,
    pub id: String,
    pub ok: bool,
    pub result_json: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RustBridgeErrorResponse {
    pub channel: String,
    pub bridge_version: String,
    pub id: String,
    pub ok: bool,
    pub error: RustBridgeErrorPayload,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(untagged)]
pub enum RustBridgeResponse {
    Success(RustBridgeSuccessResponse),
    Error(RustBridgeErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RustBridgeApprovalRequest {
    pub app: SageApp,
    pub source_label: String,
    pub request_id: String,

    #[serde(flatten)]
    pub body: RustBridgeApprovalBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RustBridgeApprovalBody {
    GetSecretKey {
        fingerprint: u32,
    },
    SendXch {
        summary: WalletSendXchParams,
    },
    CapabilityGrant {
        capability: UserBridgeCapability,
        definition: SageAppCapabilityDefinitionView,
    },
    NetworkWhitelistGrant {
        entry: SageNetworkWhitelistEntry,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct RustBridgeApprovalEvent {
    pub approval_id: String,
    pub approval: RustBridgeApprovalRequest,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RustBridgeInvokeResult {
    Immediate { response: RustBridgeResponse },
    Pending {},
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ResolveBridgeApprovalArgs {
    pub approval_id: String,
    pub approved: bool,
    pub reason: Option<String>,
}

impl RustBridgeResponse {
    pub fn success(channel: &str, id: &str, result: Value) -> RustBridgeResponse {
        RustBridgeResponse::Success(RustBridgeSuccessResponse {
            channel: channel.into(),
            bridge_version: "v1".into(),
            id: id.into(),
            ok: true,
            result_json: serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string()),
        })
    }
    pub fn error(
        channel: &str,
        id: &str,
        code: &str,
        message: impl Into<String>,
    ) -> RustBridgeResponse {
        RustBridgeResponse::Error(RustBridgeErrorResponse {
            channel: channel.into(),
            bridge_version: "v1".into(),
            id: id.into(),
            ok: false,
            error: RustBridgeErrorPayload {
                code: code.into(),
                message: message.into(),
            },
        })
    }
}
