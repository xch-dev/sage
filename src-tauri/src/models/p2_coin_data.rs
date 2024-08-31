use chia::protocol::Bytes32;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2CoinData {
    #[serde_as(as = "Hex")]
    pub coin_id: Bytes32,
    pub address: String,
    pub created_height: Option<u32>,
    pub spent_height: Option<u32>,
    pub amount: String,
}
