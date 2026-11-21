//! Dispatches the Tauri commands that map directly onto a [`Sage`] endpoint.
//!
//! These are the same endpoints the desktop app registers with `impl_endpoints_tauri!`, so the
//! sandbox stays in sync with the app as endpoints are added.

use sage::Sage;
use sage_api::{wallet_connect::*, *};
use sage_api_macro::impl_endpoints_tauri;
use serde_json::Value;

use crate::BridgeError;

impl_endpoints_tauri! {
    /// Calls the endpoint named `command`, or returns `None` if there isn't one.
    pub async fn call_endpoint(sage: &mut Sage, command: &str, req: Value) -> Option<Result<Value, BridgeError>> {
        let result = match command {
            (repeat endpoint_string => match serde_json::from_value::<Endpoint>(req) {
                Ok(req) => sage.endpoint(req) maybe_await .map_err(BridgeError::from).and_then(BridgeError::to_value),
                Err(error) => Err(BridgeError::invalid_request(&error)),
            },)
            _ => return None,
        };

        Some(result)
    }
}
