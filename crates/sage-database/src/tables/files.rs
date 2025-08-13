use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct FileUri {
    pub hash: Bytes32,
    pub uri: String,
    pub last_checked_timestamp: Option<u64>,
    pub failed_attempts: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateableNft {
    pub hash: Bytes32,
    pub minter_hash: Option<Bytes32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizedImageKind {
    Icon,
    Thumbnail,
}

#[derive(Debug, Clone)]
pub struct FileData {
    pub hash: Bytes32,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub is_hash_match: bool,
}

#[derive(Debug, Clone)]
pub struct ResizedImage {
    pub data: Vec<u8>,
    pub mime_type: Option<String>,
}

impl Database {
    pub async fn candidates_for_download(
        &self,
        check_every_seconds: i64,
        max_failed_attempts: u32,
        limit: u32,
    ) -> Result<Vec<FileUri>> {
        candidates_for_download(&self.pool, check_every_seconds, max_failed_attempts, limit).await
    }

    pub async fn thumbnail(&self, hash: Bytes32) -> Result<Option<ResizedImage>> {
        resized_image(&self.pool, hash, ResizedImageKind::Thumbnail).await
    }

    pub async fn icon(&self, hash: Bytes32) -> Result<Option<ResizedImage>> {
        resized_image(&self.pool, hash, ResizedImageKind::Icon).await
    }

    pub async fn full_file_data(&self, hash: Bytes32) -> Result<Option<FileData>> {
        full_file_data(&self.pool, hash).await
    }

    pub async fn checked_files(&self) -> Result<u64> {
        checked_files(&self.pool).await
    }

    pub async fn total_files(&self) -> Result<u64> {
        total_files(&self.pool).await
    }
}

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

    pub async fn update_checked_uri(&mut self, hash: Bytes32, uri: String) -> Result<()> {
        update_checked_uri(&mut *self.tx, hash, uri).await
    }

    pub async fn update_failed_uri(&mut self, hash: Bytes32, uri: String) -> Result<()> {
        update_failed_uri(&mut *self.tx, hash, uri).await
    }

    pub async fn update_file(
        &mut self,
        hash: Bytes32,
        data: Vec<u8>,
        mime_type: String,
        is_hash_match: bool,
    ) -> Result<()> {
        update_file(&mut *self.tx, hash, data, mime_type, is_hash_match).await
    }

    pub async fn insert_resized_image(
        &mut self,
        file_hash: Bytes32,
        kind: ResizedImageKind,
        data: Vec<u8>,
    ) -> Result<()> {
        insert_resized_image(&mut *self.tx, file_hash, kind, data).await
    }

    pub async fn icon(&mut self, hash: Bytes32) -> Result<Option<ResizedImage>> {
        resized_image(&mut *self.tx, hash, ResizedImageKind::Icon).await
    }

    pub async fn nfts_with_metadata_hash(&mut self, hash: Bytes32) -> Result<Vec<UpdateableNft>> {
        nfts_with_metadata_hash(&mut *self.tx, hash).await
    }

    pub async fn delete_file_data(&mut self, hash: Bytes32) -> Result<()> {
        delete_file_data(&mut *self.tx, hash).await
    }

    pub async fn set_uri_unchecked(&mut self, uri: String) -> Result<()> {
        set_uri_unchecked(&mut *self.tx, uri).await
    }
}

async fn insert_file(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<()> {
    let hash = hash.as_ref();

    query!("INSERT OR IGNORE INTO files (hash) VALUES (?)", hash)
        .execute(conn)
        .await?;

    Ok(())
}

async fn insert_file_uri(conn: impl SqliteExecutor<'_>, hash: Bytes32, uri: String) -> Result<()> {
    let hash = hash.as_ref();

    query!(
        "INSERT OR IGNORE INTO file_uris (file_id, uri) VALUES ((SELECT id FROM files WHERE hash = ?), ?)",
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

async fn full_file_data(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<Option<FileData>> {
    let hash = hash.as_ref();

    let row = query!(
        "SELECT hash, data, mime_type, is_hash_match FROM files WHERE hash = ?",
        hash
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        Ok(FileData {
            hash: row.hash.convert()?,
            data: row.data.unwrap_or_default(),
            mime_type: row.mime_type.unwrap_or_default(),
            is_hash_match: row.is_hash_match.unwrap_or_default(),
        })
    })
    .transpose()
}

async fn candidates_for_download(
    conn: impl SqliteExecutor<'_>,
    check_every_seconds: i64,
    max_failed_attempts: u32,
    limit: u32,
) -> Result<Vec<FileUri>> {
    query!(
        "
        SELECT hash, uri, last_checked_timestamp, failed_attempts
        FROM file_uris
        INNER JOIN files ON files.id = file_uris.file_id
        WHERE data IS NULL
        AND (last_checked_timestamp IS NULL OR unixepoch() - last_checked_timestamp >= ?)
        AND failed_attempts < ?
        LIMIT ?
        ",
        check_every_seconds,
        max_failed_attempts,
        limit
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(FileUri {
            hash: row.hash.convert()?,
            uri: row.uri,
            last_checked_timestamp: row.last_checked_timestamp.convert()?,
            failed_attempts: row.failed_attempts.convert()?,
        })
    })
    .collect()
}

async fn update_failed_uri(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    uri: String,
) -> Result<()> {
    let hash = hash.as_ref();

    query!(
        "
        UPDATE file_uris
        SET failed_attempts = failed_attempts + 1, last_checked_timestamp = unixepoch()
        WHERE file_id = (SELECT id FROM files WHERE hash = ?) AND uri = ?
        ",
        hash,
        uri
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn update_checked_uri(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    uri: String,
) -> Result<()> {
    let hash = hash.as_ref();

    query!(
        "
        UPDATE file_uris
        SET last_checked_timestamp = unixepoch()
        WHERE file_id = (SELECT id FROM files WHERE hash = ?) AND uri = ?
        ",
        hash,
        uri
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn update_file(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    data: Vec<u8>,
    mime_type: String,
    is_hash_match: bool,
) -> Result<()> {
    let hash = hash.as_ref();

    query!(
        "
        UPDATE files SET data = ?, mime_type = ?, is_hash_match = ?
        WHERE hash = ?
        AND (data IS NULL OR NOT is_hash_match)
        ",
        data,
        mime_type,
        is_hash_match,
        hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nfts_with_metadata_hash(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
) -> Result<Vec<UpdateableNft>> {
    let hash = hash.as_ref();

    query!(
        "SELECT hash, minter_hash FROM nfts INNER JOIN assets ON assets.id = asset_id WHERE metadata_hash = ?",
        hash
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| Ok(UpdateableNft {
        hash: row.hash.convert()?,
        minter_hash: row.minter_hash.convert()?,
    }))
    .collect()
}

async fn insert_resized_image(
    conn: impl SqliteExecutor<'_>,
    file_hash: Bytes32,
    kind: ResizedImageKind,
    data: Vec<u8>,
) -> Result<()> {
    let file_hash = file_hash.as_ref();
    let kind = kind as i64;

    query!(
        "INSERT INTO resized_images (file_id, kind, data) VALUES ((SELECT id FROM files WHERE hash = ?), ?, ?)",
        file_hash,
        kind,
        data
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn resized_image(
    conn: impl SqliteExecutor<'_>,
    hash: Bytes32,
    kind: ResizedImageKind,
) -> Result<Option<ResizedImage>> {
    let hash = hash.as_ref();
    let kind = kind as i64;

    let row = query!(
        "SELECT resized_images.data, mime_type
        FROM resized_images 
        INNER JOIN files ON files.id = resized_images.file_id
        WHERE files.hash = ? AND kind = ?",
        hash,
        kind
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|row| ResizedImage {
        data: row.data,
        mime_type: row.mime_type,
    }))
}

async fn delete_file_data(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<()> {
    let hash = hash.as_ref();

    query!("UPDATE files SET data = NULL WHERE hash = ?", hash)
        .execute(conn)
        .await?;

    Ok(())
}

async fn set_uri_unchecked(conn: impl SqliteExecutor<'_>, uri: String) -> Result<()> {
    query!(
        "UPDATE file_uris 
        SET failed_attempts = 0, last_checked_timestamp = NULL
        FROM files
        WHERE file_uris.file_id = files.id AND file_uris.uri = ?",
        uri
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn checked_files(conn: impl SqliteExecutor<'_>) -> Result<u64> {
    query!(
        "
        SELECT COUNT(*) AS count FROM files
        WHERE EXISTS (
            SELECT 1 FROM file_uris
            WHERE file_uris.file_id = files.id
            AND file_uris.last_checked_timestamp IS NOT NULL
        )
        "
    )
    .fetch_one(conn)
    .await?
    .count
    .try_into()
    .map_err(crate::DatabaseError::PrecisionLost)
}

async fn total_files(conn: impl SqliteExecutor<'_>) -> Result<u64> {
    query!(
        "
        SELECT COUNT(*) AS count FROM files
        WHERE EXISTS (
            SELECT 1 FROM file_uris
            WHERE file_uris.file_id = files.id
        )
        "
    )
    .fetch_one(conn)
    .await?
    .count
    .try_into()
    .map_err(crate::DatabaseError::PrecisionLost)
}
