use chia::{
    protocol::{Bytes32, Coin, CoinState},
    puzzles::LineageProof,
};

use crate::{DatabaseError, Result};

pub fn to_coin_state(
    coin: Coin,
    created_height: Option<i64>,
    spent_height: Option<i64>,
) -> Result<CoinState> {
    Ok(CoinState {
        coin,
        spent_height: spent_height.map(TryInto::try_into).transpose()?,
        created_height: created_height.map(TryInto::try_into).transpose()?,
    })
}

pub fn to_coin(parent_coin_id: &[u8], puzzle_hash: &[u8], amount: &[u8]) -> Result<Coin> {
    Ok(Coin {
        parent_coin_info: to_bytes32(parent_coin_id)?,
        puzzle_hash: to_bytes32(puzzle_hash)?,
        amount: u64::from_be_bytes(to_bytes(amount)?),
    })
}

pub fn to_lineage_proof(
    parent_parent_coin_id: &[u8],
    parent_inner_puzzle_hash: &[u8],
    parent_amount: &[u8],
) -> Result<LineageProof> {
    Ok(LineageProof {
        parent_parent_coin_info: to_bytes32(parent_parent_coin_id)?,
        parent_inner_puzzle_hash: to_bytes32(parent_inner_puzzle_hash)?,
        parent_amount: u64::from_be_bytes(to_bytes(parent_amount)?),
    })
}

pub fn to_bytes<const N: usize>(slice: &[u8]) -> Result<[u8; N]> {
    slice
        .try_into()
        .map_err(|_| DatabaseError::InvalidLength(slice.len(), N))
}

pub fn to_bytes32(slice: &[u8]) -> Result<Bytes32> {
    to_bytes(slice).map(Bytes32::new)
}
