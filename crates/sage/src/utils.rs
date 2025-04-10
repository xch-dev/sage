mod coins;
mod confirmation;
mod offer_status;
mod offer_summary;
mod parse;
mod spends;

pub use coins::*;
pub use confirmation::*;
pub use offer_status::*;
pub use parse::*;

use crate::Error;
use chia::protocol::Bytes32;

pub fn to_bytes<const N: usize>(slice: &[u8]) -> Result<[u8; N], Error> {
    slice
        .try_into()
        .map_err(|_| Error::Database(sage_database::DatabaseError::InvalidLength(slice.len(), N)))
}

pub fn to_bytes32(slice: &[u8]) -> Result<Bytes32, Error> {
    to_bytes(slice).map(Bytes32::new)
}

pub fn to_u64(slice: &[u8]) -> Result<u64, Error> {
    Ok(u64::from_be_bytes(to_bytes::<8>(slice)?))
}

pub fn to_bytes32_opt(bytes: Option<Vec<u8>>) -> Option<Bytes32> {
    bytes.map(|b| to_bytes32(&b).unwrap_or_default())
}
