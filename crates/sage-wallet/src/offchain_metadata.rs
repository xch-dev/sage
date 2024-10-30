use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffchainMetadata {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub collection: Option<Collection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    #[serde(default)]
    pub id: Option<String>,

    #[serde(default)]
    pub name: Option<String>,
}
