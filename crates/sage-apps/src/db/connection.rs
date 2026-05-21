use std::{
    fs,
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
        fs::create_dir_all(base_path)
            .with_context(|| format!("failed to create apps db directory {}", base_path.display()))?;

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

        MIGRATOR
            .run(&pool)
            .await
            .with_context(|| format!("failed to run apps database migrations {}", db_path.display()))?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn database_path(base_path: &Path) -> PathBuf {
    base_path.join(DB_FILE)
}
