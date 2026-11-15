use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

const DB_FILE: &str = "sage-apps.sqlite3";

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("src/db/migrations");

#[derive(Debug, Clone)]
pub struct AppsDb {
    pub(super) pool: SqlitePool,
}

impl AppsDb {
    pub async fn initialize(base_path: &Path) -> Result<Self> {
        fs::create_dir_all(base_path).with_context(|| {
            format!("failed to create apps db directory {}", base_path.display())
        })?;

        let db_path = database_path(base_path);

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .with_context(|| format!("failed to open apps database {}", db_path.display()))?;

        MIGRATOR.run(&pool).await.with_context(|| {
            format!(
                "failed to run apps database migrations {}",
                db_path.display()
            )
        })?;

        Ok(Self { pool })
    }

    /// Deletes the database, along with its write-ahead log, so that the next start recreates it.
    pub fn delete_files(base_path: &Path) -> io::Result<()> {
        let db_path = database_path(base_path);

        for suffix in ["", "-wal", "-shm"] {
            let mut path = db_path.clone().into_os_string();
            path.push(suffix);

            match fs::remove_file(&path) {
                Err(err) if err.kind() != io::ErrorKind::NotFound => return Err(err),
                _ => {}
            }
        }

        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn database_path(base_path: &Path) -> PathBuf {
    base_path.join(DB_FILE)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn deleted_database_is_recreated_empty() {
        let dir = tempdir().unwrap();
        let db = AppsDb::initialize(dir.path()).await.unwrap();
        db.write_setting("key", &true).await.unwrap();
        db.pool().close().await;

        AppsDb::delete_files(dir.path()).unwrap();
        // Deleting a database that is already gone is fine.
        AppsDb::delete_files(dir.path()).unwrap();

        let db = AppsDb::initialize(dir.path()).await.unwrap();
        assert_eq!(db.read_setting::<bool>("key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn database_from_a_newer_build_fails_to_open() {
        let dir = tempdir().unwrap();
        let db = AppsDb::initialize(dir.path()).await.unwrap();
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES (9999, 'from the future', TRUE, x'00', 0)",
        )
        .execute(db.pool())
        .await
        .unwrap();
        db.pool().close().await;

        assert!(AppsDb::initialize(dir.path()).await.is_err());
    }
}
