use chia::protocol::Bytes32;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::{to_bytes32, DatabaseError};

use super::IntoRow;

#[allow(unused)]
pub(crate) struct OptionSql {
    pub launcher_id: Vec<u8>,
    pub coin_id: Vec<u8>,
    pub visible: bool,
    pub is_owned: bool,
    pub created_height: Option<i64>,
    pub is_pending: Option<bool>,
}

impl FromRow<'_, SqliteRow> for OptionSql {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(OptionSql {
            launcher_id: row.try_get("launcher_id")?,
            coin_id: row.try_get("coin_id")?,
            visible: row.try_get("visible")?,
            is_owned: row.try_get("is_owned")?,
            created_height: row.try_get("created_height")?,
            is_pending: row.try_get("is_pending")?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OptionRow {
    pub launcher_id: Bytes32,
    pub coin_id: Bytes32,
    pub visible: bool,
    pub is_owned: bool,
    pub created_height: Option<u32>,
}

impl IntoRow for OptionSql {
    type Row = OptionRow;

    fn into_row(self) -> Result<OptionRow, DatabaseError> {
        Ok(OptionRow {
            launcher_id: to_bytes32(&self.launcher_id)?,
            coin_id: to_bytes32(&self.coin_id)?,
            visible: self.visible,
            is_owned: self.is_owned,
            created_height: self.created_height.map(TryInto::try_into).transpose()?,
        })
    }
}
