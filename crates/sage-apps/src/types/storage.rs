use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;
use crate::types::app::UserSageApp;
use crate::utils::unix_timestamp_ms;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum InstalledSageAppStorage {
    AppleDataStore { identifier_hex: String },
    WindowsProfile { directory_name: String },
    Unmanaged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PendingStorageCleanupTarget {
    AppleDataStore { identifier_hex: String },
    WindowsProfile { directory_name: String },
    Unmanaged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingStorageCleanupEntry {
    id: String,
    app_id: String,
    app_name: String,
    target: PendingStorageCleanupTarget,
    created_at_ms: i64,
    last_attempt_at_ms: Option<i64>,
    attempt_count: u32,
    last_error: Option<String>,
}

impl PendingStorageCleanupEntry {
    pub fn new(app: &UserSageApp, target: PendingStorageCleanupTarget, error: &str) -> Self {
        let now = unix_timestamp_ms();

        Self {
            id: Uuid::new_v4().to_string(),
            app_id: app.common.id.clone(),
            app_name: app.common.name.clone(),
            target,
            created_at_ms: now,
            last_attempt_at_ms: Some(now),
            attempt_count: 1,
            last_error: Some(error.to_string()),
        }
    }

    pub fn record_failed_attempt(&mut self, error: &str) {
        self.last_attempt_at_ms = Some(unix_timestamp_ms());
        self.attempt_count = self.attempt_count.saturating_add(1);
        self.last_error = Some(error.to_string());
    }

    pub fn target(&self) -> &PendingStorageCleanupTarget {
        &self.target
    }
}
