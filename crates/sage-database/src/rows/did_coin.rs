use chia::{
    protocol::{Bytes32, Coin, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::driver::{Did, DidInfo};

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

pub(crate) struct FullDidCoinSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub parent_parent_coin_id: Vec<u8>,
    pub parent_inner_puzzle_hash: Vec<u8>,
    pub parent_amount: Vec<u8>,
    pub launcher_id: Vec<u8>,
    pub recovery_list_hash: Option<Vec<u8>>,
    pub num_verifications_required: Vec<u8>,
    pub metadata: Vec<u8>,
    pub p2_puzzle_hash: Vec<u8>,
}

pub(crate) struct DidCoinInfoSql {
    pub coin_id: Vec<u8>,
    pub amount: Vec<u8>,
    pub p2_puzzle_hash: Vec<u8>,
    pub recovery_list_hash: Option<Vec<u8>>,
    pub created_height: Option<i64>,
    pub transaction_id: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct DidCoinInfo {
    pub coin_id: Bytes32,
    pub amount: u64,
    pub p2_puzzle_hash: Bytes32,
    pub recovery_list_hash: Option<Bytes32>,
    pub created_height: Option<u32>,
    pub transaction_id: Option<Bytes32>,
}

impl IntoRow for FullDidCoinSql {
    type Row = Did<Program>;

    fn into_row(self) -> Result<Did<Program>, DatabaseError> {
        Ok(Did {
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
            info: DidInfo {
                launcher_id: to_bytes32(&self.launcher_id)?,
                recovery_list_hash: self
                    .recovery_list_hash
                    .as_deref()
                    .map(to_bytes32)
                    .transpose()?,
                num_verifications_required: to_u64(&self.num_verifications_required)?,
                metadata: self.metadata.into(),
                p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            },
        })
    }
}

impl IntoRow for DidCoinInfoSql {
    type Row = DidCoinInfo;

    fn into_row(self) -> Result<DidCoinInfo, DatabaseError> {
        Ok(DidCoinInfo {
            coin_id: to_bytes32(&self.coin_id)?,
            amount: to_u64(&self.amount)?,
            p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            recovery_list_hash: self
                .recovery_list_hash
                .as_deref()
                .map(to_bytes32)
                .transpose()?,
            created_height: self.created_height.map(TryInto::try_into).transpose()?,
            transaction_id: self.transaction_id.as_deref().map(to_bytes32).transpose()?,
        })
    }
}
