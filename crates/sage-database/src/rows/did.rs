use chia::protocol::Bytes32;

use crate::{to_bytes32, DatabaseError};

use super::IntoRow;

pub(crate) struct DidSql {
    pub launcher_id: Vec<u8>,
    pub name: Option<String>,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct DidRow {
    pub launcher_id: Bytes32,
    pub name: Option<String>,
    pub visible: bool,
}

impl IntoRow for DidSql {
    type Row = DidRow;

    fn into_row(self) -> Result<DidRow, DatabaseError> {
        Ok(DidRow {
            launcher_id: to_bytes32(&self.launcher_id)?,
            name: self.name,
            visible: self.visible,
        })
    }
}
