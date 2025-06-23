use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

use crate::{DatabaseTx, Result};

impl DatabaseTx<'_> {
    pub async fn insert_file(&mut self, hash: Bytes32) -> Result<i64> {
        insert_file(&mut *self.tx, hash).await
    }

    pub async fn insert_file_uri(&mut self, file_id: i64, uri: String) -> Result<()> {
        insert_file_uri(&mut *self.tx, file_id, uri).await
    }
}

async fn insert_file(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<i64> {
    let hash = hash.as_ref();

    Ok(
        query!("INSERT INTO files (hash) VALUES (?) RETURNING id", hash)
            .fetch_one(conn)
            .await?
            .id,
    )
}

async fn insert_file_uri(conn: impl SqliteExecutor<'_>, file_id: i64, uri: String) -> Result<()> {
    query!(
        "INSERT INTO file_uris (file_id, uri) VALUES (?, ?)",
        file_id,
        uri
    )
    .execute(conn)
    .await?;

    Ok(())
}
