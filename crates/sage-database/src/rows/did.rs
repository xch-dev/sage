use chia::protocol::Bytes32;

use crate::{to_bytes32, DatabaseError};

use super::IntoRow;

pub(crate) struct DidSql {
    pub launcher_id: Vec<u8>,
    pub coin_id: Vec<u8>,
    pub name: Option<String>,
    pub visible: bool,
    pub is_owned: bool,
    pub created_height: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DidRow {
    pub launcher_id: Bytes32,
    pub coin_id: Bytes32,
    pub name: Option<String>,
    pub is_owned: bool,
    pub visible: bool,
    pub created_height: Option<u32>,
}

impl IntoRow for DidSql {
    type Row = DidRow;

    fn into_row(self) -> Result<DidRow, DatabaseError> {
        Ok(DidRow {
            launcher_id: to_bytes32(&self.launcher_id)?,
            coin_id: to_bytes32(&self.coin_id)?,
            name: self.name,
            is_owned: self.is_owned,
            visible: self.visible,
            created_height: self.created_height.map(TryInto::try_into).transpose()?,
        })
    }
}
