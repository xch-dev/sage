use anyhow::{Context, Result};
use sqlx::SqliteConnection;

use crate::AppsDb;

pub(crate) struct AppsDbTx {
    pub(super) conn: SqliteConnection,
    finished: bool,
}

impl AppsDb {
    pub(crate) async fn begin_immediate(&self) -> Result<AppsDbTx> {
        let mut conn = self
            .pool()
            .acquire()
            .await
            .context("failed to acquire apps db connection")?
            .detach();

        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut conn)
            .await
            .context("failed to begin immediate apps db transaction")?;

        Ok(AppsDbTx {
            conn,
            finished: false,
        })
    }
}

impl AppsDbTx {
    pub(crate) async fn commit(mut self) -> Result<()> {
        sqlx::query("COMMIT")
            .execute(&mut self.conn)
            .await
            .context("failed to commit apps db transaction")?;

        self.finished = true;
        Ok(())
    }

    pub(crate) async fn rollback(&mut self) {
        if self.finished {
            return;
        }

        let _ = sqlx::query("ROLLBACK").execute(&mut self.conn).await;
        self.finished = true;
    }
}

impl Drop for AppsDbTx {
    fn drop(&mut self) {
        if !self.finished {
            tracing::error!("AppsDbTx dropped without commit or rollback");
        }
    }
}
