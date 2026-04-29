pub mod bridge_request;
pub mod capabilities;
pub mod commands;
pub mod event_emit;
pub mod methods;
pub mod registry;
pub mod state;
pub mod ts_exports;
pub mod types;

pub use types::{
    ResolveBridgeApprovalArgs, RustBridgeApprovalEvent, RustBridgeApprovalRequest,
    RustBridgeErrorPayload, RustBridgeErrorResponse, RustBridgeInvokeResult, RustBridgeRequest,
    RustBridgeResponse, RustBridgeSuccessResponse,
};
use crate::types::SharedSageApp;

pub const USER_BRIDGE_CHANNEL: &str = "sage-bridge";
pub const SYSTEM_BRIDGE_CHANNEL: &str = "sage-system-bridge";

pub(crate) fn response_channel_for_app(app: &SharedSageApp) -> &'static str {
    if app.is_system_app() {
        return SYSTEM_BRIDGE_CHANNEL;
    }

    USER_BRIDGE_CHANNEL
}
