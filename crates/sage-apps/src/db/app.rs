use anyhow::{Context, Result};

use crate::db::AppsDb;

impl AppsDb {
    pub async fn register_app(
        &self,
        app_id: &str,
        storage_id: i64,
        origin_row_id: i64,
    ) -> Result<()> {
        let now = crate::utils::unix_timestamp_ms();

        sqlx::query(
            r#"
            INSERT INTO sage_apps (
                app_id,
                storage_id,
                origin_row_id,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
            .bind(app_id)
            .bind(storage_id)
            .bind(origin_row_id)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .with_context(|| format!("failed to register app {app_id}"))?;

        Ok(())
    }

    pub async fn unregister_app(&self, app_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM sage_apps
            WHERE app_id = ?
            "#,
        )
            .bind(app_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("failed to unregister app {app_id}"))?;

        Ok(())
    }

    pub async fn update_app_assignment(
        &self,
        app_id: &str,
        storage_id: i64,
        origin_row_id: i64,
    ) -> Result<()> {
        let now = crate::utils::unix_timestamp_ms();

        sqlx::query(
            r#"
            UPDATE sage_apps
            SET
                storage_id = ?,
                origin_row_id = ?,
                updated_at_ms = ?
            WHERE app_id = ?
            "#,
        )
            .bind(storage_id)
            .bind(origin_row_id)
            .bind(now)
            .bind(app_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("failed to update app assignment {app_id}"))?;

        Ok(())
    }
}
