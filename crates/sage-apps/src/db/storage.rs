use anyhow::{Context, Result};
use sqlx::Row;

use crate::db::AppsDb;
use crate::types::SageAppStorage;

#[derive(Debug, Clone)]
pub struct AbandonedStorage {
    pub id: i64,
    pub storage: SageAppStorage,
}

impl AppsDb {
    pub async fn register_storage(&self, storage: &SageAppStorage) -> Result<i64> {
        let storage_json = serde_json::to_string(storage).context("failed to serialize storage")?;
        let now = crate::utils::unix_timestamp_ms();

        let result = sqlx::query(
            r#"
            INSERT INTO sage_app_storages (
                storage_json,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, ?)
            "#,
        )
            .bind(storage_json)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .context("failed to insert storage")?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_app_storage(&self, app_id: &str) -> Result<Option<SageAppStorage>> {
        let row = sqlx::query(
            r#"
            SELECT storage_json
            FROM sage_apps apps
            INNER JOIN sage_app_storages storages
                ON storages.id = apps.storage_id
            WHERE apps.app_id = ?
            "#,
        )
            .bind(app_id)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("failed to get storage for app {app_id}"))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let storage_json: String = row.try_get("storage_json")?;

        serde_json::from_str::<SageAppStorage>(&storage_json)
            .with_context(|| format!("failed to deserialize storage for app {app_id}"))
            .map(Some)
    }

    pub async fn list_abandoned_managed_storages(&self) -> Result<Vec<AbandonedStorage>> {
        let rows = sqlx::query(
            r#"
            SELECT id, storage_json
            FROM sage_app_storages
            WHERE id NOT IN (
                SELECT storage_id FROM sage_apps
            )
            ORDER BY id ASC
            "#,
        )
            .fetch_all(&self.pool)
            .await
            .context("failed to list abandoned storages")?;

        rows.into_iter()
            .map(|row| {
                let id: i64 = row.try_get("id")?;
                let storage_json: String = row.try_get("storage_json")?;
                let storage = serde_json::from_str::<SageAppStorage>(&storage_json)
                    .with_context(|| format!("failed to deserialize abandoned storage {id}"))?;

                Ok(AbandonedStorage { id, storage })
            })
            .filter_map(|result| match result {
                Ok(abandoned) if matches!(abandoned.storage, SageAppStorage::Unmanaged) => None,
                other => Some(other),
            })
            .collect()
    }

    pub async fn delete_origins_for_abandoned_storage(&self, storage_id: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM sage_app_origins
            WHERE storage_id = ?
              AND storage_id NOT IN (
                  SELECT storage_id FROM sage_apps
              )
            "#,
        )
            .bind(storage_id)
            .execute(&self.pool)
            .await
            .with_context(|| {
                format!("failed to delete abandoned origins for storage row {storage_id}")
            })?;

        Ok(result.rows_affected())
    }

    pub async fn delete_abandoned_storage(&self, storage_id: i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM sage_app_storages
            WHERE id = ?
              AND id NOT IN (
                SELECT storage_id FROM sage_apps
              )
            "#,
        )
            .bind(storage_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("failed to delete abandoned storage row {storage_id}"))?;

        Ok(())
    }
}
