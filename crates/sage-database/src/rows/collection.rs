use chia::protocol::Bytes32;

use crate::{to_bytes32, DatabaseError};

use super::IntoRow;

pub(crate) struct CollectionSql {
    pub collection_id: Vec<u8>,
    pub did_id: Vec<u8>,
    pub metadata_collection_id: String,
    pub visible: bool,
    pub name: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CollectionRow {
    pub collection_id: Bytes32,
    pub did_id: Bytes32,
    pub metadata_collection_id: String,
    pub visible: bool,
    pub name: Option<String>,
    pub icon: Option<String>,
}

impl IntoRow for CollectionSql {
    type Row = CollectionRow;

    fn into_row(self) -> Result<CollectionRow, DatabaseError> {
        Ok(CollectionRow {
            collection_id: to_bytes32(&self.collection_id)?,
            did_id: to_bytes32(&self.did_id)?,
            metadata_collection_id: self.metadata_collection_id.clone(),
            visible: self.visible,
            name: self.name.clone(),
            icon: self.icon.clone(),
        })
    }
}
