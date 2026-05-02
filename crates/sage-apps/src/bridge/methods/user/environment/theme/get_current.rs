use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::capabilities::list::UserBridgeCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::shared::{BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, BridgeMethodHandleError};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentThemeView {
    pub name: String,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub most_like: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,

    pub css_vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentThemeGetCurrentResult {
    pub theme: EnvironmentThemeView,
}

#[derive(Debug, Clone, Copy)]
pub struct EnvironmentThemeGetCurrent;

#[async_trait]
impl BridgeMethod for EnvironmentThemeGetCurrent {
    fn name(&self) -> &'static str {
        "environment.theme.getCurrent"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::EnvironmentThemeGetCurrent)
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
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let theme = tools
            .host_state
            .environment
            .theme
            .current
            .lock()
            .await
            .clone()
            .ok_or_else(|| {
                BridgeMethodHandleError::internal_error("current theme is not initialized")
            })?;

        Ok(Box::new(EnvironmentThemeGetCurrentResult { theme }))
    }
}
