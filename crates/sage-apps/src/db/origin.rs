use anyhow::{Context, Result};
use sqlx::Row;

use crate::db::AppsDb;

impl AppsDb {
    pub async fn register_origin(&self, origin_id: &str, storage_id: i64) -> Result<i64> {
        let now = crate::utils::unix_timestamp_ms();

        let result = sqlx::query(
            r#"
            INSERT INTO sage_app_origins (
                origin_id,
                storage_id,
                may_contain_secrets,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, 0, ?, ?)
            "#,
        )
            .bind(origin_id)
            .bind(storage_id)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .with_context(|| format!("failed to insert origin {origin_id}"))?;

        Ok(result.last_insert_rowid())
    }

    pub async fn mark_origin_may_contain_secrets(&self, origin_id: &str) -> Result<()> {
        let now = crate::utils::unix_timestamp_ms();

        sqlx::query(
            r#"
            UPDATE sage_app_origins
            SET
                may_contain_secrets = 1,
                updated_at_ms = ?
            WHERE origin_id = ?
            "#,
        )
            .bind(now)
            .bind(origin_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("failed to mark origin as containing secrets {origin_id}"))?;

        Ok(())
    }

    pub async fn origin_may_contain_secrets(&self, origin_id: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT may_contain_secrets
            FROM sage_app_origins
            WHERE origin_id = ?
            "#,
        )
            .bind(origin_id)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("failed to query origin secret state {origin_id}"))?;

        let Some(row) = row else {
            return Ok(false);
        };

        let value: i64 = row.try_get("may_contain_secrets")?;

        Ok(value != 0)
    }

    pub async fn delete_abandoned_origins(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM sage_app_origins
            WHERE id NOT IN (
                SELECT origin_row_id FROM sage_apps
            )
            "#,
        )
            .execute(&self.pool)
            .await
            .context("failed to delete abandoned origin rows")?;

        Ok(result.rows_affected())
    }
}
