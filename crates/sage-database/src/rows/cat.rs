use chia::protocol::Bytes32;

use crate::{to_bytes32, DatabaseError};

pub(crate) struct CatSql {
    pub asset_id: Vec<u8>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct CatRow {
    pub asset_id: Bytes32,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub visible: bool,
}

impl CatSql {
    pub fn into_row(&self) -> Result<CatRow, DatabaseError> {
        Ok(CatRow {
            asset_id: to_bytes32(&self.asset_id)?,
            name: self.name.clone(),
            ticker: self.ticker.clone(),
            description: self.description.clone(),
            icon_url: self.icon_url.clone(),
            visible: self.visible,
        })
    }
}
