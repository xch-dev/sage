pub mod bridge_request;
pub mod capabilities;
pub mod commands;
pub mod event_emit;
pub mod methods;
pub mod registry;
pub mod state;
pub mod ts_exports;
pub mod types;

use crate::runtime::SageAppRuntimeKind;
pub use types::{
    ResolveBridgeApprovalArgs, RustBridgeApprovalEvent, RustBridgeApprovalRequest,
    RustBridgeErrorPayload, RustBridgeErrorResponse, RustBridgeInvokeResult, RustBridgeRequest,
    RustBridgeResponse, RustBridgeSuccessResponse,
};

pub const USER_BRIDGE_CHANNEL: &str = "sage-bridge";
pub const SYSTEM_BRIDGE_CHANNEL: &str = "sage-system-bridge";

pub(crate) fn response_channel_for_runtime_kind(runtime_kind: SageAppRuntimeKind) -> &'static str {
    match runtime_kind {
        SageAppRuntimeKind::User => USER_BRIDGE_CHANNEL,
        SageAppRuntimeKind::System => SYSTEM_BRIDGE_CHANNEL,
    }
}
