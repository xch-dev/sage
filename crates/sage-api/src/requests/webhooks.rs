use sage_config::WebhookEntry;
use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Register a new webhook.")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterWebhook {
    pub url: String,
    pub event_types: Option<Vec<String>>,
    pub secret: Option<String>,
}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Register a new webhook.")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterWebhookResponse {
    pub webhook_id: String,
}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Unregister a webhook.")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct UnregisterWebhook {
    pub webhook_id: String,
}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Unregister a webhook.")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct UnregisterWebhookResponse {}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Get all webhooks.")
)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWebhooks {}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Get all webhooks.")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWebhooksResponse {
    pub webhooks: Vec<WebhookEntry>,
}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Update a webhook.")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateWebhook {
    pub webhook_id: String,
    pub enabled: bool,
}

#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Webhooks", description = "Update a webhook.")
)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateWebhookResponse {}
