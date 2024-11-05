use chia::protocol::Bytes32;

use crate::{DatabaseError, Result};

pub fn to_bytes<const N: usize>(slice: &[u8]) -> Result<[u8; N]> {
    slice
        .try_into()
        .map_err(|_| DatabaseError::InvalidLength(slice.len(), N))
}

pub fn to_bytes32(slice: &[u8]) -> Result<Bytes32> {
    to_bytes(slice).map(Bytes32::new)
}

pub fn to_u64(slice: &[u8]) -> Result<u64> {
    Ok(u64::from_be_bytes(to_bytes::<8>(slice)?))
}
