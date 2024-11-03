use chia::protocol::{Bytes32, Coin, CoinState};

use crate::{to_bytes32, to_u64, DatabaseError};

pub(crate) struct CoinStateSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub spent_height: Option<i64>,
    pub created_height: Option<i64>,
    pub transaction_id: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CoinStateRow {
    pub coin_state: CoinState,
    pub transaction_id: Option<Bytes32>,
}

impl CoinStateSql {
    pub(crate) fn into_row(self) -> Result<CoinStateRow, DatabaseError> {
        Ok(CoinStateRow {
            coin_state: CoinState {
                coin: Coin {
                    parent_coin_info: to_bytes32(&self.parent_coin_id)?,
                    puzzle_hash: to_bytes32(&self.puzzle_hash)?,
                    amount: to_u64(&self.amount)?,
                },
                spent_height: self.spent_height.map(TryInto::try_into).transpose()?,
                created_height: self.created_height.map(TryInto::try_into).transpose()?,
            },
            transaction_id: self.transaction_id.as_deref().map(to_bytes32).transpose()?,
        })
    }
}
