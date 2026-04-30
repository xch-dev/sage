use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::types::SharedSageApp;
use crate::utils::unix_timestamp_ms;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetiredAppOriginEntry {
    id: String,
    app_id: String,
    app_name: String,
    origin_id: String,
    created_at_ms: i64,
    storage_may_contain_secrets: bool,
    cleanup_pending: bool,
}

impl RetiredAppOriginEntry {
    pub fn new(app: &SharedSageApp, cleanup_pending: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            app_id: app.id().to_string(),
            app_name: app.name().to_string(),
            origin_id: app.origin_id().to_string(),
            created_at_ms: unix_timestamp_ms(),
            storage_may_contain_secrets: app.storage_may_contain_secrets(),
            cleanup_pending,
        }
    }

    pub fn update_retirement_state(&mut self, app: &SharedSageApp, cleanup_pending: bool) {
        self.cleanup_pending = cleanup_pending;
        self.storage_may_contain_secrets =
            self.storage_may_contain_secrets || app.storage_may_contain_secrets();
    }

    pub fn matches_app_origin(&self, app_id: &str, origin_id: &str) -> bool {
        self.app_id == app_id && self.origin_id == origin_id
    }

    pub fn clear_pending_cleanup(&mut self) -> bool {
        if !self.cleanup_pending {
            return false;
        }

        self.cleanup_pending = false;
        true
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }

    pub fn created_at_ms(&self) -> i64 {
        self.created_at_ms
    }

    pub fn cleanup_pending(&self) -> bool {
        self.cleanup_pending
    }

    pub fn storage_may_contain_secrets(&self) -> bool {
        self.storage_may_contain_secrets
    }
}
