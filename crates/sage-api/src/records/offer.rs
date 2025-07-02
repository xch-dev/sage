use serde::{Deserialize, Serialize};

use super::OfferSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferRecord {
    pub offer_id: String,
    pub offer: String,
    pub status: OfferRecordStatus,
    pub creation_date: String,
    pub summary: OfferSummary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum OfferRecordStatus {
    Pending = 0,
    Active = 1,
    Completed = 2,
    Cancelled = 3,
    Expired = 4,
}
