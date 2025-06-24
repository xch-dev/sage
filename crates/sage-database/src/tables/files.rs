use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

use crate::{DatabaseTx, Result};

impl DatabaseTx<'_> {
    pub async fn insert_file(&mut self, hash: Bytes32) -> Result<()> {
        insert_file(&mut *self.tx, hash).await
    }

    pub async fn insert_file_uri(&mut self, hash: Bytes32, uri: String) -> Result<()> {
        insert_file_uri(&mut *self.tx, hash, uri).await
    }

    pub async fn file_data(&mut self, hash: Bytes32) -> Result<Option<Vec<u8>>> {
        file_data(&mut *self.tx, hash).await
    }
}

async fn insert_file(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<()> {
    let hash = hash.as_ref();

    query!("INSERT INTO files (hash) VALUES (?)", hash)
        .fetch_one(conn)
        .await?;

    Ok(())
}

async fn insert_file_uri(conn: impl SqliteExecutor<'_>, hash: Bytes32, uri: String) -> Result<()> {
    let hash = hash.as_ref();

    query!(
        "INSERT INTO file_uris (file_id, uri) VALUES ((SELECT id FROM files WHERE hash = ?), ?)",
        hash,
        uri
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn file_data(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<Option<Vec<u8>>> {
    let hash = hash.as_ref();

    let row = query!("SELECT data FROM files WHERE hash = ?", hash)
        .fetch_optional(conn)
        .await?;

    Ok(row.and_then(|row| row.data))
}
