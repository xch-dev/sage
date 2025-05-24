use chia::protocol::Bytes32;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::{to_bytes32, DatabaseError};

use super::IntoRow;

#[allow(unused)]
pub(crate) struct NftSql {
    pub launcher_id: Vec<u8>,
    pub coin_id: Vec<u8>,
    pub collection_id: Option<Vec<u8>>,
    pub minter_did: Option<Vec<u8>>,
    pub owner_did: Option<Vec<u8>>,
    pub visible: bool,
    pub sensitive_content: bool,
    pub name: Option<String>,
    pub is_owned: bool,
    pub created_height: Option<i64>,
    pub metadata_hash: Option<Vec<u8>>,
    pub is_named: Option<bool>,
    pub is_pending: Option<bool>,
    pub edition_number: Option<i64>,
    pub edition_total: Option<i64>,
}

impl FromRow<'_, SqliteRow> for NftSql {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(NftSql {
            launcher_id: row.try_get("launcher_id")?,
            coin_id: row.try_get("coin_id")?,
            collection_id: row.try_get("collection_id")?,
            minter_did: row.try_get("minter_did")?,
            owner_did: row.try_get("owner_did")?,
            visible: row.try_get("visible")?,
            sensitive_content: row.try_get("sensitive_content")?,
            name: row.try_get("name")?,
            is_owned: row.try_get("is_owned")?,
            created_height: row.try_get("created_height")?,
            metadata_hash: row.try_get("metadata_hash")?,
            is_named: row.try_get("is_named")?,
            is_pending: row.try_get("is_pending")?,
            edition_number: row.try_get("edition_number")?,
            edition_total: row.try_get("edition_total")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NftRow {
    pub launcher_id: Bytes32,
    pub coin_id: Bytes32,
    pub collection_id: Option<Bytes32>,
    pub minter_did: Option<Bytes32>,
    pub owner_did: Option<Bytes32>,
    pub visible: bool,
    pub sensitive_content: bool,
    pub name: Option<String>,
    pub is_owned: bool,
    pub created_height: Option<u32>,
    pub metadata_hash: Option<Bytes32>,
    pub edition_number: Option<u32>,
    pub edition_total: Option<u32>,
}

impl IntoRow for NftSql {
    type Row = NftRow;

    fn into_row(self) -> Result<NftRow, DatabaseError> {
        Ok(NftRow {
            launcher_id: to_bytes32(&self.launcher_id)?,
            coin_id: to_bytes32(&self.coin_id)?,
            collection_id: self.collection_id.as_deref().map(to_bytes32).transpose()?,
            minter_did: self.minter_did.as_deref().map(to_bytes32).transpose()?,
            owner_did: self.owner_did.as_deref().map(to_bytes32).transpose()?,
            visible: self.visible,
            sensitive_content: self.sensitive_content,
            name: self.name,
            is_owned: self.is_owned,
            created_height: self.created_height.map(TryInto::try_into).transpose()?,
            metadata_hash: self.metadata_hash.as_deref().map(to_bytes32).transpose()?,
            edition_number: self.edition_number.map(TryInto::try_into).transpose()?,
            edition_total: self.edition_total.map(TryInto::try_into).transpose()?,
        })
    }
}
