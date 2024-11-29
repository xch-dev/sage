use chia::{bls::PublicKey, protocol::Bytes32};

use crate::{to_bytes, to_bytes32, DatabaseError};

use super::IntoRow;

pub(crate) struct DerivationSql {
    pub p2_puzzle_hash: Vec<u8>,
    pub index: i64,
    pub hardened: bool,
    pub synthetic_key: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct DerivationRow {
    pub p2_puzzle_hash: Bytes32,
    pub index: u32,
    pub hardened: bool,
    pub synthetic_key: PublicKey,
}

impl IntoRow for DerivationSql {
    type Row = DerivationRow;

    fn into_row(self) -> Result<DerivationRow, DatabaseError> {
        Ok(DerivationRow {
            p2_puzzle_hash: to_bytes32(&self.p2_puzzle_hash)?,
            index: self.index.try_into()?,
            hardened: self.hardened,
            synthetic_key: PublicKey::from_bytes(&to_bytes(&self.synthetic_key)?)?,
        })
    }
}
