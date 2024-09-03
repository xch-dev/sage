use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CoinData {
    pub coin_id: String,
    pub address: String,
    pub created_height: Option<u32>,
    pub spent_height: Option<u32>,
    pub amount: String,
}
