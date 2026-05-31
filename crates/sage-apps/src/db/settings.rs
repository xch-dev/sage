use anyhow::{Context, Result};
use sqlx::Row;

use crate::{AppsDb, unix_timestamp_ms};

const AUTO_UPDATE_ENABLED_KEY: &str = "auto_update_enabled";

impl AppsDb {
    pub async fn get_auto_update_enabled(&self) -> Result<bool> {
        let row = sqlx::query(
            r"
            SELECT value_json
            FROM sage_app_settings
            WHERE key = ?
            ",
        )
        .bind(AUTO_UPDATE_ENABLED_KEY)
        .fetch_optional(&self.pool)
        .await
        .context("failed to read auto update setting")?;

        let Some(row) = row else {
            return Ok(false);
        };

        serde_json::from_str(&row.try_get::<String, _>("value_json")?)
            .context("failed to deserialize auto update setting")
    }

    pub async fn set_auto_update_enabled(&self, enabled: bool) -> Result<bool> {
        let now = unix_timestamp_ms();

        sqlx::query(
            r"
            INSERT INTO sage_app_settings (key, value_json, updated_at_ms)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at_ms = excluded.updated_at_ms
            ",
        )
        .bind(AUTO_UPDATE_ENABLED_KEY)
        .bind(serde_json::to_string(&enabled)?)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("failed to persist auto update setting")?;

        Ok(enabled)
    }
}
