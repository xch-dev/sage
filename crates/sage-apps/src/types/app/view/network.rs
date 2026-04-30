use serde::{Deserialize, Serialize};
use specta::Type;
use crate::types::SageNetworkWhitelistEntry;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord)]
pub struct SageNetworkWhitelistEntryView {
    scheme: String,
    host: String,
}

impl From<&SageNetworkWhitelistEntry> for SageNetworkWhitelistEntryView {
    fn from(entry: &SageNetworkWhitelistEntry) -> Self {
        Self {
            scheme: entry.scheme().to_string(),
            host: entry.host().to_string(),
        }
    }
}
