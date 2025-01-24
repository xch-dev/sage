use serde::{Deserialize, Serialize};
use specta::Type;

use crate::CatRecord;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RemoveCat {
    pub asset_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct RemoveCatResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateCat {
    pub record: CatRecord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct UpdateCatResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateDid {
    pub did_id: String,
    pub name: Option<String>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct UpdateDidResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateNft {
    pub nft_id: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct UpdateNftResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateNftCollection {
    pub collection_id: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct UpdateNftCollectionResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RedownloadNft {
    pub nft_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct RedownloadNftResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct IncreaseDerivationIndex {
    pub hardened: bool,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct IncreaseDerivationIndexResponse {}
