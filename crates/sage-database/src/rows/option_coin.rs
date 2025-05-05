use chia::{
    protocol::{Bytes32, Coin},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::driver::{OptionContract, OptionInfo};

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

pub(crate) struct OptionCoinSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub parent_parent_coin_id: Vec<u8>,
    pub parent_inner_puzzle_hash: Vec<u8>,
    pub parent_amount: Vec<u8>,
    pub launcher_id: Vec<u8>,
    pub underlying_coin_id: Vec<u8>,
    pub underlying_delegated_puzzle_hash: Vec<u8>,
    pub p2_puzzle_hash: Vec<u8>,
}

impl IntoRow for OptionCoinSql {
    type Row = OptionContract;

    fn into_row(self) -> Result<OptionContract, DatabaseError> {
        Ok(OptionContract {
            coin: Coin {
                parent_coin_info: to_bytes32(&self.parent_coin_id)?,
                puzzle_hash: to_bytes32(&self.puzzle_hash)?,
                amount: to_u64(&self.amount)?,
            },
            proof: Proof::Lineage(LineageProof {
                parent_parent_coin_info: to_bytes32(&self.parent_parent_coin_id)?,
                parent_inner_puzzle_hash: to_bytes32(&self.parent_inner_puzzle_hash)?,
                parent_amount: to_u64(&self.parent_amount)?,
            }),
            info: OptionInfo {
                launcher_id: to_bytes32(&self.launcher_id)?,
                underlying_coin_id: to_bytes32(&self.underlying_coin_id)?,
                underlying_delegated_puzzle_hash: to_bytes32(
                    &self.underlying_delegated_puzzle_hash,
                )?,
                p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            },
        })
    }
}

pub(crate) struct OptionRecordSql {
    pub coin_id: Vec<u8>,
    pub p2_puzzle_hash: Vec<u8>,
    pub created_height: Option<i64>,
    pub transaction_id: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionRecordInfo {
    pub coin_id: Bytes32,
    pub p2_puzzle_hash: Bytes32,
    pub created_height: Option<u32>,
    pub transaction_id: Option<Bytes32>,
}

impl IntoRow for OptionRecordSql {
    type Row = OptionRecordInfo;

    fn into_row(self) -> Result<OptionRecordInfo, DatabaseError> {
        Ok(OptionRecordInfo {
            coin_id: to_bytes32(&self.coin_id)?,
            p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            created_height: self.created_height.map(TryInto::try_into).transpose()?,
            transaction_id: self.transaction_id.as_deref().map(to_bytes32).transpose()?,
        })
    }
}
