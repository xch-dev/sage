use chia::protocol::Bytes32;

use crate::{to_bytes32, to_u64, DatabaseError};

use super::IntoRow;

pub(crate) struct OfferSql {
    pub offer_id: Vec<u8>,
    pub encoded_offer: String,
    pub expiration_height: Option<i64>,
    pub expiration_timestamp: Option<Vec<u8>>,
    pub fee: Vec<u8>,
    pub status: i64,
    pub inserted_timestamp: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OfferRow {
    pub offer_id: Bytes32,
    pub encoded_offer: String,
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
    pub fee: u64,
    pub status: OfferStatus,
    pub inserted_timestamp: u64,
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
            expiration_height: self.expiration_height.map(TryInto::try_into).transpose()?,
            expiration_timestamp: self
                .expiration_timestamp
                .as_deref()
                .map(to_u64)
                .transpose()?,
            fee: to_u64(&self.fee)?,
            status: match self.status {
                0 => OfferStatus::Active,
                1 => OfferStatus::Completed,
                2 => OfferStatus::Cancelled,
                3 => OfferStatus::Expired,
                _ => return Err(DatabaseError::InvalidOfferStatus(self.status)),
            },
            inserted_timestamp: to_u64(&self.inserted_timestamp)?,
        })
    }
}

pub(crate) struct OfferXchSql {
    pub offer_id: Vec<u8>,
    pub requested: bool,
    pub amount: Vec<u8>,
    pub royalty: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct OfferXchRow {
    pub offer_id: Bytes32,
    pub requested: bool,
    pub amount: u64,
    pub royalty: u64,
}

impl IntoRow for OfferXchSql {
    type Row = OfferXchRow;

    fn into_row(self) -> Result<OfferXchRow, DatabaseError> {
        Ok(OfferXchRow {
            offer_id: to_bytes32(&self.offer_id)?,
            requested: self.requested,
            amount: to_u64(&self.amount)?,
            royalty: to_u64(&self.royalty)?,
        })
    }
}

pub(crate) struct OfferNftSql {
    pub offer_id: Vec<u8>,
    pub requested: bool,
    pub launcher_id: Vec<u8>,
    pub royalty_puzzle_hash: Vec<u8>,
    pub royalty_ten_thousandths: i64,
    pub name: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub thumbnail_mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OfferNftRow {
    pub offer_id: Bytes32,
    pub requested: bool,
    pub launcher_id: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
    pub name: Option<String>,
    pub thumbnail: Option<Vec<u8>>,
    pub thumbnail_mime_type: Option<String>,
}

impl IntoRow for OfferNftSql {
    type Row = OfferNftRow;

    fn into_row(self) -> Result<OfferNftRow, DatabaseError> {
        Ok(OfferNftRow {
            offer_id: to_bytes32(&self.offer_id)?,
            requested: self.requested,
            launcher_id: to_bytes32(&self.launcher_id)?,
            royalty_puzzle_hash: to_bytes32(&self.royalty_puzzle_hash)?,
            royalty_ten_thousandths: self.royalty_ten_thousandths.try_into()?,
            name: self.name,
            thumbnail: self.thumbnail,
            thumbnail_mime_type: self.thumbnail_mime_type,
        })
    }
}

pub(crate) struct OfferCatSql {
    pub offer_id: Vec<u8>,
    pub requested: bool,
    pub asset_id: Vec<u8>,
    pub amount: Vec<u8>,
    pub royalty: Vec<u8>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OfferCatRow {
    pub offer_id: Bytes32,
    pub requested: bool,
    pub asset_id: Bytes32,
    pub amount: u64,
    pub royalty: u64,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub icon: Option<String>,
}

impl IntoRow for OfferCatSql {
    type Row = OfferCatRow;

    fn into_row(self) -> Result<OfferCatRow, DatabaseError> {
        Ok(OfferCatRow {
            offer_id: to_bytes32(&self.offer_id)?,
            requested: self.requested,
            asset_id: to_bytes32(&self.asset_id)?,
            amount: to_u64(&self.amount)?,
            royalty: to_u64(&self.royalty)?,
            name: self.name,
            ticker: self.ticker,
            icon: self.icon,
        })
    }
}
