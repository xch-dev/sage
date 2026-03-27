use sqlx::{SqlitePool, migrate};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::Database;

static DB_INDEX: AtomicU32 = AtomicU32::new(0);

/// Create an in-memory SQLite database with all migrations applied.
pub async fn test_database() -> anyhow::Result<Database> {
    let index = DB_INDEX.fetch_add(1, Ordering::SeqCst);
    let pool =
        SqlitePool::connect(&format!("file:testdb_sage_database_{index}?mode=memory&cache=shared"))
            .await?;
    migrate!("../../migrations").run(&pool).await?;
    Ok(Database::new(pool))
}
