use chia::protocol::Bytes32;

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

#[allow(unused)]
pub(crate) struct OfferSql {
    pub offer_id: Vec<u8>,
    pub encoded_offer: String,
    pub expiration_height: i64,
    pub expiration_timestamp: Vec<u8>,
    pub status: i64,
}

#[derive(Debug, Clone)]
pub struct OfferRow {
    pub offer_id: Bytes32,
    pub encoded_offer: String,
    pub expiration_height: u32,
    pub expiration_timestamp: u64,
    pub status: OfferStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum OfferStatus {
    Active = 0,
    Completed = 1,
    Cancelled = 2,
    Expired = 3,
}

impl IntoRow for OfferSql {
    type Row = OfferRow;

    fn into_row(self) -> Result<OfferRow, DatabaseError> {
        Ok(OfferRow {
            offer_id: to_bytes32(&self.offer_id)?,
            encoded_offer: self.encoded_offer,
            expiration_height: self.expiration_height.try_into()?,
            expiration_timestamp: to_u64(&self.expiration_timestamp)?,
            status: match self.status {
                0 => OfferStatus::Active,
                1 => OfferStatus::Completed,
                2 => OfferStatus::Cancelled,
                3 => OfferStatus::Expired,
                _ => return Err(DatabaseError::InvalidOfferStatus(self.status)),
            },
        })
    }
}
