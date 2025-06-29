use crate::{Convert, Database, DatabaseError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OfferStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
    Expired,
}

impl Convert<OfferStatus> for i64 {
    fn convert(self) -> Result<OfferStatus> {
        Ok(match self {
            0 => OfferStatus::Pending,
            1 => OfferStatus::Active,
            2 => OfferStatus::Completed,
            3 => OfferStatus::Cancelled,
            4 => OfferStatus::Expired,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

impl Database {}
