use serde::{Deserialize, Serialize};
use specta::Type;

use super::OfferSummary;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OfferRecord {
    pub offer_id: String,
    pub offer: String,
    pub status: OfferRecordStatus,
    pub creation_date: String,
    pub summary: OfferSummary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum OfferRecordStatus {
    Active = 0,
    Completed = 1,
    Cancelled = 2,
    Expired = 3,
}
