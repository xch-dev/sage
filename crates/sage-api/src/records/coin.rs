use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CoinRecord {
    pub coin_id: String,
    pub address: String,
    pub amount: Amount,
    pub created_height: Option<String>,
    pub spent_height: Option<String>,
}
