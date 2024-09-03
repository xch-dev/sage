use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CatRecord {
    pub asset_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub ticker: Option<String>,
    pub precision: u8,
    pub icon_url: Option<String>,
}
