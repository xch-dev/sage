use chia::{
    protocol::{Coin, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::driver::{Nft, NftInfo};

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

pub(crate) struct FullNftCoinSql {
    pub parent_coin_id: Vec<u8>,
    pub puzzle_hash: Vec<u8>,
    pub amount: Vec<u8>,
    pub parent_parent_coin_id: Vec<u8>,
    pub parent_inner_puzzle_hash: Vec<u8>,
    pub parent_amount: Vec<u8>,
    pub launcher_id: Vec<u8>,
    pub metadata: Vec<u8>,
    pub metadata_updater_puzzle_hash: Vec<u8>,
    pub current_owner: Option<Vec<u8>>,
    pub royalty_puzzle_hash: Vec<u8>,
    pub royalty_ten_thousandths: i64,
    pub p2_puzzle_hash: Vec<u8>,
}

impl IntoRow for FullNftCoinSql {
    type Row = Nft<Program>;

    fn into_row(self) -> Result<Nft<Program>, DatabaseError> {
        Ok(Nft {
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
            info: NftInfo {
                launcher_id: to_bytes32(&self.launcher_id)?,
                metadata: self.metadata.into(),
                metadata_updater_puzzle_hash: to_bytes32(&self.metadata_updater_puzzle_hash)?,
                current_owner: self.current_owner.as_deref().map(to_bytes32).transpose()?,
                royalty_puzzle_hash: to_bytes32(&self.royalty_puzzle_hash)?,
                royalty_ten_thousandths: self.royalty_ten_thousandths.try_into()?,
                p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            },
        })
    }
}
