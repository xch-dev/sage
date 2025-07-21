use crate::{Database, Result};
use sqlx::Row;
use std::time::Instant;
use tracing::{error, warn};

#[derive(Debug, Clone, Copy)]
pub struct MaintenanceStats {
    pub vacuum_duration_ms: u64,
    pub analyze_duration_ms: u64,
    pub wal_checkpoint_duration_ms: u64,
    pub total_duration_ms: u64,
    pub pages_vacuumed: i64,
    pub wal_pages_checkpointed: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct DatabaseStats {
    pub total_pages: i64,
    pub free_pages: i64,
    pub free_percentage: f64,
    pub page_size: i64,
    pub database_size_bytes: i64,
    pub free_space_bytes: i64,
    pub wal_pages: i64,
}

impl Database {
    /// Get current database statistics without performing any maintenance
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let free_pages_row = sqlx::query("PRAGMA freelist_count")
            .fetch_one(&self.pool)
            .await?;
        let free_pages: i64 = free_pages_row.try_get(0)?;

        let total_pages_row = sqlx::query("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;
        let total_pages: i64 = total_pages_row.try_get(0)?;

        let page_size_row = sqlx::query("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;
        let page_size: i64 = page_size_row.try_get(0)?;

        let wal_pages_row = sqlx::query("PRAGMA wal_checkpoint")
            .fetch_one(&self.pool)
            .await?;
        let wal_pages: i64 = wal_pages_row.try_get(1).unwrap_or(0); // log_pages

        #[allow(clippy::cast_precision_loss)]
        let free_percentage = if total_pages == 0 {
            0.0
        } else {
            (free_pages as f64 / total_pages as f64) * 100.0
        };

        Ok(DatabaseStats {
            total_pages,
            free_pages,
            free_percentage,
            page_size,
            database_size_bytes: total_pages * page_size,
            free_space_bytes: free_pages * page_size,
            wal_pages,
        })
    }

    pub async fn perform_sqlite_maintenance(&self, force_vacuum: bool) -> Result<MaintenanceStats> {
        let total_start = Instant::now();
        let mut stats = MaintenanceStats {
            vacuum_duration_ms: 0,
            analyze_duration_ms: 0,
            wal_checkpoint_duration_ms: 0,
            total_duration_ms: 0,
            pages_vacuumed: 0,
            wal_pages_checkpointed: 0,
        };

        // 1. Update table statistics with ANALYZE
        let analyze_start = Instant::now();
        sqlx::query("ANALYZE").execute(&self.pool).await?;
        stats.analyze_duration_ms = analyze_start.elapsed().as_millis() as u64;

        let wal_start = Instant::now();

        // Use TRUNCATE mode to reset WAL file size
        match sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                // Get checkpoint statistics
                let checkpoint_row = sqlx::query("PRAGMA wal_checkpoint")
                    .fetch_one(&self.pool)
                    .await?;

                let _busy: i64 = checkpoint_row.try_get(0).unwrap_or(0);
                let _log_pages: i64 = checkpoint_row.try_get(1).unwrap_or(0);
                let checkpointed: i64 = checkpoint_row.try_get(2).unwrap_or(0);

                stats.wal_pages_checkpointed = checkpointed;
                stats.wal_checkpoint_duration_ms = wal_start.elapsed().as_millis() as u64;
            }
            Err(e) => {
                warn!("WAL checkpoint failed: {}", e);
                // Convert to DatabaseError which will be handled by the Result<T> type
                return Err(e.into());
            }
        }

        // 3. Check if VACUUM is needed (unless forced)
        let should_vacuum = if force_vacuum {
            true
        } else {
            let db_stats = self.get_database_stats().await?;

            // VACUUM if more than 10% free space or more than 1000 free pages
            db_stats.free_percentage > 10.0 || db_stats.free_pages > 1000
        };

        // 4. Perform VACUUM if needed
        if should_vacuum {
            let vacuum_start = Instant::now();

            // Get page count before vacuum
            let pages_before_row = sqlx::query("PRAGMA page_count")
                .fetch_one(&self.pool)
                .await?;
            let pages_before: i64 = pages_before_row.try_get(0)?;

            match sqlx::query("VACUUM").execute(&self.pool).await {
                Ok(_) => {
                    let pages_after_row = sqlx::query("PRAGMA page_count")
                        .fetch_one(&self.pool)
                        .await?;
                    let pages_after: i64 = pages_after_row.try_get(0)?;

                    stats.pages_vacuumed = pages_before - pages_after;
                    stats.vacuum_duration_ms = vacuum_start.elapsed().as_millis() as u64;
                }
                Err(e) => {
                    error!("VACUUM failed: {}", e);
                    // Convert to DatabaseError which will be handled by the Result<T> type
                    return Err(e.into());
                }
            }
        }

        // Final optimization - update statistics again if we vacuumed
        if should_vacuum {
            let final_analyze_start = Instant::now();
            sqlx::query("ANALYZE").execute(&self.pool).await?;
            stats.analyze_duration_ms += final_analyze_start.elapsed().as_millis() as u64;
        }

        stats.total_duration_ms = total_start.elapsed().as_millis() as u64;

        Ok(stats)
    }

    /// Performs a quick maintenance routine suitable for regular automated runs
    ///
    /// This is a lighter version that only runs ANALYZE and WAL checkpoint,
    /// skipping VACUUM unless specifically needed.
    pub async fn perform_quick_maintenance(&self) -> Result<MaintenanceStats> {
        self.perform_sqlite_maintenance(false).await
    }

    /// Performs a full maintenance routine including forced VACUUM
    ///
    /// This is suitable for periodic deep maintenance (e.g., weekly or monthly)
    pub async fn perform_full_maintenance(&self) -> Result<MaintenanceStats> {
        self.perform_sqlite_maintenance(true).await
    }
}
