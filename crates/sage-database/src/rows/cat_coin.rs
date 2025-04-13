use chia::{
    protocol::{Bytes32, Coin},
    puzzles::LineageProof,
};
use chia_wallet_sdk::driver::Cat;

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

pub(crate) struct CatCoinSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub parent_parent_coin_id: Vec<u8>,
    pub parent_inner_puzzle_hash: Vec<u8>,
    pub parent_amount: Vec<u8>,
    pub p2_puzzle_hash: Vec<u8>,
}

pub(crate) struct FullCatCoinSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub parent_parent_coin_id: Vec<u8>,
    pub parent_inner_puzzle_hash: Vec<u8>,
    pub parent_amount: Vec<u8>,
    pub p2_puzzle_hash: Vec<u8>,
    pub asset_id: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct CatCoinRow {
    pub coin: Coin,
    pub lineage_proof: LineageProof,
    pub p2_puzzle_hash: Bytes32,
}

impl IntoRow for CatCoinSql {
    type Row = CatCoinRow;

    fn into_row(self) -> Result<CatCoinRow, DatabaseError> {
        Ok(CatCoinRow {
            coin: Coin {
                parent_coin_info: to_bytes32(&self.parent_coin_id)?,
                puzzle_hash: to_bytes32(&self.puzzle_hash)?,
                amount: to_u64(&self.amount)?,
            },
            lineage_proof: LineageProof {
                parent_parent_coin_info: to_bytes32(&self.parent_parent_coin_id)?,
                parent_inner_puzzle_hash: to_bytes32(&self.parent_inner_puzzle_hash)?,
                parent_amount: to_u64(&self.parent_amount)?,
            },
            p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
        })
    }
}

impl IntoRow for FullCatCoinSql {
    type Row = Cat;

    fn into_row(self) -> Result<Cat, DatabaseError> {
        Ok(Cat {
            coin: Coin {
                parent_coin_info: to_bytes32(&self.parent_coin_id)?,
                puzzle_hash: to_bytes32(&self.puzzle_hash)?,
                amount: to_u64(&self.amount)?,
            },
            lineage_proof: Some(LineageProof {
                parent_parent_coin_info: to_bytes32(&self.parent_parent_coin_id)?,
                parent_inner_puzzle_hash: to_bytes32(&self.parent_inner_puzzle_hash)?,
                parent_amount: to_u64(&self.parent_amount)?,
            }),
            p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            asset_id: to_bytes32(&self.asset_id)?,
        })
    }
}
