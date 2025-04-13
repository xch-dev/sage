use chia::protocol::{Bytes32, Coin, CoinState};

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

pub(crate) struct CoinStateSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub spent_height: Option<i64>,
    pub created_height: Option<i64>,
    pub transaction_id: Option<Vec<u8>>,
    pub kind: i64,
    pub created_unixtime: Option<i64>,
    pub spent_unixtime: Option<i64>,
}

pub(crate) struct CoinSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoinKind {
    Unknown,
    Xch,
    Cat,
    Nft,
    Did,
}

impl CoinKind {
    pub fn from_i64(value: i64) -> Self {
        match value {
            1 => Self::Xch,
            2 => Self::Cat,
            3 => Self::Nft,
            4 => Self::Did,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoinStateRow {
    pub coin_state: CoinState,
    pub transaction_id: Option<Bytes32>,
    pub kind: CoinKind,
    pub created_timestamp: Option<u32>,
    pub spent_timestamp: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct EnhancedCoinStateRow {
    pub base: CoinStateRow,
    pub offer_id: Option<String>,
    pub spend_transaction_id: Option<String>,
}

impl From<CoinStateRow> for EnhancedCoinStateRow {
    fn from(base: CoinStateRow) -> Self {
        Self {
            base,
            offer_id: None,
            spend_transaction_id: None,
        }
    }
}

impl IntoRow for CoinStateSql {
    type Row = CoinStateRow;

    fn into_row(self) -> Result<CoinStateRow, DatabaseError> {
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
            kind: CoinKind::from_i64(self.kind),
            created_timestamp: self.created_unixtime.map(|t| t as u32),
            spent_timestamp: self.spent_unixtime.map(|t| t as u32),
        })
    }
}

impl IntoRow for CoinSql {
    type Row = Coin;

    fn into_row(self) -> Result<Coin, DatabaseError> {
        Ok(Coin {
            parent_coin_info: to_bytes32(&self.parent_coin_id)?,
            puzzle_hash: to_bytes32(&self.puzzle_hash)?,
            amount: to_u64(&self.amount)?,
        })
    }
}
