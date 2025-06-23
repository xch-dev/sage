use chia::protocol::Bytes32;
use sqlx::{query, SqliteExecutor};

use crate::{Convert, Database, DatabaseError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Token,
    Nft,
    Did,
    Option,
}

impl Convert<AssetKind> for i64 {
    fn convert(self) -> Result<AssetKind> {
        Ok(match self {
            0 => AssetKind::Token,
            1 => AssetKind::Nft,
            2 => AssetKind::Did,
            3 => AssetKind::Option,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

impl Database {
    pub async fn asset_kind(&self, asset_id: Bytes32) -> Result<Option<AssetKind>> {
        asset_kind(&self.pool, asset_id).await
    }
}

async fn asset_kind(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<AssetKind>> {
    let asset_id = asset_id.as_ref();

    query!("SELECT kind FROM assets WHERE hash = ?", asset_id)
        .fetch_optional(conn)
        .await?
        .map(|row| row.kind.convert())
        .transpose()
}
