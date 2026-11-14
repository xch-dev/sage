use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::Row;

use crate::{AppsDb, unix_timestamp_ms};

const AUTO_UPDATE_ENABLED_KEY: &str = "auto_update_enabled";

impl AppsDb {
    pub async fn get_auto_update_enabled(&self) -> Result<bool> {
        Ok(self
            .read_setting(AUTO_UPDATE_ENABLED_KEY)
            .await?
            .unwrap_or(false))
    }

    pub async fn set_auto_update_enabled(&self, enabled: bool) -> Result<bool> {
        self.write_setting(AUTO_UPDATE_ENABLED_KEY, &enabled)
            .await?;
        Ok(enabled)
    }

    pub(crate) async fn read_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let row = sqlx::query("SELECT value_json FROM sage_app_settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("failed to read setting {key}"))?;

        row.map(|row| {
            serde_json::from_str(&row.try_get::<String, _>("value_json")?)
                .with_context(|| format!("failed to deserialize setting {key}"))
        })
        .transpose()
    }

    pub(crate) async fn write_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO sage_app_settings (key, value_json, updated_at_ms)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at_ms = excluded.updated_at_ms
            ",
        )
        .bind(key)
        .bind(serde_json::to_string(value)?)
        .bind(unix_timestamp_ms())
        .execute(&self.pool)
        .await
        .with_context(|| format!("failed to persist setting {key}"))?;

        Ok(())
    }
}
