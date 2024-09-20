use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CatRecord {
    pub asset_id: String,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub visible: bool,
    pub balance: Amount,
}
